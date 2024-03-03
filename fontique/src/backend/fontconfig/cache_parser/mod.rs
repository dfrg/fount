#![deny(missing_docs)]

//! A crate for parsing fontconfig cache files.
//!
//! The fontconfig cache format is a C-style binary format, containing a maze of twisty structs all alike,
//! with lots of pointers from one to another. This makes it pretty inefficient to parse the whole file,
//! especially if you're only interested in a few parts. The expected workflow of this crate is:
//!
//! 1. You read the cache file into memory (possibly using `mmap` if the file is large and performance is important).
//! 2. You construct a [`Cache`](crate::Cache::from_bytes), borrowing the memory chunk.
//! 3. You follow the various methods on `Cache` to get access to the information you want.
//!    As you follow those methods, the data will be read incrementally from the memory chunk you
//!    created in part 1.

use bytemuck::AnyBitPattern;
use std::os::raw::{c_int, c_uint};

pub mod data;
pub mod ptr;

use data::{
    CacheData, CharSetData, FontSetData, PatternData, PatternEltData, ValueData, ValueListData,
};
use ptr::{Array, Ptr};

type Result<T> = std::result::Result<T, Error>;

/// A dynamically typed value.
///
/// This is a wrapper around fontconfig's `FcValue` type.
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum Value<'buf> {
    Unknown,
    Void,
    Int(c_int),
    Float(f64),
    String(Ptr<'buf, u8>),
    Bool(c_int),
    /// Not yet supported
    Matrix(Ptr<'buf, ()>),
    CharSet(CharSet<'buf>),
    /// Not yet supported
    FtFace(Ptr<'buf, ()>),
    /// Not yet supported
    LangSet(Ptr<'buf, ()>),
    /// Not yet supported
    Range(Ptr<'buf, ()>),
}

impl<'buf> Ptr<'buf, ValueData> {
    /// Converts the raw C representation to an enum.
    pub fn to_value(&self) -> Result<Value<'buf>> {
        use Value::*;
        let payload = self.deref()?;
        unsafe {
            Ok(match payload.ty {
                -1 => Unknown,
                0 => Void,
                1 => Int(payload.val.i),
                2 => Float(payload.val.d),
                3 => String(self.relative_offset(payload.val.s)?),
                4 => Bool(payload.val.b),
                5 => Matrix(self.relative_offset(payload.val.m)?),
                6 => CharSet(self::CharSet(self.relative_offset(payload.val.c)?)),
                7 => FtFace(self.relative_offset(payload.val.f)?),
                8 => LangSet(self.relative_offset(payload.val.l)?),
                9 => Range(self.relative_offset(payload.val.r)?),
                _ => return Err(Error::InvalidEnumTag(payload.ty)),
            })
        }
    }
}

/// All the different object types supported by fontconfig.
///
/// (We currently only actually handle a few of these.)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum Object {
    Invalid = 0,
    Family,
    FamilyLang,
    Style,
    StyleLang,
    FullName,
    FullNameLang,
    Slant,
    Weight,
    Width,
    Size,
    Aspect,
    PixelSize,
    Spacing,
    Foundry,
    AntiAlias,
    HintStyle,
    Hinting,
    VerticalLayout,
    AutoHint,
    GlobalAdvance,
    File,
    Index,
    Rasterizer,
    Outline,
    Scalable,
    Dpi,
    Rgba,
    Scale,
    MinSpace,
    CharWidth,
    CharHeight,
    Matrix,
    CharSet,
    Lang,
    FontVersion,
    Capability,
    FontFormat,
    Embolden,
    EmbeddedBitmap,
    Decorative,
    LcdFilter,
    NameLang,
    FontFeatures,
    PrgName,
    Hash,
    PostscriptName,
    Color,
    Symbol,
    FontVariations,
    Variable,
    FontHasHint,
    Order,
}

const MAX_OBJECT: c_int = Object::Order as c_int;

impl TryFrom<c_int> for Object {
    type Error = Error;

    fn try_from(value: c_int) -> Result<Self> {
        if value <= MAX_OBJECT {
            Ok(unsafe { std::mem::transmute(value) })
        } else {
            Err(Error::InvalidObjectTag(value))
        }
    }
}

/// A linked list of [`Value`]s.
#[derive(Clone, Debug)]
struct ValueList<'buf>(pub Ptr<'buf, ValueListData>);

impl<'buf> ValueList<'buf> {
    fn value(&self) -> Result<Value<'buf>> {
        self.0
            .relative_offset(ptr::offset(
                std::mem::size_of_val(&self.0.deref()?.next) as isize
            ))
            .and_then(|val_ptr| val_ptr.to_value())
    }
}

/// An iterator over [`Value`]s.
#[derive(Clone, Debug)]
struct ValueListIter<'buf> {
    next: Option<Result<ValueList<'buf>>>,
}

