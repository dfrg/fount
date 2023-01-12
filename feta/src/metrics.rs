use read_fonts::TableProvider;

use crate::NormalizedCoord;

/// Global font metrics.
#[derive(Copy, Clone, Default, Debug)]
pub struct Metrics {
    /// Number of font design units per em unit.
    pub units_per_em: u16,
    /// Number of glyphs in the font.
    pub glyph_count: u16,
    /// True if the font is monospace.
    pub is_monospace: bool,
    /// True if the font provides canonical vertical metrics.
    pub has_vertical_metrics: bool,
    /// Distance from the baseline to the top of the alignment box.
    pub ascent: f32,
    /// Distance from the baseline to the bottom of the alignment box.
    pub descent: f32,
    /// Recommended additional spacing between lines.
    pub leading: f32,
    /// Distance from the vertical center baseline to the right edge of
    /// the design space.
    pub vertical_ascent: f32,
    /// Distance from the vertical center baseline to the left edge of
    /// the design space.
    pub vertical_descent: f32,
    /// Recommended additional spacing between columns.
    pub vertical_leading: f32,
    /// Distance from the baseline to the top of a typical English capital.
    pub cap_height: f32,
    /// Distance from the baseline to the top of the lowercase "x" or
    /// similar character.
    pub x_height: f32,
    /// Average width of all non-zero characters in the font.
    pub average_width: f32,
    /// Maximum advance width of all characters in the font.
    pub max_width: f32,
    /// Recommended distance from the baseline to the top of an underline
    /// decoration.
    pub underline_offset: f32,
    /// Recommended thickness of an underline decoration.
    pub underline_size: f32,
    /// Recommended distance from the baseline to the top of a strikeout
    /// decoration.
    pub strikeout_offset: f32,
    /// Recommended thickness of a strikeout decoration.
    pub strikeout_size: f32,
}

impl Metrics {
    /// Creates new metrics for the specified font and normalized variation coordinates.
    pub fn new<'a>(
        font: &impl TableProvider<'a>,
        size: f32,
        coords: &'a [NormalizedCoord],
    ) -> Self {
        let _font = font;
        let _size = size;
        let _coords = coords;
        Self::default()
    }
}
