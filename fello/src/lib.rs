//! A robust, ergonomic, high performance crate for OpenType fonts.
//!  
//! Skrifa is a mid level library that provides access to various types
//! of [`metadata`](MetadataProvider) contained in a font as well as support
//! for [`scaling`](scale) (extraction) of glyph outlines.
//!
//! It is described as "mid level" because the library is designed to sit
//! above low level font parsing (provided by [`read-fonts`](https://crates.io/crates/read-fonts))
//! and below a higher level text layout engine.
//!
//! See the [readme](https://github.com/dfrg/fontations/blob/main/skrifa/README.md)
//! for additional details.

// #![forbid(unsafe_code)]
// TODO: this is temporary-- remove when hinting is added.
#![allow(dead_code, unused_imports, unused_variables)]

/// Expose our "raw" underlying parser crate.
pub extern crate read_fonts as raw;

pub mod attribute;

// #[doc(hidden)]
pub mod charmap;
#[doc(hidden)]
pub mod font;
pub mod instance;
pub mod metrics;
pub mod palette;
#[doc(hidden)]
#[cfg(feature = "scale")]
pub mod scale;
#[doc(hidden)]

pub mod setting;
pub mod string;
pub mod variation;

// /// Mapping of characters to glyph identifiers.
// pub mod layout {}

/// Loading, scaling and hinting of glyph outlines.
pub mod outline {    
    pub struct Font {}
    pub struct Setting {}
    pub struct Scale {}
    pub struct Location {}
    pub struct LocationRef {}
    pub struct Tag {}
    pub struct GlyphId {}
    pub struct Key {}

    pub type NormalizedCoord = raw::types::F2Dot14;
}

// /// Collections of fonts supporting fallback and enumeration.
// pub mod family {}

/// Writing systems and associated typographic features.
pub mod feature {}

mod provider;

/// Useful collection of common types suitable for glob importing.
pub mod prelude {
    #[doc(no_inline)]
    pub use super::{
        font::{FontRef, UniqueId},
        instance::{LocationRef, NormalizedCoord, Size},
        GlyphId, MetadataProvider, Tag,
    };
}

pub use read_fonts::types::{GlyphId, Tag};

#[doc(inline)]
pub use provider::MetadataProvider;
