//! Color outlines loaded from the `COLR` table.

use read_fonts::tables::variations::{DeltaSetIndexMap, ItemVariationStore};
use read_fonts::types::Fixed;
use read_fonts::{tables::colr::*, types::Point, ReadError};
use read_fonts::{FontRead, ResolveOffset};

pub use read_fonts::tables::{colr::Colr, cpal::Cpal};

use super::color::Color;
use super::{color, path, Error, Pen};
use crate::prelude::NormalizedCoord;
use crate::scale::color::ColorPen;
use crate::GlyphId;
use color::Transform;

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::ops::Range;

type PathIndex = usize;
type BrushIndex = usize;

#[derive(Clone, Default, Debug)]
pub struct Context {
    brushes: Vec<(BrushData, Option<Transform>)>,
    stops: Vec<color::ColorStop>,
    paths: Vec<PathData>,
    verbs: Vec<path::Verb>,
    points: Vec<Point<f32>>,
    path_cache: HashMap<GlyphId, PathIndex>,
    visited: HashSet<GlyphId>,
    commands: Vec<Command>,
}

impl Context {
    fn reset(&mut self) {
        self.brushes.clear();
        self.stops.clear();
        self.paths.clear();
        self.verbs.clear();
        self.points.clear();
        self.path_cache.clear();
        self.visited.clear();
        self.commands.clear();
    }

    fn push_path(
        &mut self,
        glyph_id: GlyphId,
        outline_fn: &mut impl FnMut(GlyphId, &mut PathBuilder) -> Result<(), Error>,
    ) -> Result<PathIndex, Error> {
        if let Some(path_index) = self.path_cache.get(&glyph_id) {
            return Ok(*path_index);
        }
        let verb_start = self.verbs.len();
        let point_start = self.points.len();
        let mut builder = PathBuilder {
            verbs: &mut self.verbs,
            points: &mut self.points,
        };
        let res = outline_fn(GlyphId::new(0), &mut builder)?;
        let path_index = self.paths.len();
        self.paths.push(PathData {
            glyph_id,
            verbs: verb_start..self.verbs.len(),
            points: point_start..self.points.len(),
        });
        self.path_cache.insert(glyph_id, path_index);
        Ok(path_index)
    }

    fn path(&self, index: PathIndex) -> Option<path::Path> {
        let path_data = self.paths.get(index)?;
        Some(path::Path::new(
            self.verbs.get(path_data.verbs.clone())?,
            self.points.get(path_data.points.clone())?,
        ))
    }

    fn push_brush(&mut self, brush: BrushData, transform: Option<Transform>) -> BrushIndex {
        let index = self.brushes.len();
        self.brushes.push((brush, transform));
        index
    }

    fn brush(&self, index: BrushIndex) -> Option<(color::Brush, Option<Transform>)> {
        let (brush_data, transform) = self.brushes.get(index)?;
        Some((
            match brush_data {
                BrushData::Solid(color) => color::Brush::Solid(*color),
                BrushData::Gradient(gradient) => color::Brush::Gradient(color::Gradient {
                    kind: gradient.kind,
                    extend: gradient.extend,
                    stops: self.stops.get(gradient.stops.clone())?,
                }),
            },
            *transform,
        ))
    }
}

pub struct Scaler<'a> {
    context: &'a mut Context,
    font: ScalerFont<'a>,
}

impl<'a> Scaler<'a> {
    pub fn new(context: &'a mut Context, colr: Colr<'a>, coords: &'a [NormalizedCoord]) -> Self {
        Self {
            context,
            font: ScalerFont::new(colr, coords),
        }
    }

