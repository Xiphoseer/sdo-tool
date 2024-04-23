#![allow(non_snake_case)] // wasm_bindgen macro

use bstr::BStr;
use image::ImageOutputFormat;
use js_sys::{Array, Uint8Array};
use log::{info, Level};
use sdo_util::keymap::{KB_DRAW, NP_DRAW};
use signum::{
    chsets::{
        cache::{AsyncIterator, ChsetCache, FontCacheInfo, VFS},
        editor::parse_eset,
        encoding::decode_atari_str,
        printer::parse_ps24,
    },
    docs::{
        container::parse_sdoc0001_container,
        four_cc,
        hcim::{parse_image, Hcim},
        header, SDoc,
    },
    raster,
    util::FourCC,
};
use std::{fmt::Write, future::IntoFuture, io::Cursor, path::Path};
use vfs::OriginPrivateFS;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, Blob, BlobPropertyBag, CanvasRenderingContext2d, Document, Element, Event,
    FileSystemFileHandle, FileSystemGetFileOptions, FileSystemWritableFileStream,
    HtmlCanvasElement, HtmlElement, HtmlImageElement, ImageBitmap, Url,
};

mod vfs;

/*
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}
*/

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
    fs: OriginPrivateFS,
    closures: Vec<Closure<dyn FnMut(JsValue)>>,

    fc: ChsetCache,

    staged: Vec<(String, Uint8Array, FourCC)>,
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
            fs: OriginPrivateFS::new(),
            fc: ChsetCache::new(),
            closures: Vec::new(),
            staged: Vec::new(),
        };
        log::info!("New handle created!");
        Ok(h)
    }

    #[wasm_bindgen]
    pub async fn init(&mut self) -> Result<(), JsValue> {
        self.fs.init().await?;
        Ok(())
    }

    fn _write0001(&self, header: &header::Header<'_>) -> Result<(), JsValue> {
        log::info!("Created: {}", &header.ctime);
        log::info!("Modified: {}", &header.mtime);
        let el_header = self.document.create_element("section")?;
        let text = format!("Created: {}<br>Modified: {}", header.ctime, header.mtime);
        el_header.set_inner_html(&text);
        self.output.append_child(&el_header)?;
        Ok(())
    }

    fn _write_cset(&self, charsets: &[&bstr::BStr]) -> Result<(), JsValue> {
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
        let url = Url::create_object_url_with_blob(blob)?;
        let el_image = HtmlImageElement::new()?;
        el_image.set_src(&url);
        Ok(el_image)
    }

    fn _write_hcim(&self, hcim: &Hcim<'_>) -> Result<(), JsValue> {
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

    /*
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
     */

    fn parse_sdoc<'a>(&self, data: &'a [u8]) -> Result<SDoc<'a>, JsValue> {
        match parse_sdoc0001_container(data) {
            Ok((_rest, container)) => match SDoc::unpack(container) {
                Ok(res) => {
                    log("Parsing complete");
                    Ok(res)
                }
                Err(e) => Err(JsError::new(&format!("Failed to parse: {:?}", e)).into()),
            },
            Err(_e) => Err(JsError::new("Failed to parse SDO container").into()),
        }
    }

    fn _parse_eset(&self, data: &[u8]) -> Result<(), JsValue> {
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

    fn _parse_ps24(&mut self, data: &[u8]) -> Result<(), JsValue> {
        log::info!("Signum 24-Needle Printer Bitmap Font");
        match parse_ps24(data) {
            Ok((_, pset)) => {
                log::info!("Parsed Printer Font");
                let el_table = self.document.create_element("table")?;
                self.output.append_child(&el_table)?;
                for crow in pset.chars.chunks(16) {
                    let el_tr = self.document.create_element("tr")?;
                    el_table.append_child(&el_tr)?;
                    for c in crow {
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
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.output.set_inner_html("");
        self.staged.clear();
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn add_to_collection(&mut self) -> Result<(), JsValue> {
        let root_dir = self.fs.root_dir()?;
        let chset_dir = self.fs.chset_dir().await?;
        let mut opts = FileSystemGetFileOptions::new();
        opts.create(true);
        for (name, data, four_cc) in self.staged.drain(..) {
            let dir = match four_cc {
                FourCC::ESET | FourCC::PS24 | FourCC::PS09 | FourCC::LS30 => &chset_dir,
                _ => root_dir,
            };
            let r = JsFuture::from(dir.get_file_handle_with_options(&name, &opts))
                .await?
                .unchecked_into::<FileSystemFileHandle>();
            let w = JsFuture::from(r.create_writable())
                .await?
                .unchecked_into::<FileSystemWritableFileStream>();
            let o = JsFuture::from(w.write_with_buffer_source(&data)?).await?;
            assert_eq!(o, JsValue::UNDEFINED);
            console::info_3(&"Added".into(), &name.into(), &"to collection!".into());
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn stage(&mut self, name: &str, arr: Uint8Array) -> Result<(), JsValue> {
        let data = arr.to_vec();
        info!("Parsing file '{}'", name);

        if let Ok((_, four_cc)) = four_cc(&data) {
            if ACCEPT.contains(&four_cc) {
                self.staged.push((name.to_owned(), arr, four_cc))
            } else {
                log::warn!("Unknown File Format '{}'", four_cc);
                return Ok(());
            }
            let card = self.document.create_element("div")?;
            card.class_list()
                .add_2("card", decode_atari_str(&FourCC::SDOC).as_ref())?;
            let card_body = self.card_body(name, four_cc)?;
            match four_cc {
                FourCC::SDOC => {
                    let doc = self.parse_sdoc(&data)?;
                    self.sdoc_card(&card_body, &doc).await?;
                }
                FourCC::ESET => {
                    // self.parse_eset(&data)
                }
                FourCC::PS24 => {
                    // self.parse_ps24(&data)
                }
                k => {
                    log::warn!("Unknown File Format '{}'", k);
                }
            }
            card.append_child(&card_body)?;
            self.output.append_child(&card)?;
            Ok(())
        } else {
            log::warn!("File is less than 4 bytes long");
            Ok(())
        }
    }

    fn card_body(&self, name: &str, four_cc: FourCC) -> Result<Element, JsValue> {
        let card_body = self.document.create_element("div")?;
        card_body.class_list().add_1("card-body")?;
        let card_title = self.document.create_element("h5")?;
        card_title.class_list().add_1("card-title")?;
        card_title.set_text_content(Some(name));
        card_body.append_child(&card_title)?;
        let card_subtitle = self.document.create_element("h6")?;
        card_subtitle
            .class_list()
            .add_3("card-subtitle", "mb-2", "text-body-secondary")?;
        card_subtitle.set_text_content(Some(match four_cc {
            FourCC::SDOC => "Signum! Document",
            FourCC::ESET => "Signum! Editor Font",
            FourCC::PS24 => "Signum! 24-Needle Printer Font",
            FourCC::PS09 => "Signum! 9-Needle Printer Font",
            FourCC::LS30 => "Signum! Laser Printer Font",
            FourCC::BIMC => "Signum! Hardcopy Image",
            _ => "Unknown",
        }));
        card_body.append_child(&card_subtitle)?;
        Ok(card_body)
    }

    async fn sdoc_card(&mut self, card_body: &Element, doc: &SDoc<'_>) -> Result<(), JsValue> {
        let header_info = self.document.create_element("div")?;
        header_info.class_list().add_1("mb-2")?;
        let mut text = format!(
            "Created: {} | Modified: {}",
            doc.header.ctime, doc.header.mtime
        );
        if let Some(hcim) = &doc.hcim {
            write!(text, " | Embedded images: {}", hcim.header.img_count).unwrap();
        }
        header_info.set_text_content(Some(&text));
        card_body.append_child(&header_info)?;
        let chset_list = self.document.create_element("ol")?;
        chset_list
            .class_list()
            .add_2("list-group", "list-group-horizontal-md")?;
        info!("Loading charsets");
        for chset in doc.cset.names.iter().cloned().filter(|c| !c.is_empty()) {
            info!("Loading {}", chset);
            let (cls, tooltip) = {
                let cset_index = self.fc.load_cset(&self.fs, chset).await;
                console::log_2(&"Font Index".into(), &cset_index.into());
                let cset = self.fc.cset(cset_index).unwrap();
                if cset.e24().is_none() {
                    (
                        "list-group-item-danger",
                        format!("Missing Editor Font {chset}.E24"),
                    )
                } else {
                    let mut missing = vec![];
                    if cset.p24().is_none() {
                        missing.push(format!("{chset}.P24"));
                    }
                    if cset.p09().is_none() {
                        missing.push(format!("{chset}.P09"));
                    }
                    if cset.l30().is_none() {
                        missing.push(format!("{chset}.L30"));
                    }
                    if missing.is_empty() {
                        ("list-group-item-success", "All fonts present".to_string())
                    } else {
                        (
                            "list-group-item-warning",
                            format!("Missing Printer Font {}", missing.join(", ")),
                        )
                    }
                }
            };

            let chset_li = self.document.create_element("li")?;
            chset_li.class_list().add_2("list-group-item", cls)?;
            let text = decode_atari_str(chset);
            chset_li.set_text_content(Some(text.as_ref()));
            chset_li.set_attribute("title", &tooltip)?;
            chset_list.append_child(&chset_li)?;
        }
        info!("Done Loading charsets");
        card_body.append_child(&chset_list)?;

        Ok(())
    }
}

const ACCEPT: &[FourCC] = &[
    FourCC::SDOC,
    FourCC::ESET,
    FourCC::PS09,
    FourCC::PS24,
    FourCC::LS30,
    FourCC::BIMC,
];
