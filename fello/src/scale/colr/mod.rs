//! Color outlines loaded from the `COLR` table.

use read_fonts::tables::variations::{DeltaSetIndexMap, ItemVariationStore};
use read_fonts::types::{BoundingBox, F2Dot14, FWord, Fixed, UfWord};
use read_fonts::{tables::colr::*, types::Point, ReadError};
use read_fonts::{FontRead, ResolveOffset};

pub use read_fonts::tables::{
    colr::{Colr, ColrInstance, ResolvedPaint},
    cpal::Cpal,
};

use super::color::Color;
use super::{color, path, Error, Pen};
use crate::prelude::NormalizedCoord;
use crate::scale::color::ColorPen;
use crate::GlyphId;
use color::Transform;

use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::ops::Range;

/// Index for a cached path.
type PathIndex = usize;

/// Index for a cached brush.
type BrushIndex = usize;

/// Identifier used for representing a paint on the recursion blacklist.
type PaintId = usize;

#[derive(Clone, Default, Debug)]
pub struct Context {
    brushes: Vec<(BrushData, Option<Transform>)>,
    stops: Vec<color::ColorStop>,
    paths: Vec<PathData>,
    verbs: Vec<path::Verb>,
    points: Vec<Point<f32>>,
    path_cache: HashMap<GlyphId, PathIndex>,
    blacklist: HashSet<PaintId>,
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
        self.blacklist.clear();
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
        let res = outline_fn(glyph_id, &mut builder)?;
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
    instance: ColrInstance<'a>,
}

