use read_fonts::{
    tables::name::{CharIter, Name, NameRecord},
    TableProvider,
};

/// Identifier for a localized string.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct LocalizedStringId(pub u16);

impl LocalizedStringId {
    /// Copyright notice.
    pub const COPYRIGHT_NOTICE: Self = Self(0);
    /// Family name.
    pub const FAMILY_NAME: Self = Self(1);
    /// Subfamily name.
    pub const SUBFAMILY_NAME: Self = Self(2);
    /// Unique identifier.
    pub const UNIQUE_ID: Self = Self(3);
    /// Full name.
    pub const FULL_NAME: Self = Self(4);
    /// Version string.
    pub const VERSION_STRING: Self = Self(5);
    /// PostScript name.
    pub const POSTSCRIPT_NAME: Self = Self(6);
    /// Trademark.
    pub const TRADEMARK: Self = Self(7);
    /// Manufacturer name.
    pub const MANUFACTURER: Self = Self(8);
    /// Designer name.
    pub const DESIGNER: Self = Self(9);
    /// Description of the typeface.
    pub const DESCRIPTION: Self = Self(10);
    /// URL of the font vendor.
    pub const VENDOR_URL: Self = Self(11);
    /// URL of the font designer.
    pub const DESIGNER_URL: Self = Self(12);
    /// License description.
    pub const LICENSE_DESCRIPTION: Self = Self(13);
    /// URL where additional licensing information can be found.
    pub const LICENSE_URL: Self = Self(14);
    /// Typographic family name.
    pub const TYPOGRAPHIC_FAMILY_NAME: Self = Self(16);
    /// Typographic subfamily name.
    pub const TYPOGRAPHIC_SUBFAMILY_NAME: Self = Self(17);
    /// Compatible full name (Macintosh only).
    pub const COMPATIBLE_FULL_NAME: Self = Self(18);
    /// Sample text.
    pub const SAMPLE_TEXT: Self = Self(19);
    /// PostScript CID findfont name.
    pub const POSTSCRIPT_CID_NAME: Self = Self(20);
    /// WWS family name.
    pub const WWS_FAMILY_NAME: Self = Self(21);
    /// WWS subfamily name.
    pub const WWS_SUBFAMILY_NAME: Self = Self(22);
    /// Light background palette name.
    pub const LIGHT_BACKGROUND_PALETTE: Self = Self(23);
    /// Dark background palette name.
    pub const DARK_BACKGROUND_PALETTE: Self = Self(24);
    /// Variations PostScript name prefix.
    pub const VARIATIONS_POSTSCRIPT_NAME_PREFIX: Self = Self(25);
}

/// String describing some font metadata in a specific language.
#[derive(Clone)]
pub struct LocalizedString<'a> {
    name: Name<'a>,
    record: NameRecord,
}

impl<'a> LocalizedString<'a> {
    /// Returns the identifier.
    pub fn id(&self) -> LocalizedStringId {
        LocalizedStringId(self.record.name_id())
    }

    /// Returns the locale for this string.
    pub fn locale(&self) -> Option<&str> {
        get_locale(self.record.platform_id(), self.record.language_id())
    }

    /// Returns an iterator over the characters of the string.
    pub fn chars(&self) -> impl Iterator<Item = char> + 'a {
        let inner = self
            .record
            .string(self.name.string_data())
            .ok()
            .map(|name_string| name_string.chars());
        Chars { inner }
    }
}

struct Chars<'a> {
    inner: Option<CharIter<'a>>,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next()
    }
}

/// Collection of localized strings.
#[derive(Clone)]
pub struct LocalizedStringCollection<'a> {
    name: Option<Name<'a>>,
}

impl<'a> LocalizedStringCollection<'a> {
    /// Creates a new localized string collection from the specified table provider.
    pub fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            name: font.name().ok(),
        }
    }

    /// Returns the number of strings in the collection.
    pub fn len(&self) -> usize {
        self.name
            .as_ref()
            .map(|name| name.count() as usize)
            .unwrap_or_default()
    }

    /// Returns true if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the localized string at the specified index.
    pub fn get(&self, index: usize) -> Option<LocalizedString<'a>> {
        let name = self.name.clone()?;
        let record = name.name_record().get(index)?.clone();
        Some(LocalizedString { name, record })
    }

    /// Returns an iterator over the localized strings in the collection.
    pub fn iter(&self) -> impl Iterator<Item = LocalizedString<'a>> + 'a + Clone {
        self.clone().into_iter()
    }
}

#[derive(Clone)]
pub struct LocalizedStringIter<'a> {
    strings: LocalizedStringCollection<'a>,
    pos: usize,
}

impl<'a> Iterator for LocalizedStringIter<'a> {
    type Item = LocalizedString<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.strings.get(pos)
    }
}

impl<'a> IntoIterator for LocalizedStringCollection<'a> {
    type IntoIter = LocalizedStringIter<'a>;
    type Item = LocalizedString<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LocalizedStringIter {
            strings: self,
            pos: 0,
        }
    }
}

