//! Collections of colors referenced by color outlines.

use crate::provider::{List, ListData, ListElement, ListIter};
use read_fonts::tables::cpal::{ColorRecord, Cpal};
use read_fonts::TableProvider;

/// 32-bit RGBA color value.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default, Debug)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone)]
pub struct Palettes<'a> {
    cpal: Option<Cpal<'a>>,
}

impl<'a> Palettes<'a> {
    /// Creates a palette collection for the given font.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            cpal: font.cpal().ok(),
        }
    }

    /// Returns true if there are no available palettes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of available palettes.
    pub fn len(&self) -> usize {
        self.cpal
            .as_ref()
            .map(|cpal| cpal.num_palettes() as usize)
            .unwrap_or_default()
    }

    /// Returns the palette at the specified index.
    pub fn get(&self, index: usize) -> Option<Palette<'a>> {
        let cpal = self.cpal.clone()?;
        let records = cpal.color_records_array()?.ok()?;
        if index >= cpal.num_palettes() as usize {
            return None;
        }
        Some(Palette {
            cpal,
            index,
            records,
        })
    }

    /// Returns an iterator over the collection of palettes.
    pub fn iter(&self) -> impl Iterator<Item = Palette<'a>> + 'a {
        let copy = self.clone();
        (0..self.len()).filter_map(move |ix| copy.get(ix))
    }
}

#[derive(Clone)]
pub struct Palette<'a> {
    cpal: Cpal<'a>,
    index: usize,
    records: &'a [ColorRecord],
}

impl<'a> Palette<'a> {
    /// Returns the number of color entries in the palette.
    pub fn len(&self) -> usize {
        self.cpal.num_palette_entries() as usize
    }

    /// Returns true if the palette is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the color at the given index.
    pub fn get(&self, index: usize) -> Option<Color> {
        let len = self.len();
        if index >= len {
            return None;
        }
        let ix = len * self.index + index;
        self.records.get(ix).map(|color| Color {
            r: color.red(),
            g: color.green(),
            b: color.blue(),
            a: color.alpha(),
        })
    }

    /// Returns a function that wraps this palette.
    pub fn as_fn(&self, foreground: Color) -> impl Fn(u16) -> Option<Color> + 'a {
        let copy = self.clone();
        move |ix| {
            if ix == 0xFFFF {
                Some(foreground)
            } else {
                copy.get(ix as usize)
            }
        }
    }
}
