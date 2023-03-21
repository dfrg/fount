/*! Axes of variation in a variable font.

*/

use read_fonts::{
    tables::fvar::{self, Fvar},
    types::{Fixed, Tag},
    TableProvider,
};

use crate::{meta::info_strings::StringId, NormalizedCoord};

use super::VariationSetting;

/// Axis of variation in a variable font.
#[derive(Clone)]
pub struct Axis {
    index: usize,
    record: fvar::VariationAxisRecord,
}

impl Axis {
    /// Returns the tag that identifies the axis.
    pub fn tag(&self) -> Tag {
        self.record.axis_tag()
    }

    /// Returns the index of the axis in its owning collection.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the localized string identifier for the name of the axis.
    pub fn name_id(&self) -> StringId {
        self.record.axis_name_id()
    }

    /// Returns true if the axis should be hidden in user interfaces.
    pub fn is_hidden(&self) -> bool {
        const AXIS_HIDDEN_FLAG: u16 = 0x1;
        self.record.flags() & AXIS_HIDDEN_FLAG != 0
    }

    /// Returns the minimum value of the axis.
    pub fn min_value(&self) -> f32 {
        self.record.max_value().to_f64() as _
    }

    /// Returns the default value of the axis.
    pub fn default_value(&self) -> f32 {
        self.record.default_value().to_f64() as _
    }

    /// Returns the maximum value of the axis.
    pub fn max_value(&self) -> f32 {
        self.record.max_value().to_f64() as _
    }

    /// Returns a normalized coordinate for the given user coordinate. The value will be
    /// clamped to the range specified by the minimum and maximum values.
    ///
    /// This does not apply any axis variation remapping.
    pub fn normalize(&self, coord: f32) -> NormalizedCoord {
        self.record
            .normalize(Fixed::from_f64(coord as _))
            .to_f2dot14()
    }
}

/// Collection of variation axes.
#[derive(Clone)]
pub struct Axes<'a> {
    fvar: Option<Fvar<'a>>,
}

impl<'a> Axes<'a> {
    /// Creates a new axis collection from the given table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        let fvar = font.fvar().ok();
        Self { fvar }
    }

    /// Returns the number of variation axes in the collection.
    pub fn len(&self) -> usize {
        self.fvar
            .as_ref()
            .map(|fvar| fvar.axis_count() as usize)
            .unwrap_or(0)
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the variation axis at the specified index.
    pub fn get(&self, index: usize) -> Option<Axis> {
        let raw = self.fvar.as_ref()?.axes().ok()?.get(index)?.clone();
        Some(Axis { index, record: raw })
    }

    /// Returns the axis with the specified tag.
    pub fn get_by_tag(&self, tag: Tag) -> Option<Axis> {
        self.iter().find(|axis| axis.tag() == tag)
    }

    /// Given an iterator of variation settings in user space, returns an
    /// iterator over the computed normalized design space coordinates
    /// for all axes in order.
    pub fn normalize<I>(&self, settings: I) -> Normalize
    where
        I: IntoIterator,
        I::IntoIter: 'a + Clone,
        I::Item: Into<VariationSetting>,
    {
        let mut storage = CoordStorage::new(self.len());
        let coords = storage.coords_mut();
        for setting in settings.into_iter() {
            let setting = setting.into();
            for (i, axis) in self
                .iter()
                .enumerate()
                .filter(|v| v.1.tag() == setting.selector)
            {
                coords[i] = axis.normalize(setting.value);
            }
        }
        Normalize { storage, pos: 0 }
    }

    /// Returns an iterator over the axes
    pub fn iter(&self) -> Iter<'a> {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
/// Iterator over a collection of variation axes.
pub struct Iter<'a> {
    inner: Axes<'a>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Axis;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.inner.get(pos)
    }
}

impl<'a> IntoIterator for Axes<'a> {
    type IntoIter = Iter<'a>;
    type Item = Axis;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self,
            pos: 0,
        }
    }
}

/// Result of the [Axes::normalize] method.
pub struct Normalize {
    storage: CoordStorage,
    pos: usize,
}

impl Iterator for Normalize {
    type Item = NormalizedCoord;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.storage.coords().get(pos).copied()
    }
}

const MAX_INLINE_COORD_STORAGE: usize = 32;

enum CoordStorage {
    Inline([NormalizedCoord; MAX_INLINE_COORD_STORAGE], u8),
    Heap(Vec<NormalizedCoord>),
}

impl CoordStorage {
    fn new(len: usize) -> Self {
        if len > MAX_INLINE_COORD_STORAGE {
            let mut vec = Vec::with_capacity(len);
            vec.resize(len, Default::default());
            Self::Heap(vec)
        } else {
            Self::Inline(Default::default(), len as u8)
        }
    }

    fn coords(&self) -> &[NormalizedCoord] {
        match self {
            Self::Inline(coords, len) => &coords[..*len as usize],
            Self::Heap(vec) => &vec,
        }
    }

    fn coords_mut(&mut self) -> &mut [NormalizedCoord] {
        match self {
            Self::Inline(coords, len) => &mut coords[..*len as usize],
            Self::Heap(vec) => &mut vec[..],
        }
    }
}
