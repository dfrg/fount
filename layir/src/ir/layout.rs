use super::super::NameMap;
use core::fmt;
use read_fonts::tables::layout::{ClassDef, CoverageTable};
use read_fonts::types::GlyphId;
use std::collections::{BTreeSet, HashMap};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Default)]
pub struct LayoutBuilder {
    pub glyph_lists: Vec<GlyphList>,
    pub glyph_list_map: HashMap<Arc<Vec<GlyphId>>, usize>,
    pub glyph_sets: Vec<GlyphSet>,
    pub glyph_set_map: HashMap<Arc<BTreeSet<GlyphId>>, usize>,
}

impl LayoutBuilder {
    pub fn glyph_list(&mut self, glyphs: &Vec<GlyphId>) -> GlyphList {
        if let Some(index) = self.glyph_list_map.get(glyphs) {
            self.glyph_lists.get(*index).cloned().unwrap()
        } else {
            let index = self.glyph_lists.len();
            let glyphs = Arc::new(glyphs.clone());
            let list = GlyphList {
                index,
                glyphs: glyphs.clone(),
            };
            self.glyph_lists.push(list.clone());
            self.glyph_list_map.insert(glyphs, index);
            list
        }
    }

    pub fn glyph_set(&mut self, glyphs: &BTreeSet<GlyphId>) -> GlyphSet {
        if let Some(index) = self.glyph_set_map.get(glyphs) {
            self.glyph_sets.get(*index).cloned().unwrap()
        } else {
            let index = self.glyph_sets.len();
            let glyphs = Arc::new(glyphs.clone());
            let set = GlyphSet {
                index,
                glyphs: glyphs.clone(),
            };
            self.glyph_sets.push(set.clone());
            self.glyph_set_map.insert(glyphs, index);
            set
        }
    }

    pub fn glyph_set_from_coverage(&mut self, coverage: &CoverageTable) -> GlyphSet {
        let set: BTreeSet<_> = coverage.iter().collect();
        self.glyph_set(&set)
    }

    pub fn glyph_sets_from_class_def(&mut self, class_def: &ClassDef) -> HashMap<u16, GlyphSet> {
        let mut sets: HashMap<u16, BTreeSet<GlyphId>> = Default::default();
        for (glyph, class) in class_def.iter() {
            sets.entry(class).or_default().insert(glyph);
        }
        sets.into_iter()
            .map(|(k, v)| (k, self.glyph_set(&v)))
            .collect()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphSet {
    pub index: usize,
    pub glyphs: Arc<BTreeSet<GlyphId>>,
}

impl Deref for GlyphSet {
    type Target = BTreeSet<GlyphId>;

    fn deref(&self) -> &Self::Target {
        &*self.glyphs
    }
}

impl super::PrettyPrint for GlyphSet {
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &NameMap) -> fmt::Result {
        write!(f, "[")?;
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", name_map.get(*glyph))?;
        }
        write!(f, "]")
    }
}

impl GlyphSet {
    pub fn dump(&self, name_map: &super::super::NameMap) {
        print!("[");
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", name_map.get(*glyph));
        }
        print!("]");
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GlyphList {
    pub index: usize,
    pub glyphs: Arc<Vec<GlyphId>>,
}

impl Deref for GlyphList {
    type Target = [GlyphId];

    fn deref(&self) -> &Self::Target {
        &*self.glyphs
    }
}

impl GlyphList {
    pub fn dump(&self, name_map: &super::super::NameMap) {
        print!("[");
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", name_map.get(*glyph));
        }
        print!("]");
    }
}

impl super::PrettyPrint for GlyphList {
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &NameMap) -> fmt::Result {
        write!(f, "[")?;
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", name_map.get(*glyph))?;
        }
        write!(f, "]")
    }
}
