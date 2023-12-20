use super::layout::GlyphSet;
use core::fmt;

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct Filter {
    pub is_rtl: bool,
    pub ignore_bases: bool,
    pub ignore_ligatures: bool,
    pub marks: MarkFilter,
}

#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub enum MarkFilter {
    #[default]
    None,
    IgnoreAll,
    Allow(GlyphSet),
}

impl super::PrettyPrint for Filter {
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &crate::NameMap) -> fmt::Result {
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
        match &self.marks {
            MarkFilter::None => {}
            MarkFilter::IgnoreAll => {
                write!(f, " -marks")?;
            }
            MarkFilter::Allow(marks) => {
                write!(f, "marks: ")?;
                marks.pretty_print(f, name_map)?;
            }
        }
        Ok(())
    }
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
        match &self.marks {
            MarkFilter::None => {}
            MarkFilter::IgnoreAll => {
                write!(f, " -marks")?;
            }
            MarkFilter::Allow(marks) => {
                write!(f, " marks: [")?;
                for (i, mark) in marks.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", mark.to_u16())?;
                }
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}
