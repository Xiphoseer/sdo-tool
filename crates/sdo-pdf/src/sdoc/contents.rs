use std::io::{self, Write};

use pdf_create::common::MediaBox;
use signum::docs::{hcim::ImageSite, pbuf, Overrides};

use super::TextContents;

/// The `Contents` stream of a PDF
#[derive(Default)]
pub struct Contents {
    top: f32,
    left: f32,
    inner: Vec<u8>,
}

impl Contents {
    pub fn for_page(
        page_info: &pbuf::Page,
        media_box: &MediaBox,
        overrides: &Overrides,
    ) -> Contents {
        // PDF uses a unit length of 1/72 1/(18*4) of an inch by default
        //
        // Signum uses 1/54 1/(18*3) of an inch vertically and 1/90 1/(18*5) horizontally

        let width = page_info.format.width() * 72 / 90;
        let height = page_info.format.length as i32 * 72 / 54;

        assert!(width as i32 <= media_box.width, "Please file a bug!");

        let xmargin = (media_box.width - width as i32) / 2;
        let ymargin = (media_box.height - height) / 2;

        let left = {
            let left = xmargin as f32 + overrides.xoffset as f32;
            left - page_info.format.left as f32 * 8.0 / 10.0
        };
        let top = {
            let top = ymargin as f32 + overrides.yoffset as f32;
            media_box.height as f32 - top - 8.0
        };

        Contents::new(top, left)
    }

    /// Create a new stream
    pub fn new(top: f32, left: f32) -> Self {
        let mut inner = Vec::new();
        writeln!(inner, "0 g").unwrap();
        Self { inner, top, left }
    }

    pub fn image(&mut self, site: &ImageSite, key: &str) -> io::Result<()> {
        writeln!(self.inner, "q")?;
        let t = self.top - (((site.site.y + site.site.h / 2 - site._5 / 2) as f32 * 72.0) / 54.0);
        let l = self.left + ((site.site.x as f32 * 72.0) / 90.0);
        let w = (site.site.w as f32 * 72.0) / 90.0;
        let h = (site.site.h as f32 * /*72.0*/ 36.0) / 54.0;
        writeln!(self.inner, "{} 0 0 {} {} {} cm", w, h, l, t)?;
        writeln!(self.inner, "/{} Do", key)?;
        writeln!(self.inner, "Q")?;
        Ok(())
    }

    pub fn start_text(self, scale_x: f32, scale_y: f32) -> TextContents {
        let mut inner = self.inner;
        let left = self.left;
        let top = self.top;
        write!(
            inner,
            "q\nBT\n{} 0 0 {} {} {} Tm\n",
            scale_x, scale_y, left, top
        )
        .unwrap();
        TextContents::new(inner)
    }
}