impl<'buf> Iterator for ValueListIter<'buf> {
    type Item = Result<Value<'buf>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        if let Some(Ok(next)) = next {
            match next.0.deref() {
                Ok(next_payload) => {
                    if next_payload.next.0 == 0 {
                        self.next = None;
                    } else {
                        self.next = Some(next.0.relative_offset(next_payload.next).map(ValueList));
                    }
                }
                Err(e) => {
                    self.next = Some(Err(e));
                }
            }
            Some(next.value())
        } else if let Some(Err(e)) = next {
            Some(Err(e))
        } else {
            None
        }
    }
}

/// A list of properties, each one associated with a range of values.
#[derive(Clone, Debug)]
pub struct Pattern<'buf>(pub Ptr<'buf, PatternData>);

impl Pattern<'_> {
    /// Returns an iterator over the elements in this pattern.
    pub fn elts(&self) -> Result<impl Iterator<Item = PatternElt> + '_> {
        let payload = self.0.deref()?;
        let elts = self.0.relative_offset(payload.elts_offset)?;
        Ok(elts.array(payload.num)?.map(PatternElt))
    }

    /// The serialized pattern data, straight from the fontconfig cache.
    pub fn data(&self) -> Result<PatternData> {
        self.0.deref()
    }
}

/// A single element of a [`Pattern`].
///
/// This consists of an [`Object`] type, and a range of values. By convention,
/// the values are all of the same [`Value`] variant (of a type determined by the object
/// tag), but this is not actually enforced.
pub struct PatternElt<'buf>(pub Ptr<'buf, PatternEltData>);

impl<'buf> PatternElt<'buf> {
    /// An iterator over the values in this `PatternElt`.
    pub fn values(&self) -> Result<impl Iterator<Item = Result<Value<'buf>>> + 'buf> {
        Ok(ValueListIter {
            next: Some(Ok(ValueList(
                self.0.relative_offset(self.0.deref()?.values)?,
            ))),
        })
    }

    /// The object tag, describing the font property that this `PatternElt` represents.
    pub fn object(&self) -> Result<Object> {
        self.0.deref()?.object.try_into()
    }

    /// The serialized pattern elt data, straight from the fontconfig cache.
    pub fn data(&self) -> Result<PatternEltData> {
        self.0.deref()
    }
}

/// A set of fonts.
#[derive(Clone, Debug)]
pub struct FontSet<'buf>(pub Ptr<'buf, FontSetData>);

impl<'buf> FontSet<'buf> {
    /// Returns an iterator over the fonts in this set.
    pub fn fonts<'a>(&'a self) -> Result<impl Iterator<Item = Result<Pattern<'buf>>> + 'a> {
        let payload = self.0.deref()?;
        let fonts = self
            .0
            .relative_offset(payload.fonts)?
            .array(payload.nfont)?;
        let me = self.clone();
        Ok(fonts.map(move |font_offset| Ok(Pattern(me.0.relative_offset(font_offset.deref()?)?))))
    }

    /// The serialized font set data, straight from the fontconfig cache.
    pub fn data(&self) -> Result<FontSetData> {
        self.0.deref()
    }
}

