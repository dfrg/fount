use read_fonts::types::F2Dot14;

use std::sync::Arc;

pub type MasterLocations = Vec<Vec<F2Dot14>>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MasterDeltas(pub(super) Arc<[i32]>);

impl MasterDeltas {
    pub fn deltas(&self) -> &[i32] {
        &self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = i32> + Clone + '_ {
        self.deltas().iter().copied()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Value {
    pub default: i16,
    /// Values for each master
    pub deltas: Option<MasterDeltas>,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.default)?;
        if let Some(deltas) = &self.deltas {
            write!(f, "{{")?;
            for (i, delta) in deltas.iter().enumerate() {
                let val = self.default as i32 + delta;
                if i > 0 {
                    write!(f, ",")?;
                }
                write!(f, "{val}")?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Value {
            default: value,
            deltas: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Anchor {
    pub x: Value,
    pub y: Value,
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@({},{})", self.x, self.y)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct Adjustment {
    pub x: Option<Value>,
    pub y: Option<Value>,
    pub x_advance: Option<Value>,
    pub y_advance: Option<Value>,
}

impl std::fmt::Display for Adjustment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut write_val = |opt_val: &Option<Value>, tail: &str| {
            if let Some(val) = opt_val {
                write!(f, "{}{}", val, tail)
            } else {
                write!(f, "-{}", tail)
            }
        };
        write_val(&self.x, " ")?;
        write_val(&self.y, " ")?;
        write_val(&self.x_advance, " ")?;
        write_val(&self.y_advance, ")")
    }
}
