//! Convert fonts into other formats

#[cfg(feature = "bdf")]
/// Conversion to BDF
pub mod bdf {
    use crate::chsets::printer::{PSet, PSetChar, PrinterKind};
    use std::fmt;

    fn write_bdf_char<I: std::fmt::Write>(
        o: &mut I,
        index: usize,
        chr: &PSetChar,
        pk: PrinterKind,
    ) -> fmt::Result {
        writeln!(o, "STARTCHAR U+{:04x}", index)?;
        writeln!(o, "ENCODING {}", index)?;
        writeln!(o, "SWIDTH {} {}", chr.width as u32 * 500, 0)?;
        writeln!(o, "DWIDTH {} {}", chr.width as u32 * 8, 0)?;
        let half = (pk.line_height() as i32 - pk.baseline()) / 2;
        writeln!(
            o,
            "BBX {} {} {} {}",
            (chr.width * 8),
            chr.height, //pk.line_height(),
            0,
            pk.baseline() - (chr.top as i32) - (chr.height as i32) + half,
        )?;
        writeln!(o, "BITMAP")?;
        for scanline in chr.bitmap.chunks(chr.width as usize) {
            for &byte in scanline {
                write!(o, "{:02X}", byte)?;
            }
            writeln!(o)?;
        }
        writeln!(o, "ENDCHAR")?;
        Ok(())
    }

    /// Convert a printer CHSET into a BDF font
    pub fn pset_to_bdf<I: std::fmt::Write>(o: &mut I, pset: &PSet) -> fmt::Result {
        let font_descriptor = bdf::xfont::XFontDescriptor {
            foundry: "gnu".to_string(),
            family_name: "unifont".to_string(),
            weight_name: "medium".to_string(),
            slant: bdf::xfont::Slant::Roman,
            setwidth_name: "normal".to_string(),
            add_style_name: "".to_string(),
            pixel_size: pset.pk.line_height(),
            point_size: pset.pk.line_height() * 10,
            resolution_x: 75,
            resolution_y: 75,
            spacing: bdf::xfont::Spacing::CharCell,
            average_width: 80,
            charset_registry: "iso10646".to_string(),
            charset_encoding: "1".to_string(),
        };

        writeln!(o, "STARTFONT 2.1")?;
        writeln!(o, "FONT {}", font_descriptor)?;
        writeln!(o, "SIZE {} {} {}", pset.pk.line_height(), 75, 75)?;
        writeln!(
            o,
            "FONTBOUNDINGBOX {} {} {} {}",
            pset.pk.line_height(),
            pset.pk.line_height(),
            0,
            pset.pk.line_height() as i32 - pset.pk.baseline(),
        )?;
        writeln!(o, "STARTPROPERTIES 2",)?;

        let ascent = pset.pk.baseline();
        let descent = pset.pk.line_height() - pset.pk.baseline() as u32;
        writeln!(o, "FONT_ASCENT {}", ascent)?;
        writeln!(o, "FONT_DESCENT {}", descent)?;
        writeln!(o, "ENDPROPERTIES")?;

        writeln!(
            o,
            "CHARS {}",
            pset.chars.iter().filter(|c| c.width > 0).count()
        )?;

        for (index, chr) in pset.chars[1..].iter().enumerate() {
            if chr.width > 0 {
                write_bdf_char(o, index + 1, chr, pset.pk)?;
            }
        }

        writeln!(o, "ENDFONT")?;
        Ok(())
    }
}
