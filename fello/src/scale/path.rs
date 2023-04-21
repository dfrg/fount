//! Compact representation of a vector path.

use super::Pen;
use read_fonts::types::Point;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Verb {
    MoveTo,
    LineTo,
    QuadTo,
    CurveTo,
    Close,
}

pub struct Path<'a> {
    verbs: &'a [Verb],
    points: &'a [Point<f32>],
}

impl<'a> Path<'a> {
    pub(crate) fn new(verbs: &'a [Verb], points: &'a [Point<f32>]) -> Self {
        Self { verbs, points }
    }

    pub fn evaluate(&self, pen: &mut impl Pen) {
        let mut ix = 0;
        for verb in self.verbs {
            match verb {
                Verb::MoveTo => {
                    let to = self.points[ix];
                    ix += 1;
                    pen.move_to(to.x, to.y);
                }
                Verb::LineTo => {
                    let to = self.points[ix];
                    ix += 1;
                    pen.line_to(to.x, to.y);
                }
                Verb::QuadTo => {
                    let control = self.points[ix];
                    let to = self.points[ix + 1];
                    ix += 2;
                    pen.quad_to(control.x, control.y, to.x, to.y);
                }
                Verb::CurveTo => {
                    let control1 = self.points[ix];
                    let control2 = self.points[ix + 1];
                    let to = self.points[ix + 2];
                    ix += 3;
                    pen.curve_to(control1.x, control1.y, control2.x, control2.y, to.x, to.y);
                }
                Verb::Close => {
                    pen.close();
                }
            }
        }
    }
}
