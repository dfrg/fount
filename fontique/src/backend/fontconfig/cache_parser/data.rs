//! Definitions of fontconfig's raw serialized format.
//!
//! Fontconfig's cache format is defined by writing out C structs byte-for-byte.
//! This module gives a rust definition for some of those structs (more may be
//! supported in the future). Since the fontconfig structs include offsets to other
//! structs, you cannot do much with the raw data itself: you need to interpret
//! that data in the context of its location in the cache. See the [`ptr`](crate::ptr)
//! module for more details.
//!
//! In any case, you are unlikely to need access to the raw serialized format, as this
//! crate defines more convenient wrappers around this raw format.

use bytemuck::AnyBitPattern;
use std::os::raw::{c_int, c_uint};

use super::{
    ptr::{Offset, PtrOffset},
    CharSetLeaf,
};

/// The fontconfig cache header, in the raw serialized format.
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct CacheData {
    /// The magic 4 bytes marking the data as a fontconfig cache.
    pub magic: c_uint,
    /// The cache format version. We support versions 7 and 8.
    pub version: c_int,
    /// The size of the cache.
    pub size: isize,
    /// This cache caches the data of all fonts in some directory.
    /// Here is (an offset to) the name of that directory.
    pub dir: Offset<u8>,
    /// Here is an offset to an array of offsets to the names of
    /// subdirectories.
    pub dirs: Offset<Offset<u8>>,
    /// How many subdirectories are there?
    pub dirs_count: c_int,
    /// An offset to the set of fonts in this cache.
    pub set: Offset<FontSetData>,
    /// A "checksum" of this cache (but really just a timestamp).
    pub checksum: c_int,
    /// Another "checksum" of this cache (but really just a more precise timestamp).
    pub checksum_nano: c_int,
}

/// A set of fonts, in the raw serialized format.
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct FontSetData {
    /// The number of fonts in this set.
    pub nfont: c_int,
    // Capacity of the font array. Uninteresting for the serialized format.
    _sfont: c_int,
    /// Pointer to an array of fonts.
    ///
    /// All the offsets here (both outer and inner) are relative to this `FontSetData`.
    pub fonts: PtrOffset<PtrOffset<PatternData>>,
}

/// The raw serialized format of a [`Pattern`](crate::Pattern).
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct PatternData {
    /// The number of elements in this pattern.
    pub num: c_int,
    // The capacity of the elements array. For serialized data, it's probably
    // the same as `num`.
    _size: c_int,
    /// The offset of the element array.
    pub elts_offset: Offset<PatternEltData>,
    ref_count: c_int,
}

/// A single element of a [`Pattern`](crate::Pattern), in the raw serialized format.
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct PatternEltData {
    /// The object type tag.
    pub object: c_int,
    /// Offset of the linked list of values.
    pub values: PtrOffset<ValueListData>,
}

/// A linked list of [`Value`](crate::Value)s, in the raw serialized format.
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct ValueListData {
    /// An offset to the next element in the linked list.
    pub next: PtrOffset<ValueListData>,
    /// The value of the current list element.
    pub value: ValueData,
    binding: c_int,
}

/// Fontconfig's `FcValue` data type, in the raw serialized format.
#[derive(AnyBitPattern, Copy, Clone)]
#[repr(C)]
pub struct ValueData {
    /// The value's type tag.
    pub ty: c_int,
    /// The value's value.
    pub val: ValueUnion,
}

impl std::fmt::Debug for ValueData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.ty))
        // TODO: write the rest
    }
}

/// A dynamically typed value, as a raw union.
///
/// Many of fontconfig's values are stored as a tagged union. But because
/// there's no layout guarantees for tagged unions in rust, we read them
/// in the C layout, as a combination of `c_int` tag and an untagged union.
///
/// This is the untagged union part.
#[repr(C)]
#[derive(AnyBitPattern, Copy, Clone)]
#[allow(missing_docs)]
pub union ValueUnion {
    pub s: PtrOffset<u8>,
    pub i: c_int,
    pub b: c_int,
    pub d: f64,
    pub m: PtrOffset<()>, // TODO
    pub c: PtrOffset<CharSetData>,
    pub f: PtrOffset<()>,
    pub l: PtrOffset<()>, // TODO
    pub r: PtrOffset<()>, // TODO
}

/// A set of code points, in the raw serialized format.
///
/// # Implementation details
///
/// This charset is implemented as a bunch of bitsets. Each bitset (a [`CharSetLeaf`](crate::CharSetLeaf))
/// has 256 bits, and so it represents the least significant byte of the codepoint. Associated to each
/// leaf is a 16-bit offset, representing the next two least-significant bytes of the codepoint.
/// (In particular, this can represent any subset of the numbers `0` through `0x00FFFFFF`, which is
/// big enough for the unicode range.)
#[derive(AnyBitPattern, Copy, Clone, Debug)]
#[repr(C)]
pub struct CharSetData {
    // Reference count. Not interesting for us.
    ref_count: c_int,
    /// Length of both of the following arrays
    pub num: c_int,
    /// Array of offsets to leaves. Each offset is relative to the start of the array, not the
    /// start of this struct!
    pub leaves: Offset<Offset<CharSetLeaf>>,
    /// Array having the same length as `leaves`, and storing the 16-bit offsets of each leaf.
    pub numbers: Offset<u16>,
}
