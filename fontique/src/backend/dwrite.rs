use dwrote::{
    Font as DFont, FontCollection, FontFallback, TextAnalysisSource, TextAnalysisSourceMethods,
};
use hashbrown::HashMap;
use std::{borrow::Cow, sync::Arc};
use winapi::{
    ctypes::wchar_t,
    um::dwrite::{
        IDWriteFont, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
        DWRITE_FONT_WEIGHT_REGULAR, DWRITE_READING_DIRECTION,
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT,
    },
};
use wio::com::ComPtr;

use super::{
    FallbackKey, FamilyId, FamilyInfo, FamilyName, FamilyNameMap, FontInfo, GenericFamily,
    GenericFamilyMap, SourcePathMap,
};

const DEFAULT_GENERIC_FAMILIES: &[(GenericFamily, &[&str])] = &[
    (GenericFamily::Serif, &["Times New Roman"]),
    (GenericFamily::SansSerif, &["Arial"]),
    (GenericFamily::Monospace, &["Consolas"]),
    (GenericFamily::Cursive, &["Comic Sans MS"]),
    (GenericFamily::Fantasy, &["Impact"]),
    (GenericFamily::SystemUi, &["Segoe UI"]),
    (GenericFamily::Emoji, &["Segoe UI Emoji"]),
    (GenericFamily::Math, &["Cambria Math"]),
    (GenericFamily::FangSong, &["FangSong"]),
];

/// Raw access to the collection of local system fonts.
pub struct SystemFonts {
    pub name_map: Arc<FamilyNameMap>,
    pub generic_families: Arc<GenericFamilyMap>,
    source_cache: SourcePathMap,
    family_map: HashMap<FamilyId, Option<FamilyInfo>>,
    collection: FontCollection,
    fallback: Option<FontFallback>,
    utf16_buf: Vec<wchar_t>,
}

// We're only going to access this through a mutex.
unsafe impl Send for SystemFonts {}
unsafe impl Sync for SystemFonts {}

impl SystemFonts {
    pub fn new() -> Self {
        let collection = FontCollection::get_system(false);
        let mut name_map = FamilyNameMap::default();
        for name in collection.families_iter().map(|family| family.name()) {
            name_map.get_or_insert(&name);
        }
        let mut generic_families = GenericFamilyMap::default();
        for (family, names) in DEFAULT_GENERIC_FAMILIES {
            generic_families.set(
                *family,
                names
                    .iter()
                    .filter_map(|name| name_map.get(name))
                    .map(|name| name.id()),
            );
        }
        Self {
            name_map: Arc::new(name_map),
            generic_families: Arc::new(generic_families),
            source_cache: Default::default(),
            family_map: Default::default(),
            collection,
            fallback: FontFallback::get_system_fallback(),
            utf16_buf: Default::default(),
        }
    }

    pub fn family(&mut self, id: FamilyId) -> Option<FamilyInfo> {
        match self.family_map.get(&id) {
            Some(Some(family)) => return Some(family.clone()),
            Some(None) => return None,
            _ => {}
        }
        let name = self.name_map.get_by_id(id)?;
        let mut fonts: smallvec::SmallVec<[FontInfo; 4]> = Default::default();
        if let Some(family) = self.collection.get_font_family_by_name(name.name()) {
            fonts.reserve(family.get_font_count() as usize);
            for i in 0..family.get_font_count() {
                if let Some(font) =
                    FontInfo::from_dwrite(family.get_font(i), &mut self.source_cache)
                {
                    if !fonts
                        .iter()
                        .any(|f| f.source().id() == font.source().id() && f.index() == font.index())
                    {
                        fonts.push(font);
                    }
                }
            }
            if !fonts.is_empty() {
                let family = FamilyInfo::new(name.clone(), fonts);
                self.family_map.insert(id, Some(family.clone()));
                return Some(family);
            }
        }
        self.family_map.insert(id, None);
        None
    }

    pub fn fallback(&mut self, key: impl Into<FallbackKey>) -> Option<FamilyId> {
        let key = key.into();
        let text = key.script().sample()?;
        let locale = key.locale();
        self.fallback_for_text(text, locale, false)
            .map(|handle| handle.id())
    }
}

impl SystemFonts {
    fn fallback_for_text(
        &mut self,
        text: &str,
        locale: Option<&str>,
        prefer_ui: bool,
    ) -> Option<FamilyName> {
        self.utf16_buf.clear();
        for ch in text.encode_utf16() {
            self.utf16_buf.push(ch);
        }
        let text_len = self.utf16_buf.len() as u32;
        let text_source = TextAnalysisSource::from_text(
            Box::new(TextAnalysisData {
                locale,
                len: text_len,
            }),
            Cow::Borrowed(&self.utf16_buf),
        );
        let mut base_family = if prefer_ui {
            Some(smallvec::SmallVec::<[u16; 12]>::from_slice(
                &b"Segoe UI\0".map(|ch| ch as u16),
            ))
        } else {
            None
        };
        let fallback = self.fallback.as_ref()?;
        let font = {
            let mut font: *mut IDWriteFont = std::ptr::null_mut();
            let mut i = 0u32;
            while font.is_null() && i < text_len {
                let mut mapped_length = 0;
                let mut scale = 0.0;
                let hr = unsafe {
                    (*fallback.as_ptr()).MapCharacters(
                        text_source.as_ptr(),
                        i,
                        text_len - i,
                        core::ptr::null_mut(),
                        // self.collection.as_ptr(),
                        base_family
                            .as_mut()
                            .map(|name| name.as_mut_ptr())
                            .unwrap_or(core::ptr::null_mut()),
                        DWRITE_FONT_WEIGHT_REGULAR,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        &mut mapped_length,
                        &mut font,
                        &mut scale,
                    )
                };
                assert_eq!(hr, 0);
                if font.is_null() {
                    i += 1;
                }
            }
            if font.is_null() {
                None
            } else {
                Some(DFont::take(unsafe { ComPtr::from_raw(font) }))
            }
        }?;
        self.name_map.get(font.family_name().as_str()).cloned()
    }
}

impl FontInfo {
    fn from_dwrite(font: DFont, paths: &mut SourcePathMap) -> Option<Self> {
        let face = font.create_font_face();
        let files = face.get_files();
        let path = files.first()?.get_font_file_path()?;
        let data = paths.get_or_insert(&path);
        let index = face.get_index();
        Self::from_source(data, index)
    }
}

struct TextAnalysisData<'a> {
    locale: Option<&'a str>,
    len: u32,
}

impl TextAnalysisSourceMethods for TextAnalysisData<'_> {
    fn get_locale_name(&self, _text_position: u32) -> (Cow<'_, str>, u32) {
        (Cow::Borrowed(self.locale.unwrap_or("")), self.len)
    }

    /// Get the text direction for the paragraph.
    fn get_paragraph_reading_direction(&self) -> DWRITE_READING_DIRECTION {
        DWRITE_READING_DIRECTION_LEFT_TO_RIGHT
    }
}
