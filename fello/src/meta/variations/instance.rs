/*! Named instances for variable fonts.

*/

use read_fonts::{
    tables::fvar::{self, Fvar},
    TableProvider,
};

use crate::meta::info_strings::StringId;

/// Named instance of a variation.
#[derive(Clone)]
pub struct Instance<'a> {
    record: fvar::InstanceRecord<'a>,
}

impl<'a> Instance<'a> {
    /// Returns the localized string identifier for the subfamily name of the instance.
    pub fn subfamily_name_id(&self) -> StringId {
        self.record.subfamily_name_id
    }

    /// Returns the string identifier for the PostScript name of the instance.
    pub fn post_script_name_id(&self) -> Option<StringId> {
        self.record.post_script_name_id
    }

    /// Returns an iterator over the sequence of user space coordinates that define
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
pub struct Instances<'a> {
    fvar: Option<Fvar<'a>>,
}

impl<'a> Instances<'a> {
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
    pub fn get(&self, index: usize) -> Option<Instance<'a>> {
        let record = self.fvar.as_ref()?.instances().ok()?.get(index).ok()?;
        Some(Instance { record })
    }

    /// Returns an iterator over the instances in a colletion.
    pub fn iter(&self) -> Iter<'a> {
        self.clone().into_iter()
    }
}

/// Iterator over a collection of named instances.
#[derive(Clone)]
pub struct Iter<'a> {
    instances: Instances<'a>,
    pos: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Instance<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.instances.get(pos)
    }
}

impl<'a> IntoIterator for Instances<'a> {
    type IntoIter = Iter<'a>;
    type Item = Instance<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            instances: self,
            pos: 0,
        }
    }
}
