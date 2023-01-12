/*!
Font metadata reader.
*/

#![no_std]

mod char_map;
mod font;
mod glyph_metrics;
mod localized_strings;
mod metrics;
mod setting;
mod variations;

/// Raw primitives for parsing font data.
pub extern crate read_fonts as raw;

pub use raw::{CollectionRef, FileRef, FontRef};

pub use char_map::{CharMap, MapVariant};
pub use glyph_metrics::GlyphMetrics;
pub use localized_strings::{LocalizedString, LocalizedStringCollection, LocalizedStringId};
pub use metrics::Metrics;
pub use setting::Setting;
pub use variations::{
    Axis, AxisCollection, NamedInstance, NamedInstanceCollection, NormalizedCoord,
};

/// Interface for types that can provide font metadata.
pub trait MetadataProvider<'a>: raw::TableProvider<'a> + Sized {
    /// Returns the collection of variation axes.
    fn axes(&self) -> AxisCollection<'a> {
        AxisCollection::new(self)
    }

    /// Returns the collection of named variation instances.
    fn named_instances(&self) -> NamedInstanceCollection<'a> {
        NamedInstanceCollection::new(self)
    }

    /// Returns the codepoint to nominal glyph identifier mapping.
    fn char_map(&self) -> CharMap<'a> {
        CharMap::new(self)
    }

    /// Returns the collection of localized strings.
    fn localized_strings(&self) -> LocalizedStringCollection<'a> {
        LocalizedStringCollection::new(self)
    }

    /// Returns the global font metrics for the specified size in pixels per em units
    /// and normalized variation coordinates.
    ///
    /// Specifying a size of 0.0 will result in metrics that yield
    /// results in font units.
    fn metrics(&self, size: f32, coords: &'a [NormalizedCoord]) -> Metrics {
        Metrics::new(self, size, coords)
    }

    /// Returns the glyph specific metrics for the specified size in pixels per em units
    /// and normalized variation coordinates.
    ///
    /// Specifying a size of 0.0 will result in glyph metrics that yield
    /// results in font units.
    fn glyph_metrics(&self, size: f32, coords: &'a [NormalizedCoord]) -> GlyphMetrics<'a> {
        GlyphMetrics::new(self, size, coords)
    }
}

// impl<'a> MetadataProvider<'a> for FontRef<'a> {}

impl<'a, T> MetadataProvider<'a> for T where T: raw::TableProvider<'a> {}
