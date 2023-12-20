use crate::GlyphList;
use read_fonts::types::GlyphId;

#[derive(Clone, Debug)]
pub struct ReplaceAction {
    pub target: GlyphId,
    pub replacement: Replacement,
}

#[derive(Clone, Debug)]
pub enum Replacement {
    Delete,
    Single(GlyphId),
    Multiple(GlyphList),
}

#[derive(Clone, Debug)]
pub struct LigateAction {
    pub target: GlyphId,
    pub components: GlyphList,
    pub replacement: GlyphId,
}
