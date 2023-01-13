use read_fonts::TableProvider;

/// Generic type for representing a collection of metadata.
#[derive(Clone, Debug)]
pub struct Sequence<'a, T: SequenceElement<'a>> {
    data: T::Data,
}

impl<'a, T: SequenceElement<'a> + 'a> Sequence<'a, T> {
    /// Creates a new sequence from the specified table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            data: T::Data::from_font(font),
        }
    }

    /// Returns the number of elements in the sequence.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the sequence is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the element in the sequence at the specified index.
    pub fn get(&self, index: usize) -> Option<T> {
        self.data.get(index)
    }

    /// Returns an iterator over the elements in the sequence.
    pub fn iter(&self) -> impl Iterator<Item = T> + 'a + Clone {
        self.clone().into_iter()
    }
}

pub trait SequenceElement<'a>: Clone {
    type Data: SequenceData<'a, Self>;
}

pub trait SequenceData<'a, T>: Clone {
    fn from_font(font: &impl TableProvider<'a>) -> Self;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn get(&self, index: usize) -> Option<T>;
}

#[derive(Clone)]
pub struct Iter<'a, T: SequenceElement<'a>> {
    sequence: Sequence<'a, T>,
    pos: usize,
}

impl<'a, T: SequenceElement<'a> + 'a> Iterator for Iter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.sequence.get(pos)
    }
}

impl<'a, T: SequenceElement<'a> + 'a> IntoIterator for Sequence<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            sequence: self,
            pos: 0,
        }
    }
}
