//! Color outlines loaded from the `COLR` table.

use read_fonts::{tables::colr::*, types::Point, ReadError};

pub use read_fonts::tables::{colr::Colr, cpal::Cpal};

use super::{color, path, Error, Pen};
use crate::scale::color::ColorPen;
use crate::GlyphId;
use color::Transform;

use std::collections::{HashMap, HashSet};
use std::ops::Range;

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
    pub fn load(
        &mut self,
        glyph_id: GlyphId,
        mut outline_scaler: impl FnMut(GlyphId, &mut PathReader) -> Result<(), Error>,
        pen: &mut impl color::ColorPen,
    ) -> Result<(), Error> {
        self.brushes.clear();
        self.stops.clear();
        self.paths.clear();
        self.verbs.clear();
        self.points.clear();
        self.path_cache.clear();
        self.visited.clear();
        self.commands.clear();
        self.load_path(glyph_id, &mut outline_scaler)?;
        Ok(())
    }

    fn load_path(
        &mut self,
        glyph_id: GlyphId,
        outline_scaler: &mut impl FnMut(GlyphId, &mut PathReader) -> Result<(), Error>,
    ) -> Result<PathIndex, Error> {
        if let Some(path_index) = self.path_cache.get(&glyph_id) {
            return Ok(*path_index);
        }
        let verb_start = self.verbs.len();
        let point_start = self.points.len();
        let mut path_reader = PathReader {
            verbs: &mut self.verbs,
            points: &mut self.points,
        };
        let res = outline_scaler(GlyphId::new(0), &mut path_reader)?;
        let path_index = self.paths.len();
        self.paths.push(PathData {
            glyph_id,
            verbs: verb_start..self.verbs.len(),
            points: point_start..self.points.len(),
        });
        self.path_cache.insert(glyph_id, path_index);
        Ok(path_index)
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

    fn path(&self, index: PathIndex) -> Option<path::Path> {
        let path_data = self.paths.get(index)?;
        Some(path::Path::new(
            self.verbs.get(path_data.verbs.clone())?,
            self.points.get(path_data.points.clone())?,
        ))
    }
}

pub struct PathReader<'a> {
    verbs: &'a mut Vec<path::Verb>,
    points: &'a mut Vec<Point<f32>>,
}

impl Pen for PathReader<'_> {
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

pub type PathIndex = usize;
pub type BrushIndex = usize;

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
    mut paint: Paint<'a>,
) -> Result<(Option<Transform>, Paint<'a>), ReadError> {
    let mut transform = Transform::IDENTITY;
    let mut has_transform = false;
    while let Some((child_transform, child_paint)) = flatten_one_transform(&paint)? {
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
        Paint::VarTransform(transform) => (Transform::default(), transform.paint()?),
        Paint::Translate(transform) => (
            Transform::translate(
                transform.dx().to_i16() as f32,
                transform.dy().to_i16() as f32,
            ),
            transform.paint()?,
        ),
        Paint::VarTranslate(transform) => (Transform::default(), transform.paint()?),
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
        Paint::VarScale(transform) => (Transform::default(), transform.paint()?),
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
        Paint::VarScaleUniform(transform) => (Transform::default(), transform.paint()?),
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
