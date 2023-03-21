use super::math::*;
use crate::scale::Hinting;

use raw::tables::glyf::PointFlags;

pub type Point = super::Point<i32>;

#[derive(Clone, Default, Debug)]
pub struct Storage {
    /// Storage for the interpreter stack.
    pub stack: Vec<i32>,
    /// Twilight zone points.
    pub twilight: Vec<Point>,
    /// Twilight zone tags.
    pub twilight_tags: Vec<PointFlags>,
}

#[derive(Copy, Clone, Debug)]
pub struct InstanceState {
    pub graphics: GraphicsState,
    pub default_graphics: GraphicsState,
    pub ppem: u16,
    pub point_size: i32,
    pub scale: i32,
    pub coord_count: u16,
    pub compat: bool,
    pub mode: Hinting,
}

impl Default for InstanceState {
    fn default() -> Self {
        Self {
            graphics: GraphicsState::default(),
            default_graphics: GraphicsState::default(),
            ppem: 0,
            point_size: 0,
            scale: 0,
            coord_count: 0,
            compat: false,
            mode: Hinting::VerticalSubpixel,
        }
    }
}

impl InstanceState {
    /// Returns true if hinting is enabled for this state.
    pub fn hinting_enabled(&self) -> bool {
        self.graphics.instruct_control & 1 == 0
    }

    /// Returns true if compatibility mode is enabled for this state.
    pub fn compat_enabled(&self) -> bool {
        self.compat
    }
}

#[derive(Copy, Clone, Debug)]
pub struct GraphicsState {
    pub auto_flip: bool,
    pub control_value_cutin: i32,
    pub delta_base: u16,
    pub delta_shift: u16,
    pub instruct_control: u8,
    pub min_distance: i32,
    pub scan_control: bool,
    pub scan_type: i32,
    pub single_width_cutin: i32,
    pub single_width: i32,
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self {
            auto_flip: true,
            control_value_cutin: 68,
            delta_base: 9,
            delta_shift: 3,
            instruct_control: 0,
            min_distance: 64,
            scan_control: false,
            scan_type: 0,
            single_width_cutin: 0,
            single_width: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum CoordMode {
    #[default]
    Both,
    X,
    Y,
}

pub struct ProjectState {
    pub mode: CoordMode,
    pub dual_mode: CoordMode,
    pub pv: Point,
    pub dv: Point,
    pub fv: Point,
    pub fdotp: i32,
    pub move_mode: CoordMode,
}

impl Default for ProjectState {
    fn default() -> Self {
        let vector = Point::new(0x4000, 0);
        Self {
            mode: CoordMode::Both,
            dual_mode: CoordMode::Both,
            pv: vector,
            dv: vector,
            fv: vector,
            fdotp: 0x4000,
            move_mode: CoordMode::Both,
        }
    }
}

impl ProjectState {
    pub fn update(&mut self) {
        if self.fv.x == 0x4000 {
            self.fdotp = self.pv.x as i32;
        } else if self.fv.y == 0x4000 {
            self.fdotp = self.pv.y as i32;
        } else {
            let px = self.pv.x as i32;
            let py = self.pv.y as i32;
            let fx = self.fv.x as i32;
            let fy = self.fv.y as i32;
            self.fdotp = (px * fx + py * fy) >> 14;
        }
        self.mode = CoordMode::Both;
        if self.pv.x == 0x4000 {
            self.mode = CoordMode::X;
        } else if self.pv.y == 0x4000 {
            self.mode = CoordMode::Y;
        }
        self.dual_mode = CoordMode::Both;
        if self.dv.x == 0x4000 {
            self.dual_mode = CoordMode::X;
        } else if self.dv.y == 0x4000 {
            self.dual_mode = CoordMode::Y;
        }
        self.move_mode = CoordMode::Both;
        if self.fdotp == 0x4000 {
            if self.fv.x == 0x4000 {
                self.move_mode = CoordMode::X;
            } else if self.fv.y == 0x4000 {
                self.move_mode = CoordMode::Y;
            }
        }
        if self.fdotp.abs() < 0x400 {
            self.fdotp = 0x4000;
        }
    }

    #[inline(always)]
    pub fn project(&self, v1: Point, v2: Point) -> i32 {
        match self.mode {
            CoordMode::X => v1.x - v2.x,
            CoordMode::Y => v1.y - v2.y,
            CoordMode::Both => {
                let x = v1.x - v2.x;
                let y = v1.y - v2.y;
                dot14(x, y, self.pv.x as i32, self.pv.y as i32)
            }
        }
    }

    #[inline(always)]
    pub fn fast_project(&self, v: Point) -> i32 {
        self.project(v, Point::new(0, 0))
    }

    #[inline(always)]
    pub fn dual_project(&self, v1: Point, v2: Point) -> i32 {
        match self.dual_mode {
            CoordMode::X => v1.x - v2.x,
            CoordMode::Y => v1.y - v2.y,
            CoordMode::Both => {
                let x = v1.x - v2.x;
                let y = v1.y - v2.y;
                dot14(x, y, self.dv.x as i32, self.dv.y as i32)
            }
        }
    }

    #[inline(always)]
    pub fn fast_dual_project(&self, v: Point) -> i32 {
        self.dual_project(v, Point::new(0, 0))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub enum RoundMode {
    HalfGrid,
    #[default]
    Grid,
    DoubleGrid,
    DownToGrid,
    UpToGrid,
    Off,
    Super,
    Super45,
}

pub struct RoundState {
    pub mode: RoundMode,
    pub threshold: i32,
    pub phase: i32,
    pub period: i32,
}

impl Default for RoundState {
    fn default() -> Self {
        Self {
            mode: RoundMode::Grid,
            threshold: 0,
            phase: 0,
            period: 64,
        }
    }
}

impl RoundState {
    pub fn round(&self, distance: i32) -> i32 {
        use RoundMode::*;
        match self.mode {
            HalfGrid => {
                if distance >= 0 {
                    (floor(distance) + 32).max(0)
                } else {
                    (-(floor(-distance) + 32)).min(0)
                }
            }
            Grid => {
                if distance >= 0 {
                    round(distance).max(0)
                } else {
                    (-round(-distance)).min(0)
                }
            }
            DoubleGrid => {
                if distance >= 0 {
                    round_pad(distance, 32).max(0)
                } else {
                    (-round_pad(-distance, 32)).min(0)
                }
            }
            DownToGrid => {
                if distance >= 0 {
                    floor(distance).max(0)
                } else {
                    (-floor(-distance)).min(0)
                }
            }
            UpToGrid => {
                if distance >= 0 {
                    ceil(distance).max(0)
                } else {
                    (-ceil(-distance)).min(0)
                }
            }
            Super => {
                if distance >= 0 {
                    let val =
                        ((distance + (self.threshold - self.phase)) & -self.period) + self.phase;
                    if val < 0 {
                        self.phase
                    } else {
                        val
                    }
                } else {
                    let val =
                        -(((self.threshold - self.phase) - distance) & -self.period) - self.phase;
                    if val > 0 {
                        -self.phase
                    } else {
                        val
                    }
                }
            }
            Super45 => {
                if distance >= 0 {
                    let val = (((distance + (self.threshold - self.phase)) / self.period)
                        * self.period)
                        + self.phase;
                    if val < 0 {
                        self.phase
                    } else {
                        val
                    }
                } else {
                    let val = -((((self.threshold - self.phase) - distance) / self.period)
                        * self.period)
                        - self.phase;
                    if val > 0 {
                        -self.phase
                    } else {
                        val
                    }
                }
            }
            Off => distance,
        }
    }
}
