use super::{
    attributes::Attributes,
    charmap::Charmap,
    info_strings::InfoStrings,
    metrics::{GlyphMetrics, Metrics},
    variations::{axis::Axes, instance::Instances},
};

use crate::{NormalizedCoord, NormalizedCoords, Size};

/// Interface for types that can provide font metadata.
pub trait MetadataProvider<'a>: raw::TableProvider<'a> + Sized {
    /// Returns the stretch, style and weight attributes.
    fn attributes(&self) -> Attributes {
        Attributes::new(self)
    }

    /// Returns the collection of variations.
    fn axes(&self) -> Axes<'a> {
        Axes::new(self)
    }

    /// Returns the collection of named variation instances.
    fn instances(&self) -> Instances<'a> {
        Instances::new(self)
    }

    /// Returns the collection of informational strings.
    fn info_strings(&self) -> InfoStrings<'a> {
        InfoStrings::new(self)
    }

    /// Returns the global font metrics for the specified size and normalized variation
    /// coordinates.
    fn metrics(&self, size: Size, coords: NormalizedCoords<'a>) -> Metrics {
        Metrics::new(self, size, coords)
    }

    /// Returns the glyph specific metrics for the specified size and normalized variation
    /// coordinates.
    fn glyph_metrics(&self, size: Size, coords: NormalizedCoords<'a>) -> GlyphMetrics<'a> {
        GlyphMetrics::new(self, size, coords)
    }

    /// Returns the codepoint to nominal glyph identifier mapping.
    fn charmap(&self) -> Charmap<'a> {
        Charmap::new(self)
    }
}

/// Blanket implementation of `MetadataProvider` for any type that implements
/// `TableProvider`.
impl<'a, T> MetadataProvider<'a> for T where T: raw::TableProvider<'a> {}
