/*!
Font metadata reader.
*/

#![no_std]

mod char_map;
mod glyph_metrics;
mod localized_strings;
mod metrics;
mod sequence;
mod setting;
mod variations;

/// Raw primitives for parsing font data.
pub extern crate read_fonts as raw;

pub use raw::{CollectionRef, FileRef, FontRef};

pub use char_map::{CharMap, MapVariant};
pub use glyph_metrics::GlyphMetrics;
pub use localized_strings::{EncodedString, LocalizedString, LocalizedStringId, LocalizedStrings};
pub use metrics::{BoundingBox, Decoration, Metrics};
pub use sequence::Sequence;
pub use setting::Setting;
pub use variations::{NamedInstance, NamedInstances, NormalizedCoord, VariationAxis};

/// Interface for types that can provide font metadata.
pub trait MetadataProvider<'a>: raw::TableProvider<'a> + Sized {
    /// Returns the collection of variation axes.
    fn variation_axes(&self) -> Sequence<'a, VariationAxis<'a>> {
        Sequence::new(self)
    }

    /// Returns the collection of named variation instances.
    fn named_instances(&self) -> NamedInstances<'a> {
        NamedInstances::new(self)
    }

    /// Returns the codepoint to nominal glyph identifier mapping.
    fn char_map(&self) -> CharMap<'a> {
        CharMap::new(self)
    }

    /// Returns the collection of localized strings.
    fn localized_strings(&self) -> LocalizedStrings<'a> {
        LocalizedStrings::new(self)
    }

    /// Returns the global font metrics for the specified size in pixels per em units
    /// and normalized variation coordinates.
    ///
    /// If `size` is `None`, resulting metric values will be in font units.
    fn metrics(&self, size: Option<f32>, coords: &'a [NormalizedCoord]) -> Metrics {
        Metrics::new(self, size, coords)
    }

    /// Returns the glyph specific metrics for the specified size in pixels per em units
    /// and normalized variation coordinates.
    ///
    /// If `size` is `None`, resulting metric values will be in font units.
    fn glyph_metrics(&self, size: Option<f32>, coords: &'a [NormalizedCoord]) -> GlyphMetrics<'a> {
        GlyphMetrics::new(self, size, coords)
    }
}

impl<'a, T> MetadataProvider<'a> for T where T: raw::TableProvider<'a> {}
