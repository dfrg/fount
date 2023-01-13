use read_fonts::{
    tables::{
        avar::Avar,
        fvar::{self, Fvar},
    },
    types::{F2Dot14, Fixed, Scalar, Tag},
    TableProvider,
};

use crate::sequence::{Sequence, SequenceData, SequenceElement};
use crate::{LocalizedStringId, Setting};

/// Type for a normalized variation coordinate.
pub type NormalizedCoord = F2Dot14;

/// Variation axis.
#[derive(Clone)]
pub struct VariationAxis<'a> {
    index: usize,
    record: fvar::VariationAxisRecord,
    avar: Option<Avar<'a>>,
}

impl<'a> VariationAxis<'a> {
    /// Returns the tag that identifies the axis.
    pub fn tag(&self) -> Tag {
        self.record.axis_tag()
    }

    /// Returns the index of the axis in its owning collection.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the localized string identifier for the name of the axis.
    pub fn name_id(&self) -> LocalizedStringId {
        LocalizedStringId(self.record.axis_name_id())
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
    pub fn normalize(&self, coord: f32) -> NormalizedCoord {
        let mut normalized = self.record.normalize(Fixed::from_f64(coord as _));
        if let Some(Ok(segment_map)) = self
            .avar
            .as_ref()
            .and_then(|avar| avar.axis_segment_maps().get(self.index))
        {
            normalized = segment_map.apply(normalized);
        }
        let bits = i32::from_be_bytes(normalized.to_be_bytes());
        NormalizedCoord::from_raw((((bits + 2) >> 2) as i16).to_be_bytes())
    }
}

impl<'a> SequenceElement<'a> for VariationAxis<'a> {
    type Data = VariationAxisSequence<'a>;
}

#[derive(Clone)]
pub struct VariationAxisSequence<'a> {
    fvar: Option<Fvar<'a>>,
    avar: Option<Avar<'a>>,
}

impl<'a> SequenceData<'a, VariationAxis<'a>> for VariationAxisSequence<'a> {
    fn from_font(font: &impl TableProvider<'a>) -> Self {
        let fvar = font.fvar().ok();
        let avar = font.avar().ok();
        Self { fvar, avar }
    }

    fn len(&self) -> usize {
        self.fvar
            .as_ref()
            .map(|fvar| fvar.axis_count() as usize)
            .unwrap_or(0)
    }

    fn get(&self, index: usize) -> Option<VariationAxis<'a>> {
        let record = self.fvar.as_ref()?.axes().ok()?.get(index)?.clone();
        Some(VariationAxis {
            index,
            record,
            avar: self.avar.clone(),
        })
    }
}

impl<'a> Sequence<'a, VariationAxis<'a>> {
    /// Returns the axis with the specified tag.
    pub fn get_by_tag(&self, tag: Tag) -> Option<VariationAxis<'a>> {
        self.iter().find(|axis| axis.tag() == tag)
    }

    /// Returns an iterator over pairs of axis index and normalized coordinate
    /// for the specified sequence of variation settings.
    pub fn normalize<I>(
        &self,
        variations: I,
    ) -> impl Iterator<Item = (usize, NormalizedCoord)> + 'a + Clone
    where
        I: IntoIterator,
        I::IntoIter: 'a + Clone,
        I::Item: Into<Setting<f32>>,
    {
        let copy = self.clone();
        variations.into_iter().filter_map(move |setting| {
            let setting = setting.into();
            let axis = copy.get_by_tag(setting.selector)?;
            Some((axis.index(), axis.normalize(setting.value)))
        })
    }
}

/// Collection of variation axes.
#[derive(Clone)]
pub struct VariationAxes<'a> {
    fvar: Option<Fvar<'a>>,
    avar: Option<Avar<'a>>,
}

