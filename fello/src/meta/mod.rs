//! High level interface to font metadata.

pub mod attributes;
pub mod charmap;
pub mod info_strings;
pub mod metrics;
pub mod variations;

mod provider;

pub use provider::MetadataProvider;