/// A set of code points.
#[derive(Clone, Debug)]
pub struct CharSet<'buf>(pub Ptr<'buf, CharSetData>);

impl<'buf> CharSet<'buf> {
    /// Returns an iterator over the leaf bitsets.
    pub fn leaves(&self) -> Result<impl Iterator<Item = Result<CharSetLeaf>> + 'buf> {
        let payload = self.0.deref()?;
        let leaf_array = self.0.relative_offset(payload.leaves)?;
        Ok(leaf_array.array(payload.num)?.map(move |leaf_offset| {
            leaf_array
                .relative_offset(leaf_offset.deref()?)
                .and_then(|leaf_ptr| leaf_ptr.deref())
        }))
    }

    /// Returns an iterator over the 16-bit leaf offsets.
    pub fn numbers(&self) -> Result<Array<'buf, u16>> {
        let payload = self.0.deref()?;
        self.0.relative_offset(payload.numbers)?.array(payload.num)
    }

    /// Creates an iterator over the codepoints in this charset.
    pub fn chars(&self) -> Result<impl Iterator<Item = Result<u32>> + 'buf> {
        // TODO: this iterator-mangling is super-grungy and shouldn't allocate.
        // This would be super easy to write using generators; the main issue is that
        // the early-return-on-decode errors make the control flow tricky to express
        // with combinators and closures.
        fn transpose_result_iter<T: 'static, I: Iterator<Item = T> + 'static>(
            res: Result<I>,
        ) -> impl Iterator<Item = Result<T>> {
            match res {
                Ok(iter) => Box::new(iter.map(|x| Ok(x))) as Box<dyn Iterator<Item = Result<T>>>,
                Err(e) => Box::new(Some(Err(e)).into_iter()) as Box<dyn Iterator<Item = Result<T>>>,
            }
        }

        let leaves = self.leaves()?;
        let numbers = self.numbers()?;
        Ok(leaves.zip(numbers).flat_map(|(leaf, number)| {
            let iter = (move || {
                let number = (number.deref()? as u32) << 8;
                Ok(leaf?.iter().map(move |x| x as u32 + number))
            })();
            transpose_result_iter(iter)
        }))
    }

    /// The `CharSetLeaf` at the given index, if there is one.
    pub fn leaf_at(&self, idx: usize) -> Result<Option<CharSetLeaf>> {
        let payload = self.0.deref()?;
        let leaf_array = self.0.relative_offset(payload.leaves)?;
        leaf_array
            .array(payload.num)?
            .get(idx)
            .map(|ptr| {
                leaf_array
                    .relative_offset(ptr.deref()?)
                    .and_then(|leaf_ptr| leaf_ptr.deref())
            })
            .transpose()
    }

    /// Checks whether this charset contains a given codepoint.
    pub fn contains(&self, ch: u32) -> Result<bool> {
        let hi = ((ch >> 8) & 0xffff) as u16;
        let lo = (ch & 0xff) as u8;
        match self.numbers()?.as_slice()?.binary_search(&hi) {
            // The unwrap will succeed because numbers and leaves have the same length.
            Ok(idx) => Ok(self.leaf_at(idx)?.unwrap().contains_byte(lo)),
            Err(_) => Ok(false),
        }
    }
}

/// A set of bytes, represented as a bitset.
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct CharSetLeaf {
    /// The bits in the set, all 256 of them.
    pub map: [u32; 8],
}

impl CharSetLeaf {
    /// Checks whether this set contains the given byte.
    pub fn contains_byte(&self, byte: u8) -> bool {
        let map_idx = (byte >> 5) as usize;
        let bit_idx = (byte & 0x1f) as u32;

        (self.map[map_idx] >> bit_idx) & 1 != 0
    }

