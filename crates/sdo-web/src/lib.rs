use image::ImageOutputFormat;
use js_sys::{Array, Uint8Array};
use log::{info, Level};
use sdo_util::keymap::{KB_DRAW, NP_DRAW};
use signum::{
    chsets::{editor::parse_eset, encoding::decode_atari_str, printer::parse_ps24},
    docs::{
        container::parse_sdoc0001_container,
        four_cc,
        hcim::{parse_image, Hcim},
        header, SDoc,
    },
    raster,
    util::FourCC,
};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use web_sys::{
    window, Blob, BlobPropertyBag, CanvasRenderingContext2d, Document, Event, HtmlCanvasElement,
    HtmlElement, HtmlImageElement, ImageBitmap, Url,
};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    if let Err(e) = console_log::init_with_level(Level::Debug) {
        error(&format!("Failed to set up logger: {}", e));
    }

    Ok(())
}

// Use `js_namespace` here to bind `console.log(..)` instead of just
// `log(..)`
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn error_val(e: JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_val(key: &str, e: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_array(name: &str, e: Array);
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
#[allow(dead_code)]
pub struct Module {
    callback: Closure<dyn FnMut(Event)>,
}

#[wasm_bindgen]
pub struct Handle {
    document: Document,
    output: HtmlElement,
    closures: Vec<Closure<dyn FnMut(JsValue)>>,
}

#[wasm_bindgen]
impl Handle {
    #[wasm_bindgen(constructor)]
    pub fn new(output: HtmlElement) -> Result<Handle, JsValue> {
        let h = Self {
            document: window()
                .ok_or(JsValue::NULL)?
                .document()
                .ok_or(JsValue::NULL)?,
            output,
            closures: Vec::new(),
        };
        log::info!("New handle created!");
        Ok(h)
    }

    fn write0001(&self, header: &header::Header<'_>) -> Result<(), JsValue> {
        log::info!("Created: {}", &header.ctime);
        log::info!("Modified: {}", &header.mtime);
        let el_header = self.document.create_element("section")?;
        let text = format!("Created: {}<br>Modified: {}", header.ctime, header.mtime);
        el_header.set_inner_html(&text);
        self.output.append_child(&el_header)?;
        Ok(())
    }

    fn write_cset(&self, charsets: &[&bstr::BStr]) -> Result<(), JsValue> {
        let el_cset = self.document.create_element("section")?;
        let ar = Array::new();
        let mut html = "<h3>Character Sets</h3><ol>".to_string();
        for chr in charsets {
            let name = decode_atari_str(chr.as_ref());
            html.push_str("<li>");
            ar.push(js_sys::JsString::from(name.as_ref()).as_ref());
            html.push_str(&name);
            html.push_str("</li>");
        }
        html.push_str("</ol>");
        log_array("cset", ar);
        el_cset.set_inner_html(&html);
        self.output.append_child(&el_cset)?;
        Ok(())
    }

    fn page_as_blob(&self, page: &raster::Page) -> Result<Blob, JsValue> {
        let mut buffer = Cursor::new(Vec::<u8>::new());
        page.to_alpha_image()
            .write_to(&mut buffer, ImageOutputFormat::Png)
            .unwrap();
        Blob::new_with_u8_array_sequence_and_options(
            &Array::from_iter([Uint8Array::from(buffer.get_ref().as_slice())]),
            &{
                let mut bag = BlobPropertyBag::new();
                bag.type_("image/png");
                bag
            },
        )
    }

    fn blob_image_el(blob: &Blob) -> Result<HtmlImageElement, JsValue> {
        let url = Url::create_object_url_with_blob(&blob)?;
        let el_image = HtmlImageElement::new()?;
        el_image.set_src(&url);
        Ok(el_image)
    }

    fn write_hcim(&self, hcim: &Hcim<'_>) -> Result<(), JsValue> {
        let el_hcim = self.document.create_element("section")?;
        let heading = self.document.create_element("h3")?;
        heading.set_inner_html("Embedded Images");
        el_hcim.append_child(&heading)?;
        for (i, _im) in hcim.images.iter().enumerate() {
            match parse_image(_im.0) {
                Ok((_rest, image)) => {
                    let blob = self.page_as_blob(&image.image.into())?;

                    let el_figure = self.document.create_element("figure")?;
                    let el_image = Self::blob_image_el(&blob)?;
                    el_figure.append_child(&el_image)?;

                    let el_figcaption = self.document.create_element("figcaption")?;
                    el_figcaption.set_inner_html(&image.key);

                    el_figure.append_child(&el_figcaption)?;

                    el_hcim.append_child(&el_figure)?;
                }
                Err(e) => {
                    log::error!("Failed to parse image {}: {}", i, e);
                }
            }
        }
        if let Ok(tebu) = serde_wasm_bindgen::to_value(hcim) {
            log_val("hcim", &tebu);
        }
        self.output.append_child(&el_hcim)?;
        Ok(())
    }

    pub fn parse_sdoc(&self, data: &[u8]) -> Result<(), JsValue> {
        match parse_sdoc0001_container(&data) {
            Ok((_rest, container)) => {
                let doc = match SDoc::unpack(container) {
                    Ok(res) => {
                        log("Parsing complete");
                        res
                    }
                    Err(_e) => {
                        log(&format!("Failed to parse: {:?}", _e));
                        return Ok(());
                    }
                };

                self.write0001(&doc.header)?;

                // cset
                self.write_cset(&doc.charsets)?;

                // sysp
                if let Ok(sysp) = serde_wasm_bindgen::to_value(&doc.sysp) {
                    log_val("sysp", &sysp);
                }

                // pbuf
                if let Ok(pbuf) = serde_wasm_bindgen::to_value(&doc.pbuf) {
                    log_val("pbuf", &pbuf);
                }

                // tebu
                if let Ok(tebu) = serde_wasm_bindgen::to_value(&doc.tebu) {
                    log_val("tebu", &tebu);
                }

                // hcim
                if let Some(hcim) = &doc.hcim {
                    self.write_hcim(hcim)?;
                }

                for (key, val) in &doc.other {
                    if let Ok(bytes) = serde_wasm_bindgen::to_value(&val.0) {
                        log_val(&key.to_string(), &bytes)
                    }
                }
            }
            Err(_) => {
                console_log!("Failed to parse SDO container");
            }
        }
        Ok(())
    }

    fn parse_eset(&self, data: &[u8]) -> Result<(), JsValue> {
        log::info!("Signum Editor Bitmap Font");
        match parse_eset(data) {
            Ok((_, eset)) => {
                log::info!("Parsed Editor Font");
                let kb_img = KB_DRAW
                    .to_page(&eset)
                    .or(Err("Failed to draw Keyboard Map"))?;
                let np_img = NP_DRAW
                    .to_page(&eset)
                    .or(Err("Failed to draw Numpad Map"))?;

                let kb_blob = self.page_as_blob(&kb_img)?;
                let np_blob = self.page_as_blob(&np_img)?;

                let kb_img_el = Self::blob_image_el(&kb_blob)?;
                let np_img_el = Self::blob_image_el(&np_blob)?;

                self.output.append_child(&kb_img_el)?;
                self.output.append_child(&np_img_el)?;
            }
            Err(e) => {
                log::error!("Failed to parse editor font: {}", e);
            }
        }
        Ok(())
    }

    fn parse_ps24(&mut self, data: &[u8]) -> Result<(), JsValue> {
        log::info!("Signum 24-Needle Printer Bitmap Font");
        match parse_ps24(data) {
            Ok((_, pset)) => {
                log::info!("Parsed Printer Font");
                let el_table = self.document.create_element("table")?;
                self.output.append_child(&el_table)?;
                for (_rdx, crow) in pset.chars.chunks(16).enumerate() {
                    let el_tr = self.document.create_element("tr")?;
                    el_table.append_child(&el_tr)?;
                    for (_idx, c) in crow.iter().enumerate() {
                        //log::info!("Char {:x}{:x} {}x{}", rdx, idx, c.width, c.height);
                        let el_td = self.document.create_element("td")?;
                        el_tr.append_child(&el_td)?;
                        if c.height > 0 {
                            let page = raster::Page::from(c);
                            let blob = self.page_as_blob(&page)?;
                            let img_el = Self::blob_image_el(&blob)?;
                            el_td.append_child(&img_el)?;
                        }
                    }
                }

                let char_capital_a = &pset.chars[b'A' as usize];
                let page = raster::Page::from(char_capital_a);
                let blob = self.page_as_blob(&page)?;

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
                    ctx.set_fill_style(&"green".into());
                    //ctx.fill_rect(0.0, 0.0, 150.0, 100.0);
                    ctx.draw_image_with_image_bitmap_and_dw_and_dh(
                        &img, 10.0, 10.0, w as f64, h as f64,
                    )
                    .unwrap();

                    // Implement the rest of https://potrace.sourceforge.net/potrace.pdf
                    for (x, y) in page.vertices() {
                        ctx.fill_rect((9 + x * 10) as f64, (9 + y * 10) as f64, 2.0, 2.0);
                    }

                    ctx.set_stroke_style(&"blue".into());
                    if let Some(mut iter) = page.first_outline() {
                        log_val("Test", &JsValue::TRUE);
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
            }
            Err(e) => {
                log::error!("Failed to parse printer font: {}", e);
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn do_stuff(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        info!("Parsing file '{}'", name);
        self.output.set_inner_html("");

        if let Ok((_, four_cc)) = four_cc(data) {
            match four_cc {
                FourCC::SDOC => self.parse_sdoc(data),
                FourCC::ESET => self.parse_eset(data),
                FourCC::PS24 => self.parse_ps24(data),
                k => {
                    log::warn!("Unknown File Format '{}'", k);
                    Ok(())
                }
            }
        } else {
            log::warn!("File is less than 4 bytes long");
            Ok(())
        }
    }
}
