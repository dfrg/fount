//! Font enumeration and fallback.

#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]

extern crate alloc;

mod attributes;
mod backend;
mod collection;
mod fallback;
mod family;
mod family_name;
mod font;
mod generic;
mod matching;
mod scan;
mod script;
mod source;

#[cfg(feature = "std")]
mod source_cache;

pub use icu_locid::LanguageIdentifier as Language;
pub use peniko::Blob;

pub use attributes::{Attributes, Stretch, Style, Weight};
pub use collection::{Collection, CollectionOptions};
pub use fallback::FallbackKey;
pub use family::{FamilyId, FamilyInfo};
pub use font::{AxisInfo, FontInfo, Synthesis};
pub use generic::GenericFamily;
pub use script::Script;
pub use source::{SourceId, SourceInfo, SourceKind};

#[cfg(feature = "std")]
pub use source_cache::{SourceCache, SourceCacheOptions};
