#![allow(dead_code)]

use std::{collections::HashMap, fmt, hint::unreachable_unchecked};

use data::BIT_STRING;

pub mod data;
pub mod parser;

#[derive(Debug, Clone)]
/// The header of a PCF file
pub struct PCFHeader {
    pub tables: Vec<PCFHeaderEntry>,
}

impl PCFHeader {
    pub fn get(&self, kind: PCFTableKind) -> Option<TableRef> {
        self.tables.iter().find(|&t| t.kind == kind).map(|t| t.pos)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PCFTableKind(pub u32);

impl PCFTableKind {
    pub const PROPERTIES: Self = Self(1 << 0);
    pub const ACCELERATORS: Self = Self(1 << 1);
    pub const METRICS: Self = Self(1 << 2);
    pub const BITMAPS: Self = Self(1 << 3);
    pub const INK_METRICS: Self = Self(1 << 4);
    pub const BDF_ENCODINGS: Self = Self(1 << 5);
    pub const SWIDTHS: Self = Self(1 << 6);
    pub const GLYPH_NAMES: Self = Self(1 << 7);
    pub const BDF_ACCELERATORS: Self = Self(1 << 8);
}

pub const PCF_DEFAULT_FORMAT: u32 = 0x00000000;
pub const PCF_INKBOUNDS: u32 = 0x00000200;
pub const PCF_ACCEL_W_INKBOUNDS: u32 = 0x00000100;
pub const PCF_COMPRESSED_METRICS: u32 = 0x00000100;

#[derive(Debug, Copy, Clone)]
pub struct TableRef {
    /// The size of the table
    pub size: u32,
    /// The offset of the table
    pub offset: u32,
}

impl TableRef {
    pub fn of<'a>(&self, buffer: &'a [u8]) -> Option<&'a [u8]> {
        let start = self.offset as usize;
        let end = start + self.size as usize;
        buffer.get(start..end)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PCFHeaderEntry {
    /// The type of table at the location
    pub kind: PCFTableKind,
    /// The format modifiers of that table
    pub format: u32,
    /// The reference to the table
    pub pos: TableRef,
}

#[derive(Debug, Clone)]
pub struct PCFGlyphNames {
    pub names: Vec<String>,
}

#[derive(Clone)]
pub enum PropVal {
    None,
    Int(u32),
    String(String),
}

impl Default for PropVal {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Debug for PropVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => Option::<()>::fmt(&None, f),
            Self::Int(i) => i.fmt(f),
            Self::String(i) => i.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PCFProperties {
    average_width: PropVal,
    cap_height: PropVal,
    charset_collections: PropVal,
    charset_encoding: PropVal,
    charset_registry: PropVal,
    copyright: PropVal,
    family_name: PropVal,
    foundry: PropVal,
    font: PropVal,
    fontname_registry: PropVal,
    full_name: PropVal,
    pixel_size: PropVal,
    point_size: PropVal,
    quad_width: PropVal,
    resolution: PropVal,
    resolution_x: PropVal,
    resolution_y: PropVal,
    setwidth_name: PropVal,
    spacing: PropVal,
    weight: PropVal,
    weight_name: PropVal,
    x_height: PropVal,

    misc: HashMap<String, PropVal>,
}

#[derive(Debug, Clone)]
pub struct PCFScalableWidths {
    pub swidths: Vec<i32>,
}
#[derive(Debug, Clone)]
pub struct PCFBDFEncodings {
    pub min_char_or_byte2: i16,
    pub max_char_or_byte2: i16,
    pub min_byte1: i16,
    pub max_byte1: i16,
    pub default_char: i16,
    pub glyphindeces: Vec<i16>,
}

#[derive(Debug, Clone)]
pub struct XChar {
    pub metrics: XCharMetrics,
    pub bitmap: Option<BitMap>,
    pub swidth: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct XCharMetrics {
    pub left_sided_bearing: i16,
    pub right_side_bearing: i16,
    pub character_width: i16,
    pub character_ascent: i16,
    pub character_descent: i16,
    pub character_attributes: u16,
}

#[derive(Debug, Clone)]
pub struct PCFMetricBounds {
    pub min: XCharMetrics,
    pub max: XCharMetrics,
}

#[derive(Debug, Clone)]
pub struct PCFAccelerators {
    /// if for all `i`, `max(metrics[i].rightSideBearing - metrics[i].characterWidth) <= minbounds.leftSideBearing`
    pub no_overlap: u8,
    /// Means the perchar field of the XFontStruct can be NULL
    pub constant_metrics: u8,
    /// `constant_metrics` true and forall characters:
    ///
    /// - the left side bearing==0
    /// - the right side bearing== the character's width
    /// - the character's ascent==the font's ascent
    /// - the character's descent==the font's descent
    pub terminal_font: u8,
    /// monospace font like courier
    pub constant_width: u8,
    /// Means that all inked bits are within the rectangle with x between `[0,charwidth]`
    /// and y between `[-descent,ascent]`. So no ink overlaps another char when drawing
    pub ink_inside: u8,
    /// true if the ink metrics differ from the metrics somewhere
    pub ink_metrics: u8,
    /// 0=>left to right, 1=>right to left
    pub draw_direction: u8,

    pub font_ascent: i32,
    pub font_descent: i32,
    pub max_overlap: i32,

    pub bounds: PCFMetricBounds,
    pub ink_bounds: Option<PCFMetricBounds>,
}

#[derive(Debug, Clone)]
pub struct PCFMetrics {
    pub metrics: Vec<XCharMetrics>,
}

#[derive(Debug, Copy, Clone)]
pub enum ByteOrder {
    Reverse = 0b00,      // LSBit first, LSByte first,
    BitReverse = 0b01,   // LSBit first, MSByte first,
    LittleEndian = 0b10, // MSBit first, LSByte first,
    BigEndian = 0b11,    // MSBit first, MSByte first,
}

impl ByteOrder {
    pub fn from_bits(i: u8) -> ByteOrder {
        match i & 3 {
            0 => ByteOrder::Reverse,
            1 => ByteOrder::BitReverse,
            2 => ByteOrder::LittleEndian,
            3 => ByteOrder::BigEndian,
            _ => unsafe { unreachable_unchecked() },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BitWidth {
    Bytes = 0,
    Shorts = 1,
    Ints = 2,
}

impl BitWidth {
    pub fn from_bits(i: u8) -> Option<BitWidth> {
        match i & 3 {
            0 => Some(BitWidth::Bytes),
            1 => Some(BitWidth::Shorts),
            2 => Some(BitWidth::Ints),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct BitMap(Vec<u8>, u32);

impl fmt::Debug for BitMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let clen = self.1 as usize;
        write!(f, "+")?;
        for _ in 0..clen {
            write!(f, "--------")?;
        }
        writeln!(f, "+")?;
        for line in self.0.chunks(clen) {
            write!(f, "|")?;
            for byte in line {
                write!(f, "{}", BIT_STRING[*byte as usize])?;
            }
            writeln!(f, "|")?;
        }
        write!(f, "+")?;
        for _ in 0..clen {
            write!(f, "--------")?;
        }
        writeln!(f, "+")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PCFBitmaps {
    //offsets: Vec<i32>,
    /// How to read the values
    ///
    /// - the byte order (format&4 => LSByte first)
    /// - the bit order (format&8 => LSBit first)
    pub order: ByteOrder,

    /// how each row in each glyph's bitmap is padded (format&3)
    ///
    /// 0=>bytes, 1=>shorts, 2=>ints
    pub pad_width: BitWidth,

    /// what the bits are stored in (bytes, shorts, ints) (format>>4)&3
    ///
    /// 0=>bytes, 1=>shorts, 2=>ints
    pub store_width: BitWidth,
}