#[rustfmt::skip]
const LOCALE_MAP: [(u32, &str); 334] = [
    (0x10000, "en"), (0x10001, "fr"), (0x10002, "de"), (0x10003, "it"), (0x10004, "nl"),
    (0x10005, "sv"), (0x10006, "es"), (0x10007, "da"), (0x10008, "pt"), (0x10009, "no"),
    (0x1000A, "he"), (0x1000B, "ja"), (0x1000C, "ar"), (0x1000D, "fi"), (0x1000E, "el"),
    (0x1000F, "is"), (0x10010, "mt"), (0x10011, "tr"), (0x10012, "hr"), (0x10013, "zh-tw"),
    (0x10014, "ur"), (0x10015, "hi"), (0x10016, "th"), (0x10017, "ko"), (0x10018, "lt"),
    (0x10019, "pl"), (0x1001A, "hu"), (0x1001B, "et"), (0x1001C, "lv"), (0x1001E, "fo"),
    (0x1001F, "fa"), (0x10020, "ru"), (0x10021, "zh-cn"), (0x10022, "nl"), (0x10023, "ga"),
    (0x10024, "sq"), (0x10025, "ro"), (0x10026, "cs"), (0x10027, "sk"), (0x10028, "sl"),
    (0x10029, "yi"), (0x1002A, "sr"), (0x1002B, "mk"), (0x1002C, "bg"), (0x1002D, "uk"),
    (0x1002E, "be"), (0x1002F, "uz"), (0x10030, "kk"), (0x10031, "az"), (0x10031, "az"),
    (0x10032, "ar"), (0x10033, "hy"), (0x10034, "ka"), (0x10035, "mo"), (0x10036, "ky"),
    (0x10037, "tg"), (0x10038, "tk"), (0x10039, "mn"), (0x10039, "mn"), (0x1003A, "mn"),
    (0x1003B, "ps"), (0x1003C, "ku"), (0x1003D, "ks"), (0x1003E, "sd"), (0x1003F, "bo"),
    (0x10040, "ne"), (0x10041, "sa"), (0x10042, "mr"), (0x10043, "bn"), (0x10044, "as"),
    (0x10045, "gu"), (0x10046, "pa"), (0x10047, "or"), (0x10048, "ml"), (0x10049, "kn"),
    (0x1004A, "ta"), (0x1004B, "te"), (0x1004C, "si"), (0x1004D, "my"), (0x1004E, "km"),
    (0x1004F, "lo"), (0x10050, "vi"), (0x10051, "id"), (0x10052, "tl"), (0x10053, "ms"),
    (0x10054, "ms"), (0x10055, "am"), (0x10056, "ti"), (0x10057, "om"), (0x10058, "so"),
    (0x10059, "sw"), (0x1005A, "rw"), (0x1005B, "rn"), (0x1005C, "ny"), (0x1005D, "mg"),
    (0x1005E, "eo"), (0x10080, "cy"), (0x10081, "eu"), (0x10082, "ca"), (0x10083, "la"),
    (0x10084, "qu"), (0x10085, "gn"), (0x10086, "ay"), (0x10087, "tt"), (0x10088, "ug"),
    (0x10089, "dz"), (0x1008A, "jw"), (0x1008B, "su"), (0x1008C, "gl"), (0x1008D, "af"),
    (0x1008E, "br"), (0x1008F, "iu"), (0x10090, "gd"), (0x10091, "gv"), (0x10092, "ga"),
    (0x10093, "to"), (0x10094, "el"), (0x10095, "ik"), (0x10096, "az"), (0x30001, "ar"),
    (0x30004, "zh"), (0x30009, "en"), (0x30401, "ar"), (0x30402, "bg"), (0x30403, "ca"),
    (0x30404, "zh-tw"), (0x30405, "cs"), (0x30406, "da"), (0x30407, "de"), (0x30408, "el"),
    (0x30409, "en"), (0x3040A, "es"), (0x3040B, "fi"), (0x3040C, "fr"), (0x3040D, "he"),
    (0x3040E, "hu"), (0x3040F, "is"), (0x30410, "it"), (0x30411, "ja"), (0x30412, "ko"),
    (0x30413, "nl"), (0x30414, "no"), (0x30415, "pl"), (0x30416, "pt"), (0x30417, "rm"),
    (0x30418, "ro"), (0x30419, "ru"), (0x3041A, "hr"), (0x3041B, "sk"), (0x3041C, "sq"),
    (0x3041D, "sv"), (0x3041E, "th"), (0x3041F, "tr"), (0x30420, "ur"), (0x30421, "id"),
    (0x30422, "uk"), (0x30423, "be"), (0x30424, "sl"), (0x30425, "et"), (0x30426, "lv"),
    (0x30427, "lt"), (0x30428, "tg"), (0x30429, "fa"), (0x3042A, "vi"), (0x3042B, "hy"),
    (0x3042C, "az"), (0x3042D, "eu"), (0x3042E, "wen"), (0x3042F, "mk"), (0x30430, "st"),
    (0x30431, "ts"), (0x30432, "tn"), (0x30433, "ven"), (0x30434, "xh"), (0x30435, "zu"),
    (0x30436, "af"), (0x30437, "ka"), (0x30438, "fo"), (0x30439, "hi"), (0x3043A, "mt"),
    (0x3043B, "se"), (0x3043C, "ga"), (0x3043D, "yi"), (0x3043E, "ms"), (0x3043F, "kk"),
    (0x30440, "ky"), (0x30441, "sw"), (0x30442, "tk"), (0x30443, "uz"), (0x30444, "tt"),
    (0x30445, "bn"), (0x30446, "pa"), (0x30447, "gu"), (0x30448, "or"), (0x30449, "ta"),
    (0x3044A, "te"), (0x3044B, "kn"), (0x3044C, "ml"), (0x3044D, "as"), (0x3044E, "mr"),
    (0x3044F, "sa"), (0x30450, "mn"), (0x30451, "bo"), (0x30452, "cy"), (0x30453, "km"),
    (0x30454, "lo"), (0x30455, "my"), (0x30456, "gl"), (0x30457, "kok"), (0x30458, "mni"),
    (0x30459, "sd"), (0x3045A, "syr"), (0x3045B, "si"), (0x3045C, "chr"), (0x3045D, "iu"),
    (0x3045E, "am"), (0x30460, "ks"), (0x30461, "ne"), (0x30462, "fy"), (0x30463, "ps"),
    (0x30464, "phi"), (0x30465, "div"), (0x30468, "ha"), (0x3046A, "yo"), (0x30470, "ibo"),
    (0x30471, "kau"), (0x30472, "om"), (0x30473, "ti"), (0x30474, "gn"), (0x30475, "haw"),
    (0x30476, "la"), (0x30477, "so"), (0x30479, "pap"), (0x30481, "mi"), (0x30801, "ar"),
    (0x30804, "zh-cn"), (0x30807, "de"), (0x30809, "en"), (0x3080A, "es"), (0x3080C, "fr"),
    (0x30810, "it"), (0x30812, "ko"), (0x30813, "nl"), (0x30814, "nn"), (0x30816, "pt"),
    (0x30818, "mo"), (0x30819, "ru"), (0x3081A, "sr"), (0x3081D, "sv"), (0x30820, "ur"),
    (0x30827, "lt"), (0x3082C, "az"), (0x3083C, "gd"), (0x3083E, "ms"), (0x30843, "uz"),
    (0x30845, "bn"), (0x30846, "ar"), (0x30850, "mn"), (0x30851, "bo"), (0x30851, "dz"),
    (0x30860, "ks"), (0x30861, "ne"), (0x30873, "ti"), (0x30C01, "ar"), (0x30C04, "zh-hk"),
    (0x30C07, "de"), (0x30C09, "en"), (0x30C0A, "es"), (0x30C0C, "fr"), (0x30C1A, "sr"),
    (0x31001, "ar"), (0x31004, "zh-sg"), (0x31007, "de"), (0x31009, "en"), (0x3100A, "es"),
    (0x3100C, "fr"), (0x31401, "ar"), (0x31404, "zh-mo"), (0x31407, "de"), (0x31409, "en"),
    (0x3140A, "es"), (0x3140C, "fr"), (0x3141A, "bs"), (0x31801, "ar"), (0x31809, "en"),
    (0x3180A, "es"), (0x3180C, "fr"), (0x31C01, "ar"), (0x31C09, "en"), (0x31C0A, "es"),
    (0x31C0C, "fr"), (0x32001, "ar"), (0x32009, "en"), (0x3200A, "es"), (0x3200C, "fr"),
    (0x32401, "ar"), (0x32409, "en"), (0x3240A, "es"), (0x3240C, "fr"), (0x32801, "ar"),
    (0x32809, "en"), (0x3280A, "es"), (0x3280C, "fr"), (0x32C01, "ar"), (0x32C09, "en"),
    (0x32C0A, "es"), (0x32C0C, "fr"), (0x33001, "ar"), (0x33009, "en"), (0x3300A, "es"),
    (0x3300C, "fr"), (0x33401, "ar"), (0x33409, "en"), (0x3340A, "es"), (0x3340C, "fr"),
    (0x33801, "ar"), (0x3380A, "es"), (0x3380C, "fr"), (0x33C01, "ar"), (0x33C09, "en"),
    (0x33C0A, "es"), (0x33C0C, "fr"), (0x34001, "ar"), (0x34009, "en"), (0x3400A, "es"),
    (0x34409, "en"), (0x3440A, "es"), (0x34809, "en"), (0x3480A, "es"), (0x34C0A, "es"),
    (0x3500A, "es"), (0x3540A, "es"), (0x3E40A, "es"), (0x3E40C, "fr"),
];

fn get_locale(platform_id: u16, language_id: u16) -> Option<&'static str> {
    let key = (platform_id as u32) << 16 | language_id as u32;
    match LOCALE_MAP.binary_search_by(|x| x.0.cmp(&key)) {
        Ok(idx) => LOCALE_MAP.get(idx).map(|pair| pair.1),
        _ => None,
    }
}
