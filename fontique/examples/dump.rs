fn main() {
    use fontique::*;
    let start = std::time::Instant::now();
    let mut fonts = Collection::new(Default::default());
    let load_time = start.elapsed();
    // let fam = fonts.fallback_families(b"Latn").next().unwrap();
    // let _ = fonts
    //     .family(fam)
    //     .unwrap()
    //     .match_font(Stretch::NORMAL, Style::Normal, Weight::SEMI_BOLD, true)
    //     .unwrap();
    // let font_select_time = start.elapsed();
    println!("init: {:?}", load_time);
    // println!("init + select latin bold: {:?}", font_select_time);
    println!("=====================================");

    // let sysui = fonts
    //     .generic_families(GenericFamily::Monospace)
    //     .next()
    //     .unwrap();
    // let sysui = fonts.family(sysui).unwrap();
    // println!("{}", sysui.name());
    // let stretch = Stretch::SEMI_CONDENSED;
    // let style = Style::Italic;
    // let weight = Weight::LIGHT;
    // let sysui_font = sysui.match_font(stretch, style, weight, true).unwrap();
    // let synth = sysui_font.synthesis(stretch, style, weight);
    // println!("{sysui_font:?}");
    // println!("{synth:?}");

    let mut results = vec![];
    let han_locales = ["zh-Hans", "zh-Hant", "zh-Hant-HK", "ja", "ko", "zh-tw"];
    for (script, _) in Script::all_samples() {
        if script.0 == *b"Hani" {
            for &locale in &han_locales {
                let family = fonts.fallback_families((*script, locale)).next();
                if let Some(family) = family {
                    results.push((
                        *script,
                        Some(locale),
                        fonts.family_name(family).unwrap().to_string(),
                    ));
                } else {
                    results.push((*script, Some(locale), "----".to_owned()));
                }
            }
        } else {
            let family = fonts.fallback_families(*script).next();
            if let Some(family) = family {
                results.push((
                    *script,
                    None,
                    fonts.family_name(family).unwrap().to_string(),
                ));
            } else {
                results.push((*script, None, "----".to_owned()));
            }
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));

    for result in &results {
        if let Some(locale) = result.1 {
            println!("[{:?} {:?}] {}", result.0, locale, result.2);
        } else {
            println!("[{:?}] {}", result.0, result.2);
        }
    }

    let mut family_names = fonts
        .family_names()
        .map(|name| name.to_string())
        .collect::<Vec<_>>();
    family_names.sort();
    let mut count = 0;
    for family_name in &family_names {
        let family_id = fonts.family_id(family_name).unwrap();
        let Some(family) = fonts.family(family_id) else {
            continue;
        };
        dump_family(&family);
        count += family.fonts().len();
    }
    println!("{count} total fonts");
    for &family in GenericFamily::all() {
        print!("[{:?}]: ", family);
        let ids = fonts.generic_families(family).collect::<Vec<_>>();
        let names = ids
            .iter()
            .filter_map(|id| fonts.family_name(*id).map(|s| s.to_string()))
            .collect::<Vec<_>>();
        println!("{:?}", names);
    }
    return;
}

fn dump_family(family: &fontique::FamilyInfo) {
    println!("[{}]", family.name());
    let default_font = family.default_font().unwrap();
    for font in family.fonts() {
        if font.source().id() == default_font.source().id() && font.index() == default_font.index()
        {
            print!("*");
        } else {
            print!(" ")
        }
        println!(" {:?}", font);
    }
}
