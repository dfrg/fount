use read_fonts::{tables::cmap, types::GlyphId, TableProvider};

/// Mapping of codepoints to nominal glyph identifiers.
// #[derive(Clone)]
pub struct CharMap<'a> {
    map: Option<cmap::CmapSubtable<'a>>,
    vs_map: Option<cmap::Cmap14<'a>>,
}

impl<'a> CharMap<'a> {
    /// Creates a new character map from the specified table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        let (map, vs_map) = if let Ok(cmap) = font.cmap() {
            let mut map = None;
            let mut vs_map = None;
            for record in cmap.encoding_records() {
                if let Ok(subtable) = record.subtable(cmap.offset_data()) {
                    match subtable {
                        cmap::CmapSubtable::Format12(_) => {
                            map = Some(subtable);
                        }
                        cmap::CmapSubtable::Format4(_) => {
                            if map.is_none() {
                                map = Some(subtable);
                            }
                        }
                        cmap::CmapSubtable::Format14(subtable) => {
                            vs_map = Some(subtable);
                        }
                        _ => {}
                    }
                }
            }
            (map, vs_map)
        } else {
            (None, None)
        };
        Self { map, vs_map }
    }

    /// Maps a codepoint to a nominal glyph identifier. Returns `None` if a mapping does
    /// not exist.
    pub fn map(&self, codepoint: impl Into<u32>) -> Option<GlyphId> {
        match self.map.as_ref()? {
            cmap::CmapSubtable::Format4(subtable) => subtable.map_codepoint(codepoint),
            cmap::CmapSubtable::Format12(subtable) => subtable.map_codepoint(codepoint),
            _ => None,
        }
    }

    /// Maps a codepoint and variation selector to a nominal glyph identifier.
    pub fn map_variant(
        &self,
        codepoint: impl Into<u32>,
        selector: impl Into<u32>,
    ) -> Option<MapVariant> {
        // use read_fonts::types::{Scalar, Uint24};
        let _codepoint = codepoint.into();
        let _selector = selector.into();
        let _map = self.vs_map.as_ref()?;
        // let selector_records = map.var_selector();
        // let selector_record = match selector_records.binary_search_by(|record| {
        //     <Uint24 as Into<u32>>::into(record.var_selector()).cmp(&selector)
        // }) {
        //     Ok(idx) => selector_records.get(idx)?,
        //     _ => return None,
        // };
        // let default_uvs = selector_record.default_uvs(map.offset_data())?.ok()?;

        // TODO: finish this logic.
        None
    }
}

/// Result of the mapping a codepoint with a variation selector.
#[derive(Copy, Clone, Debug)]
pub enum MapVariant {
    /// Use the default mapping.
    UseDefault,
    /// Use the specified variant.
    Variant(GlyphId),
}
