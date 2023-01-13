use read_fonts::{
    tables::name::{CharIter, Name, NameRecord, NameString},
    TableProvider,
};

use core::fmt;

use crate::sequence::{SequenceData, SequenceElement};

/// Identifier for a localized string.
///
/// A set of pre-defined identifiers exist for accessing names and other various metadata
/// about the font and those are provided as associated constants on this type.
///
/// IDs 26 to 255, inclusive, are reserved for future standard names. IDs 256 to 32767,
/// inclusive, are reserved for font-specific names such as those referenced by a font’s
/// layout features.
///
/// For more detail, see <https://learn.microsoft.com/en-us/typography/opentype/spec/name#name-ids>
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct LocalizedStringId(pub u16);

impl LocalizedStringId {
    /// Copyright notice.
    pub const COPYRIGHT_NOTICE: Self = Self(0);

    /// Font Family name. The Font Family name is used in combination with Font Subfamily name (name ID 2),
    /// and should be shared among at most four fonts that differ only in weight or style (as described below).
    pub const FAMILY_NAME: Self = Self(1);

    /// Font Subfamily name. The Font Subfamily name is used in combination with Font Family name (name ID 1),
    /// and distinguishes the fonts in a group with the same Font Family name. This should be used for style
    /// and weight variants only (as described below).
    pub const SUBFAMILY_NAME: Self = Self(2);

    /// Unique font identifier.
    pub const UNIQUE_ID: Self = Self(3);

    /// Full font name that reflects all family and relevant subfamily descriptors. The full font name is generally
    /// a combination of name IDs 1 and 2, or of name IDs 16 and 17, or a similar human-readable variant.
    pub const FULL_NAME: Self = Self(4);

    /// Version string. Should begin with the syntax “Version number.number” (upper case, lower case, or mixed,
    /// with a space between “Version” and the number).
    pub const VERSION_STRING: Self = Self(5);

    /// PostScript name for the font; Name ID 6 specifies a string which is used to invoke a PostScript language font
    /// that corresponds to this OpenType font. When translated to ASCII, the name string must be no longer than 63
    /// characters and restricted to the printable ASCII subset, codes 33 to 126, except for the 10 characters '[', ']',
    /// '(', ')', '{', '}', '<', '>', '/', '%'.
    pub const POSTSCRIPT_NAME: Self = Self(6);

    /// Trademark; this is used to save any trademark notice/information for this font. Such information should be based
    /// on legal advice. This is distinctly separate from the copyright.
    pub const TRADEMARK: Self = Self(7);

    /// Manufacturer name.
    pub const MANUFACTURER: Self = Self(8);

    /// Designer; name of the designer of the typeface.
    pub const DESIGNER: Self = Self(9);

    /// Description; description of the typeface. Can contain revision information, usage recommendations, history, features, etc.
    pub const DESCRIPTION: Self = Self(10);

    /// URL Vendor; URL of font vendor (with protocol, e.g., http://, ftp://). If a unique serial number is embedded in the URL,
    /// it can be used to register the font.
    pub const VENDOR_URL: Self = Self(11);

    /// URL Designer; URL of typeface designer (with protocol, e.g., http://, ftp://).
    pub const DESIGNER_URL: Self = Self(12);

    /// License Description; description of how the font may be legally used, or different example scenarios for licensed use.
    /// This field should be written in plain language, not legalese.
    pub const LICENSE_DESCRIPTION: Self = Self(13);

    /// URL where additional licensing information can be found.
    pub const LICENSE_URL: Self = Self(14);

    /// Typographic Family name: The typographic family grouping doesn’t impose any constraints on the number of faces within it,
    /// in contrast with the 4-style family grouping (ID 1), which is present both for historical reasons and to express style
    /// linking groups.
    pub const TYPOGRAPHIC_FAMILY_NAME: Self = Self(16);

