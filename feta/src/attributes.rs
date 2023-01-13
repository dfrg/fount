use read_fonts::TableProvider;

/// Visual width of a font-- a relative change from the normal aspect
/// ratio from 0.5 to 2.0.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Stretch(pub f32);

impl Stretch {
    /// Width that is 50% of normal.
    pub const ULTRA_CONDENSED: Self = Self(0.5);

    /// Width that is 62.5% of normal.
    pub const EXTRA_CONDENSED: Self = Self(0.625);

    /// Width that is 75% of normal.
    pub const CONDENSED: Self = Self(0.75);

    /// Width that is 87.5% of normal.
    pub const SEMI_CONDENSED: Self = Self(0.875);

    /// Width that is 100% of normal.
    pub const NORMAL: Self = Self(1.0);

    /// Width that is 112.5% of normal.
    pub const SEMI_EXPANDED: Self = Self(1.125);

    /// Width that is 125% of normal.
    pub const EXPANDED: Self = Self(1.25);

    /// Width that is 150% of normal.
    pub const EXTRA_EXPANDED: Self = Self(1.5);

    /// Width that is 200% of normal.
    pub const ULTRA_EXPANDED: Self = Self(2.0);
}

impl Default for Stretch {
    fn default() -> Self {
        Self(1.0)
    }
}

/// Visual style or 'slope' of a font.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub enum Style {
    #[default]
    Normal,
    Italic,
    /// Oblique style with an optional angle in degrees, counter-clockwise
    /// from the vertical.
    Oblique(Option<f32>),
}

/// Visual weight class of a font on a scale from 1.0 to 1000.0.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Weight(pub f32);

impl Weight {
    /// Weight value of 100.
    pub const THIN: Self = Self(100.0);

    /// Weight value of 200.
    pub const EXTRA_LIGHT: Self = Self(200.0);

    /// Weight value of 300.
    pub const LIGHT: Self = Self(300.0);

    /// Weight value of 350.
    pub const SEMI_LIGHT: Self = Self(350.0);

    /// Weight value of 400.
    pub const NORMAL: Self = Self(400.0);

    /// Weight value of 500.
    pub const MEDIUM: Self = Self(500.0);

    /// Weight value of 600.
    pub const SEMI_BOLD: Self = Self(600.0);

    /// Weight value of 700.
    pub const BOLD: Self = Self(700.0);

    /// Weight value of 800.
    pub const EXTRA_BOLD: Self = Self(800.0);

    /// Weight value of 900.
    pub const BLACK: Self = Self(900.0);

    /// Weight value of 950.
    pub const EXTRA_BLACK: Self = Self(950.0);
}

impl Default for Weight {
    fn default() -> Self {
        Self(400.0)
    }
}

pub fn from_font<'a>(font: &impl TableProvider<'a>) -> (Stretch, Style, Weight) {
    let mut stretch = Stretch::default();
    let mut style = Style::default();
    let mut weight = Weight::default();
    if let Ok(os2) = font.os2() {
        weight = Weight(os2.us_weight_class().clamp(1, 1000) as f32);
        stretch = match os2.us_weight_class() {
            1 => Stretch::ULTRA_CONDENSED,
            2 => Stretch::EXTRA_CONDENSED,
            3 => Stretch::CONDENSED,
            4 => Stretch::SEMI_CONDENSED,
            5 => Stretch::NORMAL,
            6 => Stretch::SEMI_EXPANDED,
            7 => Stretch::EXPANDED,
            8 => Stretch::EXTRA_EXPANDED,
            9 => Stretch::ULTRA_EXPANDED,
            _ => Stretch::NORMAL,
        };
        const FS_SELECTION_ITALIC: u16 = 1;
        const FS_SELECTION_OBLIQUE: u16 = 1 << 9;
        let fs_selection = os2.fs_selection();
        if fs_selection & FS_SELECTION_ITALIC != 0 {
            style = Style::Italic;
        } else if fs_selection & FS_SELECTION_OBLIQUE != 0 {
            let angle = font
                .post()
                .map(|post| post.italic_angle().to_f64() as f32)
                .ok();
            style = Style::Oblique(angle);
        }
    } else if let Ok(head) = font.head() {
        const MAC_STYLE_BOLD: u16 = 1;
        const MAC_STYLE_ITALIC: u16 = 2;
        if head.mac_style() & MAC_STYLE_BOLD != 0 {
            weight = Weight(700.0);
        }
        if head.mac_style() & MAC_STYLE_ITALIC != 0 {
            style = Style::Italic;
        }
    }
    (stretch, style, weight)
}
