use read_fonts::{
    tables::{hmtx::Hmtx, hvar::Hvar},
    types::GlyphId,
    TableProvider,
};

use crate::NormalizedCoord;

/// Glyph specified metrics.
#[derive(Clone)]
pub struct GlyphMetrics<'a> {
    scale: f32,
    hmtx: Option<Hmtx<'a>>,
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
        let hmtx = font.hmtx().ok();
        let hvar = font.hvar().ok();
        Self {
            scale,
            hmtx,
            hvar,
            coords,
        }
    }

    /// Returns the advance width for the specified glyph.
    pub fn advance_width(&self, glyph_id: GlyphId) -> f32 {
        let mut advance = self
            .hmtx
            .as_ref()
            .map(|hmtx| {
                let default_advance = hmtx
                    .h_metrics()
                    .last()
                    .map(|metric| metric.advance())
                    .unwrap_or(0);
                hmtx.h_metrics()
                    .get(glyph_id.to_u16() as usize)
                    .map(|metric| metric.advance())
                    .unwrap_or(default_advance) as i32
            })
            .unwrap_or(0);
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
            .hmtx
            .as_ref()
            .map(|hmtx| {
                hmtx.h_metrics()
                    .get(gid_index)
                    .map(|metric| metric.side_bearing())
                    .unwrap_or_else(|| {
                        hmtx.left_side_bearings()
                            .get(gid_index.saturating_sub(hmtx.h_metrics().len()))
                            .map(|lsb| lsb.get())
                            .unwrap_or(0)
                    }) as i32
            })
            .unwrap_or(0);
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