    pub fn load(
        &mut self,
        glyph_id: GlyphId,
        palette_fn: impl Fn(u16) -> color::Color,
        mut outline_fn: impl FnMut(GlyphId, &mut PathBuilder) -> Result<(), Error>,
        pen: &mut impl color::ColorPen,
    ) -> Result<(), Error> {
        self.context.reset();
        self.context.visited.insert(glyph_id);
        let paint = self.font.base_glyph_paint(glyph_id)?;
        self.load_paint(paint, &palette_fn, &mut outline_fn, 0)?;
        for command in &self.context.commands {
            match command {
                Command::PushTransform(transform) => {
                    pen.push_transform(*transform);
                }
                Command::PopTransform => {
                    pen.pop_transform();
                }
                Command::PushClip { glyph_id, path } => {
                    pen.push_clip(*glyph_id, &self.context.path(*path).unwrap());
                }
                Command::PopClip => {
                    pen.pop_clip();
                }
                Command::PushLayer(mode) => {
                    pen.push_layer(*mode);
                }
                Command::PopLayer => {
                    pen.pop_layer();
                }
                Command::Fill(brush) => {
                    let (brush, transform) = self.context.brush(*brush).unwrap();
                    pen.fill(&brush, transform);
                }
                Command::FillPath {
                    glyph_id,
                    path,
                    brush,
                } => {
                    let (brush, brush_transform) = self.context.brush(*brush).unwrap();
                    pen.fill_path(
                        *glyph_id,
                        &self.context.path(*path).unwrap(),
                        &brush,
                        brush_transform,
                    )
                }
            }
        }
        Ok(())
    }

    fn load_paint(
        &mut self,
        paint: Paint<'a>,
        palette_fn: &impl Fn(u16) -> color::Color,
        outline_fn: &mut impl FnMut(GlyphId, &mut PathBuilder) -> Result<(), Error>,
        recurse_depth: u32,
    ) -> Result<(), Error> {
        let (transform, paint) = flatten_all_transforms(&self.font, paint)?;
        // At this point, we know paint is not a transform. Process the other
        // variants.
        match paint {
            Paint::ColrLayers(layers) => {
                self.maybe_push_transform(&transform);
                let start = layers.first_layer_index() as usize;
                let end = start + layers.num_layers() as usize;
                for layer_ix in start..end {
                    let child_paint = self.font.layer_paint(layer_ix)?;
                    self.load_paint(child_paint, palette_fn, outline_fn, recurse_depth + 1)?;
                }
                self.maybe_pop_transform(&transform);
            }
            Paint::ColrGlyph(glyph) => {
                self.maybe_push_transform(&transform);
                let glyph_id = glyph.glyph_id();
                if self.context.visited.contains(&glyph_id) {
                    // Spec says to do nothing if glyph on recursion blacklist.
                    return Ok(());
                }
                self.context.visited.insert(glyph_id);
                let child_paint = self.font.base_glyph_paint(glyph.glyph_id())?;
                self.load_paint(child_paint, palette_fn, outline_fn, recurse_depth + 1)?;
                self.maybe_pop_transform(&transform);
            }
            Paint::Composite(composite) => {
                self.maybe_push_transform(&transform);
                // Push an empty layer to isolate the blend.
                self.context.commands.push(Command::PushLayer(None));
                // Evaluate the backdrop paint graph.
                let backdrop_paint = composite.backdrop_paint()?;
                self.load_paint(backdrop_paint, palette_fn, outline_fn, recurse_depth + 1)?;
                // Push a layer with the requested composite mode.
                self.context
                    .commands
                    .push(Command::PushLayer(Some(composite.composite_mode())));
                // Evaluate the source paint graph.
                let source_paint = composite.source_paint()?;
                self.load_paint(source_paint, palette_fn, outline_fn, recurse_depth + 1)?;
                // Pop the composite layer.
                self.context.commands.push(Command::PopLayer);
                // Pop the isolation layer.
                self.context.commands.push(Command::PopLayer);
                self.maybe_pop_transform(&transform);
            }
            Paint::Glyph(glyph) => {
                self.maybe_push_transform(&transform);
                let glyph_id = glyph.glyph_id();
                let path_index = self.context.push_path(glyph_id, outline_fn)?;
                let child_paint = glyph.paint()?;
                let (child_transform, child_paint) =
                    flatten_all_transforms(&self.font, child_paint)?;
                if let Some(brush) = self.load_brush_paint(&child_paint, palette_fn)? {
                    // Optimization: if the child paint graph is a transform
                    // sequence followed by a brush, emit a single fill path
                    // command.
                    let brush_index = self.context.push_brush(brush, child_transform);
                    self.context.commands.push(Command::FillPath {
                        glyph_id,
                        path: path_index,
                        brush: brush_index,
                    });
                } else {
                    // Otherwise, push a clip and recurse into the child paint.
                    self.context.commands.push(Command::PushClip {
                        glyph_id,
                        path: path_index,
                    });
                    self.maybe_push_transform(&child_transform);
                    self.load_paint(child_paint, palette_fn, outline_fn, recurse_depth + 1)?;
                    self.maybe_pop_transform(&child_transform);
                    self.context.commands.push(Command::PopClip);
                }
                self.maybe_pop_transform(&transform);
            }
            _ => {
                let brush = self
                    .load_brush_paint(&paint, palette_fn)?
                    .expect("all non-brush paints should have been processed by this point");
                // The remaining transform applies only to the brush.
                let brush_index = self.context.push_brush(brush, transform);
                self.context.commands.push(Command::Fill(brush_index));
            }
        }
        Ok(())
    }

