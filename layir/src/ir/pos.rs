use std::collections::{BTreeMap, BTreeSet, HashMap};

use read_fonts::types::GlyphId;

use super::value::{Adjustment, Anchor};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct AdjustmentAction {
    pub glyph: GlyphId,
    pub adjustment: Adjustment,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SingleMarkAttachment {
    pub base: GlyphId,
    pub base_anchor: Anchor,
    pub mark: GlyphId,
    pub mark_anchor: Anchor,
}

impl std::fmt::Display for SingleMarkAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {} << {} {}",
            self.base.to_u16(),
            self.base_anchor,
            self.mark.to_u16(),
            self.mark_anchor
        )
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct GroupedMarkAttachment {
    pub base: GlyphId,
    pub base_anchor: Anchor,
    //pub marks: Vec<(Anchor, Vec<GlyphId>)>,
    pub marks: BTreeMap<Anchor, BTreeSet<GlyphId>>,
}

impl std::fmt::Display for GroupedMarkAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", self.base.to_u16(), self.base_anchor)?;
        for (anchor, glyphs) in &self.marks {
            write!(f, "    [")?;
            for (i, glyph) in glyphs.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", glyph.to_u16())?;
            }
            writeln!(f, "] {}", *anchor)?;
        }
        Ok(())
    }
}

impl GroupedMarkAttachment {
    pub fn flatten(&self) -> impl Iterator<Item = SingleMarkAttachment> + '_ {
        self.marks.iter().flat_map(|(mark_anchor, mark_glyphs)| {
            mark_glyphs.iter().map(|&mark| SingleMarkAttachment {
                base: self.base,
                base_anchor: self.base_anchor.clone(),
                mark,
                mark_anchor: mark_anchor.clone(),
            })
        })
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct MarkAttachmentAction {
    pub groups: Vec<GroupedMarkAttachment>,
}

impl MarkAttachmentAction {
    pub fn flatten(&self) -> impl Iterator<Item = SingleMarkAttachment> + '_ {
        self.groups.iter().flat_map(|group| group.flatten())
    }

    pub fn merge_flattened(&mut self, flattened: impl Iterator<Item = SingleMarkAttachment>) {
        let mut group_ix_by_base: HashMap<(GlyphId, Anchor), usize> = self
            .groups
            .iter()
            .enumerate()
            .map(|(ix, group)| ((group.base, group.base_anchor.clone()), ix))
            .collect();
        for attachment in flattened {
            let group_ix = if let Some(group_ix) =
                group_ix_by_base.get(&(attachment.base, attachment.base_anchor.clone()))
            {
                *group_ix
            } else {
                let group_ix = self.groups.len();
                self.groups.push(GroupedMarkAttachment {
                    base: attachment.base,
                    base_anchor: attachment.base_anchor.clone(),
                    marks: Default::default(),
                });
                group_ix_by_base
                    .insert((attachment.base, attachment.base_anchor.clone()), group_ix);
                group_ix
            };
            let grouped_attachments = &mut self.groups[group_ix].marks;
            if let Some(existing_group) = grouped_attachments
                .iter_mut()
                .find(|g| *g.0 == attachment.mark_anchor)
            {
                existing_group.1.insert(attachment.mark);
            } else {
                grouped_attachments
                    .insert(attachment.mark_anchor.clone(), [attachment.mark].into());
            }
        }
    }

    // pub fn append_mark_base(
    //     &mut self,
    //     rcx: &RaiseContext,
    //     subtable: &MarkBasePosFormat1,
    // ) -> Result<(), ReadError> {
    //     let base_array = subtable.base_array()?;
    //     let base_records = base_array.base_records();
    //     let mark_array = subtable.mark_array()?;
    //     let mark_records = mark_array.mark_records();
    //     let cov_ix_to_mark_glyph: HashMap<usize, GlyphId> =
    //         subtable.mark_coverage()?.iter().enumerate().collect();
    //     for (base_ix, base_glyph) in subtable.base_coverage()?.iter().enumerate() {
    //         let base_record = base_records.get(base_ix)?;
    //         for (base_anchor_ix, base_anchor) in base_record
    //             .base_anchors(base_array.offset_data())
    //             .iter()
    //             .enumerate()
    //         {
    //             let Some(base_anchor) = base_anchor else {
    //                 continue;
    //             };
    //             let base_anchor = base_anchor?;
    //             let base_anchor = rcx.raise_anchor(&base_anchor)?;
    //             let mut group = GroupedMarkAttachment {
    //                 base: base_glyph,
    //                 base_anchor: base_anchor,
    //                 marks: Default::default(),
    //             };
    //             let mut attachments: HashMap<Anchor, Vec<GlyphId>> = Default::default();
    //             for (mark_ix, mark_record) in mark_records.iter().enumerate() {
    //                 let mark_class = mark_record.mark_class() as usize;
    //                 if mark_class != base_anchor_ix {
    //                     continue;
    //                 }
    //                 let Some(mark_glyph) = cov_ix_to_mark_glyph.get(&mark_ix) else {
    //                     continue;
    //                 };
    //                 let mark_anchor = mark_record.mark_anchor(mark_array.offset_data())?;
    //                 let mark_anchor = rcx.raise_anchor(&mark_anchor)?;
    //                 attachments
    //                     .entry(mark_anchor)
    //                     .or_default()
    //                     .push(*mark_glyph);
    //             }
    //             if !attachments.is_empty() {
    //                 for (_, glyphs) in &mut attachments {
    //                     glyphs.sort();
    //                 }
    //                 group.marks.extend(attachments.drain());
    //                 group.marks.sort();
    //                 self.groups.push(group);
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}

pub enum PositionAction {
    Adjustment(Vec<AdjustmentAction>),
    MarkAttachment(MarkAttachmentAction),
}
