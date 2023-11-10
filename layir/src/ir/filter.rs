use std::collections::BTreeSet;

use read_fonts::types::GlyphId;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Filter {
    pub ignore_bases: bool,
    pub ignore_ligatures: bool,
    pub is_rtl: bool,
    pub mark_filter: Option<BTreeSet<GlyphId>>,
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "filter:")?;
        if self.is_rtl {
            write!(f, " rtl")?;
        }
        if self.ignore_bases {
            write!(f, " -bases")?;
        }
        if self.ignore_ligatures {
            write!(f, " -ligatures")?;
        }
        if let Some(marks) = &self.mark_filter {
            write!(f, " marks: [")?;
            for (i, mark) in marks.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", mark.to_u16())?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}