    fn load_brush_paint(
        &mut self,
        paint: &Paint<'a>,
        palette_fn: &impl Fn(u16) -> color::Color,
    ) -> Result<Option<BrushData>, Error> {
        Ok(Some(match paint {
            Paint::Solid(solid) => {
                let mut color = palette_fn(solid.palette_index());
                let alpha = solid.alpha().to_f32();
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                BrushData::Solid(color)
            }
            Paint::VarSolid(solid) => {
                let mut color = palette_fn(solid.palette_index());
                let deltas = self.font.deltas::<1>(solid.var_index_base());
                let alpha = (solid.alpha().to_fixed() + deltas[0]).to_f64() as f32;
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                BrushData::Solid(color)
            }
            Paint::LinearGradient(gradient) => {
                let (stops, extend) = self.push_color_stops(gradient.color_line()?, palette_fn)?;
                let x0 = gradient.x0().to_i16() as f32;
                let y0 = gradient.y0().to_i16() as f32;
                let x1 = gradient.x1().to_i16() as f32;
                let y1 = gradient.y1().to_i16() as f32;
                // TODO: handle x2/y2
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Linear {
                        start: Point::new(x0, y0),
                        end: Point::new(x1, y1),
                    },
                    extend,
                    stops,
                })
            }
            Paint::VarLinearGradient(gradient) => {
                let (stops, extend) =
                    self.push_var_color_stops(gradient.color_line()?, palette_fn)?;
                let deltas = self.font.deltas::<6>(gradient.var_index_base());
                let x0 =
                    (Fixed::from_i32(gradient.x0().to_i16() as i32) + deltas[0]).to_f64() as f32;
                let y0 =
                    (Fixed::from_i32(gradient.y0().to_i16() as i32) + deltas[1]).to_f64() as f32;
                let x1 =
                    (Fixed::from_i32(gradient.x1().to_i16() as i32) + deltas[2]).to_f64() as f32;
                let y1 =
                    (Fixed::from_i32(gradient.y1().to_i16() as i32) + deltas[3]).to_f64() as f32;
                // TODO: handle x2/y2
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Linear {
                        start: Point::new(x0, y0),
                        end: Point::new(x1, y1),
                    },
                    extend,
                    stops,
                })
            }
            Paint::RadialGradient(gradient) => {
                let (stops, extend) = self.push_color_stops(gradient.color_line()?, palette_fn)?;
                let x0 = gradient.x0().to_i16() as f32;
                let y0 = gradient.y0().to_i16() as f32;
                let r0 = gradient.radius0().to_u16() as f32;
                let x1 = gradient.x1().to_i16() as f32;
                let y1 = gradient.y1().to_i16() as f32;
                let r1 = gradient.radius1().to_u16() as f32;
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Radial {
                        start_center: Point::new(x0, y0),
                        start_radius: r0,
                        end_center: Point::new(x1, y1),
                        end_radius: r1,
                    },
                    extend,
                    stops,
                })
            }
            Paint::VarRadialGradient(gradient) => {
                let (stops, extend) =
                    self.push_var_color_stops(gradient.color_line()?, palette_fn)?;
                let deltas = self.font.deltas::<6>(gradient.var_index_base());
                let x0 =
                    (Fixed::from_i32(gradient.x0().to_i16() as i32) + deltas[0]).to_f64() as f32;
                let y0 =
                    (Fixed::from_i32(gradient.y0().to_i16() as i32) + deltas[1]).to_f64() as f32;
                let r0 = (Fixed::from_i32(gradient.radius0().to_u16() as i32) + deltas[2]).to_f64()
                    as f32;
                let x1 =
                    (Fixed::from_i32(gradient.x1().to_i16() as i32) + deltas[3]).to_f64() as f32;
                let y1 =
                    (Fixed::from_i32(gradient.y1().to_i16() as i32) + deltas[4]).to_f64() as f32;
                let r1 = (Fixed::from_i32(gradient.radius1().to_u16() as i32) + deltas[5]).to_f64()
                    as f32;
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Radial {
                        start_center: Point::new(x0, y0),
                        start_radius: r0,
                        end_center: Point::new(x1, y1),
                        end_radius: r1,
                    },
                    extend,
                    stops,
                })
            }
            Paint::SweepGradient(..) => unimplemented!(),
            Paint::VarSweepGradient(..) => unimplemented!(),
            _ => return Ok(None),
        }))
    }

    fn push_color_stops(
        &mut self,
        color_line: ColorLine,
        palette_fn: &impl Fn(u16) -> color::Color,
    ) -> Result<(Range<usize>, Extend), Error> {
        let start = self.context.stops.len();
        self.context
            .stops
            .extend(color_line.color_stops().iter().map(|stop| {
                let mut color = palette_fn(stop.palette_index());
                let alpha = stop.alpha().to_f32();
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                color::ColorStop {
                    offset: stop.stop_offset().to_f32(),
                    color,
                }
            }));
        let end = self.context.stops.len();
        Ok((start..end, color_line.extend()))
    }

    fn push_var_color_stops(
        &mut self,
        color_line: VarColorLine,
        palette_fn: &impl Fn(u16) -> color::Color,
    ) -> Result<(Range<usize>, Extend), Error> {
        let start = self.context.stops.len();
        self.context
            .stops
            .extend(color_line.color_stops().iter().map(|stop| {
                let deltas = self.font.deltas::<2>(stop.var_index_base());
                let mut color = palette_fn(stop.palette_index());
                let alpha = (stop.alpha().to_fixed() + deltas[1]).to_f64() as f32;
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                color::ColorStop {
                    offset: (stop.stop_offset().to_fixed() + deltas[0]).to_f64() as f32,
                    color,
                }
            }));
        let end = self.context.stops.len();
        Ok((start..end, color_line.extend()))
    }

    fn maybe_push_transform(&mut self, transform: &Option<Transform>) {
        if let Some(transform) = transform {
            self.context
                .commands
                .push(Command::PushTransform(*transform));
        }
    }

    fn maybe_pop_transform(&mut self, transform: &Option<Transform>) {
        if transform.is_some() {
            self.context.commands.push(Command::PopTransform);
        }
    }
}

