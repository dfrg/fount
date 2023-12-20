mod filter;
mod layout;
mod pos;
mod raise;
mod sub;
mod value;

use core::fmt;
use std::collections::BTreeSet;

use read_fonts::types::Tag;

pub use filter::*;
pub use layout::*;
pub use pos::*;
pub use sub::*;
pub use value::*;

pub use raise::RaiseContext;

use crate::NameMap;

#[derive(Debug)]
pub enum Action {
    Replace(ReplaceAction),
    Ligate(LigateAction),
    Adjust(AdjustAction),
    MarkAttach(MarkAttachAction),
    Contextual(ContextualAction),
}

#[derive(Debug)]
pub struct ContextualAction {
    pub backtrack: Vec<GlyphSet>,
    pub input: Vec<GlyphSet>,
    pub lookahead: Vec<GlyphSet>,
    pub actions: Vec<(u16, u16)>,
}

#[derive(Default, Debug)]
pub struct ActionGroup {
    pub feature_users: BTreeSet<FeatureUser>,
    pub filter: Filter,
    pub actions: Vec<Action>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FeatureUser {
    pub script: Tag,
    pub language: Tag,
    pub feature: Tag,
}

impl fmt::Display for FeatureUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.script, self.language, self.feature)
    }
}

impl fmt::Debug for FeatureUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.script, self.language, self.feature)
    }
}

#[derive(Default, Debug)]
pub struct Feature {
    pub script: Tag,
    pub language: Tag,
    pub feature: Tag,
    pub action_groups: Vec<usize>,
}

#[derive(Default, Debug)]
pub struct Layout {
    pub action_groups: Vec<ActionGroup>,
    pub features: Vec<Feature>,
}

impl Layout {
    pub fn dump(&self, name_map: &super::NameMap) {
        for group in &self.action_groups {
            println!("{:?} {}", group.feature_users, group.filter);
            for action in &group.actions {
                match action {
                    Action::Replace(replace) => {
                        print!("    {} -> ", name_map.get(replace.target));
                        match &replace.replacement {
                            Replacement::Delete => {
                                println!("[]");
                            }
                            Replacement::Single(gid) => {
                                println!("{}", name_map.get(*gid));
                            }
                            Replacement::Multiple(list) => {
                                list.dump(name_map);
                                println!();
                            }
                        }
                    }
                    Action::Ligate(ligate) => {
                        print!("    [");
                        for (i, comp) in core::iter::once(ligate.target)
                            .chain(ligate.components.iter().copied())
                            .enumerate()
                        {
                            if i > 0 {
                                print!(", ");
                            }
                            print!("{}", name_map.get(comp));
                        }
                        println!("] -> {}", name_map.get(ligate.replacement));
                    }
                    Action::Adjust(_) => {}
                    Action::MarkAttach(attach) => {
                        println!("    {:?} {}", attach.base, attach.base_anchor);
                        for (anchor, glyphs) in &attach.marks {
                            print!("        [");
                            for (i, glyph) in glyphs.iter().enumerate() {
                                if i > 0 {
                                    print!(", ");
                                }
                                print!("{}", name_map.get(*glyph));
                            }
                            println!("] {}", *anchor);
                        }
                    }
                    Action::Contextual(contextual) => {}
                }
            }
        }
    }

    fn pretty_print_action_group(
        &self,
        f: &mut fmt::Formatter,
        name_map: &NameMap,
        group: &ActionGroup,
    ) -> fmt::Result {
        writeln!(f, "{:?} {}", group.feature_users, group.filter)?;
        for action in &group.actions {
            match action {
                Action::Replace(replace) => {
                    write!(f, "    {} -> ", name_map.get(replace.target))?;
                    match &replace.replacement {
                        Replacement::Delete => {
                            writeln!(f, "[]")?;
                        }
                        Replacement::Single(gid) => {
                            writeln!(f, "{}", name_map.get(*gid))?;
                        }
                        Replacement::Multiple(list) => {
                            list.pretty_print(f, name_map)?;
                            writeln!(f)?;
                        }
                    }
                }
                Action::Ligate(ligate) => {
                    write!(f, "    [")?;
                    for (i, comp) in core::iter::once(ligate.target)
                        .chain(ligate.components.iter().copied())
                        .enumerate()
                    {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", name_map.get(comp))?;
                    }
                    writeln!(f, "] -> {}", name_map.get(ligate.replacement))?;
                }
                Action::Adjust(_) => {}
                Action::MarkAttach(attach) => {
                    writeln!(f, "    {:?} {}", attach.base, attach.base_anchor)?;
                    for (anchor, glyphs) in &attach.marks {
                        write!(f, "        [")?;
                        for (i, glyph) in glyphs.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", name_map.get(*glyph))?;
                        }
                        writeln!(f, "] {}", *anchor)?;
                    }
                }
                Action::Contextual(contextual) => {
                    write!(f, "    backtrack: ")?;
                    contextual.backtrack.as_slice().pretty_print(f, name_map)?;
                    writeln!(f)?;
                    write!(f, "    input: ")?;
                    contextual.input.as_slice().pretty_print(f, name_map)?;
                    writeln!(f)?;
                    write!(f, "    lookahead: ")?;
                    contextual.lookahead.as_slice().pretty_print(f, name_map)?;
                    writeln!(f)?;
                    writeln!(f, "    =======================================")?;
                    for (seq_ix, lookup_ix) in &contextual.actions {
                        writeln!(
                            f,
                            "        (input start: {}, action index: {})",
                            *seq_ix, *lookup_ix
                        )?;
                        self.pretty_print_action_group(
                            f,
                            name_map,
                            &self.action_groups[*lookup_ix as usize],
                        )?;
                    }
                    writeln!(f, "    =======================================")?;
                }
            }
        }
        Ok(())
    }
}

pub struct LayoutPrettyPrinter<'a>(pub &'a Layout, pub &'a NameMap);

impl fmt::Display for LayoutPrettyPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.pretty_print(f, &self.1)
    }
}

impl PrettyPrint for Layout {
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &NameMap) -> fmt::Result {
        for group in &self.action_groups {
            self.pretty_print_action_group(f, name_map, group)?;
        }
        Ok(())
    }
}

pub trait PrettyPrint {
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &NameMap) -> fmt::Result;
}

impl<T> PrettyPrint for &[T]
where
    T: PrettyPrint,
{
    fn pretty_print(&self, f: &mut fmt::Formatter, name_map: &NameMap) -> fmt::Result {
        write!(f, "[")?;
        for (i, value) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            value.pretty_print(f, name_map)?;
        }
        write!(f, "]")
    }
}
