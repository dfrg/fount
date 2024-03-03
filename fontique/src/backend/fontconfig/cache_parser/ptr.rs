//! Pointers and offset's within fontconfig's cache.
//!
//! Fontconfig's cache is stuctured as a collection of structs.
//! These structs are encoded in the cache by
//! writing their bytes into a cache file. (So the format is architecture-dependent.)
//! The structs reference each other using relative offsets, so for example in the struct
//!
//! ```C
//! struct FcPattern {
//!     int num;
//!     int size;
//!     intptr_t elts_offset;
//!     int ref;
//! }
//! ```
//!
//! the elements `num`, `size`, and `ref` are just plain old data, and the element `elts_offset`
//! says that there is some other struct (which happens to be an `FcPatternElt` in this case)
//! stored at the location `base_offset + elts_offset`, where `base_offset` is the offset
//! of the `FcPattern`. Note that `elts_offset` is signed: it can be negative.
//!
//! We encode these offsets using `Offset`, so for example the struct above gets translated to
//!
//! ```ignore
//! struct PatternData {
//!     num: c_int,
//!     size: c_int,
//!     elts_offset: Offset<PatternElt>,
//!     ref: c_int,
//! }
//! ```
//!
//! Sometimes, the structs in fontconfig contain pointers instead of offsets, like for example
//!
//! ```C
//! struct FcPatternElt {
//!     FcObject object;
//!     FcValueList *values;
//! }
//! ```
//!
//! In this case, fontconfig actually handles two cases: if the lowest-order bit of `values` is 0
//! it's treated as a normal pointer, but if the lowest-order bit is 1 then that bit is set
//! to zero and `values` is treated as an offset. When the struct came from a cache file that
//! was serialized to disk (which we always are in this crate), it should always be in the "offset" case.
//! That is, these pointers get treated almost the same as offsets, except that we need to
//! sanity-check the low-order bit and then set it to zero. We encode these as `PtrOffset`,
//! so for example the struct above gets translated to
//!
//! ```ignore
//! struct PatternEltData {
//!     object: c_int,
//!     values: PtrOffset<ValueList>,
//! }

use bytemuck::AnyBitPattern;
use std::os::raw::c_int;

use super::Error;

type Result<T> = std::result::Result<T, Error>;

/// A relative offset to another struct in the cache, which is encoded as a pointer in fontconfig.
///
/// See [`Offset`] for more on offsets in fontconfig and how we handle them in this crate.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct PtrOffset<T: Copy>(pub isize, std::marker::PhantomData<T>);

unsafe impl<T: Copy> bytemuck::Zeroable for PtrOffset<T> {}
unsafe impl<T: Copy + 'static> bytemuck::Pod for PtrOffset<T> {}

/// This is basically equivalent to `TryInto<Offset<T>, Error=Error>`, but having this
/// alias makes type inference work better.
pub trait IntoOffset: AnyBitPattern + Copy {
    /// Into an offset of what type?
    type Item: AnyBitPattern + Copy;

    /// Turns `self` into an `Offset`.
    fn into_offset(self) -> Result<Offset<Self::Item>>;
}

impl<T: AnyBitPattern + Copy> IntoOffset for PtrOffset<T> {
    type Item = T;

    fn into_offset(self) -> Result<Offset<T>> {
        if self.0 & 1 == 0 {
            Err(Error::BadPointer(self.0))
        } else {
            Ok(Offset(self.0 & !1, std::marker::PhantomData))
        }
    }
}

impl<T: AnyBitPattern + Copy> IntoOffset for Offset<T> {
    type Item = T;

    fn into_offset(self) -> Result<Offset<T>> {
        Ok(self)
    }
}

/// A relative offset to another struct in the cache.
///
/// # Implementation details
///
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Offset<T: Copy>(isize, std::marker::PhantomData<T>);

pub(crate) fn offset<T: Copy>(off: isize) -> Offset<T> {
    Offset(off, std::marker::PhantomData)
}

unsafe impl<T: Copy> bytemuck::Zeroable for Offset<T> {}
unsafe impl<T: Copy + 'static> bytemuck::Pod for Offset<T> {}

/// A reference to a fontconfig struct that's been serialized in a buffer.
#[derive(Clone)]
pub struct Ptr<'buf, S> {
    /// We point at this `offset`, relative to the buffer.
    pub offset: isize,
    /// The buffer that we point into.
    pub buf: &'buf [u8],
    pub(crate) marker: std::marker::PhantomData<S>,
}

impl<'buf, S> std::fmt::Debug for Ptr<'buf, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ptr").field("offset", &self.offset).finish()
    }
}

/// A reference to an array of serialized fontconfig structs.
#[derive(Clone, Debug)]
pub struct Array<'buf, T> {
    buf: &'buf [u8],
    offset: usize,
    size: isize,
    marker: std::marker::PhantomData<T>,
}

