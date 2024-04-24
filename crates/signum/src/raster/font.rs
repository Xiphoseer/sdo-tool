use bstr::BStr;

use crate::chsets::editor::ESet;

use super::{DrawPrintErr, Page};

/// Render a signum 7 bit text to an image
pub fn render_editor_text(text: &BStr, eset: &ESet) -> Result<Page, DrawPrintErr> {
    let width = text
        .iter()
        .copied()
        .map(|i| u32::from(eset.chars[i as usize].width) + 1)
        .sum::<u32>()
        + 8;
    let mut x = 4;
    let mut page = Page::new(width, 24);
    for ci in text.iter() {
        let ch = &eset.chars[*ci as usize];
        page.draw_echar(x, 0, ch)?;
        x += u16::from(ch.width) + 1;
    }
    Ok(page)
}
