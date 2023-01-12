use read_fonts::types::Tag;

use core::str::FromStr;

/// Setting defined by a tag selector and a value.
#[derive(Copy, Clone, Debug)]
pub struct Setting<T> {
    /// Tag that specifies the target setting.
    pub selector: Tag,
    /// Value for the variation.
    pub value: T,
}

impl<T> From<(Tag, T)> for Setting<T> {
    fn from(s: (Tag, T)) -> Self {
        Self {
            selector: s.0,
            value: s.1,
        }
    }
}

impl<T> From<(&str, T)> for Setting<T> {
    fn from(s: (&str, T)) -> Self {
        Self {
            selector: Tag::from_str(s.0).unwrap_or_default(),
            value: s.1,
        }
    }
}

impl<T> From<([u8; 4], T)> for Setting<T> {
    fn from(s: ([u8; 4], T)) -> Self {
        Self {
            selector: Tag::new_checked(&s.0[..]).unwrap_or_default(),
            value: s.1,
        }
    }
}
