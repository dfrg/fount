use super::id::*;
use super::*;
use std::collections::HashMap;
use std::sync::Arc;
use swash::text::Script;
use swash::{Attributes, CacheKey, Stretch, Style, Weight};

#[derive(Clone)]
pub struct FamilyData {
    pub name: Arc<str>,
    pub has_stretch: bool,
    pub fonts: Vec<(FontId, Stretch, Weight, Style)>,
}

#[derive(Clone)]
pub struct FontData {
    pub family: FontFamilyId,
    pub source: FontSourceId,
    pub index: u32,
    pub attributes: Attributes,
    pub cache_key: CacheKey,
}

#[derive(Clone)]
pub enum SourceData {
    Path(Arc<str>),
    Data(Arc<Vec<u8>>),
}

#[derive(Clone, Default)]
pub struct CollectionData {
    pub is_user: bool,
    pub families: Vec<Arc<FamilyData>>,
    pub fonts: Vec<FontData>,
    pub sources: Vec<SourceData>,
    pub family_map: HashMap<Arc<str>, FontFamilyId>,
}

impl CollectionData {
    pub fn family_id(&self, name: &str) -> Option<FontFamilyId> {
        let mut lowercase_buf = LowercaseString::new();
        let lowercase_name = lowercase_buf.get(name)?;
        if let Some(family_id) = self.family_map.get(lowercase_name) {
            Some(*family_id)
        } else {
            None
        }
    }

    pub fn family(&self, id: FontFamilyId) -> Option<FontFamily> {
        let family = self.families.get(id.to_usize())?;
        Some(FontFamily {
            id,
            kind: FontFamilyKind::Dynamic(family.clone()),
        })
    }

    pub fn family_by_name(&self, name: &str) -> Option<FontFamily> {
        self.family(self.family_id(name)?)
    }     

    pub fn font(&self, id: FontId) -> Option<Font> {
        let font = self.fonts.get(id.to_usize())?;
        Some(Font {
            id,
            family: font.family,
            source: font.source,
            index: font.index,
            attributes: font.attributes,
        })
    }

    pub fn source(&self, id: FontSourceId) -> Option<FontSource> {
        let source = self.sources.get(id.to_usize())?;
        Some(FontSource {
            id,
            kind: match source {
                SourceData::Path(path) => FontSourceKind::Path(path.clone()),
                SourceData::Data(data) => FontSourceKind::Data(data.clone()),
            },
        })
    }

    pub fn clone_into(&self, other: &mut Self) {
        other.families.clear();
        other.fonts.clear();
        other.sources.clear();
        other.family_map.clear();
        other.families.extend(self.families.iter().cloned());
        other.fonts.extend(self.fonts.iter().cloned());
        other.sources.extend(self.sources.iter().cloned());
        for (name, families) in &self.family_map {
            other.family_map.insert(name.clone(), families.clone());
        }
    }
}

#[derive(Default)]
pub struct FallbackData {
    pub default_families: Vec<FontFamilyId>,
    pub script_fallbacks: HashMap<[u8; 4], Vec<FontFamilyId>>,
    pub generic_families: [Vec<FontFamilyId>; GENERIC_FAMILY_COUNT],
    pub cjk_families: [Vec<FontFamilyId>; CJK_FAMILY_COUNT],
}

impl FallbackData {
    pub fn default_families(&self) -> &[FontFamilyId] {
        &self.default_families
    }

    pub fn generic_families(&self, family: GenericFontFamily) -> &[FontFamilyId] {
        self.generic_families
            .get(family as usize)
            .map(|families| families.as_ref())
            .unwrap_or(&[])
    }

    pub fn fallback_families(&self, script: Script, locale: Option<Locale>) -> &[FontFamilyId] {
        if script == Script::Han {
            let cjk = locale.map(|l| l.cjk() as usize).unwrap_or(0);
            return &self.cjk_families[cjk];
        }
        let tag = super::script_tags::script_tag(script);
        match self.script_fallbacks.get(&tag) {
            Some(families) => &families,
            _ => &self.default_families,
        }
    }

