mod bytecode;
mod cache;
mod interpret;
mod math;
mod state;

use super::scaler::ScalerFont;
use crate::scale::Hinting;

use interpret::{Interpreter, Stack, Zone};
use state::InstanceState;

use read_fonts::{
    tables::glyf::PointFlags,
    types::{F26Dot6, Point},
};

/// Slot for the hinting cache.
#[derive(Copy, Clone, Default, Debug)]
pub enum Slot {
    /// Uncached font.
    #[default]
    Uncached,
    /// Font and size cache indices.
    Cached {
        font_index: usize,
        size_index: usize,
    },
}

#[derive(Copy, Clone, Default, Debug)]
pub struct HintConfig {
    hinting: Option<Hinting>,
    is_enabled: bool,
    slot: Option<Slot>,
}

impl HintConfig {
    pub fn new(hinting: Option<Hinting>) -> Self {
        Self {
            hinting,
            is_enabled: hinting.is_some(),
            slot: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn reset(&mut self) {
        self.is_enabled = self.hinting.is_some();
    }
}

/// Aggregate state from the scaler that is necessary for hinting
/// a glyph.
pub struct HintGlyph<'a> {
    pub font: &'a ScalerFont<'a>,
    pub config: &'a mut HintConfig,
    pub points: &'a mut [Point<F26Dot6>],
    pub original: &'a mut [Point<F26Dot6>],
    pub unscaled: &'a mut [Point<i32>],
    pub flags: &'a mut [PointFlags],
    pub contours: &'a mut [u16],
    pub point_base: usize,
    pub contour_base: usize,
    pub phantom: &'a mut [Point<F26Dot6>],
    pub ins: &'a [u8],
    pub is_composite: bool,
}

#[derive(Clone, Default, Debug)]
pub struct HintContext {
    /// Storage for the interpreter stack.
    pub stack: Vec<i32>,
    /// Twilight zone points.
    pub twilight: Vec<Point<i32>>,
    /// Twilight zone tags.
    pub twilight_tags: Vec<PointFlags>,
    cache: cache::Cache,
}

impl HintContext {
    pub fn hint(&mut self, glyph: HintGlyph) -> bool {
        if glyph.config.slot.is_none() {
            let max_twilight = glyph.font.max_twilight as usize + 4;
            self.twilight.resize(max_twilight * 3, Point::default());
            self.twilight_tags
                .resize(max_twilight, PointFlags::default());
            self.stack.resize(glyph.font.max_stack as usize, 0);
            let (font_entry, instance, slot) = self
                .cache
                .find_or_create_entries(&glyph.font, glyph.config.hinting.unwrap_or_default());
            if !font_entry.is_current | !instance.is_current {
                let (cvt, store) = instance.entry.store.split_at_mut(font_entry.entry.cvt_len);
                let (fdefs, idefs) = font_entry
                    .entry
                    .definitions
                    .split_at_mut(font_entry.entry.max_fdefs);
                let glyph_zone = Zone::new(&mut [], &mut [], &mut [], &mut [], &[]);
                let max_twilight = self.twilight_tags.len();
                let (unscaled, rest) = self.twilight.split_at_mut(max_twilight);
                let (original, points) = rest.split_at_mut(max_twilight);
                let twilight_contours = [max_twilight as u16];
                let twilight = Zone::new(
                    unscaled,
                    original,
                    points,
                    &mut self.twilight_tags[..],
                    &twilight_contours,
                );
                let mut hinter = Interpreter::new(
                    store,
                    cvt,
                    fdefs,
                    idefs,
                    twilight,
                    glyph_zone,
                    glyph.font.coords,
                    glyph.font.axis_count,
                );
                if !font_entry.is_current {
                    let mut state = InstanceState::default();
                    if !hinter.run_fpgm(&mut state, Stack::new(&mut self.stack), glyph.font.fpgm) {
                        glyph.config.is_enabled = false;
                        return false;
                    }
                }
                if !instance.is_current {
                    instance.entry.state = InstanceState::default();
                    if !hinter.run_prep(
                        &mut instance.entry.state,
                        glyph.config.hinting.unwrap_or_default(),
                        Stack::new(&mut self.stack),
                        glyph.font.fpgm,
                        glyph.font.prep,
                        glyph.font.ppem,
                        glyph.font.scale.to_bits(),
                    ) {
                        glyph.config.is_enabled = false;
                        return false;
                    }
                }
            }
            glyph.config.slot = Some(slot);
        }
        let (font_entry, instance) = self.cache.from_slot(glyph.config.slot.unwrap());
        if !instance.state.hinting_enabled() {
            return true;
        }
        let point_base = glyph.point_base;
        if glyph.is_composite && glyph.point_base != 0 {
            for c in &mut glyph.contours[glyph.contour_base..] {
                *c -= point_base as u16;
            }
        }
        let (scaled, original, phantom) = unsafe {
            use core::slice::from_raw_parts_mut;
            (
                from_raw_parts_mut(glyph.points.as_mut_ptr() as *mut _, glyph.points.len()),
                from_raw_parts_mut(glyph.original.as_mut_ptr() as *mut _, glyph.original.len()),
                from_raw_parts_mut(glyph.phantom.as_mut_ptr() as *mut _, glyph.phantom.len()),
            )
        };
        let glyph_zone = Zone::new(
            glyph.unscaled,
            original,
            &mut scaled[point_base..],
            &mut glyph.flags[point_base..],
            &glyph.contours[glyph.contour_base..],
        );
        let twilight_len = self.twilight_tags.len();
        let twilight_contours = [twilight_len as u16];
        let (twilight_original, rest) = self.twilight.split_at_mut(twilight_len);
        let (twilight_unscaled, twilight_points) = rest.split_at_mut(twilight_len);
        let twilight = Zone::new(
            twilight_unscaled,
            twilight_original,
            twilight_points,
            &mut self.twilight_tags[..],
            &twilight_contours,
        );
        let (cvt, store) = instance.store.split_at_mut(font_entry.cvt_len);
        let (fdefs, idefs) = font_entry.definitions.split_at_mut(font_entry.max_fdefs);
        let mut hinter = Interpreter::new(
            store,
            cvt,
            fdefs,
            idefs,
            twilight,
            glyph_zone,
            glyph.font.coords,
            glyph.font.axis_count,
        );
        let result = hinter.run(
            &mut instance.state,
            Stack::new(&mut self.stack),
            glyph.font.fpgm,
            glyph.font.prep,
            glyph.ins,
            glyph.is_composite,
        );
        if !instance.state.compat_enabled() {
            for (i, p) in (scaled[scaled.len() - 4..]).iter().enumerate() {
                phantom[i] = *p;
            }
        }
        if glyph.is_composite && point_base != 0 {
            for c in &mut glyph.contours[glyph.contour_base..] {
                *c += point_base as u16;
            }
        }
        result
    }
}
