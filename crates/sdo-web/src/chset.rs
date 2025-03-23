use bstr::BStr;
use log::warn;
use sdo_util::keymap::{KB_DRAW, NP_DRAW};
use signum::{
    chsets::{
        editor::{parse_eset, ESet},
        encoding::Mapping,
        printer::{parse_pset, PSet},
        FontKind,
    },
    raster::{self, render_editor_text, render_printer_char},
    util::FileFormatKind,
};
use std::fmt::Write;
use wasm_bindgen::{prelude::Closure, JsCast, JsError, JsValue};
use web_sys::{window, CanvasRenderingContext2d, Element, HtmlCanvasElement, ImageBitmap};

use crate::{blob_image_el, js_error_with_cause, page_to_blob};

impl super::Handle {
    /// Render a keyboard layout for the editor charset
    fn eset_kb(&self, eset: &ESet<'_>) -> Result<(), JsValue> {
        let container = self.document.create_element("div")?;
        container.class_list().add_1("overflow-x-auto")?;
        let kb_img = KB_DRAW
            .to_page(eset)
            .or(Err("Failed to draw Keyboard Map"))?;
        let np_img = NP_DRAW.to_page(eset).or(Err("Failed to draw Numpad Map"))?;

        let kb_blob = page_to_blob(&kb_img)?;
        let np_blob = page_to_blob(&np_img)?;

        let kb_img_el = blob_image_el(&kb_blob)?;
        let np_img_el = blob_image_el(&np_blob)?;

        container.append_child(&kb_img_el)?;
        container.append_child(&np_img_el)?;

        self.output.append_child(&container)?;
        Ok(())
    }

