//! Representation of color outlines.

use read_fonts::types::{GlyphId, Point};

pub use read_fonts::tables::colr::{CompositeMode, Extend};

pub use super::path::Path;
pub use super::transform::Transform;

/// 32-bit RGBA color value.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Debug)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Offset and color of a transition point in a gradient.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
#[repr(C)]
pub struct ColorStop {
    pub offset: f32,
    pub color: Color,
}

/// Properties for the supported gradient types.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GradientKind {
    /// Gradient that transitions between two or more colors along a line.
    Linear {
        /// Starting point.
        start: Point<f32>,
        /// Ending point.
        end: Point<f32>,
    },
    /// Gradient that transitions between two or more colors that radiate from an origin.
    Radial {
        /// Center of start circle.
        start_center: Point<f32>,
        /// Radius of start circle.
        start_radius: f32,
        /// Center of end circle.
        end_center: Point<f32>,
        /// Radius of end circle.
        end_radius: f32,
    },
    /// Gradient that transitions between two or more colors that rotate around a center
    /// point.
    Sweep {
        /// Center point.
        center: Point<f32>,
        /// Start angle of the sweep, counter-clockwise of the x-axis.
        start_angle: f32,
        /// End angle of the sweep, counter-clockwise of the x-axis.
        end_angle: f32,
    },
}

/// Definition of a gradient that transitions between two or more colors.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Gradient<'a> {
    /// Kind and properties of the gradient.
    pub kind: GradientKind,
    /// Extend mode.
    pub extend: Extend,
    /// Color stop collection.
    pub stops: &'a [ColorStop],
}

#[derive(Copy, Clone, Debug)]
pub enum Brush<'a> {
    Solid(Color),
    Gradient(Gradient<'a>),
}

/// Interface for processing a color outline.
pub trait ColorPen {
    fn push_transform(&mut self, transform: Transform);
    fn pop_transform(&mut self);
    fn push_clip(&mut self, glyph_id: GlyphId, path: &Path);
    fn pop_clip(&mut self);
    fn push_layer(&mut self, mode: Option<CompositeMode>);
    fn pop_layer(&mut self);
    fn fill(&mut self, brush: &Brush, brush_transform: Option<Transform>);
    fn fill_path(
        &mut self,
        glyph_id: GlyphId,
        path: &Path,
        brush: &Brush,
        brush_transform: Option<Transform>,
    );
}
