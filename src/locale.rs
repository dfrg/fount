
/// Language with an optional script and region that identifies a locale.
#[derive(Copy, Clone)]
pub struct Locale {
    language: [u8; 3],
    language_len: u8,
    script: [u8; 4],
    region: [u8; 3],
    region_len: u8,
    cjk: Cjk,
}

impl Locale {
    /// Parses a locale from a BCP 47 language tag.
    pub fn new(tag: &str) -> Option<Self> {
        let mut locale = Self {
            language: [0; 3],
            language_len: 0,
            script: [0; 4],
            region: [0; 3],
            region_len: 0,
            cjk: Cjk::None,
        };
        let mut has_region = false;
        let mut zh = false;
        for (i, part) in tag.split('-').enumerate() {
            let bytes = part.as_bytes();
            let len = bytes.len();
            match i {
                0 => {
                    match len {
                        2 => {
                            let a = bytes[0];
                            let b = bytes[1];
                            if a.is_ascii_lowercase() && b.is_ascii_lowercase() {
                                match (a, b) {
                                    (b'z', b'h') => zh = true,
                                    (b'j', b'a') => locale.cjk = Cjk::Japanese,
                                    (b'k', b'o') => locale.cjk = Cjk::Korean,
                                    _ => {},
                                };
                                locale.language[0] = a;
                                locale.language[1] = b;
                                locale.language_len = 2;
                            }
                        }
                        3 => {
                            let a = bytes[0];
                            let b = bytes[1];
                            let c = bytes[2];
                            if a.is_ascii_lowercase() && b.is_ascii_lowercase() && c.is_ascii_lowercase() {
                                zh = a == b'z' && b == b'h' && c == b'o';
                                locale.language[0] = a;
                                locale.language[1] = b;
                                locale.language[2] = c;
                                locale.language_len = 3;
                            }
                        }
                        _ => return None,
                    };
                }
                1 => match len {
                    2 => {
                        let a = bytes[0].to_ascii_uppercase();
                        let b = bytes[1].to_ascii_uppercase();
                        if a.is_ascii_uppercase() && b.is_ascii_uppercase() {
                            locale.region[0] = a;
                            locale.region[1] = b;
                            locale.region_len = 2;
                            has_region = true;
                        }
                    }
                    3 => {
                        let a = bytes[0];
                        let b = bytes[1];
                        let c = bytes[2];
                        if a.is_ascii_digit() && b.is_ascii_digit() && c.is_ascii_digit() {
                            locale.region[0] = a;
                            locale.region[1] = b;
                            locale.region[2] = c;
                            locale.region_len = 3;
                            has_region = true;
                        }
                    }
                    4 => {
                        let a = bytes[0];
                        let b = bytes[1];
                        let c = bytes[2];
                        let d = bytes[3];
                        if a.is_ascii_uppercase() && b.is_ascii_lowercase() && c.is_ascii_uppercase() && d.is_ascii_lowercase() {
                            locale.script[0] = a;
                            locale.script[1] = b;
                            locale.script[2] = c;
                            locale.script[3] = d;
                        }                        
                    }
                    _ => break,
                },
                2 => {
                    if has_region || len != 2 {
                        break;
                    }
                    let a = bytes[0].to_ascii_uppercase();
                    let b = bytes[1].to_ascii_uppercase();
                    if a.is_ascii_uppercase() && b.is_ascii_uppercase() {
                        locale.region[0] = a;
                        locale.region[1] = b;
                        locale.region_len = 2;
                        has_region = true;
                    }                    
                }
                _ => break,
            }
        }
        if zh {
            locale.cjk = match locale.script().unwrap_or("") {
                "Hans" => Cjk::Simplified,
                _ => Cjk::Traditional,
            };
        }
        Some(locale)
    }

    /// Returns the language component.
    pub fn language(&self) -> &str {
        core::str::from_utf8(&self.language[..self.language_len as usize]).unwrap_or("")
    }

    /// Returns the script component.
    pub fn script(&self) -> Option<&str> {
        if self.script[0] != 0 {
            core::str::from_utf8(&self.script).ok()
        } else {
            None
        }
    }

    /// Returns the region component.
    pub fn region(&self) -> Option<&str> {
        if self.region[0] != 0 {
            core::str::from_utf8(&self.region).ok()
        } else {
            None
        }
    }   
    
    pub(crate) fn cjk(&self) -> Cjk {
        self.cjk
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Cjk {
    None = 0,
    Simplified = 1,
    Traditional = 2,
    Japanese = 3,
    Korean = 4,
}
