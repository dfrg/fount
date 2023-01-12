use read_fonts::{
    tables::{
        avar::Avar,
        fvar::{self, Fvar},
    },
    types::{F2Dot14, Fixed, Scalar, Tag},
    TableProvider,
};

use crate::{LocalizedStringId, Setting};

/// Type for a normalized variation coordinate.
pub type NormalizedCoord = F2Dot14;

/// Variation axis.
#[derive(Clone)]
pub struct Axis<'a> {
    index: usize,
    record: fvar::VariationAxisRecord,
    avar: Option<Avar<'a>>,
}

impl<'a> Axis<'a> {
    /// Returns the tag that identifies the axis.
    pub fn tag(&self) -> Tag {
        self.record.axis_tag()
    }

    /// Returns the index of the axis in its owning collection.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the string identifier for the axis name.
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

    /// Returns the inner variation axis record.
    pub fn raw(&self) -> &fvar::VariationAxisRecord {
        &self.record
    }
}

/// Collection of variation axes.
#[derive(Clone)]
pub struct AxisCollection<'a> {
    fvar: Option<Fvar<'a>>,
    avar: Option<Avar<'a>>,
}

impl<'a> AxisCollection<'a> {
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
    pub fn get(&self, index: usize) -> Option<Axis<'a>> {
        let raw = self.fvar.as_ref()?.axes().ok()?.get(index)?.clone();
        Some(Axis {
            index,
            record: raw,
            avar: self.avar.clone(),
        })
    }

    /// Returns the axis with the specified tag.
    pub fn get_by_tag(&self, tag: Tag) -> Option<Axis<'a>> {
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
    pub fn iter(&self) -> impl Iterator<Item = Axis<'a>> + 'a + Clone {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
/// Iterator over a collection of variation axes.
pub struct AxisIter<'a> {
    inner: AxisCollection<'a>,
    pos: usize,
}

impl<'a> Iterator for AxisIter<'a> {
    type Item = Axis<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.inner.get(pos)
    }
}

impl<'a> IntoIterator for AxisCollection<'a> {
    type IntoIter = AxisIter<'a>;
    type Item = Axis<'a>;

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
    /// Returns the string identifier for the instance name.
    pub fn name_id(&self) -> LocalizedStringId {
        LocalizedStringId(self.record.subfamily_name_id)
    }

    /// Returns the string identifier for the instance PostScript name.
    pub fn post_script_name_id(&self) -> Option<LocalizedStringId> {
        self.record.post_script_name_id.map(LocalizedStringId)
    }

    /// Returns an iterator over the sequence of user coordinates that define
    /// the instance.
    pub fn coordinates(&self) -> impl Iterator<Item = f32> + 'a + Clone {
        self.record
            .coordinates
            .iter()
            .map(|coord| coord.get().to_f64() as _)
    }

    /// Returns the inner instance record.
    pub fn raw(&self) -> &fvar::InstanceRecord<'a> {
        &self.record
    }
}

/// Collection of named variation instances.
#[derive(Clone)]
pub struct NamedInstanceCollection<'a> {
    fvar: Option<Fvar<'a>>,
}

impl<'a> NamedInstanceCollection<'a> {
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
pub struct InstanceIter<'a> {
    instances: NamedInstanceCollection<'a>,
    pos: usize,
}

impl<'a> Iterator for InstanceIter<'a> {
    type Item = NamedInstance<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.instances.get(pos)
    }
}

impl<'a> IntoIterator for NamedInstanceCollection<'a> {
    type IntoIter = InstanceIter<'a>;
    type Item = NamedInstance<'a>;

    fn into_iter(self) -> Self::IntoIter {
        InstanceIter {
            instances: self,
            pos: 0,
        }
    }
}
