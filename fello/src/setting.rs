/*! Generic type for selecting features and variations.

*/

use read_fonts::types::Tag;

use core::str::FromStr;

/// Setting defined by a selector tag and an associated value.
///
/// The general structure of (tag, value) pairs is used to specify both
/// variations and features.
///
/// In the case of variations, the selector tag chooses a variation axis
/// and the value defines the position on that axis in user space
/// coordinates.
///
/// For features, the selector specifies a
/// [feature tag](https://learn.microsoft.com/en-us/typography/opentype/spec/featuretags)
/// and the value can have one of two meanings. Most features, such as `liga`, can be toggled,
/// and are enabled or disabled by a non-zero or zero value, respectively. Features like
/// `aalt` use the value as an index to select an alternate glyph from a set.  
#[derive(Copy, Clone, Debug)]
pub struct Setting<T> {
    /// Tag that specifies the target setting.
    pub selector: Tag,
    /// The desired value for the setting.
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
