use layir::*;
use read_fonts::{
    tables::{
        gpos::{MarkBasePosFormat1, PositionLookup},
        layout::Lookup,
    },
    FontRef, ReadError, TableProvider,
};

fn main() {
    //let font_path = "c:/work/content/fonts/NotoRashiHebrew-Regular.ttf";
    let font_path = "c:/work/content/fonts/googlefonts/ofl/oswald/Oswald[wght].ttf";
    let font_data = std::fs::read(font_path).unwrap();
    let font = FontRef::new(&font_data).unwrap();
    let mark2base_lookups = collect_mark_to_base_lookups(&font).unwrap();
    let mut action = MarkAttachmentAction::default();
    let name_map = NameMap::new(&font).unwrap();
    let mut raise_cx = font
        .gdef()
        .and_then(|gdef| RaiseContext::new(&gdef, None))
        .unwrap_or_default();

    let mut layout = Layout::default();
    if let Ok(gsub) = font.gsub() {
        raise_cx.raise_gsub(&gsub, &mut layout).unwrap();
    }
    if let Ok(gpos) = font.gpos() {
        raise_cx.raise_gpos(&gpos, &mut layout).unwrap();
    }

    let dump = LayoutPrettyPrinter(&layout, &name_map).to_string();
    println!("{dump}");
    // println!("{:#?}", layout);
    return;

    // for feature in &pos_features {
    //     println!(
    //         "[{}/{}/{}]",
    //         feature.script, feature.language, feature.feature
    //     );
    //     for group in &feature.action_groups {
    //         println!("? {}", group.filter);
    //         for action in &group.actions {
    //             match action {
    //                 PositionAction::MarkAttachment(a) => {
    //                     for group in &a.groups {
    //                         print!("{group}");
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    return;

    // let ivs = font
    //     .gdef()
    //     .ok()
    //     .and_then(|gdef| gdef.item_var_store())
    //     .transpose()
    //     .ok()
    //     .flatten();
    // let locations = ivs.as_ref().map(|ivs| master_locations(ivs).ok()).flatten();
    // let vary = match (&locations, &ivs) {
    //     (Some(locations), Some(ivs)) => Some((locations, ivs)),
    //     _ => None,
    // };
    let mut mark2base = vec![];

    for lookup in &mark2base_lookups {
        for subtable in lookup.subtables().iter().filter_map(|t| t.ok()) {
            mark2base.push(raise_cx.raise_mark_to_base(&subtable).unwrap());
        }
    }
    // for group in &mark2base {
    //     action.merge_flattened(group.flatten());
    // }
    // let flattened = action.flatten().collect::<Vec<_>>();
    // let mut merged = MarkAttachmentAction::default();
    // merged.merge_flattened(flattened.iter().cloned());
    // assert_eq!(action, merged);

    // // for group in &action.attachments {
    // //     print!("{}", group);
    // // }
    // // println!("\n[flattened]\n");
    // // for attachment in action.flatten() {
    // //     print!("{}", attachment);
    // // }

    // let flat_subset = &flattened[40..=80];
    // let mut merged_subset = MarkAttachmentAction::default();
    // merged_subset.merge_flattened(flat_subset.iter().cloned());

    // for group in &merged_subset.groups {
    //     print!("{}", group);
    // }
    // println!("\n[flattened]\n");
    // for attachment in flat_subset {
    //     print!("{}", attachment);
    // }
    // println!("{:#?}", action);
    println!("Hello, world!");
}

fn collect_mark_to_base_lookups<'a>(
    font: &FontRef<'a>,
) -> Result<Vec<Lookup<'a, MarkBasePosFormat1<'a>>>, ReadError> {
    let mut lookups = vec![];
    let gpos = font.gpos()?;
    for lookup in gpos
        .lookup_list()?
        .lookups()
        .iter()
        .filter_map(|lookup| lookup.ok())
    {
        match lookup {
            PositionLookup::MarkToBase(lookup) => lookups.push(lookup),
            _ => {}
        }
    }
    Ok(lookups)
}