struct ScalerFont<'a> {
    colr: Colr<'a>,
    coords: &'a [NormalizedCoord],
    index_map: Option<DeltaSetIndexMap<'a>>,
    var_store: Option<ItemVariationStore<'a>>,
}

impl<'a> ScalerFont<'a> {
    fn new(colr: Colr<'a>, coords: &'a [NormalizedCoord]) -> Self {
        let index_map = colr.var_index_map().map(|res| res.ok()).flatten();
        let var_store = colr.item_variation_store().map(|res| res.ok()).flatten();
        Self {
            colr,
            coords,
            index_map,
            var_store,
        }
    }

    fn base_glyph_paint(&self, glyph_id: GlyphId) -> Result<Paint<'a>, Error> {
        let list = self
            .colr
            .base_glyph_list()
            .transpose()?
            .ok_or(Error::NoSources)?;
        let records = list.base_glyph_paint_records();
        let record = match records.binary_search_by(|rec| rec.glyph_id().cmp(&glyph_id)) {
            Ok(ix) => &records[ix],
            _ => return Err(Error::GlyphNotFound(glyph_id)),
        };
        Ok(record.paint(list.offset_data())?)
    }

    fn color_layers(
        &self,
        start: usize,
        end: usize,
    ) -> Result<impl Iterator<Item = Result<Paint<'a>, ReadError>> + '_, Error> {
        let layers = self
            .colr
            .layer_list()
            .transpose()?
            .ok_or(Error::NoSources)?;
        let offsets = layers
            .paint_offsets()
            .get(start..end)
            .ok_or(Error::Read(ReadError::OutOfBounds))?;
        Ok(offsets
            .into_iter()
            .map(move |offset| Paint::read(layers.offset_data())))
    }

    fn layer_paint(&self, index: usize) -> Result<Paint<'a>, Error> {
        let layers = self
            .colr
            .layer_list()
            .transpose()?
            .ok_or(Error::NoSources)?;
        let offset = layers
            .paint_offsets()
            .get(index)
            .ok_or(Error::Read(ReadError::OutOfBounds))?;
        Ok(offset.get().resolve(layers.offset_data())?)
    }

    fn deltas<const N: usize>(&self, var_base: u32) -> [Fixed; N] {
        let mut result = [Fixed::ZERO; N];
        if self.coords.is_empty() || self.var_store.is_none() {
            return result;
        }
        let var_store = self.var_store.as_ref().unwrap();
        for i in 0..N {
            let var_idx = var_base + i as u32;
            if let Some(index_map) = self.index_map.as_ref() {
                let Ok(delta_index) = index_map.get(var_idx) else {
                    continue;
                };
                result[i] = var_store
                    .compute_delta(delta_index, self.coords)
                    .unwrap_or_default();
            }
        }
        result
    }
}