impl<'a> Scaler<'a> {
    pub fn new(context: &'a mut Context, colr: Colr<'a>, coords: &'a [NormalizedCoord]) -> Self {
        Self {
            context,
            instance: ColrInstance::new(colr, coords),
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
        if self.instance.version() == 0 {
            self.load_v0(glyph_id, &palette_fn, &mut outline_fn)?;
        } else {
            if let Ok(Some((paint, paint_id))) = self.instance.v1_base_glyph(glyph_id) {
                self.context.blacklist.insert(paint_id);
                self.load_paint(
                    paint.resolve(&self.instance)?,
                    &palette_fn,
                    &mut outline_fn,
                    0,
                )?;
            } else {
                self.load_v0(glyph_id, &palette_fn, &mut outline_fn)?;
            }
        }
        let bounds = self
            .instance
            .v1_clip_box(glyph_id)
            .ok()
            .flatten()
            .map(|cbox| cbox.resolve(&self.instance))
            .map(|cbox| BoundingBox {
                x_min: cbox.x_min.to_f64() as f32,
                y_min: cbox.y_min.to_f64() as f32,
                x_max: cbox.x_max.to_f64() as f32,
                y_max: cbox.y_max.to_f64() as f32,
            })
            .unwrap_or(BoundingBox {
                x_min: -4096.0,
                y_min: -4096.0,
                x_max: 4096.0,
                y_max: 4096.0,
            });
        pen.bounds(bounds);
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

    fn load_v0(
        &mut self,
        glyph_id: GlyphId,
        palette_fn: &impl Fn(u16) -> color::Color,
        mut outline_fn: &mut impl FnMut(GlyphId, &mut PathBuilder) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let layer_range = self
            .instance
            .v0_base_glyph(glyph_id)?
            .ok_or(Error::GlyphNotFound(glyph_id))?;
        for layer_ix in layer_range {
            let (glyph_id, color_index) = self.instance.v0_layer(layer_ix)?;
            let path_index = self.context.push_path(glyph_id, &mut outline_fn)?;
            let color = palette_fn(color_index);
            let brush_index = self.context.push_brush(BrushData::Solid(color), None);
            self.context.commands.push(Command::FillPath {
                glyph_id,
                path: path_index,
                brush: brush_index,
            });
        }
        Ok(())
    }

    fn load_paint(
        &mut self,
        paint: ResolvedPaint<'a>,
        palette_fn: &impl Fn(u16) -> color::Color,
        outline_fn: &mut impl FnMut(GlyphId, &mut PathBuilder) -> Result<(), Error>,
        recurse_depth: u32,
    ) -> Result<(), Error> {
        let (transform, paint) = flatten_all_transforms(&self.instance, paint)?;
        // At this point, we know paint is not a transform. Process the other
        // variants.
        match paint {
            ResolvedPaint::ColrLayers { range } => {
                self.maybe_push_transform(&transform);
                for layer_ix in range {
                    let (child_paint, child_paint_id) = self.instance.v1_layer(layer_ix)?;
                    if !self.context.blacklist.contains(&child_paint_id) {
                        self.context.blacklist.insert(child_paint_id);
                        self.load_paint(
                            child_paint.resolve(&self.instance)?,
                            palette_fn,
                            outline_fn,
                            recurse_depth + 1,
                        )?;
                        self.context.blacklist.remove(&child_paint_id);
                    }
                }
                self.maybe_pop_transform(&transform);
            }
            ResolvedPaint::ColrGlyph { glyph_id } => {
                self.maybe_push_transform(&transform);
                let (child_paint, child_paint_id) = self
                    .instance
                    .v1_base_glyph(glyph_id)?
                    .ok_or(Error::GlyphNotFound(glyph_id))?;
                if !self.context.blacklist.contains(&child_paint_id) {
                    self.context.blacklist.insert(child_paint_id);
                    self.load_paint(
                        child_paint.resolve(&self.instance)?,
                        palette_fn,
                        outline_fn,
                        recurse_depth + 1,
                    )?;
                    self.context.blacklist.remove(&child_paint_id);
                }
                self.maybe_pop_transform(&transform);
            }
            ResolvedPaint::Composite {
                source_paint,
                mode,
                backdrop_paint,
            } => {
                self.maybe_push_transform(&transform);
                // Push an empty layer to isolate the blend.
                self.context.commands.push(Command::PushLayer(None));
                // Evaluate the backdrop paint graph.
                self.load_paint(
                    backdrop_paint.resolve(&self.instance)?,
                    palette_fn,
                    outline_fn,
                    recurse_depth + 1,
                )?;
                // Push a layer with the requested composite mode.
                self.context.commands.push(Command::PushLayer(Some(mode)));
                // Evaluate the source paint graph.
                self.load_paint(
                    source_paint.resolve(&self.instance)?,
                    palette_fn,
                    outline_fn,
                    recurse_depth + 1,
                )?;
                // Pop the composite layer.
                self.context.commands.push(Command::PopLayer);
                // Pop the isolation layer.
                self.context.commands.push(Command::PopLayer);
                self.maybe_pop_transform(&transform);
            }
            ResolvedPaint::Glyph { glyph_id, paint } => {
                self.maybe_push_transform(&transform);
                let path_index = self.context.push_path(glyph_id, outline_fn)?;
                let child_paint = paint.resolve(&self.instance)?;
                let (child_transform, child_paint) =
                    flatten_all_transforms(&self.instance, child_paint)?;
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
        paint: &ResolvedPaint<'a>,
        palette_fn: &impl Fn(u16) -> color::Color,
    ) -> Result<Option<BrushData>, Error> {
        Ok(Some(match paint {
            ResolvedPaint::Solid {
                palette_index,
                alpha,
            } => {
                let mut color = palette_fn(*palette_index);
                let alpha = alpha.to_f64() as f32;
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                BrushData::Solid(color)
            }
            ResolvedPaint::LinearGradient {
                x0,
                y0,
                x1,
                y1,
                x2,
                y2,
                color_stops,
                extend,
            } => {
                let stops = self.push_stops(color_stops, palette_fn)?;
                let p0 = Point::new(x0, y0).map(|x| x.to_f64() as f32);
                let p1 = Point::new(x1, y1).map(|x| x.to_f64() as f32);
                let p2 = Point::new(x2, y2).map(|x| x.to_f64() as f32);
                let perp_to_p2p0 = p2 - p0;
                let perp_to_p2p0 = Point::new(perp_to_p2p0.y, -perp_to_p2p0.x);
                let p3 = p0 + project_point(p1 - p0, perp_to_p2p0);
                let start = p0;
                let end = p3;
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Linear { start, end },
                    extend: *extend,
                    stops,
                })
            }
            ResolvedPaint::RadialGradient {
                x0,
                y0,
                radius0,
                x1,
                y1,
                radius1,
                color_stops,
                extend,
            } => {
                let stops = self.push_stops(color_stops, palette_fn)?;
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Radial {
                        start_center: Point::new(x0, y0).map(|x| x.to_f64() as f32),
                        start_radius: radius0.to_f64() as f32,
                        end_center: Point::new(x1, y1).map(|x| x.to_f64() as f32),
                        end_radius: radius1.to_f64() as f32,
                    },
                    extend: *extend,
                    stops,
                })
            }
            ResolvedPaint::SweepGradient {
                center_x,
                center_y,
                start_angle,
                end_angle,
                color_stops,
                extend,
            } => {
                let stops = self.push_stops(color_stops, palette_fn)?;
                BrushData::Gradient(GradientData {
                    kind: color::GradientKind::Sweep {
                        center: Point::new(center_x, center_y).map(|x| x.to_f64() as f32),
                        start_angle: start_angle.to_f64() as f32,
                        end_angle: end_angle.to_f64() as f32,
                    },
                    extend: *extend,
                    stops,
                })
            }
            _ => return Ok(None),
        }))
    }

