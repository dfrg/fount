/// Quick and cheesy itemization example to test fallback font selection
/// by script.

use fount::*;
use swash::text::{Codepoint as _, Script};

fn main() {
    let fcx = FontContext::new(&FontLibrary::default());
    let text = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    print_items(&text, None, &fcx);
}

fn print_items(s: &str, locale: Option<Locale>, fcx: &FontContext) {
    let items = itemize(s, locale, fcx);
    for (i, item) in items.iter().enumerate() {
        println!("{}: {}: {:?}\n  {:?}", i, &item.1, &item.2, &item.0);
    }
}

fn itemize(s: &str, locale: Option<Locale>, fcx: &FontContext) -> Vec<(String, String, Vec<String>)> {
    let mut items = Vec::new();
    let mut run = String::default();
    let mut last_script = s
        .chars()
        .map(|ch| ch.script())
        .find(|&script| real_script(script))
        .unwrap_or(Script::Latin);
    let mut chars = s.chars();
    let mut next_ch = chars.next();
    while let Some(ch) = next_ch {
        let script = ch.script();
        let is_real = real_script(script);
        let mut is_emoji = ch.is_extended_pictographic();
        if is_emoji || (script != last_script && is_real) {
            flush_run(&mut run, fcx, last_script, locale, &mut items);
            if is_emoji {
                run.push(ch);
                next_ch = chars.next();
                while is_emoji {
                    is_emoji = false;
                    while let Some(ch) = next_ch {
                        use swash::text::ClusterBreak::*;
                        match ch.cluster_break() {
                            EX => {
                                run.push(ch);
                                next_ch = chars.next();
                            }
                            ZWJ => {
                                run.push(ch);
                                next_ch = chars.next();
                                is_emoji = next_ch
                                    .map(|ch| ch.is_extended_pictographic())
                                    .unwrap_or(false);
                                if is_emoji {
                                    run.push(next_ch.unwrap());
                                    next_ch = chars.next();
                                }
                                break;
                            }
                            _ => {
                                next_ch = Some(ch);
                                break;
                            }
                        }
                    }
                }
                let family_names = fcx
                    .generic_families(GenericFontFamily::Emoji)
                    .iter()
                    .filter_map(|id| fcx.family(*id))
                    .map(|family| family.name().to_owned())
                    .collect::<Vec<_>>();
                items.push((run.clone(), "Emoji".into(), family_names));
                run.clear();
            } else {
                last_script = script;
            }
        } else {
            run.push(ch);
            next_ch = chars.next();
        }
    }
    flush_run(&mut run, fcx, last_script, locale, &mut items);
    items
}

fn flush_run(
    run: &mut String,
    fcx: &FontContext,
    script: Script,
    locale: Option<Locale>,
    items: &mut Vec<(String, String, Vec<String>)>,
) {
    if !run.is_empty() {
        let family_names = fcx
            .fallback_families(script, locale)
            .iter()
            .filter_map(|id| fcx.family(*id))
            .map(|family| family.name().to_owned())
            .collect::<Vec<_>>();
        items.push((run.clone(), format!("{:?}", script), family_names));
        run.clear();
    }
}

fn real_script(script: Script) -> bool {
    script != Script::Common && script != Script::Unknown && script != Script::Inherited
}
