use read_fonts::{
    tables::cmap::{self, Cmap, Cmap14, CmapSubtable, PlatformId},
    types::{GlyphId, Uint24},
    TableProvider,
};

struct Map<'a> {
    map: CmapSubtable<'a>,
    is_symbol: bool,
}

impl<'a> Map<'a> {
    fn new(map: CmapSubtable<'a>, is_symbol: bool) -> Self {
        Self { map, is_symbol }
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

/// Result of the mapping a codepoint with a variation selector.
#[derive(Copy, Clone, Debug)]
pub enum MapVariant {
    /// The variation selector should be ignored and the default mapping
    /// of the character should be used.
    Default,
    /// The variant glyph mapped by a codepoint and associated variation
    /// selector.
    Variant(GlyphId),
}

/// Mapping of codepoints to nominal glyph identifiers.
// #[derive(Clone)]
pub struct CharMap<'a> {
    map: Option<Map<'a>>,
    vs_map: Option<Cmap14<'a>>,
}

impl<'a> CharMap<'a> {
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
        let map = self.vs_map.as_ref()?;
        let codepoint = codepoint.into();
        let selector = selector.into();
        let selector_records = map.var_selector();
        let selector_record = match selector_records.binary_search_by(|record| {
            <Uint24 as Into<u32>>::into(record.var_selector()).cmp(&selector)
        }) {
            Ok(idx) => selector_records.get(idx)?,
            _ => return None,
        };
        if let Some(Ok(default_uvs)) = selector_record.default_uvs(map.offset_data()) {
            let ranges = default_uvs.ranges();
            let mut lo = 0;
            let mut hi = ranges.len();
            while lo < hi {
                let i = (lo + hi) / 2;
                let range = &ranges[i];
                let start = range.start_unicode_value().into();
                if codepoint < start {
                    hi = i;
                } else if codepoint > (start + range.additional_count() as u32) {
                    lo = i + 1;
                } else {
                    return Some(MapVariant::Default);
                }
            }
        }
        if let Some(Ok(non_default_uvs)) = selector_record.non_default_uvs(map.offset_data()) {
            let mapping = non_default_uvs.uvs_mapping();
            let ix = mapping
                .binary_search_by(|rec| {
                    <Uint24 as Into<u32>>::into(rec.unicode_value()).cmp(&codepoint)
                })
                .ok()?;
            return Some(MapVariant::Variant(GlyphId::new(
                mapping.get(ix)?.glyph_id(),
            )));
        }
        None
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
    for rec in records {
        if let (PlatformId::Windows, ENCODING_MS_SYMBOL) = (rec.platform_id(), rec.encoding_id()) {
            if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                match subtable {
                    CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                        return Some(Map::new(subtable, true));
                    }
                    _ => {}
                }
            }
        }
    }
    // First, search for a UCS4 mapping.
    // According to FreeType, the most interesting table (Windows, UCS4) often appears
    // last, so search in reverse order.
    for rec in records.iter().rev() {
        match (rec.platform_id(), rec.encoding_id()) {
            (PlatformId::Windows, ENCODING_MS_ID_UCS_4)
            | (PlatformId::Unicode, ENCODING_APPLE_ID_UNICODE_32) => {
                if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                    match subtable {
                        CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                            return Some(Map::new(subtable, false));
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    // Now simply search for any Unicode mapping, again in reverse.
    for rec in records.iter().rev() {
        match (rec.platform_id(), rec.encoding_id()) {
            (PlatformId::ISO, _)
            | (PlatformId::Unicode, _)
            | (PlatformId::Windows, ENCODING_MS_ID_UCS_4)
            | (PlatformId::Windows, ENCODING_MS_UNICODE_CS) => {
                if let Ok(subtable) = rec.subtable(cmap.offset_data()) {
                    match subtable {
                        CmapSubtable::Format4(_) | CmapSubtable::Format12(_) => {
                            return Some(Map::new(subtable, false));
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
fn find_variant_selector_subtable<'a>(cmap: &Cmap<'a>) -> Option<Cmap14<'a>> {
    const ENCODING_APPLE_ID_VARIANT_SELECTOR: u16 = 5;
    for rec in cmap.encoding_records() {
        if let (PlatformId::Unicode, ENCODING_APPLE_ID_VARIANT_SELECTOR) =
            (rec.platform_id(), rec.encoding_id())
        {
            if let Ok(CmapSubtable::Format14(subtable)) = rec.subtable(cmap.offset_data()) {
                return Some(subtable);
            }
        }
    }
    None
}