    /// This method generates fallback data for a scanned collection from the precomputed
    /// family names in a static collection.
    pub fn fill_from_static(
        &mut self,
        collection: &CollectionData,
        static_collection: &StaticCollectionData,
    ) {
        self.default_families.clear();
        self.default_families.extend(
            static_collection
                .default_families
                .iter()
                .filter_map(|id| static_collection.family_name(*id))
                .filter_map(|name| collection.family_id(name)),
        );
        for script_fallbacks in static_collection.script_fallbacks {
            let families = script_fallbacks
                .families
                .iter()
                .filter_map(|id| static_collection.family_name(*id))
                .filter_map(|name| collection.family_id(name))
                .collect::<Vec<_>>();
            if !families.is_empty() {
                self.script_fallbacks
                    .insert(script_fallbacks.script, families);
            }
        }
        for i in 0..GENERIC_FAMILY_COUNT {
            self.generic_families[i] = static_collection.generic_families[i]
                .iter()
                .filter_map(|id| static_collection.family_name(*id))
                .filter_map(|name| collection.family_id(name))
                .collect::<Vec<_>>();
        }
        for i in 0..CJK_FAMILY_COUNT {
            self.cjk_families[i] = static_collection.cjk_families[i]
                .iter()
                .filter_map(|id| static_collection.family_name(*id))
                .filter_map(|name| collection.family_id(name))
                .collect::<Vec<_>>();
        }
    }
}

#[derive(Default)]
pub struct ScannedCollectionData {
    pub collection: CollectionData,
    pub fallback: FallbackData,
}

pub enum SystemCollectionData {
    Static(&'static StaticCollectionData),
    Scanned(ScannedCollectionData),
}

impl SystemCollectionData {
    pub fn source_paths(&self) -> SourcePaths {
        match self {
            Self::Static(data) => SourcePaths {
                inner: SourcePathsInner::Static(data.search_paths),
                pos: 0,
            },
            Self::Scanned(data) => SourcePaths {
                inner: SourcePathsInner::Static(&[]),
                pos: 0,
            },
        }
    }    

    pub fn family(&self, id: FontFamilyId) -> Option<FontFamily> {
        match self {
            Self::Static(data) => {
                let family = data.families.get(id.to_usize())?;
                Some(FontFamily {
                    id,
                    kind: FontFamilyKind::Static(family.name, family.fonts),
                })
            }
            Self::Scanned(data) => data.collection.family(id),
        }
    }    

    pub fn family_by_name(&self, name: &str) -> Option<FontFamily> {
        self.family(self.family_id(name)?)
    }    

    pub fn font(&self, id: FontId) -> Option<Font> {
        match self {
            Self::Static(data) => {
                let font = data.fonts.get(id.to_usize())?;
                Some(Font {
                    id,
                    family: font.family,
                    source: font.source,
                    index: font.index,
                    attributes: font.attributes,
                })
            }
            Self::Scanned(data) => data.collection.font(id),
        }
    }    

    pub fn source(&self, id: FontSourceId) -> Option<FontSource> {
        match self {
            Self::Static(data) => {
                let source = data.sources.get(id.to_usize())?;
                Some(FontSource {
                    id,
                    kind: FontSourceKind::FileName(source.file_name),
                })
            }
            Self::Scanned(data) => data.collection.source(id),
        }
    }    

    pub fn default_families(&self) -> &[FontFamilyId] {
        match self {
            Self::Static(data) => data.default_families,
            Self::Scanned(data) => data.fallback.default_families(),
        }
    }    

    pub fn generic_families(&self, family: GenericFontFamily) -> &[FontFamilyId] {
        match self {
            Self::Static(data) => data
                .generic_families
                .get(family as usize)
                .copied()
                .unwrap_or(&[]),
            Self::Scanned(data) => data.fallback.generic_families(family),
        }
    }    

    pub fn fallback_families(&self, script: Script, locale: Option<Locale>) -> &[FontFamilyId] {
        match self {
            Self::Static(data) => data.fallback_families(script, locale),
            Self::Scanned(data) => data.fallback.fallback_families(script, locale),
        }
    }    

