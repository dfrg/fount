use read_fonts::TableProvider;

use crate::NormalizedCoord;

/// Metrics for an underline or strikeout decoration.
#[derive(Copy, Clone, Default, Debug)]
pub struct Decoration {
    /// Offset of the decoration from the baseline.
    pub offset: f32,
    /// Thickness of the decoration.
    pub thickness: f32,
}

/// Metrics for an underline or strikeout decoration.
#[derive(Copy, Clone, Default, Debug)]
pub struct BoundingBox {
    /// Minimum x coordinate.
    pub x_min: f32,
    /// Minimum y coordinate.
    pub y_min: f32,
    /// Maximum x coordinate.
    pub x_max: f32,
    /// Maximum y coordinate.
    pub y_max: f32,
}

/// Global font metrics.
#[derive(Copy, Clone, Default, Debug)]
pub struct Metrics {
    /// Number of font design units per em unit.
    pub units_per_em: u16,
    /// Number of glyphs in the font.
    pub glyph_count: u16,
    /// True if the font is not proportionally spaced.
    pub is_monospace: bool,
    /// Italic angle in counter-clockwise degrees from the vertical. Zero for upright text,
    /// negative for text that leans to the right
    pub italic_angle: f32,
    /// Distance from the baseline to the top of the alignment box.
    pub ascent: f32,
    /// Distance from the baseline to the bottom of the alignment box.
    pub descent: f32,
    /// Recommended additional spacing between lines.
    pub leading: f32,
    /// Distance from the baseline to the top of a typical English capital.
    pub cap_height: Option<f32>,
    /// Distance from the baseline to the top of the lowercase "x" or
    /// similar character.
    pub x_height: Option<f32>,
    /// Average width of all non-zero characters in the font.
    pub average_width: Option<f32>,
    /// Maximum advance width of all characters in the font.
    pub max_width: f32,
    /// Metrics for an underline decoration.
    pub underline: Option<Decoration>,
    /// Metrics for a strikeout decoration.
    pub strikeout: Option<Decoration>,
    /// Union of minimum and maximum extents for all glyphs in the font.
    pub bounds: BoundingBox,
}

impl Metrics {
    /// Creates new metrics for the specified font, size in pixels per em, and
    /// normalized variation coordinates.
    ///
    /// If `size` is `None`, resulting metric values will be in font units.
    pub fn new<'a>(
        font: &impl TableProvider<'a>,
        size: Option<f32>,
        coords: &'a [NormalizedCoord],
    ) -> Self {
        let head = font.head();
        let mut metrics = Metrics {
            units_per_em: head.map(|head| head.units_per_em()).unwrap_or_default(),
            ..Default::default()
        };
        let scale = if let Some(size) = size {
            let size = size.abs();
            let upem = metrics.units_per_em.max(1) as f32;
            if size == 0.0 {
                1.0
            } else {
                size / upem
            }
        } else {
            1.0
        };
        if let Ok(head) = font.head() {
            metrics.bounds.x_min = head.x_min() as f32 * scale;
            metrics.bounds.y_min = head.y_min() as f32 * scale;
            metrics.bounds.x_max = head.x_max() as f32 * scale;
            metrics.bounds.y_max = head.y_max() as f32 * scale;
        }
        if let Ok(maxp) = font.maxp() {
            metrics.glyph_count = maxp.num_glyphs();
        }
        if let Ok(post) = font.post() {
            metrics.is_monospace = post.is_fixed_pitch() != 0;
            metrics.italic_angle = post.italic_angle().to_f64() as f32;
            metrics.underline = Some(Decoration {
                offset: post.underline_position().to_i16() as f32 * scale,
                thickness: post.underline_thickness().to_i16() as f32 * scale,
            });
        }
        let hhea = font.hhea();
        if let Ok(hhea) = &hhea {
            metrics.max_width = hhea.x_max_extent().to_i16() as f32 * scale;
        }
        // Choosing proper line metrics is a challenge due to the changing
        // spec, backward compatibility and broken fonts.
        //
        // We use the same strategy as FreeType:
        // 1. Use the OS/2 metrics if the table exists and the USE_TYPO_METRICS
        //    flag is set.
        // 2. Otherwise, use the hhea metrics.
        // 3. If they are zero and the OS/2 table exists:
        //    3a. Use the typo metrics if they are non-zero
        //    3b. Otherwise, use the win metrics
        let os2 = font.os2().ok();
        let mut used_typo_metrics = false;
        if let Some(os2) = &os2 {
            const USE_TYPO_METRICS: u16 = 1 << 7;
            if os2.fs_selection() & USE_TYPO_METRICS != 0 {
                metrics.ascent = os2.s_typo_ascender() as f32 * scale;
                metrics.descent = os2.s_typo_descender() as f32 * scale;
                metrics.leading = os2.s_typo_line_gap() as f32 * scale;
                used_typo_metrics = true;
            }
            metrics.average_width = Some(os2.x_avg_char_width() as f32 * scale);
            metrics.cap_height = os2.s_cap_height().map(|v| v as f32 * scale);
            metrics.x_height = os2.sx_height().map(|v| v as f32 * scale);
            metrics.strikeout = Some(Decoration {
                offset: os2.y_strikeout_position() as f32 * scale,
                thickness: os2.y_strikeout_size() as f32 * scale,
            });
        }
        if !used_typo_metrics {
            if let Ok(hhea) = font.hhea() {
                metrics.ascent = hhea.ascender().to_i16() as f32 * scale;
                metrics.descent = hhea.descender().to_i16() as f32 * scale;
                metrics.leading = hhea.line_gap().to_i16() as f32 * scale;
            }
            if metrics.ascent == 0.0 && metrics.descent == 0.0 {
                if let Some(os2) = &os2 {
                    if os2.s_typo_ascender() != 0 || os2.s_typo_descender() != 0 {
                        metrics.ascent = os2.s_typo_ascender() as f32 * scale;
                        metrics.descent = os2.s_typo_descender() as f32 * scale;
                        metrics.leading = os2.s_typo_line_gap() as f32 * scale;
                    } else {
                        metrics.ascent = os2.us_win_ascent() as f32 * scale;
                        // winDescent is always positive while other descent values are negative. Negate it
                        // to ensure we return consistent metrics.
                        metrics.descent = -(os2.us_win_descent() as f32 * scale);
                    }
                }
            }
        }
        if !coords.is_empty() {
            if let Ok(mvar) = font.mvar() {
                use read_fonts::tables::mvar::tags::*;
                macro_rules! metric_delta {
                    ($tag: ident) => {
                        mvar.metric_delta($tag, coords).unwrap_or_default().to_f64() as f32 * scale
                    };
                }
                metrics.ascent += metric_delta!(HASC);
                metrics.descent += metric_delta!(HDSC);
                metrics.leading += metric_delta!(HLGP);
                if let Some(cap_height) = &mut metrics.cap_height {
                    *cap_height += metric_delta!(CPHT);
                }
                if let Some(x_height) = &mut metrics.x_height {
                    *x_height += metric_delta!(XHGT);
                }
                if let Some(underline) = &mut metrics.underline {
                    underline.offset += metric_delta!(UNDO);
                    underline.thickness += metric_delta!(UNDS);
                }
                if let Some(strikeout) = &mut metrics.strikeout {
                    strikeout.offset += metric_delta!(STRO);
                    strikeout.thickness += metric_delta!(STRS);
                }
            }
        }
        metrics
    }
}
