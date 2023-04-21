//! Named instances for variable fonts.

use read_fonts::{
    tables::fvar::{self, Fvar},
    TableProvider,
};

use crate::{
    instance::{Location, NormalizedCoord},
    string::StringId,
};

use super::Axes;

/// Named instance of a variation.
#[derive(Clone)]
pub struct NamedInstance<'a> {
    axes: Axes<'a>,
    record: fvar::InstanceRecord<'a>,
}

impl<'a> NamedInstance<'a> {
    /// Returns the localized string identifier for the subfamily name of the instance.
    pub fn subfamily_name_id(&self) -> StringId {
        self.record.subfamily_name_id
    }

    /// Returns the string identifier for the PostScript name of the instance.
    pub fn postscript_name_id(&self) -> Option<StringId> {
        self.record.post_script_name_id
    }

    /// Returns an iterator over the sequence of user space coordinates that define
    /// the instance, one coordinate per axis.
    pub fn user_coords(&self) -> impl Iterator<Item = f32> + 'a + Clone {
        self.record
            .coordinates
            .iter()
            .map(|coord| coord.get().to_f64() as _)
    }

    /// Computes a location in normalized variation space for this instance.
    pub fn location(&self) -> Location {
        let mut location = Location::new(self.axes.len());
        self.location_to_slice(location.coords_mut());
        location
    }

    /// Computes a location in normalized variation space for this instance and
    /// stores the result in the given slice.
    pub fn location_to_slice(&self, location: &mut [NormalizedCoord]) {
        let settings = self
            .axes
            .iter()
            .map(|axis| axis.tag())
            .zip(self.user_coords());
        self.axes.location_to_slice(settings, location);
    }
}

/// Collection of named variation instances.
#[derive(Clone)]
pub struct NamedInstances<'a> {
    axes: Axes<'a>,
}

impl<'a> NamedInstances<'a> {
    /// Creates a new instance collection from the given table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            axes: Axes::new(font),
        }
    }

    /// Returns the number of instances in the collection.
    pub fn len(&self) -> usize {
        self.axes
            .fvar
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
        let record = self.axes.fvar.as_ref()?.instances().ok()?.get(index).ok()?;
        Some(NamedInstance {
            axes: self.axes.clone(),
            record,
        })
    }

    /// Returns an iterator over the instances in a colletion.
    pub fn iter(&self) -> Iter<'a> {
        self.clone().into_iter()
    }
}

/// Iterator over a collection of named instances.
#[derive(Clone)]
pub struct Iter<'a> {
    instances: NamedInstances<'a>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = NamedInstance<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.instances.get(pos)
    }
}

impl<'a> IntoIterator for NamedInstances<'a> {
    type IntoIter = Iter<'a>;
    type Item = NamedInstance<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            instances: self,
            pos: 0,
        }
    }
}
