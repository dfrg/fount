use super::cache_parser::*;
use std::io::Read;
use std::path::PathBuf;

#[derive(Default)]
pub struct CachedFont {
    pub family: String,
    pub style: String,
    pub path: PathBuf,
    pub index: u32,
    pub coverage: Coverage,
}

impl CachedFont {
    fn clear(&mut self) {
        self.family.clear();
        self.style.clear();
        self.path.clear();
        self.index = 0;
        self.coverage.clear();
    }
}

pub fn parse_caches(paths: &[PathBuf], mut f: impl FnMut(&CachedFont)) {
    let mut buffer = vec![];
    let mut cached_font = CachedFont::default();
    for path in paths {
        let Ok(dir) = path.canonicalize().and_then(std::fs::read_dir) else {
            return;
        };
        for path in dir.filter_map(|entry| entry.ok()).map(|entry| entry.path()) {
            buffer.clear();
            let Ok(file_size) = path.metadata() else {
                continue;
            };
            buffer.resize(file_size.len() as usize, 0);
            let Ok(mut file) = std::fs::OpenOptions::new().read(true).open(&path) else {
                continue;
            };
            let Ok(_) = file.read(&mut buffer) else {
                continue;
            };
            let Ok(set) = Cache::from_bytes(&buffer).and_then(|cache| cache.set()) else {
                continue;
            };
            let Ok(fonts) = set.fonts() else { continue };
            for font in fonts.flatten() {
                if parse_font(&font, &mut cached_font).is_some() {
                    f(&cached_font);
                }
            }
        }
    }
}

fn parse_font(pattern: &Pattern, font: &mut CachedFont) -> Option<()> {
    font.clear();
    for elt in pattern.elts().ok()? {
        let Ok(obj) = elt.object() else {
            continue;
        };
        match obj {
            Object::Family => {
                for val in elt.values().ok()? {
                    let val = val.ok()?;
                    if let Value::String(s) = val {
                        font.family.clear();
                        font.family
                            .push_str(core::str::from_utf8(s.str().ok()?).ok()?);
                    }
                }
            }
            Object::Style => {
                for val in elt.values().ok()? {
                    let val = val.ok()?;
                    if let Value::String(s) = val {
                        font.style.clear();
                        font.style
                            .push_str(core::str::from_utf8(s.str().ok()?).ok()?);
                    }
                }
            }
            Object::File => {
                for val in elt.values().ok()? {
                    let val = val.ok()?;
                    if let Value::String(s) = val {
                        font.path.clear();
                        font.path.push(core::str::from_utf8(s.str().ok()?).ok()?);
                        if font.path.extension() == Some(std::ffi::OsStr::new("t1")) {
                            return None;
                        }
                    }
                }
            }
            Object::Index => {
                for val in elt.values().ok()? {
                    let val = val.ok()?;
                    if let Value::Int(i) = val {
                        font.index = i as u32;
                        // Ignore named instances
                        if font.index >> 16 != 0 {
                            return None;
                        }
                    }
                }
            }
            Object::CharSet => {
                for val in elt.values().ok()? {
                    let val = val.ok()?;
                    if let Value::CharSet(set) = val {
                        font.coverage.clear();
                        font.coverage
                            .numbers
                            .extend_from_slice(set.numbers().ok()?.as_slice().ok()?);
                        for leaf in set.leaves().ok()? {
                            let leaf = leaf.ok()?;
                            font.coverage
                                .leaves
                                .push(unsafe { core::mem::transmute(leaf) });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if !font.family.is_empty() && !font.path.as_os_str().is_empty() {
        Some(())
    } else {
        None
    }
}

#[derive(Clone, Default)]
pub struct Coverage {
    numbers: Vec<u16>,
    leaves: Vec<[u32; 8]>,
}

impl Coverage {
    pub fn compute_for_str(&self, s: &str) -> usize {
        s.chars()
            .map(|ch| self.contains(ch as _).unwrap_or(false) as usize)
            .sum()
    }

    pub fn contains(&self, ch: u32) -> Option<bool> {
        let hi = ((ch >> 8) & 0xffff) as u16;
        match self.numbers.binary_search(&hi) {
            // The unwrap will succeed because numbers and leaves have the same length.
            Ok(idx) => {
                let leaf = self.leaves.get(idx)?;
                let lo = (ch & 0xff) as u8;
                let map_idx = (lo >> 5) as usize;
                let bit_idx = (lo & 0x1f) as u32;
                Some((leaf[map_idx] >> bit_idx) & 1 != 0)
            }
            Err(_) => Some(false),
        }
    }

    fn clear(&mut self) {
        self.numbers.clear();
        self.leaves.clear();
    }
}