pub struct PathBuilder<'a> {
    verbs: &'a mut Vec<path::Verb>,
    points: &'a mut Vec<Point<f32>>,
}

impl Pen for PathBuilder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.verbs.push(path::Verb::MoveTo);
        self.points.push(Point::new(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.verbs.push(path::Verb::LineTo);
        self.points.push(Point::new(x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.verbs.push(path::Verb::QuadTo);
        self.points.push(Point::new(cx0, cy0));
        self.points.push(Point::new(x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.verbs.push(path::Verb::CurveTo);
        self.points.push(Point::new(cx0, cy0));
        self.points.push(Point::new(cx1, cy1));
        self.points.push(Point::new(x, y));
    }

    fn close(&mut self) {
        self.verbs.push(path::Verb::Close);
    }
}

#[derive(Clone, Debug)]
pub struct GradientData {
    pub kind: color::GradientKind,
    pub extend: Extend,
    pub stops: Range<usize>,
}

#[derive(Clone, Debug)]
pub enum BrushData {
    Solid(color::Color),
    Gradient(GradientData),
}

#[derive(Clone, Debug)]
pub struct PathData {
    glyph_id: GlyphId,
    verbs: Range<usize>,
    points: Range<usize>,
}

#[derive(Clone, Debug)]
pub enum Command {
    PushTransform(Transform),
    PopTransform,
    PushClip {
        glyph_id: GlyphId,
        path: PathIndex,
    },
    PopClip,
    PushLayer(Option<CompositeMode>),
    PopLayer,
    Fill(BrushIndex),
    FillPath {
        glyph_id: GlyphId,
        path: PathIndex,
        brush: BrushIndex,
    },
}

fn flatten_all_transforms<'a>(
    font: &ScalerFont,
    mut paint: Paint<'a>,
) -> Result<(Option<Transform>, Paint<'a>), ReadError> {
    let mut transform = Transform::IDENTITY;
    let mut has_transform = false;
    while let Some((child_transform, child_paint)) = flatten_one_transform(font, &paint)? {
        transform = transform * child_transform;
        paint = child_paint;
        has_transform = true;
    }
    if has_transform {
        Ok((Some(transform), paint))
    } else {
        Ok((None, paint))
    }
}

fn flatten_one_transform<'a>(
    font: &ScalerFont,
    paint: &Paint<'a>,
) -> Result<Option<(Transform, Paint<'a>)>, ReadError> {
    Ok(Some(match paint {
        Paint::Transform(transform) => {
            let paint = transform.paint()?;
            let transform = transform.transform()?;
            (
                Transform {
                    xx: transform.xx().to_f64() as f32,
                    yx: transform.yx().to_f64() as f32,
                    xy: transform.xy().to_f64() as f32,
                    yy: transform.yy().to_f64() as f32,
                    dx: transform.dx().to_f64() as f32,
                    dy: transform.dy().to_f64() as f32,
                },
                paint,
            )
        }
        Paint::VarTransform(transform) => {
            let paint = transform.paint()?;
            let transform = transform.transform()?;
            let deltas = font.deltas::<6>(transform.var_index_base());
            (
                Transform {
                    xx: (transform.xx() + deltas[0]).to_f64() as f32,
                    yx: (transform.yx() + deltas[1]).to_f64() as f32,
                    xy: (transform.xy() + deltas[2]).to_f64() as f32,
                    yy: (transform.yy() + deltas[3]).to_f64() as f32,
                    dx: (transform.dx() + deltas[4]).to_f64() as f32,
                    dy: (transform.dy() + deltas[5]).to_f64() as f32,
                },
                paint,
            )
        }
        Paint::Translate(transform) => (
            Transform::translate(
                transform.dx().to_i16() as f32,
                transform.dy().to_i16() as f32,
            ),
            transform.paint()?,
        ),
        Paint::VarTranslate(transform) => {
            let paint = transform.paint()?;
            let deltas = font.deltas::<2>(transform.var_index_base());
            (
                Transform::translate(
                    (Fixed::from_i32(transform.dx().to_i16() as i32) + deltas[0]).to_f64() as f32,
                    (Fixed::from_i32(transform.dy().to_i16() as i32) + deltas[1]).to_f64() as f32,
                ),
                paint,
            )
        }
        Paint::Rotate(transform) => (
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()),
            transform.paint()?,
        ),
        Paint::VarRotate(transform) => (
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()),
            transform.paint()?,
        ),
        Paint::RotateAroundCenter(transform) => (
            Transform::rotate((transform.angle().to_f32() * 180.0).to_radians()).around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        ),
        Paint::VarRotateAroundCenter(transform) => (Transform::default(), transform.paint()?),
        Paint::Scale(transform) => (
            Transform::scale(transform.scale_x().to_f32(), transform.scale_y().to_f32()),
            transform.paint()?,
        ),
        Paint::VarScale(transform) => {
            let paint = transform.paint()?;
            let deltas = font.deltas::<2>(transform.var_index_base());
            (
                Transform::scale(
                    (transform.scale_x().to_fixed() + deltas[0]).to_f64() as f32,
                    (transform.scale_y().to_fixed() + deltas[1]).to_f64() as f32,
                ),
                paint,
            )
        }
        Paint::ScaleAroundCenter(transform) => (
            Transform::scale(transform.scale_x().to_f32(), transform.scale_y().to_f32())
                .around_center(
                    transform.center_x().to_i16() as f32,
                    transform.center_y().to_i16() as f32,
                ),
            transform.paint()?,
        ),
        Paint::VarScaleAroundCenter(transform) => (Transform::default(), transform.paint()?),
        Paint::ScaleUniform(transform) => (
            Transform::scale(transform.scale().to_f32(), transform.scale().to_f32()),
            transform.paint()?,
        ),
        Paint::ScaleUniformAroundCenter(transform) => (
            Transform::scale(transform.scale().to_f32(), transform.scale().to_f32()).around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        ),
        Paint::VarScaleUniform(transform) => {
            let paint = transform.paint()?;
            let deltas = font.deltas::<1>(transform.var_index_base());
            let scale = (transform.scale().to_fixed() + deltas[0]).to_f64() as f32;
            (Transform::scale(scale, scale), paint)
        }
        Paint::VarScaleUniformAroundCenter(transform) => (Transform::default(), transform.paint()?),
        Paint::Skew(transform) => (
            Transform::skew(
                (transform.x_skew_angle().to_f32() * 180.0).to_radians(),
                (transform.y_skew_angle().to_f32() * 180.0).to_radians(),
            ),
            transform.paint()?,
        ),
        Paint::VarSkew(transform) => (Transform::default(), transform.paint()?),
        Paint::SkewAroundCenter(transform) => (
            Transform::skew(
                (transform.x_skew_angle().to_f32() * 180.0).to_radians(),
                (transform.y_skew_angle().to_f32() * 180.0).to_radians(),
            )
            .around_center(
                transform.center_x().to_i16() as f32,
                transform.center_y().to_i16() as f32,
            ),
            transform.paint()?,
        ),
        Paint::VarSkewAroundCenter(transform) => (Transform::default(), transform.paint()?),
        _ => return Ok(None),
    }))
}
