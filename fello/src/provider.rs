use super::{
    attribute::Attributes,
    charmap::Charmap,
    instance::{LocationRef, NormalizedCoord, Size},
    metrics::{GlyphMetrics, Metrics},
    string::{LocalizedStrings, StringId},
    palette::{Palettes},
    variation::{Axes, NamedInstances},
};

use core::ops::Range;

/// Interface for types that can provide font metadata.
pub trait MetadataProvider<'a>: raw::TableProvider<'a> + Sized {
    /// Returns the primary attributes for font classification-- stretch,
    /// style and weight.
    fn attributes(&self) -> Attributes {
        Attributes::new(self)
    }

    /// Returns the collection of variation axes.
    fn axes(&self) -> Axes<'a> {
        Axes::new(self)
    }

    /// Returns the collection of named variation instances.
    fn named_instances(&self) -> NamedInstances<'a> {
        NamedInstances::new(self)
    }

    /// Returns an iterator over the collection of localized strings for the
    /// given informational string identifier.
    fn localized_strings(&self, id: StringId) -> LocalizedStrings<'a> {
        LocalizedStrings::new(self, id)
    }

    /// Returns the global font metrics for the specified size and location in
    /// normalized variation space.
    fn metrics(&self, size: Size, location: impl Into<LocationRef<'a>>) -> Metrics {
        Metrics::new(self, size, location)
    }

    /// Returns the glyph specific metrics for the specified size and location
    /// in normalized variation space.
    fn glyph_metrics(&self, size: Size, location: impl Into<LocationRef<'a>>) -> GlyphMetrics<'a> {
        GlyphMetrics::new(self, size, location)
    }

    /// Returns the character to nominal glyph identifier mapping.
    fn charmap(&self) -> Charmap<'a> {
        Charmap::new(self)
    }

    /// Returns the collection of color palettes.
    fn palettes(&self) -> Palettes<'a> {
        Palettes::new(self)
    }
}

/// Blanket implementation of `MetadataProvider` for any type that implements
/// `TableProvider`.
impl<'a, T> MetadataProvider<'a> for T where T: raw::TableProvider<'a> {}

pub trait ListData<T: ListElement>: Clone {
    fn len(&self) -> usize;
    fn get(&self, index: usize) -> Option<T>;
}

pub trait ListElement: Sized {
    type Data: ListData<Self>;
}

#[derive(Clone)]
pub struct List<T: ListElement> {
    pub(crate) data: T::Data,
}

impl<T: ListElement> List<T> {
    /// Returns the number of elements in the collection.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.data.len() == 0
    }

    /// Returns the element at the specifed index.
    pub fn get(&self, index: usize) -> Option<T> {
        self.data.get(index)
    }

    /// Returns an iterator over the elements in the collection.
    pub fn iter(&self) -> ListIter<T> {
        ListIter {
            data: self.data.clone(),
            iter: 0..self.data.len()
        }
    }
}

#[derive(Clone)]
pub struct ListIter<T: ListElement> {
    data: T::Data,
    iter: Range<usize>,
}

impl<T: ListElement> Iterator for ListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let ix = self.iter.next()?;
        self.data.get(ix)
    }
}

impl<T: ListElement> ExactSizeIterator for ListIter<T> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<T: ListElement> DoubleEndedIterator for ListIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ix = self.iter.next_back()?;
        self.data.get(ix)
    }
}

impl<T: ListElement> IntoIterator for List<T> {
    type IntoIter = ListIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