    pub(super) fn parse_eset<'a>(&self, data: &'a [u8]) -> Result<ESet<'a>, JsValue> {
        let (_, eset) =
            parse_eset(data).map_err(|e| js_error_with_cause(e, "Failed to parse editor font"))?;
        Ok(eset)
    }

    pub(super) fn parse_pset<'a>(&self, data: &'a [u8]) -> Result<PSet<'a>, JsValue> {
        let (_, pset) = parse_pset::<signum::nom::error::Error<&'a [u8]>>(data)
            .map_err(|e| js_error_with_cause(e, "Failed to parse printer font"))?;
        Ok(pset)
    }

    pub(super) fn show_eset(&self, eset: &ESet<'_>) -> Result<(), JsValue> {
        self.eset_kb(eset)?;
        Ok(())
    }

    fn show_mapping(&self, mapping: &Mapping, name: &str, built_in: bool) -> Result<(), JsValue> {
        let h3 = self.document.create_element("h3")?;
        h3.set_text_content(Some("Mapping"));
        self.output.append_child(&h3)?;
        if built_in {
            let alert = self.document.create_element("div")?;
            alert.class_list().add_2("alert", "alert-light")?;
            //alert.append_with_str_1("The font ")?;
            let code = self.document.create_element("kbd")?;
            code.set_inner_html(name);
            alert.append_child(&code)?;
            alert.append_with_str_1(
                " is a well-known font associated with the following (built-in) unicode mapping:",
            )?;
            self.output.append_child(&alert)?;
        }

        let el_table_responsive = self.document.create_element("div")?;
        el_table_responsive.class_list().add_1("table-responsive")?;
        let el_table = self.document.create_element("table")?;
        el_table.class_list().add_2("table", "mapping")?;
        let dx = 16;
        let el_tr_head = self.document.create_element("tr")?;
        el_table.append_child(&el_tr_head)?;
        let el_th0 = self.document.create_element("th")?;
        el_tr_head.append_child(&el_th0)?;
        for i in 0..dx {
            let el_th = self.document.create_element("th")?;
            el_th.append_with_str_1(&format!("_{i:X}"))?;
            el_tr_head.append_child(&el_th)?;
        }
        for (y, crow) in mapping.rows().enumerate() {
            let el_tr = self.document.create_element("tr")?;
            el_table.append_child(&el_tr)?;

            let el_th_row = self.document.create_element("th")?;
            el_th_row.append_with_str_1(&format!("{y:X}_"))?;
            el_tr.append_child(&el_th_row)?;
            for (x, c) in crow.enumerate() {
                let el_td = self.document.create_element("td")?;
                let _i = y * dx + x;
                if !matches!(*c, [char::REPLACEMENT_CHARACTER] | ['\0']) {
                    let mut text = String::new();
                    for char in c {
                        write!(text, "&#x{:04X};", u32::from(*char)).unwrap();
                    }
                    el_td.set_inner_html(&text);

                    let br = self.document.create_element("br")?;
                    el_td.append_child(&br)?;

                    let sub = self.document.create_element("small")?;
                    let mut sub_text = String::new();
                    for char in c {
                        if !sub_text.is_empty() {
                            write!(sub_text, " ").unwrap();
                        }
                        write!(sub_text, "U+{:04X}", u32::from(*char)).unwrap();
                    }
                    sub.set_inner_html(&sub_text);
                    el_td.append_child(&sub)?;
                }
                el_tr.append_child(&el_td)?;
            }
        }
        el_table_responsive.append_child(&el_table)?;
        self.output.append_child(&el_table_responsive)?;

        Ok(())
    }

    fn show_pset(&self, pset: &PSet<'_>) -> Result<(), JsValue> {
        let h3 = self.document.create_element("h3")?;
        h3.set_text_content(Some("Characters"));
        self.output.append_child(&h3)?;

        let el_table_responsive = self.document.create_element("div")?;
        el_table_responsive.class_list().add_1("table-responsive")?;
        let el_table = self.document.create_element("table")?;
        el_table.class_list().add_2("table", "pset")?;
        let dx = 16;
        let el_tr_head = self.document.create_element("tr")?;
        el_table.append_child(&el_tr_head)?;
        let el_th0 = self.document.create_element("th")?;
        el_tr_head.append_child(&el_th0)?;
        for i in 0..dx {
            let el_th = self.document.create_element("th")?;
            el_th.append_with_str_1(&format!("_{i:X}"))?;
            el_tr_head.append_child(&el_th)?;
        }
        for (y, crow) in pset.chars.chunks(dx).enumerate() {
            let el_tr = self.document.create_element("tr")?;
            el_table.append_child(&el_tr)?;

            let el_th_row = self.document.create_element("th")?;
            el_th_row.append_with_str_1(&format!("{y:X}_"))?;
            el_tr.append_child(&el_th_row)?;
            for (x, c) in crow.iter().enumerate() {
                let el_td = self.document.create_element("td")?;
                let i = y * dx + x;
                el_tr.append_child(&el_td)?;
                if let Some(special) = c.special() {
                    warn!("pset char special {}: {}", i, special);
                }
                if c.height > 0 && c.width > 0 {
                    let page = raster::Page::from(c);
                    let blob = page_to_blob(&page)?;
                    let img_el = blob_image_el(&blob)?;
                    el_td.append_child(&img_el)?;
                }
            }
        }
        el_table_responsive.append_child(&el_table)?;
        self.output.append_child(&el_table_responsive)?;

        Ok(())
    }

    pub(super) async fn show_font(
        &self,
        font_kind: FontKind,
        name: &str,
        data: &[u8],
    ) -> Result<(), JsValue> {
        let h2 = self.document.create_element("h2")?;
        h2.set_text_content(Some(name));
        h2.append_with_str_1(" ")?;

        let small = self.document.create_element("small")?;
        small
            .class_list()
            .add_2("text-secondary", "d-inline-block")?;
        small.set_text_content(Some(font_kind.file_format_name()));
        h2.append_child(&small)?;
        self.output.append_child(&h2)?;

        let (font_name, _ext) = match name.rsplit_once('.') {
            Some((name, ext)) => (name, ext),
            None => (name, font_kind.extension()),
        };

        match font_kind {
            FontKind::Editor => {
                let eset = self.parse_eset(data)?;
                self.show_eset(&eset)?;
            }
            FontKind::Printer(_) => {
                let pset = self.parse_pset(data)?;
                self.show_pset(&pset)?;
            }
        }

        if let Some(mapping) = sdo_fonts::mappings::lookup(font_name) {
            self.show_mapping(mapping, font_name, true)?;
        }
        Ok(())
    }

    fn _trace_letter(&mut self, pset: &PSet<'_>) -> Result<(), JsValue> {
        let char_capital_a = &pset.chars[b'A' as usize];
        let page = raster::Page::from(char_capital_a);
        let blob = page_to_blob(&page)?;

        let window = window().ok_or("expected window")?;
        let _p = window.create_image_bitmap_with_blob(&blob)?;

        let canvas = self
            .document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        canvas.set_width(700);
        canvas.set_height(900);
        self.output.append_child(&canvas)?;
        let ctx = canvas
            .get_context("2d")?
            .ok_or("context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let callback = Closure::new(move |_v: JsValue| {
            let img = _v.dyn_into::<ImageBitmap>().unwrap();
            let w = img.width() * 10;
            let h = img.height() * 10;
            ctx.set_fill_style_str("green");
            //ctx.fill_rect(0.0, 0.0, 150.0, 100.0);
            ctx.draw_image_with_image_bitmap_and_dw_and_dh(&img, 10.0, 10.0, w as f64, h as f64)
                .unwrap();

            // Implement the rest of https://potrace.sourceforge.net/potrace.pdf
            for (x, y) in page.vertices() {
                ctx.fill_rect((9 + x * 10) as f64, (9 + y * 10) as f64, 2.0, 2.0);
            }

            ctx.set_stroke_style_str("blue");
            if let Some(mut iter) = page.first_outline() {
                super::log_val("Test", &JsValue::TRUE);
                let (x0, y0) = iter.next().unwrap();
                ctx.begin_path();
                ctx.move_to((10 + x0 * 10) as f64, (10 + y0 * 10) as f64);
                for (x, y) in iter {
                    ctx.line_to((10 + x * 10) as f64, (10 + y * 10) as f64);
                }
                ctx.stroke();
            }
        });
        let _ = _p.then(&callback);
        self.closures.push(callback);
        Ok(())
    }

    pub(super) fn eset_card(
        &self,
        list_item: &Element,
        eset: &ESet<'_>,
        name: &str,
    ) -> Result<(), JsValue> {
        let chset = name.split_once('.').map(|a| a.0).unwrap_or(name);
        let text = BStr::new(chset.as_bytes());
        let page = render_editor_text(text, eset).map_err(|v| {
            let err = format!("Failed to render editor font name: {}", v);
            JsError::new(&err)
        })?;
        let blob = page_to_blob(&page)?;
        let img = blob_image_el(&blob)?;
        list_item.append_child(&img)?;
        Ok(())
    }

    pub(super) fn pset_card(
        &self,
        list_item: &Element,
        pset: &PSet<'_>,
        name: &str,
    ) -> Result<(), JsValue> {
        let ch = name
            .chars()
            .next()
            .and_then(|c| c.try_into().ok())
            .unwrap_or(b'A');
        let page = render_printer_char(ch, pset)
            .ok_or_else(|| JsError::new("Failed to render printer char"))?;
        let (width, height) = (page.bit_width(), page.bit_height());
        log::trace!("Page generated ({width}x{height})");
        if width > 0 && height > 0 {
            let blob = page_to_blob(&page)?;
            let img = blob_image_el(&blob)?;
            list_item.append_child(&img)?;
        }
        Ok(())
    }
}
