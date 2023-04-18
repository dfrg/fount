use read_fonts::types::Fixed;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub xx: f32,
    pub yx: f32,
    pub xy: f32,
    pub yy: f32,
    pub dx: f32,
    pub dy: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self::new(&[1., 0., 0., 1., 0., 0.]);

    pub const fn new(elements: &[f32; 6]) -> Self {
        Self {
            xx: elements[0],
            yx: elements[1],
            xy: elements[2],
            yy: elements[3],
            dx: elements[4],
            dy: elements[5],
        }
    }
    pub fn scale(x: f32, y: f32) -> Self {
        Self::new(&[x, 0., 0., y, 0., 0.])
    }

    pub fn translate(x: f32, y: f32) -> Self {
        Self::new(&[1., 0., 0., 1., x, y])
    }

    pub fn rotate(th: f32) -> Self {
        let (s, c) = th.sin_cos();
        Self::new(&[c, s, -s, c, 0., 0.])
    }

    pub fn skew(x: f32, y: f32) -> Self {
        Self::new(&[1., y.tan(), x.tan(), 1., 0., 0.])
    }

    pub fn around_center(&self, x: f32, y: f32) -> Self {
        Self::translate(x, y) * *self * Self::translate(-x, -y)
    }

    // pub fn transform_point(&self, point: &Point) -> Point {
    //     Point {
    //         x: point.x * self.xx + point.y * self.yx + self.dx,
    //         y: point.y * self.yy + point.y * self.xy + self.dy,
    //     }
    // }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(&[
            self.xx * other.xx + self.xy * other.yx,
            self.yx * other.xx + self.yy * other.yx,
            self.xx * other.xy + self.xy * other.yy,
            self.yx * other.xy + self.yy * other.yy,
            self.xx * other.dx + self.xy * other.dy + self.dx,
            self.yx * other.dx + self.yy * other.dy + self.dy,
        ])
    }
}
