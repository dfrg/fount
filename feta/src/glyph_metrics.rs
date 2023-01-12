use read_fonts::{
    tables::{hmtx::LongMetric, hvar::Hvar},
    types::{BigEndian, GlyphId},
    TableProvider,
};

use crate::NormalizedCoord;

/// Glyph specified metrics.
#[derive(Clone)]
pub struct GlyphMetrics<'a> {
    scale: f32,
    h_metrics: &'a [LongMetric],
    default_advance_width: u16,
    lsbs: &'a [BigEndian<i16>],
    hvar: Option<Hvar<'a>>,
    coords: &'a [NormalizedCoord],
}

impl<'a> GlyphMetrics<'a> {
    /// Creates new glyph metrics from the specified font, size in pixels per em units, and normalized
    /// variation coordinates.
    ///
    /// Specifying a size of 0.0 will result in glyph metrics that yield
    /// results in font units.
    pub fn new(font: &impl TableProvider<'a>, size: f32, coords: &'a [NormalizedCoord]) -> Self {
        let upem = font
            .head()
            .map(|head| head.units_per_em())
            .unwrap_or_default()
            .max(1);
        let size = size.abs();
        let scale = if size == 0.0 { 1.0 } else { size / upem as f32 };
        let (h_metrics, default_advance_width, lsbs) = font
            .hmtx()
            .map(|hmtx| {
                let h_metrics = hmtx.h_metrics();
                let default_advance_width = h_metrics.last().map(|m| m.advance.get()).unwrap_or(0);
                let lsbs = hmtx.left_side_bearings();
                (h_metrics, default_advance_width, lsbs)
            })
            .unwrap_or_default();
        let hvar = font.hvar().ok();
        Self {
            scale,
            h_metrics,
            default_advance_width,
            lsbs,
            hvar,
            coords,
        }
    }

    /// Returns the advance width for the specified glyph.
    pub fn advance_width(&self, glyph_id: GlyphId) -> f32 {
        let mut advance = self
            .h_metrics
            .get(glyph_id.to_u16() as usize)
            .map(|metric| metric.advance())
            .unwrap_or(self.default_advance_width) as i32;
        if let Some(hvar) = &self.hvar {
            advance += hvar
                .advance_width_delta(glyph_id, self.coords)
                // FreeType truncates metric deltas...
                .map(|delta| delta.to_f64() as i32)
                .unwrap_or(0);
        }
        advance as f32 * self.scale
    }

    /// Returns the left side bearing for the specified glyph.
    pub fn left_side_bearing(&self, glyph_id: GlyphId) -> f32 {
        let gid_index = glyph_id.to_u16() as usize;
        let mut lsb = self
            .h_metrics
            .get(gid_index)
            .map(|metric| metric.side_bearing())
            .unwrap_or_else(|| {
                self.lsbs
                    .get(gid_index.saturating_sub(self.h_metrics.len()))
                    .map(|lsb| lsb.get())
                    .unwrap_or_default()
            }) as i32;
        if let Some(hvar) = &self.hvar {
            lsb += hvar
                .lsb_delta(glyph_id, self.coords)
                // FreeType truncates metric deltas...
                .map(|delta| delta.to_f64() as i32)
                .unwrap_or(0);
        }
        lsb as f32 * self.scale
    }
}
