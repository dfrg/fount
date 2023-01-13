/*!
Font metadata reader.
*/

#![no_std]

mod attributes;
mod charmap;
mod glyph_metrics;
mod localized_strings;
mod metrics;
mod setting;
mod variations;

/// Raw primitives for parsing font data.
pub extern crate read_fonts as raw;

pub use raw::{CollectionRef, FileRef, FontRef};

pub use attributes::{Stretch, Style, Weight};
pub use charmap::{Charmap, MapVariant};
pub use glyph_metrics::GlyphMetrics;
pub use localized_strings::{EncodedString, LocalizedString, LocalizedStringList, StringId};
pub use metrics::{BoundingBox, Decoration, Metrics};
pub use setting::SelectorValue;
pub use variations::{
    NamedInstance, NamedInstanceList, NormalizedCoord, VariationAxis, VariationAxisList,
};

/// Interface for types that can provide font metadata.
pub trait MetadataProvider<'a>: raw::TableProvider<'a> + Sized {
    /// Returns the list of variation axes.
    fn variation_axes(&self) -> VariationAxisList<'a> {
        VariationAxisList::new(self)
    }

    /// Returns the list of named variation instances.
    fn named_instances(&self) -> NamedInstanceList<'a> {
        NamedInstanceList::new(self)
    }

    /// Returns the codepoint to nominal glyph identifier mapping.
    fn charmap(&self) -> Charmap<'a> {
        Charmap::new(self)
    }

    /// Returns the list of localized strings.
    fn localized_strings(&self) -> LocalizedStringList<'a> {
        LocalizedStringList::new(self)
    }

    /// Returns the stretch, style and weight attributes.
    fn attributes(&self) -> (Stretch, Style, Weight) {
        attributes::from_font(self)
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
