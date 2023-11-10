//! Support for "raising" binary font data to IR.

use read_fonts::{
    tables::{
        gdef::Gdef,
        gpos::{Gpos, MarkBasePosFormat1, MarkMarkPosFormat1, PositionLookup, ValueRecord},
        layout::{DeviceOrVariationIndex, LookupFlag},
        variations::{DeltaSetIndex, ItemVariationStore},
    },
    types::{GlyphId, Tag},
    FontData, ReadError,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use super::filter::Filter;
use super::value::{Adjustment, Anchor, MasterDeltas, MasterLocations, Value};
use super::{
    ActionGroup, GroupedMarkAttachment, MarkAttachmentAction, PositionAction, PositionFeature,
};

#[derive(Default)]
pub struct RaiseContext {
    master_locations: MasterLocations,
    deltas: HashMap<(u16, u16), MasterDeltas>,
    mark_sets: Vec<BTreeSet<GlyphId>>,
    mark_classes: HashMap<u16, BTreeSet<GlyphId>>,
}

impl RaiseContext {
    pub fn new(gdef: &Gdef, master_locations: Option<MasterLocations>) -> Result<Self, ReadError> {
        let mut cx = Self::default();
        if let Some(Ok(ivs)) = gdef.item_var_store() {
            cx.master_locations = if let Some(locations) = master_locations {
                locations
            } else {
                self::master_locations(&ivs)?
            };
            for (outer_ix, data) in ivs.item_variation_data().iter().enumerate() {
                let outer_ix = outer_ix as u16;
                let Some(data) = data else {
                    continue;
                };
                let data = data?;
                for inner_ix in 0..data.item_count() {
                    let mut deltas = Vec::with_capacity(cx.master_locations.len());
                    let delta_ix = DeltaSetIndex {
                        outer: outer_ix,
                        inner: inner_ix,
                    };
                    for location in &cx.master_locations {
                        deltas.push(ivs.compute_delta(delta_ix, &location)?);
                    }
                    cx.deltas
                        .insert((outer_ix, inner_ix), MasterDeltas(deltas.into()));
                }
            }
        }
        if let Some(Ok(mark_sets)) = gdef.mark_glyph_sets_def() {
            for coverage in mark_sets.coverages().iter() {
                cx.mark_sets.push(coverage?.iter().collect());
            }
        }
        if let Some(Ok(mark_classes)) = gdef.mark_attach_class_def() {
            for (glyph, class) in mark_classes.iter() {
                cx.mark_classes.entry(class).or_default().insert(glyph);
            }
        }
        Ok(cx)
    }

    pub fn deltas(&self, outer_ix: u16, inner_ix: u16) -> Option<MasterDeltas> {
        self.deltas.get(&(outer_ix, inner_ix)).cloned()
    }

    pub fn mark_glyph_set(&self, index: usize) -> Option<&BTreeSet<GlyphId>> {
        self.mark_sets.get(index)
    }

    pub fn marks_by_class(&self, class: u16) -> Option<&BTreeSet<GlyphId>> {
        self.mark_classes.get(&class)
    }
}

/// Values and anchors.
impl RaiseContext {
    pub fn raise_value_record(
        &self,
        record: &ValueRecord,
        data: FontData,
    ) -> Result<Adjustment, ReadError> {
        let mut adj = Adjustment::default();
        let raise_value = |val, var| {
            if let Some(val) = val {
                let deltas = self.master_deltas(var);
                Ok(Some(Value {
                    default: val,
                    deltas,
                }))
            } else {
                Ok(None)
            }
        };
        adj.x = raise_value(record.x_placement(), record.x_placement_device(data))?;
        adj.y = raise_value(record.y_placement(), record.y_placement_device(data))?;
        adj.x_advance = raise_value(record.x_advance(), record.x_advance_device(data))?;
        adj.y_advance = raise_value(record.y_advance(), record.y_advance_device(data))?;
        Ok(adj)
    }

    pub fn raise_anchor(
        &self,
        table: &read_fonts::tables::gpos::AnchorTable,
    ) -> Result<Anchor, ReadError> {
        use read_fonts::tables::gpos::AnchorTable::*;
        let (x, y) = match table {
            Format1(t) => (t.x_coordinate().into(), t.y_coordinate().into()),
            Format2(t) => (t.x_coordinate().into(), t.y_coordinate().into()),
            Format3(t) => {
                let (x, y) = (t.x_coordinate(), t.y_coordinate());
                let mut x: Value = x.into();
                let mut y: Value = y.into();
                x.deltas = self.master_deltas(t.x_device());
                y.deltas = self.master_deltas(t.y_device());
                (x, y)
            }
        };
        Ok(Anchor { x, y })
    }

    fn master_deltas(
        &self,
        var: Option<Result<DeviceOrVariationIndex, ReadError>>,
    ) -> Option<MasterDeltas> {
        if let Some(Ok(DeviceOrVariationIndex::VariationIndex(vi))) = var {
            let outer = vi.delta_set_outer_index();
            let inner = vi.delta_set_inner_index();
            self.deltas(outer, inner)
        } else {
            None
        }
    }
}

/// LookupFlag to Filter
impl RaiseContext {
    pub fn raise_lookup_flag(
        &self,
        flag: LookupFlag,
        mark_filtering_set: u16,
    ) -> Result<Filter, ReadError> {
        let mut filter = Filter::default();
        filter.is_rtl = flag.right_to_left();
        filter.ignore_bases = flag.ignore_base_glyphs();
        filter.ignore_ligatures = flag.ignore_ligatures();
        // If a mark filtering set is specified, this supersedes any mark
        // attachment type indication in the lookup flag. If the IGNORE_MARKS
        // bit is set, this supersedes any mark filtering set or mark
        // attachment type indications.
        if flag.ignore_marks() {
            // Create an empty set
            filter.mark_filter = Some(Default::default());
        } else if flag.use_mark_filtering_set() {
            filter.mark_filter = Some(
                self.mark_glyph_set(mark_filtering_set as usize)
                    .ok_or(ReadError::MalformedData("missing mark filtering set"))?
                    .clone(),
            );
        } else if let Some(attach_type) = flag.mark_attachment_type_mask() {
            filter.mark_filter = Some(
                self.marks_by_class(attach_type)
                    .ok_or(ReadError::MalformedData("invalid mark attachment type"))?
                    .clone(),
            );
        }
        Ok(filter)
    }
}

/// GPOS.
impl RaiseContext {
    pub fn raise_gpos(&self, gpos: &Gpos) -> Result<Vec<PositionFeature>, ReadError> {
        let mut features = vec![];
        let script_list = gpos.script_list()?;
        let feature_list = gpos.feature_list()?;
        let lookup_list = gpos.lookup_list()?;
        for script in script_list.script_records() {
            let script_tag = script.script_tag();
            let script = script.script(script_list.offset_data())?;
            for (lang_tag, lang) in script
                .lang_sys_records()
                .iter()
                .map(|rec| (rec.lang_sys_tag(), rec.lang_sys(script.offset_data()).ok()))
                .chain(
                    script
                        .default_lang_sys()
                        .transpose()
                        .ok()
                        .flatten()
                        .map(|lang| (Tag::new(b"DFLT"), Some(lang))),
                )
            {
                let Some(lang) = lang else {
                    continue;
                };
                for feature_ix in lang.feature_indices() {
                    let feature_ix = feature_ix.get() as usize;
                    let feature = feature_list
                        .feature_records()
                        .get(feature_ix)
                        .ok_or(ReadError::OutOfBounds)?;
                    let feature_tag = feature.feature_tag();
                    let feature = feature.feature(feature_list.offset_data())?;
                    let mut lookup_indices = feature
                        .lookup_list_indices()
                        .iter()
                        .map(|ix| ix.get() as usize)
                        .collect::<Vec<_>>();
                    lookup_indices.sort();
                    let mut feature = PositionFeature::default();
                    feature.script = script_tag;
                    feature.language = lang_tag;
                    feature.feature = feature_tag;
                    for &lookup_ix in &lookup_indices {
                        let lookup = lookup_list.lookups().get(lookup_ix)?;
                        match lookup {
                            PositionLookup::Single(lookup) => {
                                let mut group = ActionGroup::default();
                                group.filter = self.raise_lookup_flag(
                                    lookup.lookup_flag(),
                                    lookup.mark_filtering_set(),
                                )?;
                                // for subtable in lookup.subtables().iter().filter_map(|s| s.ok()) {
                                //     let action = self.raise_mark_to_base(&subtable)?;
                                //     group.actions.push(PositionAction::MarkAttachment(action));
                                // }
                                feature.action_groups.push(group);
                            }
                            PositionLookup::Pair(lookup) => {
                                let mut group = ActionGroup::default();
                                group.filter = self.raise_lookup_flag(
                                    lookup.lookup_flag(),
                                    lookup.mark_filtering_set(),
                                )?;
                                // for subtable in lookup.subtables().iter().filter_map(|s| s.ok()) {
                                //     let action = self.raise_mark_to_base(&subtable)?;
                                //     group.actions.push(PositionAction::MarkAttachment(action));
                                // }
                                feature.action_groups.push(group);
                            }
                            PositionLookup::MarkToBase(lookup) => {
                                let mut group = ActionGroup::default();
                                group.filter = self.raise_lookup_flag(
                                    lookup.lookup_flag(),
                                    lookup.mark_filtering_set(),
                                )?;
                                for subtable in lookup.subtables().iter().filter_map(|s| s.ok()) {
                                    let action = self.raise_mark_to_base(&subtable)?;
                                    group.actions.push(PositionAction::MarkAttachment(action));
                                }
                                feature.action_groups.push(group);
                            }
                            PositionLookup::MarkToMark(lookup) => {
                                let mut group = ActionGroup::default();
                                group.filter = self.raise_lookup_flag(
                                    lookup.lookup_flag(),
                                    lookup.mark_filtering_set(),
                                )?;
                                for subtable in lookup.subtables().iter().filter_map(|s| s.ok()) {
                                    let action = self.raise_mark_to_mark(&subtable)?;
                                    group.actions.push(PositionAction::MarkAttachment(action));
                                }
                                feature.action_groups.push(group);
                            }
                            _ => {}
                        }
                    }
                    features.push(feature);
                }
            }
        }
        Ok(features)
    }

    pub fn raise_mark_to_base(
        &self,
        subtable: &MarkBasePosFormat1,
    ) -> Result<MarkAttachmentAction, ReadError> {
        let mut res = MarkAttachmentAction::default();
        let base_array = subtable.base_array()?;
        let base_records = base_array.base_records();
        let mark_array = subtable.mark_array()?;
        let mark_records = mark_array.mark_records();
        let cov_ix_to_mark_glyph: HashMap<usize, GlyphId> =
            subtable.mark_coverage()?.iter().enumerate().collect();
        for (base_ix, base_glyph) in subtable.base_coverage()?.iter().enumerate() {
            let base_record = base_records.get(base_ix)?;
            for (base_anchor_ix, base_anchor) in base_record
                .base_anchors(base_array.offset_data())
                .iter()
                .enumerate()
            {
                let Some(base_anchor) = base_anchor else {
                    continue;
                };
                let base_anchor = base_anchor?;
                let base_anchor = self.raise_anchor(&base_anchor)?;
                let mut group = GroupedMarkAttachment {
                    base: base_glyph,
                    base_anchor: base_anchor,
                    marks: Default::default(),
                };
                let mut marks: BTreeMap<Anchor, BTreeSet<GlyphId>> = Default::default();
                for (mark_ix, mark_record) in mark_records.iter().enumerate() {
                    let mark_class = mark_record.mark_class() as usize;
                    if mark_class != base_anchor_ix {
                        continue;
                    }
                    let Some(mark_glyph) = cov_ix_to_mark_glyph.get(&mark_ix) else {
                        continue;
                    };
                    let mark_anchor = mark_record.mark_anchor(mark_array.offset_data())?;
                    let mark_anchor = self.raise_anchor(&mark_anchor)?;
                    marks.entry(mark_anchor).or_default().insert(*mark_glyph);
                }
                if !marks.is_empty() {
                    group.marks.append(&mut marks);
                    res.groups.push(group);
                }
            }
        }
        Ok(res)
    }

    pub fn raise_mark_to_mark(
        &self,
        subtable: &MarkMarkPosFormat1,
    ) -> Result<MarkAttachmentAction, ReadError> {
        let mut res = MarkAttachmentAction::default();
        let base_array = subtable.mark2_array()?;
        let base_records = base_array.mark2_records();
        let mark_array = subtable.mark1_array()?;
        let mark_records = mark_array.mark_records();
        let cov_ix_to_mark_glyph: HashMap<usize, GlyphId> =
            subtable.mark1_coverage()?.iter().enumerate().collect();
        for (base_ix, base_glyph) in subtable.mark2_coverage()?.iter().enumerate() {
            let base_record = base_records.get(base_ix)?;
            for (base_anchor_ix, base_anchor) in base_record
                .mark2_anchors(base_array.offset_data())
                .iter()
                .enumerate()
            {
                let Some(base_anchor) = base_anchor else {
                    continue;
                };
                let base_anchor = base_anchor?;
                let base_anchor = self.raise_anchor(&base_anchor)?;
                let mut group = GroupedMarkAttachment {
                    base: base_glyph,
                    base_anchor: base_anchor,
                    marks: Default::default(),
                };
                let mut marks: BTreeMap<Anchor, BTreeSet<GlyphId>> = Default::default();
                for (mark_ix, mark_record) in mark_records.iter().enumerate() {
                    let mark_class = mark_record.mark_class() as usize;
                    if mark_class != base_anchor_ix {
                        continue;
                    }
                    let Some(mark_glyph) = cov_ix_to_mark_glyph.get(&mark_ix) else {
                        continue;
                    };
                    let mark_anchor = mark_record.mark_anchor(mark_array.offset_data())?;
                    let mark_anchor = self.raise_anchor(&mark_anchor)?;
                    marks.entry(mark_anchor).or_default().insert(*mark_glyph);
                }
                if !marks.is_empty() {
                    group.marks.append(&mut marks);
                    res.groups.push(group);
                }
            }
        }
        Ok(res)
    }
}

pub fn master_locations(ivs: &ItemVariationStore) -> Result<MasterLocations, ReadError> {
    let mut locations = vec![];
    let region_list = ivs.variation_region_list()?;
    for region in region_list.variation_regions().iter() {
        let region = region?;
        locations.push(
            region
                .region_axes()
                .iter()
                .map(|axis| axis.peak_coord())
                .collect(),
        );
    }
    Ok(locations)
}