    /// Creates an iterator over bits in this set.
    pub fn iter(self) -> CharSetLeafIter {
        CharSetLeafIter {
            leaf: self,
            map_idx: 0,
        }
    }
}

impl IntoIterator for CharSetLeaf {
    type Item = u8;
    type IntoIter = CharSetLeafIter;
    fn into_iter(self) -> CharSetLeafIter {
        self.iter()
    }
}

/// An iterator over bits in a [`CharSetLeaf`](crate::CharSetLeaf),
/// created by [`CharSetLeaf::iter`](crate::CharSetLeaf::iter).
#[derive(Clone, Debug)]
pub struct CharSetLeafIter {
    leaf: CharSetLeaf,
    map_idx: u8,
}

impl Iterator for CharSetLeafIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        let len = self.leaf.map.len() as u8;
        if self.map_idx >= len {
            None
        } else {
            let bits = &mut self.leaf.map[self.map_idx as usize];
            if *bits != 0 {
                let ret = bits.trailing_zeros() as u8;
                *bits &= !(1 << ret);
                Some(ret + (self.map_idx << 5))
            } else {
                while self.map_idx < len && self.leaf.map[self.map_idx as usize] == 0 {
                    self.map_idx += 1;
                }
                self.next()
            }
        }
    }
}

/// All the possible errors we can encounter when parsing the cache file.
#[derive(Clone, Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Invalid magic number {0:#x}")]
    BadMagic(c_uint),

    #[error("Unsupported version {0}")]
    UnsupportedVersion(c_int),

    #[error("Bad pointer {0}")]
    BadPointer(isize),

    #[error("Bad offset {0}")]
    BadOffset(isize),

    #[error("Bad alignment (expected {expected_alignment}) for offset {offset}")]
    BadAlignment {
        expected_alignment: usize,
        offset: usize,
    },

    #[error("Bad length {0}")]
    BadLength(isize),

    #[error("Invalid enum tag {0}")]
    InvalidEnumTag(c_int),

    #[error("Invalid object tag {0}")]
    InvalidObjectTag(c_int),

    #[error("Unterminated string at {0}")]
    UnterminatedString(isize),

    #[error("Wrong size: header expects {expected} bytes, buffer is {actual} bytes")]
    WrongSize { expected: isize, actual: isize },
}

/// The fontconfig cache header.
#[derive(Clone, Debug)]
pub struct Cache<'buf>(Ptr<'buf, CacheData>);

impl<'buf> Cache<'buf> {
    /// Read a cache from a slice of bytes.
    pub fn from_bytes(buf: &'buf [u8]) -> Result<Self> {
        use Error::*;

        let len = std::mem::size_of::<CacheData>();
        if buf.len() < len {
            Err(WrongSize {
                expected: len as isize,
                actual: buf.len() as isize,
            })
        } else {
            let cache: CacheData = bytemuck::try_pod_read_unaligned(&buf[0..len])
                .expect("but we checked the length...");

            if cache.magic != 4228054020 {
                Err(BadMagic(cache.magic))
            } else if cache.version != 7 && cache.version != 8 {
                Err(UnsupportedVersion(cache.version))
            } else if cache.size != buf.len() as isize {
                Err(WrongSize {
                    expected: cache.size,
                    actual: buf.len() as isize,
                })
            } else {
                Ok(Cache(Ptr {
                    buf,
                    offset: 0,
                    marker: std::marker::PhantomData,
                }))
            }
        }
    }

    /// The [`FontSet`](crate::FontSet) stored in this cache.
    pub fn set(&self) -> Result<FontSet<'buf>> {
        Ok(FontSet(self.0.relative_offset(self.0.deref()?.set)?))
    }

    /// The serialized cache data, straight from the fontconfig cache.
    pub fn data(&self) -> Result<CacheData> {
        self.0.deref()
    }
}
