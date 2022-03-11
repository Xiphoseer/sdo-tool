use std::{collections::BTreeMap, fmt};

pub mod xfont;

pub struct Font<'a> {
    pub chars: Vec<Char<'a>>,
    pub font_descriptor: xfont::XFontDescriptor,
    pub bounding_box: BoundingBox,
    pub properties: BTreeMap<String, u32>,
    pub size: Size,
}

impl fmt::Display for Font<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "STARTFONT 2.1")?;
        writeln!(f, "FONT {}", self.font_descriptor)?;
        writeln!(f, 
            "SIZE {} {} {}",
            self.size.point_size, self.size.xdpi, self.size.ydpi
        )?;
        writeln!(f, 
            "FONTBOUNDINGBOX {} {} {} {}",
            self.bounding_box.width,
            self.bounding_box.height,
            self.bounding_box.xoff,
            self.bounding_box.yoff
        )?;
        writeln!(f, "STARTPROPERTIES {}", self.properties.len())?;
        for (k, v) in &self.properties {
            writeln!(f, "{} {}", k, v)?;
        }
        writeln!(f, "ENDPROPERTIES")?;

        writeln!(f, "CHARS {}", self.chars.len())?;

        for chr in &self.chars {
            chr.fmt(f)?;
        }
        writeln!(f, "ENDFONT")?;
        Ok(())
    }
}

pub struct Char<'a> {
    pub unicode: u32,
    pub encoding: u32,
    pub scalable_width: Width<u32>,
    pub device_width: Width<u32>,
    pub bounding_box: BoundingBox,
    pub pixels: &'a [&'a [u8]],
}

impl fmt::Display for Char<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "STARTCHAR U+{:04x}", self.unicode)?;
        writeln!(f, "ENCODING {}", self.encoding)?;
        writeln!(f, "SWIDTH {} {}", self.scalable_width.x, self.scalable_width.y)?;
        writeln!(f, "DWIDTH {} {}", self.device_width.x, self.device_width.y)?;
        writeln!(f, 
            "BBX {} {} {} {}",
            self.bounding_box.width,
            self.bounding_box.height,
            self.bounding_box.xoff,
            self.bounding_box.yoff
        )?;
        writeln!(f, "BITMAP")?;
        for &scanline in self.pixels {
            for &word in scanline {
                writeln!(f, "{:02X}", word)?;
            }
        }
        writeln!(f, "ENDCHAR")?;
        Ok(())
    }
}

pub struct Size {
    pub point_size: u32,
    pub xdpi: u32,
    pub ydpi: u32,
}

pub struct Width<I> {
    pub x: I,
    pub y: I,
}

pub struct BoundingBox {
    pub width: u32,
    pub height: u32,
    pub xoff: i32,
    pub yoff: i32,
}