    /// Typographic Subfamily name: This allows font designers to specify a subfamily name within the typographic family grouping.
    /// This string must be unique within a particular typographic family.
    pub const TYPOGRAPHIC_SUBFAMILY_NAME: Self = Self(17);

    /// Compatible Full (Macintosh only); On the Macintosh, the menu name is constructed using the FOND resource. This usually matches
    /// the Full Name. If you want the name of the font to appear differently than the Full Name, you can insert the Compatible Full
    /// Name in ID 18.
    pub const COMPATIBLE_FULL_NAME: Self = Self(18);

    /// Sample text; This can be the font name, or any other text that the designer thinks is the best sample to display the font in.
    pub const SAMPLE_TEXT: Self = Self(19);

    /// PostScript CID findfont name; Its presence in a font means that the nameID 6 holds a PostScript font name that is meant to be
    /// used with the “composefont” invocation in order to invoke the font in a PostScript interpreter.
    pub const POSTSCRIPT_CID_NAME: Self = Self(20);

    /// WWS Family Name. Used to provide a WWS-conformant family name in case the entries for IDs 16 and 17 do not conform to the WWS model.
    pub const WWS_FAMILY_NAME: Self = Self(21);

    /// WWS Subfamily Name. Used in conjunction with ID 21, this ID provides a WWS-conformant subfamily name (reflecting only weight, width
    /// and slope attributes) in case the entries for IDs 16 and 17 do not conform to the WWS model.
    pub const WWS_SUBFAMILY_NAME: Self = Self(22);

    /// Light Background Palette. This ID, if used in the CPAL table’s Palette Labels Array, specifies that the corresponding color palette in
    /// the CPAL table is appropriate to use with the font when displaying it on a light background such as white.
    pub const LIGHT_BACKGROUND_PALETTE: Self = Self(23);

    /// Dark Background Palette. This ID, if used in the CPAL table’s Palette Labels Array, specifies that the corresponding color palette in
    /// the CPAL table is appropriate to use with the font when displaying it on a dark background such as black.
    pub const DARK_BACKGROUND_PALETTE: Self = Self(24);

    /// Variations PostScript Name Prefix. If present in a variable font, it may be used as the family prefix in the PostScript Name Generation
    /// for Variation Fonts algorithm.
    pub const VARIATIONS_POSTSCRIPT_NAME_PREFIX: Self = Self(25);
}

impl fmt::Debug for LocalizedStringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match *self {
            Self::COPYRIGHT_NOTICE => "COPYRIGHT_NOTICE",
            Self::FAMILY_NAME => "FAMILY_NAME",
            Self::SUBFAMILY_NAME => "SUBFAMILY_NAME",
            Self::UNIQUE_ID => "UNIQUE_ID",
            Self::FULL_NAME => "FULL_NAME",
            Self::VERSION_STRING => "VERSION_STRING",
            Self::POSTSCRIPT_NAME => "POSTSCRIPT_NAME",
            Self::TRADEMARK => "TRADEMARK",
            Self::MANUFACTURER => "MANUFACTURER",
            Self::DESIGNER => "DESIGNER",
            Self::DESCRIPTION => "DESCRIPTION",
            Self::VENDOR_URL => "VENDOR_URL",
            Self::DESIGNER_URL => "DESIGNER_URL",
            Self::LICENSE_DESCRIPTION => "LICENSE_DESCRIPTION",
            Self::LICENSE_URL => "LICENSE_URL",
            Self::TYPOGRAPHIC_FAMILY_NAME => "TYPOGRAPHIC_FAMILY_NAME",
            Self::TYPOGRAPHIC_SUBFAMILY_NAME => "TYPOGRAPHIC_SUBFAMILY_NAME",
            Self::COMPATIBLE_FULL_NAME => "COMPATIBLE_FULL_NAME",
            Self::SAMPLE_TEXT => "SAMPLE_TEXT",
            Self::POSTSCRIPT_CID_NAME => "POSTSCRIPT_CID_NAME",
            Self::WWS_FAMILY_NAME => "WWS_FAMILY_NAME",
            Self::WWS_SUBFAMILY_NAME => "WWS_SUBFAMILY_NAME",
            Self::LIGHT_BACKGROUND_PALETTE => "LIGHT_BACKGROUND_PALETTE",
            Self::DARK_BACKGROUND_PALETTE => "DARK_BACKGROUND_PALETTE",
            Self::VARIATIONS_POSTSCRIPT_NAME_PREFIX => "VARIATIONS_POSTSCRIPT_NAME_PREFIX",
            _ => return write!(f, "LocalizedStringId({})", self.0),
        };
        f.write_str(name)
    }
}

