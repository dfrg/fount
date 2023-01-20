use read_fonts::{
    tables::{glyf::Glyf, hmtx::LongMetric, hvar::Hvar, loca::Loca},
    types::{BigEndian, GlyphId},
    TableProvider,
};

use crate::{BoundingBox, NormalizedCoord};

/// Glyph specified metrics.
#[derive(Clone)]
pub struct GlyphMetrics<'a> {
    glyph_count: u16,
    scale: f32,
    h_metrics: &'a [LongMetric],
    default_advance_width: u16,
    lsbs: &'a [BigEndian<i16>],
    hvar: Option<Hvar<'a>>,
    loca_glyf: Option<(Loca<'a>, Glyf<'a>)>,
    coords: &'a [NormalizedCoord],
}

impl<'a> GlyphMetrics<'a> {
    /// Creates new glyph metrics from the specified font, size in pixels per em units, and normalized
    /// variation coordinates.
    ///
    /// If `size` is `None`, resulting metric values will be in font units.
    pub fn new(
        font: &impl TableProvider<'a>,
        size: Option<f32>,
        coords: &'a [NormalizedCoord],
    ) -> Self {
        let glyph_count = font
            .maxp()
            .map(|maxp| maxp.num_glyphs())
            .unwrap_or_default();
        let upem = font
            .head()
            .map(|head| head.units_per_em())
            .unwrap_or_default()
            .max(1);
        let size = size.unwrap_or(0.0).abs();
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
        let loca_glyf = if let (Ok(loca), Ok(glyf)) = (font.loca(None), font.glyf()) {
            Some((loca, glyf))
        } else {
            None
        };
        Self {
            glyph_count,
            scale,
            h_metrics,
            default_advance_width,
            lsbs,
            hvar,
            loca_glyf,
            coords,
        }
    }

    /// Returns the advance width for the specified glyph.
    ///
    /// If normalized coordinates were providing with constructing glyph metrics and
    /// an `HVAR` table is present, applies the appropriate delta.
    pub fn advance_width(&self, glyph_id: GlyphId) -> Option<f32> {
        if glyph_id.to_u16() >= self.glyph_count {
            return None;
        }
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
        Some(advance as f32 * self.scale)
    }

    /// Returns the left side bearing for the specified glyph.
    ///
    /// If normalized coordinates were providing with constructing glyph metrics and
    /// an `HVAR` table is present, applies the appropriate delta.
    pub fn left_side_bearing(&self, glyph_id: GlyphId) -> Option<f32> {
        if glyph_id.to_u16() >= self.glyph_count {
            return None;
        }
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
        Some(lsb as f32 * self.scale)
    }

    /// Returns the bounding box for the specified glyph.
    ///
    /// Note that variations are not reflected in the bounding box returned by
    /// this method.
    pub fn bounds(&self, glyph_id: GlyphId) -> Option<BoundingBox> {
        let (loca, glyf) = self.loca_glyf.as_ref()?;
        Some(match loca.get_glyf(glyph_id, glyf).ok()? {
            Some(glyph) => BoundingBox {
                x_min: glyph.x_min() as f32,
                y_min: glyph.y_min() as f32,
                x_max: glyph.x_max() as f32,
                y_max: glyph.y_max() as f32,
            },
            // Empty glyphs have an empty bounding box
            None => BoundingBox::default(),
        })
    }
}
