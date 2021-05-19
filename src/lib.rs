#![allow(dead_code, unused_variables)]

#[path = "platform/windows.rs"]
mod platform;

mod context;
mod data;
mod id;
mod library;
mod locale;
mod scan;
mod script_tags;

pub use context::FontContext;
pub use data::SourcePaths;
pub use id::{FontFamilyId, FontId, FontSourceId};
pub use library::FontLibrary;
pub use locale::Locale;

use data::*;
use std::sync::Arc;
use swash::{Attributes, Stretch, Style, Weight};

/// Describes a generic font family.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum GenericFontFamily {
    Serif,
    SansSerif,
    Monospace,
    SystemUI,
    Cursive,
    Emoji,
}

/// Entry for a font family in a font library.
#[derive(Clone)]
pub struct FontFamily {
    id: FontFamilyId,
    kind: FontFamilyKind,
}

impl FontFamily {
    /// Returns the identifier for the font family.
    pub fn id(&self) -> FontFamilyId {
        self.id
    }

    /// Returns the name of the font family.
    pub fn name(&self) -> &str {
        match &self.kind {
            FontFamilyKind::Static(name, _) => name,
            FontFamilyKind::Dynamic(data) => &data.name,
        }
    }

    /// Returns an iterator over the fonts that are members of the family.
    pub fn fonts<'a>(&'a self) -> impl Iterator<Item = FontId> + Clone + 'a {
        let fonts = match &self.kind {
            FontFamilyKind::Static(_, fonts) => *fonts,
            FontFamilyKind::Dynamic(data) => &data.fonts,
        };
        fonts.iter().map(|font| font.0)
    }
}

#[derive(Clone)]
enum FontFamilyKind {
    Static(&'static str, &'static [(FontId, Stretch, Weight, Style)]),
    Dynamic(Arc<FamilyData>),
}

/// Iterator over the font families in a font library.
#[derive(Clone)]
pub struct Families {
    user: Arc<(u64, CollectionData)>,
    library: FontLibrary,
    pos: usize,
    stage: u8,
}

impl Iterator for Families {
    type Item = FontFamily;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stage == 0 {
                let len = self.user.1.families.len();
                if self.pos >= len {
                    self.stage = 1;
                    continue;
                }
                let pos = self.pos;
                self.pos += 1;
                return self.user.1.family(FontFamilyId::new_user(pos as u32));
            } else {
                let pos = self.pos;
                self.pos += 1;
                return self.library.inner.system.family(FontFamilyId::new(pos as u32));
            }    
        }
    }
}

/// Entry for a font in a font library.
#[derive(Copy, Clone)]
pub struct Font {
    id: FontId,
    family: FontFamilyId,
    source: FontSourceId,
    index: u32,
    attributes: Attributes,
}

impl Font {
    /// Returns the identifier for the font.
    pub fn id(&self) -> FontId {
        self.id
    }

    /// Returns the identifier for the family that contains the font.
    pub fn family(&self) -> FontFamilyId {
        self.family
    }

    /// Returns the identifier for the source that contains the font.
    pub fn source(&self) -> FontSourceId {
        self.source
    }

    /// Returns the index of the font within the corresponding source.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the primary font attributes.
    pub fn attributes(&self) -> Attributes {
        self.attributes
    }
}

/// Entry for a font source in a font library.
#[derive(Clone)]
pub struct FontSource {
    id: FontSourceId,
    kind: FontSourceKind,
}

impl FontSource {
    /// Returns the identifier for the font source.
    pub fn id(&self) -> FontSourceId {
        self.id
    }

    /// Returns the kind of the font source.
    pub fn kind(&self) -> &FontSourceKind {
        &self.kind
    }
}

/// The kind of a font source.
#[derive(Clone)]
pub enum FontSourceKind {
    /// File name of the source. Pair with [`SourcePaths`] to locate the file.
    FileName(&'static str),
    /// Full path to a font file.
    Path(Arc<str>),
    /// Shared buffer containing font data.
    Data(Arc<Vec<u8>>),
}

/// Context that describes the result of font registration.
#[derive(Clone, Default)]
pub struct Registration {
    /// List of font families that were registered.
    pub families: Vec<FontFamilyId>,
    /// List of fonts that were registered.
    pub fonts: Vec<FontId>,  
}
