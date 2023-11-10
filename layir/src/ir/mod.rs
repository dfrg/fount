mod filter;
mod pos;
mod raise;
mod value;

use read_fonts::types::Tag;

pub use filter::*;
pub use pos::*;
pub use value::*;

pub use raise::RaiseContext;

#[derive(Debug)]
pub struct ActionGroup<T> {
    pub filter: Filter,
    pub actions: Vec<T>,
}

impl<T> Default for ActionGroup<T> {
    fn default() -> Self {
        Self {
            filter: Filter::default(),
            actions: vec![],
        }
    }
}

pub struct Feature<T> {
    pub script: Tag,
    pub language: Tag,
    pub feature: Tag,
    pub action_groups: Vec<ActionGroup<T>>,
}

impl<T> Default for Feature<T> {
    fn default() -> Self {
        Self {
            script: Tag::default(),
            language: Tag::default(),
            feature: Tag::default(),
            action_groups: Default::default(),
        }
    }
}

pub type PositionFeature = Feature<PositionAction>;
