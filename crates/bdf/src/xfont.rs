use std::fmt;

pub enum Slant {
    Roman,
    Italic,
    Oblique,
    ReverseItalic,
    ReverseOblique,
    Other,
    Polymorphic(u8),
}

impl fmt::Display for Slant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Slant::Roman => write!(f, "r"),
            Slant::Italic => write!(f, "i"),
            Slant::Oblique => write!(f, "o"),
            Slant::ReverseItalic => write!(f, "ri"),
            Slant::ReverseOblique => write!(f, "ro"),
            Slant::Other => write!(f, "ot"),
            Slant::Polymorphic(x) => write!(f, "{}", x),
        }
    }
}

pub enum Spacing {
    Proportional,
    Monospaced,
    CharCell,
}

impl fmt::Display for Spacing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Spacing::Proportional => write!(f, "p"),
            Spacing::Monospaced => write!(f, "m"),
            Spacing::CharCell => write!(f, "c"),
        }
    }
}

pub struct XFontDescriptor {
    /// Type foundry - vendor or supplier of this font
    pub foundry: String,
    /// Typeface family
    pub family_name: String,
    /// Weight of type
    pub weight_name: String,
    /// Slant (upright, italic, oblique, reverse italic, reverse oblique, or "other")
    pub slant: Slant,
    /// Proportionate width (e.g. normal, condensed, narrow, expanded/double-wide)
    pub setwidth_name: String,
    /// Additional style (e.g. (Sans) Serif, Informal, Decorated)
    pub add_style_name: String,
    /// Size of characters, in pixels; 0 (Zero) means a scalable font
    pub pixel_size: u32,
    /// Size of characters, in tenths of points
    pub point_size: u32,
    /// Horizontal resolution in dots per inch (DPI), for which the font was designed
    pub resolution_x: u32,
    /// Vertical resolution, in DPI
    pub resolution_y: u32,
    /// monospaced, proportional, or "character cell"
    pub spacing: Spacing,
    /// Average width of characters of this font; 0 means scalable font
    pub average_width: u32,
    /// Registry defining this character set
    pub charset_registry: String,
    /// Registry's character encoding scheme for this set
    pub charset_encoding: String,
}

impl fmt::Display for XFontDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            self.foundry,
            self.family_name,
            self.weight_name,
            self.slant,
            self.setwidth_name,
            self.add_style_name,
            self.pixel_size,
            self.point_size,
            self.resolution_x,
            self.resolution_y,
            self.spacing,
            self.average_width,
            self.charset_registry,
            self.charset_encoding
        )
    }
}