/// String containing a name or other font metadata in a specific language.
#[derive(Clone)]
pub struct LocalizedString<'a> {
    name: Name<'a>,
    record: NameRecord,
}

impl<'a> LocalizedString<'a> {
    /// Returns the string identifier.
    ///
    /// Some identifiers are pre-defined and are available as associated constants
    /// on [LocalizedStringId]. Others are used to specify names for variation axes,
    /// name instances, color palettes, etc.
    ///
    /// For a full description, see <https://learn.microsoft.com/en-us/typography/opentype/spec/name#name-ids>
    pub fn id(&self) -> LocalizedStringId {
        LocalizedStringId(self.record.name_id())
    }

    /// Returns the language for this string.
    pub fn language(&self) -> Option<EncodedString<'a>> {
        let id = self.record.language_id();
        // For version 1 name tables, prefer language tags:
        // https://learn.microsoft.com/en-us/typography/opentype/spec/name#naming-table-version-1
        let inner = if self.name.version() == 1 && id >= 0x8000 {
            let index = (id - 0x8000) as usize;
            let language = self
                .name
                .lang_tag_record()?
                .get(index)?
                .lang_tag(self.name.string_data())
                .ok()?;
            EncodedStringInner::Encoded(language)
        } else {
            EncodedStringInner::Str(language_id_to_bcp47(id)?)
        };
        Some(EncodedString(inner))
    }

    /// Returns the encoded string.
    pub fn string(&self) -> Option<EncodedString<'a>> {
        Some(EncodedString(EncodedStringInner::Encoded(
            self.record.string(self.name.string_data()).ok()?,
        )))
    }
}

/// Representation of the encoded data in a localized string.
#[derive(Clone)]
pub struct EncodedString<'a>(EncodedStringInner<'a>);

#[derive(Clone)]
enum EncodedStringInner<'a> {
    Str(&'a str),
    Encoded(NameString<'a>),
}

impl<'a> EncodedString<'a> {
    /// Returns an iterator over the sequence of characters in the string.
    pub fn chars(&self) -> impl Iterator<Item = char> + 'a {
        let inner = match &self.0 {
            EncodedStringInner::Str(s) => CharsInner::Str(s.chars()),
            EncodedStringInner::Encoded(s) => CharsInner::Encoded(s.chars()),
        };
        Chars { inner }
    }
}

struct Chars<'a> {
    inner: CharsInner<'a>,
}

enum CharsInner<'a> {
    Str(core::str::Chars<'a>),
    Encoded(CharIter<'a>),
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            CharsInner::Str(iter) => iter.next(),
            CharsInner::Encoded(iter) => iter.next(),
        }
    }
}

impl<'a> SequenceElement<'a> for LocalizedString<'a> {
    type Data = LocalizedStringSequence<'a>;
}

#[derive(Clone)]
pub struct LocalizedStringSequence<'a> {
    name: Option<Name<'a>>,
}

impl<'a> SequenceData<'a, LocalizedString<'a>> for LocalizedStringSequence<'a> {
    fn new(font: &impl TableProvider<'a>) -> Self {
        Self {
            name: font.name().ok(),
        }
    }

    /// Returns the number of strings in the collection.
    fn len(&self) -> usize {
        self.name
            .as_ref()
            .map(|name| name.count() as usize)
            .unwrap_or_default()
    }