    pub fn family_id(&self, name: &str) -> Option<FontFamilyId> {
        match self {
            Self::Static(data) => data.family_id(name),
            Self::Scanned(data) => data.collection.family_id(name),
        }
    }    
}

pub struct StaticFamilyData {
    pub name: &'static str,
    pub lowercase_name: &'static str,
    pub has_stretch: bool,
    pub fonts: &'static [(FontId, Stretch, Weight, Style)],
}

pub struct StaticFontData {
    pub family: FontFamilyId,
    pub attributes: Attributes,
    pub source: FontSourceId,
    pub index: u32,
}

pub struct StaticSourceData {
    pub file_name: &'static str,
}

pub struct StaticScriptFallbacks {
    pub script: [u8; 4],
    pub families: &'static [FontFamilyId],
}

const GENERIC_FAMILY_COUNT: usize = 6;
const CJK_FAMILY_COUNT: usize = 5;

pub struct StaticCollectionData {
    pub search_paths: &'static [&'static str],
    pub families: &'static [StaticFamilyData],
    pub fonts: &'static [StaticFontData],
    pub sources: &'static [StaticSourceData],
    pub default_families: &'static [FontFamilyId],
    pub script_fallbacks: &'static [StaticScriptFallbacks],
    pub generic_families: [&'static [FontFamilyId]; GENERIC_FAMILY_COUNT],
    pub cjk_families: [&'static [FontFamilyId]; CJK_FAMILY_COUNT],
}

impl StaticCollectionData {
    pub fn family_id(&self, name: &str) -> Option<FontFamilyId> {
        let mut lowercase_buf = LowercaseString::new();
        let lowercase_name = lowercase_buf.get(name)?;
        match self
            .families
            .binary_search_by(|x| x.lowercase_name.cmp(&lowercase_name))
        {
            Ok(index) => Some(FontFamilyId::new(index as u32)),
            _ => None,
        }
    }

    pub fn fallback_families(&self, script: Script, locale: Option<Locale>) -> &[FontFamilyId] {
        if script == Script::Han {
            let cjk = locale.map(|l| l.cjk() as usize).unwrap_or(0);
            return self.cjk_families[cjk];
        }
        let tag = super::script_tags::script_tag(script);
        match self
            .script_fallbacks
            .binary_search_by(|x| x.script.cmp(&tag))
        {
            Ok(index) => self
                .script_fallbacks
                .get(index)
                .map(|x| x.families)
                .unwrap_or(&[]),
            _ => self.default_families,
        }
    }

    pub fn family_name(&self, id: FontFamilyId) -> Option<&'static str> {
        self.families.get(id.to_usize()).map(|family| family.name)
    }    
}

/// Iterator over file system paths that contain fonts.
///
/// This iterator is returned by the [`source_paths`](super::FontContext::source_paths) method
/// of [`FontContext`](super::FontContext).
#[derive(Copy, Clone)]
pub struct SourcePaths<'a> {
    inner: SourcePathsInner<'a>,
    pos: usize,
}

impl<'a> Iterator for SourcePaths<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            SourcePathsInner::Static(paths) => {
                if self.pos > paths.len() {
                    None
                } else {
                    let pos = self.pos;
                    self.pos += 1;
                    paths.get(pos).copied()
                }
            }
            SourcePathsInner::Dynamic(paths) => {
                if self.pos > paths.len() {
                    None
                } else {
                    let pos = self.pos;
                    self.pos += 1;
                    paths.get(pos).map(|s| s.as_str())
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
enum SourcePathsInner<'a> {
    Static(&'static [&'static str]),
    Dynamic(&'a Vec<String>),
}

pub struct LowercaseString {
    buf: [u8; 128],
    heap: String,
}

impl LowercaseString {
    pub fn new() -> Self {
        Self {
            buf: [0u8; 128],
            heap: Default::default(),
        }
    }

    pub fn get<'a>(&'a mut self, name: &str) -> Option<&'a str> {
        if name.len() <= self.buf.len() && name.is_ascii() {
            let mut end = 0;
            for c in name.as_bytes() {
                self.buf[end] = c.to_ascii_lowercase();
                end += 1;
            }
            std::str::from_utf8(&self.buf[..end]).ok()
        } else {
            self.heap = name.to_lowercase();
            Some(&self.heap)
        }
    }
}
