/*! Mapping characters to glyph identifiers.

*/

use read_fonts::{
    tables::cmap::{self, Cmap, Cmap14, CmapSubtable, PlatformId},
    types::{GlyphId, Uint24},
    TableProvider,
};

pub use read_fonts::tables::cmap::MapVariant;

/// Indices of selected mapping subtables.
#[derive(Copy, Clone, Default, Debug)]
pub struct SelectedMaps {
    /// Index of Unicode or symbol mapping subtable.
    pub mapping: Option<u16>,
    /// True if the above is a symbol mapping.
    pub is_symbol: bool,
    /// Index of Unicode variation selector sutable.
    pub variants: Option<u16>,
}

struct Map<'a> {
    map: CmapSubtable<'a>,
    index: u16,
    is_symbol: bool,
}

impl<'a> Map<'a> {
    fn new(map: CmapSubtable<'a>, index: u16, is_symbol: bool) -> Self {
        Self {
            map,
            index,
            is_symbol,
        }
    }

    fn map(&self, codepoint: u32) -> Option<GlyphId> {
        match &self.map {
            cmap::CmapSubtable::Format4(subtable) => subtable.map_codepoint(codepoint),
            cmap::CmapSubtable::Format12(subtable) => subtable.map_codepoint(codepoint),
            _ => None,
        }
    }

    fn adjust_symbol_pua(&self, codepoint: u32) -> u32 {
        // From HarfBuzz:
        // For symbol-encoded OpenType fonts, we duplicate the
        // U+F000..F0FF range at U+0000..U+00FF.  That's what
        // Windows seems to do, and that's hinted about at:
        // https://docs.microsoft.com/en-us/typography/opentype/spec/recom
        // under "Non-Standard (Symbol) Fonts". */
        if codepoint <= 0x00FF {
            codepoint + 0xF000
        } else {
            0
        }
    }
}

/// Mapping of codepoints to nominal glyph identifiers.
// #[derive(Clone)]
pub struct Charmap<'a> {
    map: Option<Map<'a>>,
    vs_map: Option<(Cmap14<'a>, u16)>,
}

impl<'a> Charmap<'a> {
    /// Creates a new character map from the specified table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        let (map, vs_map) = if let Ok(cmap) = font.cmap() {
            (
                find_symbol_or_unicode_subtable(&cmap),
                find_variant_selector_subtable(&cmap),
            )
        } else {
            (None, None)
        };
        Self { map, vs_map }
    }

    /// Returns the selected mapping subtables.
    pub fn selected_maps(&self) -> SelectedMaps {
        let (mapping, is_symbol) = self
            .map
            .as_ref()
            .map(|map| (Some(map.index), map.is_symbol))
            .unwrap_or_default();

        SelectedMaps {
            mapping,
            is_symbol,
            variants: self.vs_map.as_ref().map(|map| map.1),
        }
    }

    /// Returns true if a symbol mapping was selected.
    pub fn is_symbol(&self) -> bool {
        self.map.as_ref().map(|x| x.is_symbol).unwrap_or(false)
    }

    /// Maps a codepoint to a nominal glyph identifier. Returns `None` if a mapping does
    /// not exist.
    pub fn map(&self, codepoint: impl Into<u32>) -> Option<GlyphId> {
        let map = self.map.as_ref()?;
        let codepoint = codepoint.into();
        if let Some(glyph_id) = map.map(codepoint) {
            return Some(glyph_id);
        }
        if map.is_symbol {
            return map.map(map.adjust_symbol_pua(codepoint));
        }
        None
    }

    /// Maps a codepoint and variation selector to a nominal glyph identifier.
    pub fn map_variant(
        &self,
        codepoint: impl Into<u32>,
        selector: impl Into<u32>,
    ) -> Option<MapVariant> {
        let map = &self.vs_map.as_ref()?.0;
        map.map_variant(codepoint, selector)
    }
}

/// Find the best subtable that supports a Unicode mapping.
///
/// The strategy is a combination of those used in FreeType and HarfBuzz.
fn find_symbol_or_unicode_subtable<'a>(cmap: &Cmap<'a>) -> Option<Map<'a>> {
    const ENCODING_MS_SYMBOL: u16 = 0;
    const ENCODING_MS_UNICODE_CS: u16 = 1;
    const ENCODING_MS_ID_UCS_4: u16 = 10;
    const ENCODING_APPLE_ID_UNICODE_32: u16 = 4;
    let records = cmap.encoding_records();
    // HarfBuzz prefers a symbol subtable.
    for (i, rec) in records.iter().enumerate() {
        if let (PlatformId::Windows, ENCODING_MS_SYMBOL) = (rec.platform_id(), rec.encoding_id()) {
            if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                match subtable {
                    CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                        return Some(Map::new(subtable, i as u16, true));
                    }
                    _ => {}
                }
            }
        }
    }
    // First, search for a UCS4 mapping.
    // According to FreeType, the most interesting table (Windows, UCS4) often appears
    // last, so search in reverse order.
    for (i, rec) in records.iter().enumerate().rev() {
        match (rec.platform_id(), rec.encoding_id()) {
            (PlatformId::Windows, ENCODING_MS_ID_UCS_4)
            | (PlatformId::Unicode, ENCODING_APPLE_ID_UNICODE_32) => {
                if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                    match subtable {
                        CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                            return Some(Map::new(subtable, i as u16, false));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    // Now simply search for any Unicode mapping, again in reverse.
    for (i, rec) in records.iter().enumerate().rev() {
        match (rec.platform_id(), rec.encoding_id()) {
            (PlatformId::ISO, _)
            | (PlatformId::Unicode, _)
            | (PlatformId::Windows, ENCODING_MS_ID_UCS_4)
            | (PlatformId::Windows, ENCODING_MS_UNICODE_CS) => {
                if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                    match subtable {
                        CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                            return Some(Map::new(subtable, i as u16, false));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Searches for a format 14 subtable for mapping variant selector sequences.
fn find_variant_selector_subtable<'a>(cmap: &Cmap<'a>) -> Option<(Cmap14<'a>, u16)> {
    const ENCODING_APPLE_ID_VARIANT_SELECTOR: u16 = 5;
    for (i, rec) in cmap.encoding_records().iter().enumerate() {
        if let (PlatformId::Unicode, ENCODING_APPLE_ID_VARIANT_SELECTOR) =
            (rec.platform_id(), rec.encoding_id())
        {
            if let Ok(CmapSubtable::Format14(subtable)) = rec.subtable(cmap.offset_data()) {
                return Some((subtable, i as u16));
            }
        }
    }
    None
}