    fn get(&self, index: usize) -> Option<LocalizedString<'a>> {
        let name = self.name.clone()?;
        let record = name.name_record().get(index)?.clone();
        Some(LocalizedString { name, record })
    }
}

/// Converts an OpenType language identifier to a BCP-47 language tag.
fn language_id_to_bcp47(language_id: u16) -> Option<&'static str> {
    match LANGUAGE_ID_TO_BCP47.binary_search_by(|entry| entry.0.cmp(&language_id)) {
        Ok(ix) => LANGUAGE_ID_TO_BCP47.get(ix).map(|entry| entry.1),
        _ => None,
    }
}

/// Mapping of OpenType name table language identifier to BCP-47 language tag.
/// Borrowed from Skia: https://skia.googlesource.com/skia/+/refs/heads/main/src/sfnt/SkOTTable_name.cpp#98
const LANGUAGE_ID_TO_BCP47: &[(u16, &str)] = &[
    /* A mapping from Mac Language Designators to BCP 47 codes.
     *  The following list was constructed more or less manually.
     *  Apple now uses BCP 47 (post OSX10.4), so there will be no new entries.
     */
    (0, "en"),        //English
    (1, "fr"),        //French
    (2, "de"),        //German
    (3, "it"),        //Italian
    (4, "nl"),        //Dutch
    (5, "sv"),        //Swedish
    (6, "es"),        //Spanish
    (7, "da"),        //Danish
    (8, "pt"),        //Portuguese
    (9, "nb"),        //Norwegian
    (10, "he"),       //Hebrew
    (11, "ja"),       //Japanese
    (12, "ar"),       //Arabic
    (13, "fi"),       //Finnish
    (14, "el"),       //Greek
    (15, "is"),       //Icelandic
    (16, "mt"),       //Maltese
    (17, "tr"),       //Turkish
    (18, "hr"),       //Croatian
    (19, "zh-Hant"),  //Chinese (Traditional)
    (20, "ur"),       //Urdu
    (21, "hi"),       //Hindi
    (22, "th"),       //Thai
    (23, "ko"),       //Korean
    (24, "lt"),       //Lithuanian
    (25, "pl"),       //Polish
    (26, "hu"),       //Hungarian
    (27, "et"),       //Estonian
    (28, "lv"),       //Latvian
    (29, "se"),       //Sami
    (30, "fo"),       //Faroese
    (31, "fa"),       //Farsi (Persian)
    (32, "ru"),       //Russian
    (33, "zh-Hans"),  //Chinese (Simplified)
    (34, "nl"),       //Dutch
    (35, "ga"),       //Irish(Gaelic)
    (36, "sq"),       //Albanian
    (37, "ro"),       //Romanian
    (38, "cs"),       //Czech
    (39, "sk"),       //Slovak
    (40, "sl"),       //Slovenian
    (41, "yi"),       //Yiddish
    (42, "sr"),       //Serbian
    (43, "mk"),       //Macedonian
    (44, "bg"),       //Bulgarian
    (45, "uk"),       //Ukrainian
    (46, "be"),       //Byelorussian
    (47, "uz"),       //Uzbek
    (48, "kk"),       //Kazakh
    (49, "az-Cyrl"),  //Azerbaijani (Cyrillic)
    (50, "az-Arab"),  //Azerbaijani (Arabic)
    (51, "hy"),       //Armenian
    (52, "ka"),       //Georgian
    (53, "mo"),       //Moldavian
    (54, "ky"),       //Kirghiz
    (55, "tg"),       //Tajiki
    (56, "tk"),       //Turkmen
    (57, "mn-Mong"),  //Mongolian (Traditional)
    (58, "mn-Cyrl"),  //Mongolian (Cyrillic)
    (59, "ps"),       //Pashto
    (60, "ku"),       //Kurdish
    (61, "ks"),       //Kashmiri
    (62, "sd"),       //Sindhi
    (63, "bo"),       //Tibetan
    (64, "ne"),       //Nepali
    (65, "sa"),       //Sanskrit
    (66, "mr"),       //Marathi
    (67, "bn"),       //Bengali
    (68, "as"),       //Assamese
    (69, "gu"),       //Gujarati
    (70, "pa"),       //Punjabi
    (71, "or"),       //Oriya
    (72, "ml"),       //Malayalam
    (73, "kn"),       //Kannada
    (74, "ta"),       //Tamil
    (75, "te"),       //Telugu
    (76, "si"),       //Sinhalese
    (77, "my"),       //Burmese
    (78, "km"),       //Khmer
    (79, "lo"),       //Lao
    (80, "vi"),       //Vietnamese
    (81, "id"),       //Indonesian
    (82, "tl"),       //Tagalog
    (83, "ms-Latn"),  //Malay (Roman)
    (84, "ms-Arab"),  //Malay (Arabic)
    (85, "am"),       //Amharic
    (86, "ti"),       //Tigrinya
    (87, "om"),       //Oromo
    (88, "so"),       //Somali
    (89, "sw"),       //Swahili
    (90, "rw"),       //Kinyarwanda/Ruanda
    (91, "rn"),       //Rundi
    (92, "ny"),       //Nyanja/Chewa
    (93, "mg"),       //Malagasy
    (94, "eo"),       //Esperanto
    (128, "cy"),      //Welsh
    (129, "eu"),      //Basque
    (130, "ca"),      //Catalan
    (131, "la"),      //Latin
    (132, "qu"),      //Quechua
    (133, "gn"),      //Guarani
    (134, "ay"),      //Aymara
    (135, "tt"),      //Tatar
    (136, "ug"),      //Uighur
    (137, "dz"),      //Dzongkha
    (138, "jv-Latn"), //Javanese (Roman)
    (139, "su-Latn"), //Sundanese (Roman)
    (140, "gl"),      //Galician
    (141, "af"),      //Afrikaans
    (142, "br"),      //Breton
    (143, "iu"),      //Inuktitut
    (144, "gd"),      //Scottish (Gaelic)
    (145, "gv"),      //Manx (Gaelic)
    (146, "ga"),      //Irish (Gaelic with Lenition)
    (147, "to"),      //Tongan
    (148, "el"),      //Greek (Polytonic) Note: ISO 15924 does not have an equivalent script name.
    (149, "kl"),      //Greenlandic
    (150, "az-Latn"), //Azerbaijani (Roman)
    (151, "nn"),      //Nynorsk
    /* A mapping from Windows LCID to BCP 47 codes.
     *  This list is the sorted, curated output of tools/win_lcid.cpp.
     *  Note that these are sorted by value for quick binary lookup, and not logically by lsb.
     *  The 'bare' language ids (e.g. 0x0001 for Arabic) are ommitted
     *  as they do not appear as valid language ids in the OpenType specification.
     */
    (0x0401, "ar-SA"),        //Arabic
    (0x0402, "bg-BG"),        //Bulgarian
    (0x0403, "ca-ES"),        //Catalan
    (0x0404, "zh-TW"),        //Chinese (Traditional)
    (0x0405, "cs-CZ"),        //Czech
    (0x0406, "da-DK"),        //Danish
    (0x0407, "de-DE"),        //German
    (0x0408, "el-GR"),        //Greek
    (0x0409, "en-US"),        //English
    (0x040a, "es-ES_tradnl"), //Spanish
    (0x040b, "fi-FI"),        //Finnish
    (0x040c, "fr-FR"),        //French
    (0x040d, "he-IL"),        //Hebrew
    (0x040d, "he"),           //Hebrew
    (0x040e, "hu-HU"),        //Hungarian
    (0x040e, "hu"),           //Hungarian
    (0x040f, "is-IS"),        //Icelandic
    (0x0410, "it-IT"),        //Italian
    (0x0411, "ja-JP"),        //Japanese
    (0x0412, "ko-KR"),        //Korean
    (0x0413, "nl-NL"),        //Dutch
    (0x0414, "nb-NO"),        //Norwegian (Bokmål)
    (0x0415, "pl-PL"),        //Polish
    (0x0416, "pt-BR"),        //Portuguese
    (0x0417, "rm-CH"),        //Romansh
    (0x0418, "ro-RO"),        //Romanian
    (0x0419, "ru-RU"),        //Russian
    (0x041a, "hr-HR"),        //Croatian
    (0x041b, "sk-SK"),        //Slovak
    (0x041c, "sq-AL"),        //Albanian
    (0x041d, "sv-SE"),        //Swedish
    (0x041e, "th-TH"),        //Thai
    (0x041f, "tr-TR"),        //Turkish
    (0x0420, "ur-PK"),        //Urdu
    (0x0421, "id-ID"),        //Indonesian
    (0x0422, "uk-UA"),        //Ukrainian
    (0x0423, "be-BY"),        //Belarusian
    (0x0424, "sl-SI"),        //Slovenian
    (0x0425, "et-EE"),        //Estonian
    (0x0426, "lv-LV"),        //Latvian
    (0x0427, "lt-LT"),        //Lithuanian
    (0x0428, "tg-Cyrl-TJ"),   //Tajik (Cyrillic)
    (0x0429, "fa-IR"),        //Persian
    (0x042a, "vi-VN"),        //Vietnamese
    (0x042b, "hy-AM"),        //Armenian
    (0x042c, "az-Latn-AZ"),   //Azeri (Latin)
    (0x042d, "eu-ES"),        //Basque
    (0x042e, "hsb-DE"),       //Upper Sorbian
    (0x042f, "mk-MK"),        //Macedonian (FYROM)
    (0x0432, "tn-ZA"),        //Setswana
    (0x0434, "xh-ZA"),        //isiXhosa
    (0x0435, "zu-ZA"),        //isiZulu
    (0x0436, "af-ZA"),        //Afrikaans
    (0x0437, "ka-GE"),        //Georgian
    (0x0438, "fo-FO"),        //Faroese
    (0x0439, "hi-IN"),        //Hindi
    (0x043a, "mt-MT"),        //Maltese
    (0x043b, "se-NO"),        //Sami (Northern)
    (0x043e, "ms-MY"),        //Malay
    (0x043f, "kk-KZ"),        //Kazakh
    (0x0440, "ky-KG"),        //Kyrgyz
    (0x0441, "sw-KE"),        //Kiswahili
    (0x0442, "tk-TM"),        //Turkmen
    (0x0443, "uz-Latn-UZ"),   //Uzbek (Latin)
    (0x0443, "uz"),           //Uzbek
    (0x0444, "tt-RU"),        //Tatar
    (0x0445, "bn-IN"),        //Bengali
    (0x0446, "pa-IN"),        //Punjabi
    (0x0447, "gu-IN"),        //Gujarati
    (0x0448, "or-IN"),        //Oriya
    (0x0449, "ta-IN"),        //Tamil
    (0x044a, "te-IN"),        //Telugu
    (0x044b, "kn-IN"),        //Kannada
    (0x044c, "ml-IN"),        //Malayalam
    (0x044d, "as-IN"),        //Assamese
    (0x044e, "mr-IN"),        //Marathi
    (0x044f, "sa-IN"),        //Sanskrit
    (0x0450, "mn-Cyrl"),      //Mongolian (Cyrillic)
    (0x0451, "bo-CN"),        //Tibetan
    (0x0452, "cy-GB"),        //Welsh
    (0x0453, "km-KH"),        //Khmer
    (0x0454, "lo-LA"),        //Lao
    (0x0456, "gl-ES"),        //Galician
    (0x0457, "kok-IN"),       //Konkani
    (0x045a, "syr-SY"),       //Syriac
    (0x045b, "si-LK"),        //Sinhala
    (0x045d, "iu-Cans-CA"),   //Inuktitut (Syllabics)
    (0x045e, "am-ET"),        //Amharic
    (0x0461, "ne-NP"),        //Nepali
    (0x0462, "fy-NL"),        //Frisian
    (0x0463, "ps-AF"),        //Pashto
    (0x0464, "fil-PH"),       //Filipino
    (0x0465, "dv-MV"),        //Divehi
    (0x0468, "ha-Latn-NG"),   //Hausa (Latin)
    (0x046a, "yo-NG"),        //Yoruba
    (0x046b, "quz-BO"),       //Quechua
    (0x046c, "nso-ZA"),       //Sesotho sa Leboa
    (0x046d, "ba-RU"),        //Bashkir
    (0x046e, "lb-LU"),        //Luxembourgish
    (0x046f, "kl-GL"),        //Greenlandic
    (0x0470, "ig-NG"),        //Igbo
    (0x0478, "ii-CN"),        //Yi
    (0x047a, "arn-CL"),       //Mapudungun
    (0x047c, "moh-CA"),       //Mohawk
    (0x047e, "br-FR"),        //Breton
    (0x0480, "ug-CN"),        //Uyghur
    (0x0481, "mi-NZ"),        //Maori
    (0x0482, "oc-FR"),        //Occitan
    (0x0483, "co-FR"),        //Corsican
    (0x0484, "gsw-FR"),       //Alsatian
    (0x0485, "sah-RU"),       //Yakut
    (0x0486, "qut-GT"),       //K'iche
    (0x0487, "rw-RW"),        //Kinyarwanda
    (0x0488, "wo-SN"),        //Wolof
    (0x048c, "prs-AF"),       //Dari
    (0x0491, "gd-GB"),        //Scottish Gaelic
    (0x0801, "ar-IQ"),        //Arabic
    (0x0804, "zh-Hans"),      //Chinese (Simplified)
    (0x0807, "de-CH"),        //German
    (0x0809, "en-GB"),        //English
    (0x080a, "es-MX"),        //Spanish
    (0x080c, "fr-BE"),        //French
    (0x0810, "it-CH"),        //Italian
    (0x0813, "nl-BE"),        //Dutch
    (0x0814, "nn-NO"),        //Norwegian (Nynorsk)
    (0x0816, "pt-PT"),        //Portuguese
    (0x081a, "sr-Latn-CS"),   //Serbian (Latin)
    (0x081d, "sv-FI"),        //Swedish
    (0x082c, "az-Cyrl-AZ"),   //Azeri (Cyrillic)
    (0x082e, "dsb-DE"),       //Lower Sorbian
    (0x082e, "dsb"),          //Lower Sorbian
    (0x083b, "se-SE"),        //Sami (Northern)
    (0x083c, "ga-IE"),        //Irish
    (0x083e, "ms-BN"),        //Malay
    (0x0843, "uz-Cyrl-UZ"),   //Uzbek (Cyrillic)
    (0x0845, "bn-BD"),        //Bengali
    (0x0850, "mn-Mong-CN"),   //Mongolian (Traditional Mongolian)
    (0x085d, "iu-Latn-CA"),   //Inuktitut (Latin)
    (0x085f, "tzm-Latn-DZ"),  //Tamazight (Latin)
    (0x086b, "quz-EC"),       //Quechua
    (0x0c01, "ar-EG"),        //Arabic
    (0x0c04, "zh-Hant"),      //Chinese (Traditional)
    (0x0c07, "de-AT"),        //German
    (0x0c09, "en-AU"),        //English
    (0x0c0a, "es-ES"),        //Spanish
    (0x0c0c, "fr-CA"),        //French
    (0x0c1a, "sr-Cyrl-CS"),   //Serbian (Cyrillic)
    (0x0c3b, "se-FI"),        //Sami (Northern)
    (0x0c6b, "quz-PE"),       //Quechua
    (0x1001, "ar-LY"),        //Arabic
    (0x1004, "zh-SG"),        //Chinese (Simplified)
    (0x1007, "de-LU"),        //German
    (0x1009, "en-CA"),        //English
    (0x100a, "es-GT"),        //Spanish
    (0x100c, "fr-CH"),        //French
    (0x101a, "hr-BA"),        //Croatian (Latin)
    (0x103b, "smj-NO"),       //Sami (Lule)
    (0x1401, "ar-DZ"),        //Arabic
    (0x1404, "zh-MO"),        //Chinese (Traditional)
    (0x1407, "de-LI"),        //German
    (0x1409, "en-NZ"),        //English
    (0x140a, "es-CR"),        //Spanish
    (0x140c, "fr-LU"),        //French
    (0x141a, "bs-Latn-BA"),   //Bosnian (Latin)
    (0x141a, "bs"),           //Bosnian
    (0x143b, "smj-SE"),       //Sami (Lule)
    (0x143b, "smj"),          //Sami (Lule)
    (0x1801, "ar-MA"),        //Arabic
    (0x1809, "en-IE"),        //English
    (0x180a, "es-PA"),        //Spanish
    (0x180c, "fr-MC"),        //French
    (0x181a, "sr-Latn-BA"),   //Serbian (Latin)
    (0x183b, "sma-NO"),       //Sami (Southern)
    (0x1c01, "ar-TN"),        //Arabic
    (0x1c09, "en-ZA"),        //English
    (0x1c0a, "es-DO"),        //Spanish
    (0x1c1a, "sr-Cyrl-BA"),   //Serbian (Cyrillic)
    (0x1c3b, "sma-SE"),       //Sami (Southern)
    (0x1c3b, "sma"),          //Sami (Southern)
    (0x2001, "ar-OM"),        //Arabic
    (0x2009, "en-JM"),        //English
    (0x200a, "es-VE"),        //Spanish
    (0x201a, "bs-Cyrl-BA"),   //Bosnian (Cyrillic)
    (0x201a, "bs-Cyrl"),      //Bosnian (Cyrillic)
    (0x203b, "sms-FI"),       //Sami (Skolt)
    (0x203b, "sms"),          //Sami (Skolt)
    (0x2401, "ar-YE"),        //Arabic
    (0x2409, "en-029"),       //English
    (0x240a, "es-CO"),        //Spanish
    (0x241a, "sr-Latn-RS"),   //Serbian (Latin)
    (0x243b, "smn-FI"),       //Sami (Inari)
    (0x2801, "ar-SY"),        //Arabic
    (0x2809, "en-BZ"),        //English
    (0x280a, "es-PE"),        //Spanish
    (0x281a, "sr-Cyrl-RS"),   //Serbian (Cyrillic)
    (0x2c01, "ar-JO"),        //Arabic
    (0x2c09, "en-TT"),        //English
    (0x2c0a, "es-AR"),        //Spanish
    (0x2c1a, "sr-Latn-ME"),   //Serbian (Latin)
    (0x3001, "ar-LB"),        //Arabic
    (0x3009, "en-ZW"),        //English
    (0x300a, "es-EC"),        //Spanish
    (0x301a, "sr-Cyrl-ME"),   //Serbian (Cyrillic)
    (0x3401, "ar-KW"),        //Arabic
    (0x3409, "en-PH"),        //English
    (0x340a, "es-CL"),        //Spanish
    (0x3801, "ar-AE"),        //Arabic
    (0x380a, "es-UY"),        //Spanish
    (0x3c01, "ar-BH"),        //Arabic
    (0x3c0a, "es-PY"),        //Spanish
    (0x4001, "ar-QA"),        //Arabic
    (0x4009, "en-IN"),        //English
    (0x400a, "es-BO"),        //Spanish
    (0x4409, "en-MY"),        //English
    (0x440a, "es-SV"),        //Spanish
    (0x4809, "en-SG"),        //English
    (0x480a, "es-HN"),        //Spanish
    (0x4c0a, "es-NI"),        //Spanish
    (0x500a, "es-PR"),        //Spanish
    (0x540a, "es-US"),        //Spanish
];
