use super::{bytecode::Definition, state::InstanceState, ScalerFont, Slot};
use crate::{scale::Hinting, FontKey, NormalizedCoord};

pub struct CacheEntry<'a, T> {
    pub is_current: bool,
    pub entry: &'a mut T,
}

#[derive(Clone, Debug)]
pub struct Cache {
    /// Cached font entries.
    fonts: Vec<FontEntry>,
    /// Cached font size entries.
    sizes: Vec<SizeEntry>,
    /// Counter for cache eviction.
    epoch: u64,
    /// Max cache size.
    max_entries: usize,
    /// Entry for an uncached font.
    uncached_font: FontEntry,
    /// Entry for an uncached font size.
    uncached_size: SizeEntry,
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            fonts: vec![],
            sizes: vec![],
            epoch: 0,
            max_entries: 8,
            uncached_font: Default::default(),
            uncached_size: Default::default(),
        }
    }
}

impl Cache {
    pub fn find_or_create_entries(
        &mut self,
        font: &ScalerFont,
        hinting: Hinting,
    ) -> (CacheEntry<FontEntry>, CacheEntry<SizeEntry>, Slot) {
        let epoch = self.epoch;
        self.epoch += 1;
        let (font_current, font_index) = self.find_font(font.key);
        let (size_current, size_index) =
            self.find_size(font.key, font.coords, font.scale.to_bits(), hinting);
        let font_entry = if font_index == !0 {
            &mut self.uncached_font
        } else {
            &mut self.fonts[font_index]
        };
        let size_entry = if size_index == !0 {
            &mut self.uncached_size
        } else {
            &mut self.sizes[size_index]
        };
        if !font_current {
            font_entry.key = font.key.unwrap_or_default();
            font_entry.epoch = epoch;
            font_entry.definitions.clear();
            font_entry.definitions.resize(
                font.max_function_defs as usize + font.max_instruction_defs as usize,
                Definition::default(),
            );
            font_entry.max_fdefs = font.max_function_defs as usize;
            font_entry.cvt_len = font.cvt.len();
        }
        font_entry.epoch = epoch;
        if !size_current {
            size_entry.key = font.key.unwrap_or_default();
            size_entry.state = InstanceState::default();
            size_entry.mode = hinting;
            size_entry.scale = font.scale.to_bits();
            size_entry.coords.clear();
            size_entry.coords.extend_from_slice(font.coords);
            let cvt_len = font.cvt.len();
            size_entry.store.clear();
            size_entry
                .store
                .resize(cvt_len + font.max_storage as usize, 0);
            font.scale_cvt(Some(font.scale.to_bits()), &mut size_entry.store);
        }
        size_entry.epoch = epoch;
        (
            CacheEntry {
                is_current: font_current,
                entry: font_entry,
            },
            CacheEntry {
                is_current: size_current,
                entry: size_entry,
            },
            if font.key.is_some() {
                Slot::Cached {
                    font_index,
                    size_index,
                }
            } else {
                Slot::Uncached
            },
        )
    }

    pub fn from_slot(&mut self, slot: Slot) -> (&mut FontEntry, &mut SizeEntry) {
        match slot {
            Slot::Uncached => (&mut self.uncached_font, &mut self.uncached_size),
            Slot::Cached {
                font_index,
                size_index,
            } => (&mut self.fonts[font_index], &mut self.sizes[size_index]),
        }
    }

    fn find_font(&mut self, font_id: Option<FontKey>) -> (bool, usize) {
        let font_id = match font_id {
            Some(font_id) => font_id,
            None => return (false, !0),
        };
        let mut lowest_epoch = self.epoch;
        let mut lowest_index = 0;
        for (i, font) in self.fonts.iter().enumerate() {
            if font.key == font_id {
                return (true, i);
            }
            if font.epoch < lowest_epoch {
                lowest_epoch = font.epoch;
                lowest_index = i;
            }
        }
        if self.fonts.len() < self.max_entries {
            lowest_index = self.fonts.len();
            self.fonts.push(FontEntry::default());
        }
        (false, lowest_index)
    }

    fn find_size(
        &mut self,
        font_id: Option<FontKey>,
        coords: &[NormalizedCoord],
        scale: i32,
        mode: Hinting,
    ) -> (bool, usize) {
        let font_id = match font_id {
            Some(font_id) => font_id,
            None => return (false, !0),
        };
        let mut lowest_epoch = self.epoch;
        let mut lowest_index = 0;
        let vary = !coords.is_empty();
        for (i, size) in self.sizes.iter().enumerate() {
            if size.epoch < lowest_epoch {
                lowest_epoch = size.epoch;
                lowest_index = i;
            }
            if size.key == font_id
                && size.scale == scale
                && size.mode == mode
                && (!vary || (coords == &size.coords[..]))
            {
                return (true, i);
            }
        }
        if self.sizes.len() < self.max_entries {
            lowest_index = self.sizes.len();
            self.sizes.push(SizeEntry::default());
        }
        (false, lowest_index)
    }
}

/// Entry for a cached font.
#[derive(Clone, Default, Debug)]
pub struct FontEntry {
    pub key: FontKey,
    pub definitions: Vec<Definition>,
    pub max_fdefs: usize,
    pub cvt_len: usize,
    pub epoch: u64,
}

/// Entry for a cached font size (and variation).
#[derive(Clone, Default, Debug)]
pub struct SizeEntry {
    pub key: FontKey,
    pub state: InstanceState,
    pub mode: Hinting,
    pub coords: Vec<NormalizedCoord>,
    pub scale: i32,
    pub store: Vec<i32>,
    pub epoch: u64,
}