    fn push_stops(
        &mut self,
        color_stops: &ColorStops<'a>,
        palette_fn: &impl Fn(u16) -> color::Color,
    ) -> Result<Range<usize>, Error> {
        let start = self.context.stops.len();
        self.context
            .stops
            .extend(color_stops.resolve(&self.instance).map(|stop| {
                let mut color = palette_fn(stop.palette_index);
                let alpha = stop.alpha.to_f64() as f32;
                if alpha != 1.0 {
                    color.a = (color.a as f32 * alpha) as u8;
                }
                color::ColorStop {
                    offset: stop.offset.to_f64() as f32,
                    color,
                }
            }));
        let end = self.context.stops.len();
        Ok(start..end)
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

fn linear_endpoints(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> [Point<f32>; 2] {
    let p0 = Point::new(x0, y0);
    let p1 = Point::new(x1, y1);
    let p2 = Point::new(x2, y2);
    let perp_to_p2p0 = p2 - p0;
    let perp_to_p2p0 = Point::new(perp_to_p2p0.y, -perp_to_p2p0.x);
    let p3 = p0 + project_point(p1 - p0, perp_to_p2p0);
    [p0, p3]
}

fn project_point(a: Point<f32>, b: Point<f32>) -> Point<f32> {
    let b_len = (b.x * b.x + b.y * b.y).sqrt();
    if b_len == 0.0 {
        return Point::default();
    }
    let a_dot_b = a.x * b.x + a.y * b.y;
    let normalized = Point::new(b.x / b_len, b.y / b_len);
    normalized * (a_dot_b / b_len)
}

fn cross_product(a: Point<f32>, b: Point<f32>) -> f32 {
    a.x * b.y - a.y * b.x
}

fn flatten_all_transforms<'a>(
    instance: &ColrInstance<'a>,
    mut paint: ResolvedPaint<'a>,
) -> Result<(Option<Transform>, ResolvedPaint<'a>), ReadError> {
    let mut transform = Transform::IDENTITY;
    let mut has_transform = false;
    while let Some((child_transform, child_paint)) = flatten_one_transform(instance, &paint)? {
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
    instance: &ColrInstance<'a>,
    paint: &ResolvedPaint<'a>,
) -> Result<Option<(Transform, ResolvedPaint<'a>)>, ReadError> {
    fn map_angle(angle: Fixed) -> f32 {
        (angle.to_f64() as f32 * 180.0).to_radians()
    }
    Ok(Some(match paint {
        ResolvedPaint::Transform {
            xx,
            yx,
            xy,
            yy,
            dx,
            dy,
            paint,
        } => {
            let paint = paint.resolve(instance)?;
            (
                Transform {
                    xx: xx.to_f64() as f32,
                    yx: yx.to_f64() as f32,
                    xy: xy.to_f64() as f32,
                    yy: yy.to_f64() as f32,
                    dx: dx.to_f64() as f32,
                    dy: dy.to_f64() as f32,
                },
                paint,
            )
        }
        ResolvedPaint::Translate { dx, dy, paint } => {
            let paint = paint.resolve(instance)?;
            (
                Transform::translate(dx.to_f64() as f32, dy.to_f64() as f32),
                paint,
            )
        }
        ResolvedPaint::Scale {
            scale_x,
            scale_y,
            around_center,
            paint,
        } => (
            Transform::scale(scale_x.to_f64() as f32, scale_y.to_f64() as f32)
                .maybe_around_center(*around_center),
            paint.resolve(instance)?,
        ),
        ResolvedPaint::Rotate {
            angle,
            around_center,
            paint,
        } => (
            Transform::rotate(map_angle(*angle)).maybe_around_center(*around_center),
            paint.resolve(instance)?,
        ),
        ResolvedPaint::Skew {
            x_skew_angle,
            y_skew_angle,
            around_center,
            paint,
        } => (
            Transform::skew(-map_angle(*x_skew_angle), map_angle(*y_skew_angle))
                .maybe_around_center(*around_center),
            paint.resolve(instance)?,
        ),
        _ => return Ok(None),
    }))
}

impl Transform {
    fn maybe_around_center(self, center: Option<Point<Fixed>>) -> Self {
        if let Some(center) = center {
            self.around_center(center.x.to_f64() as f32, center.y.to_f64() as f32)
        } else {
            self
        }
    }
}