impl<'a> VariationAxes<'a> {
    /// Creates a new axis collection from the given table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        let fvar = font.fvar().ok();
        let avar = font.avar().ok();
        Self { fvar, avar }
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
    pub fn get(&self, index: usize) -> Option<VariationAxis<'a>> {
        let raw = self.fvar.as_ref()?.axes().ok()?.get(index)?.clone();
        Some(VariationAxis {
            index,
            record: raw,
            avar: self.avar.clone(),
        })
    }

    /// Returns the axis with the specified tag.
    pub fn get_by_tag(&self, tag: Tag) -> Option<VariationAxis<'a>> {
        self.iter().find(|axis| axis.tag() == tag)
    }

    /// Returns an iterator over pairs of axis index and normalized coordinate
    /// for the specified sequence of variation settings.
    pub fn normalize<I>(
        &self,
        variations: I,
    ) -> impl Iterator<Item = (usize, NormalizedCoord)> + 'a + Clone
    where
        I: IntoIterator,
        I::IntoIter: 'a + Clone,
        I::Item: Into<Setting<f32>>,
    {
        let copy = self.clone();
        variations.into_iter().filter_map(move |setting| {
            let setting = setting.into();
            let axis = copy.get_by_tag(setting.selector)?;
            Some((axis.index(), axis.normalize(setting.value)))
        })
    }

    /// Returns an iterator over the axes
    pub fn iter(&self) -> impl Iterator<Item = VariationAxis<'a>> + 'a + Clone {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
/// Iterator over a collection of variation axes.
pub struct AxisIter<'a> {
    inner: VariationAxes<'a>,
    pos: usize,
}

impl<'a> Iterator for AxisIter<'a> {
    type Item = VariationAxis<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.inner.get(pos)
    }
}

impl<'a> IntoIterator for VariationAxes<'a> {
    type IntoIter = AxisIter<'a>;
    type Item = VariationAxis<'a>;

    fn into_iter(self) -> Self::IntoIter {
        AxisIter {
            inner: self,
            pos: 0,
        }
    }
}

/// Named instance of a variation.
#[derive(Clone)]
pub struct NamedInstance<'a> {
    record: fvar::InstanceRecord<'a>,
}

impl<'a> NamedInstance<'a> {
    /// Returns the localized string identifier for the subfamily name of the instance.
    pub fn subfamily_name_id(&self) -> LocalizedStringId {
        LocalizedStringId(self.record.subfamily_name_id)
    }

    /// Returns the string identifier for the PostScript name of the instance.
    pub fn post_script_name_id(&self) -> Option<LocalizedStringId> {
        self.record.post_script_name_id.map(LocalizedStringId)
    }

    /// Returns an iterator over the sequence of unnormalized user space coordinates that define
    /// the instance, one coordinate per axis.
    pub fn coords(&self) -> impl Iterator<Item = f32> + 'a + Clone {
        self.record
            .coordinates
            .iter()
            .map(|coord| coord.get().to_f64() as _)
    }
}

/// Collection of named variation instances.
#[derive(Clone)]
pub struct NamedInstances<'a> {
    fvar: Option<Fvar<'a>>,
}

impl<'a> NamedInstances<'a> {
    /// Creates a new instance collection from the given table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            fvar: font.fvar().ok(),
        }
    }

    /// Returns the number of instances in the collection.
    pub fn len(&self) -> usize {
        self.fvar
            .as_ref()
            .map(|fvar| fvar.instance_count() as usize)
            .unwrap_or(0)
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the instance at the specified index.
    pub fn get(&self, index: usize) -> Option<NamedInstance<'a>> {
        let record = self.fvar.as_ref()?.instances().ok()?.get(index).ok()?;
        Some(NamedInstance { record })
    }

    /// Returns an iterator over the instances in a colletion.
    pub fn iter(&self) -> impl Iterator<Item = NamedInstance<'a>> + 'a + Clone {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
pub struct NamedInstanceIter<'a> {
    instances: NamedInstances<'a>,
    pos: usize,
}

impl<'a> Iterator for NamedInstanceIter<'a> {
    type Item = NamedInstance<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.instances.get(pos)
    }
}

impl<'a> IntoIterator for NamedInstances<'a> {
    type IntoIter = NamedInstanceIter<'a>;
    type Item = NamedInstance<'a>;

    fn into_iter(self) -> Self::IntoIter {
        NamedInstanceIter {
            instances: self,
            pos: 0,
        }
    }
}