impl<'buf, T: AnyBitPattern> Array<'buf, T> {
    fn new(buf: &'buf [u8], offset: isize, size: c_int) -> Result<Self> {
        let len = std::mem::size_of::<T>();
        let total_len = len
            .checked_mul(size as usize)
            .ok_or(Error::BadLength(size as isize))?;

        if offset < 0 {
            Err(Error::BadOffset(offset))
        } else {
            let end = (offset as usize)
                .checked_add(total_len)
                .ok_or(Error::BadLength(size as isize))?;
            if end > buf.len() {
                Err(Error::BadOffset(end as isize))
            } else {
                Ok(Array {
                    buf,
                    offset: offset as usize,
                    size: size as isize,
                    marker: std::marker::PhantomData,
                })
            }
        }
    }

    /// The number of elements in this array.
    pub fn len(&self) -> usize {
        self.size as usize
    }

    /// Is this array empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Retrieve an element at a given index, if that index isn't too big.
    pub fn get(&self, idx: usize) -> Option<Ptr<'buf, T>> {
        if (idx as isize) < self.size {
            let len = std::mem::size_of::<T>() as isize;
            Some(Ptr {
                buf: self.buf,
                offset: self.offset as isize + (idx as isize) * len,
                marker: std::marker::PhantomData,
            })
        } else {
            None
        }
    }

    /// View this array as a rust slice.
    ///
    /// This conversion might fail if the alignment is wrong. That definitely won't happen if `T` has
    /// a two-byte alignment. It's *probably* fine in general, but don't blame me if it isn't.
    pub fn as_slice(&self) -> Result<&'buf [T]> {
        let len = std::mem::size_of::<T>() * self.size as usize;
        bytemuck::try_cast_slice(&self.buf[self.offset..(self.offset + len)]).map_err(|_| {
            Error::BadAlignment {
                offset: self.offset,
                expected_alignment: std::mem::align_of::<T>(),
            }
        })
    }
}

impl<'buf, T: AnyBitPattern> Iterator for Array<'buf, T> {
    type Item = Ptr<'buf, T>;

    fn next(&mut self) -> Option<Ptr<'buf, T>> {
        if self.size <= 0 {
            None
        } else {
            let len = std::mem::size_of::<T>();
            let ret = Ptr {
                buf: self.buf,
                offset: self.offset as isize,
                marker: std::marker::PhantomData,
            };
            self.offset += len;
            self.size -= 1;
            Some(ret)
        }
    }
}

impl<'buf> Ptr<'buf, u8> {
    /// Assuming that this `Ptr<u8>` is pointing to the beginning of a null-terminated string,
    /// return that string.
    pub fn str(&self) -> Result<&'buf [u8]> {
        let offset = self.offset;
        if offset < 0 || offset > self.buf.len() as isize {
            Err(Error::BadOffset(offset))
        } else {
            let buf = &self.buf[(offset as usize)..];
            let null_offset = buf
                .iter()
                .position(|&c| c == 0)
                .ok_or(Error::UnterminatedString(offset))?;
            Ok(&buf[..null_offset])
        }
    }
}

impl<'buf, S: AnyBitPattern> Ptr<'buf, S> {
    /// Turn `offset` into a pointer, assuming that it's an offset relative to this pointer.
    ///
    /// In order to be certain about which offsets are relative to what, you'll need to check
    /// the fontconfig source. But generally, offsets stored in a struct are relative to the
    /// base address of that struct. So for example, to access the `dir` field in
    /// [`Cache`](crate::Cache) you could call `cache.relative_offset(cache.deref()?.dir)?`.
    /// This will give you a `Ptr<u8>` pointing to the start of the directory name.
    pub fn relative_offset<Off: IntoOffset>(&self, offset: Off) -> Result<Ptr<'buf, Off::Item>> {
        let offset = offset.into_offset()?;
        Ok(Ptr {
            buf: self.buf,
            offset: self
                .offset
                .checked_add(offset.0)
                .ok_or(Error::BadOffset(offset.0))?,
            marker: std::marker::PhantomData,
        })
    }

    /// "Dereference" this pointer, returning a plain struct.
    pub fn deref(&self) -> Result<S> {
        let len = std::mem::size_of::<S>() as isize;
        if self.offset + len >= self.buf.len() as isize {
            Err(Error::BadOffset(self.offset))
        } else {
            // We checked at construction time that the buffer has enough elements for the payload,
            // so the slice will succeed.
            Ok(bytemuck::try_pod_read_unaligned(
                &self.buf[(self.offset as usize)..((self.offset + len) as usize)],
            )
            .expect("but we checked the length..."))
        }
    }

    /// Treating this pointer as a reference to the start of an array of length `count`,
    /// return an iterator over that array.
    pub fn array(&self, count: c_int) -> Result<Array<'buf, S>> {
        Array::new(self.buf, self.offset, count)
    }
}
