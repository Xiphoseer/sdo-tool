use bstr::BStr;

use crate::chsets::{editor::ESet, printer::PSet};

use super::{DrawPrintErr, Page};

/// Render a signum 7 bit text to an image
pub fn render_editor_text(text: &BStr, eset: &ESet) -> Result<Page, DrawPrintErr> {
    let width = text
        .iter()
        .copied()
        .map(|i| {
            if let Some(ch) = &eset.chars.get(i as usize) {
                u32::from(ch.width) + 1
            } else {
                16
            }
        })
        .sum::<u32>()
        + 12;
    let mut x = 6;
    let mut page = Page::new(width, 30);
    for ci in text.iter() {
        if let Some(ch) = eset.chars.get(*ci as usize) {
            page.draw_echar(x, 2, ch)?;
            x += u16::from(ch.width) + 1;
        } else {
            x += 16;
        }
    }
    Ok(page)
}

/// Get a character from a page
pub fn render_printer_char(char: u8, pset: &PSet<'_>) -> Option<Page> {
    let pchar = pset.chars.get(char as usize)?;
    Some(Page::from(pchar))
}
