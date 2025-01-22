#![allow(non_snake_case)] // wasm_bindgen macro

use bstr::BStr;
use convert::page_to_blob;
use dom::blob_image_el;
use glue::{
    fs_file_handle_get_file, js_directory_get_file_handle, js_error_with_cause, js_file_data,
    js_input_file_list, js_input_files_iter, slice_to_blob,
};
use js_sys::{Array, JsString, Uint8Array};
use log::{info, warn, Level};
use sdo_pdf::{generate_pdf, MetaInfo};
use sdo_util::keymap::{KB_DRAW, NP_DRAW};
use signum::{
    chsets::{
        cache::{AsyncIterator, ChsetCache, VfsDirEntry, VFS},
        editor::{parse_eset, ESet},
        encoding::decode_atari_str,
        printer::{parse_pset, PSet, PrinterKind},
        FontKind,
    },
    docs::{
        container::parse_sdoc0001_container,
        four_cc,
        hcim::{parse_image, Hcim, ImageSite},
        header, pbuf,
        tebu::PageText,
        DocumentInfo, GenerationContext, Overrides, SDoc,
    },
    raster::{self, render_doc_page, render_editor_text, render_printer_char},
    util::FourCC,
};
use std::{ffi::OsStr, fmt::Write, io::BufWriter};
use vfs::{DirEntry, OriginPrivateFS};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, window, Blob, CanvasRenderingContext2d, Document, Element, Event,
    FileSystemFileHandle, FileSystemGetFileOptions, FileSystemWritableFileStream,
    HtmlAnchorElement, HtmlCanvasElement, HtmlElement, HtmlInputElement, ImageBitmap,
};

mod convert;
mod dom;
mod glue;
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

fn js_four_cc(arr: &Uint8Array) -> Option<FourCC> {
    if arr.length() <= 4 {
        None
    } else {
        Some(FourCC::new([
            arr.get_index(0),
            arr.get_index(1),
            arr.get_index(2),
            arr.get_index(3),
        ]))
    }
}

pub struct ActiveDocument {
    sdoc: SDoc<'static>,
    di: DocumentInfo,
    pd: FontKind,
    name: String,
}

impl GenerationContext for ActiveDocument {
    fn image_sites(&self) -> &[ImageSite] {
        self.sdoc.image_sites()
    }

    fn document_info(&self) -> &DocumentInfo {
        &self.di
    }

    fn text_pages(&self) -> &[PageText] {
        &self.sdoc.tebu.pages
    }

    fn page_at(&self, index: usize) -> Option<&pbuf::Page> {
        self.sdoc.pbuf.page_at(index)
    }
}

#[wasm_bindgen]
pub struct Handle {
    document: Document,
    output: HtmlElement,
    input: HtmlInputElement,
    fs: OriginPrivateFS,
    #[allow(dead_code)]
    closures: Vec<Closure<dyn FnMut(JsValue)>>,
    fc: ChsetCache,

    active: Option<ActiveDocument>,
}

#[wasm_bindgen]
impl Handle {
    #[wasm_bindgen(constructor)]
    pub fn new(output: HtmlElement, input: HtmlInputElement) -> Result<Handle, JsValue> {
        let window = window().ok_or(JsError::new("failed to get window global"))?;
        let document = window
            .document()
            .ok_or(JsError::new("failed to get document"))?;
        let h = Self {
            document,
            output,
            fs: OriginPrivateFS::new(),
            fc: ChsetCache::new(),
            closures: Vec::new(),
            input,
            active: None,
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
            ar.push(JsString::from(name.as_ref()).as_ref());
            html.push_str(&name);
            html.push_str("</li>");
        }
        html.push_str("</ol>");
        log_array("cset", ar);
        el_cset.set_inner_html(&html);
        self.output.append_child(&el_cset)?;
        Ok(())
    }

    fn _write_hcim(&self, hcim: &Hcim<'_>) -> Result<(), JsValue> {
        let el_hcim = self.document.create_element("section")?;
        let heading = self.document.create_element("h3")?;
        heading.set_inner_html("Embedded Images");
        el_hcim.append_child(&heading)?;
        for (i, im) in hcim.images.iter().enumerate() {
            match parse_image(im) {
                Ok((_rest, image)) => {
                    let blob = page_to_blob(&image.image.into())?;

                    let el_figure = self.document.create_element("figure")?;
                    let el_image = blob_image_el(&blob)?;
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

    fn parse_sdoc<'a>(&self, data: &'a [u8]) -> Result<SDoc<'a>, JsValue> {
        match parse_sdoc0001_container::<signum::nom::error::Error<&'a [u8]>>(data) {
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

    fn parse_eset<'a>(&self, data: &'a [u8]) -> Result<ESet<'a>, JsValue> {
        let (_, eset) =
            parse_eset(data).map_err(|e| js_error_with_cause(e, "Failed to parse editor font"))?;
        Ok(eset)
    }

    fn parse_pset<'a>(&mut self, data: &'a [u8]) -> Result<PSet<'a>, JsValue> {
        let (_, pset) = parse_pset::<signum::nom::error::Error<&'a [u8]>>(data)
            .map_err(|e| js_error_with_cause(e, "Failed to parse printer font"))?;
        Ok(pset)
    }

    fn show_eset(&mut self, eset: &ESet<'_>) -> Result<(), JsValue> {
        self.eset_kb(eset)?;
        Ok(())
    }

    fn show_pset(&mut self, pset: &PSet<'_>) -> Result<(), JsValue> {
        let h3 = self.document.create_element("h3")?;
        h3.set_text_content(Some("Characters"));
        self.output.append_child(&h3)?;

        let el_table_responsive = self.document.create_element("div")?;
        el_table_responsive.class_list().add_1("table-responsive")?;
        let el_table = self.document.create_element("table")?;
        el_table.class_list().add_1("table")?;
        for crow in pset.chars.chunks(16) {
            let el_tr = self.document.create_element("tr")?;
            el_table.append_child(&el_tr)?;
            for c in crow {
                let el_td = self.document.create_element("td")?;
                el_tr.append_child(&el_td)?;
                if c.height > 0 {
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
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) -> Result<(), JsValue> {
        self.output.set_inner_html("");
        self.active = None;
        Ok(())
    }

    fn input_file(&self, name: &str) -> Result<web_sys::File, JsValue> {
        js_input_files_iter(&self.input)?
            .flatten()
            .find(|file| file.name() == name)
            .ok_or_else(|| JsError::new("File not found").into())
    }

    #[wasm_bindgen(js_name = addToCollection)]
    pub async fn add_to_collection(&mut self) -> Result<usize, JsValue> {
        self.fc.reset();
        let root_dir = self.fs.root_dir()?;
        let chset_dir = self.fs.chset_dir().await?;
        let opts = FileSystemGetFileOptions::new();
        opts.set_create(true);
        let mut count = 0;
        for file in js_input_files_iter(&self.input)? {
            let file = file?;
            let data = js_file_data(&file).await?;

            let four_cc =
                js_four_cc(&data).ok_or_else(|| JsError::new("Failed to parse file format"))?;
            let name = file.name().to_uppercase();
            // (name, data, four_cc)
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
            let o = JsFuture::from(w.close()).await?;
            assert_eq!(o, JsValue::UNDEFINED);
            console::info_3(&"Added".into(), &name.into(), &"to collection!".into());
            count += 1;
        }
        Ok(count)
    }

    #[wasm_bindgen(js_name = exportToPdf)]
    pub async fn export_to_pdf(&mut self) -> Result<Blob, JsError> {
        let active_doc = self
            .active
            .as_ref()
            .ok_or_else(|| JsError::new("no active document"))?;
        let overrides = Overrides {
            xoffset: 0,
            yoffset: 0,
        };
        let mut meta = MetaInfo {
            title: Some(active_doc.name.clone()),
            ..MetaInfo::default()
        };
        meta.with_dates(&active_doc.sdoc.header);
        let pk = match active_doc.pd {
            FontKind::Editor => Err(JsError::new("editor font not supported")),
            FontKind::Printer(printer_kind) => Ok(printer_kind),
        }?;
        let pdf = generate_pdf(&self.fc, pk, &meta, &overrides, active_doc)?;
        let vec = Vec::new();
        let mut writer = BufWriter::new(vec);
        pdf.write(&mut writer)?;
        let bytes = writer.into_inner()?;
        let blob = slice_to_blob(&bytes, "application/pdf")
            .map_err(|_v| JsError::new("failed to create pdf blob"))?;
        Ok(blob)
    }

    #[wasm_bindgen]
    pub async fn render(&mut self, requested_index: usize) -> Result<Blob, JsValue> {
        if let Some(ActiveDocument { sdoc, di, pd, .. }) = &self.active {
            if let Some(page_text) = sdoc.tebu.pages.get(requested_index) {
                let index = page_text.index as usize;
                log::info!("Rendering page {} ({})", requested_index, index);
                if let Some((pbuf_entry, _)) = sdoc.pbuf.pages[index].as_ref() {
                    let page = render_doc_page(
                        page_text,
                        pbuf_entry,
                        sdoc.image_sites(),
                        di,
                        *pd,
                        &self.fc,
                    );

                    let blob = page_to_blob(&page)?;
                    Ok(blob)
                } else {
                    warn!("Missing page {index}");
                    Err("Missing page in pbuf".into())
                }
            } else {
                Err("Missing page in tebu".into())
            }
        } else {
            Err("No document active for rendering".into())
        }
    }

    #[wasm_bindgen(js_name = hasActive)]
    pub fn has_active(&self) -> bool {
        self.active.is_some()
    }

    #[wasm_bindgen(js_name = activePageCount)]
    pub fn active_page_count(&self) -> Option<usize> {
        self.active
            .as_ref()
            .map(|active| active.sdoc.tebu.pages.len())
    }

    async fn show_staged(&mut self, name: &str) -> Result<(), JsValue> {
        let file = self.input_file(name)?;
        let data = js_file_data(&file).await?.to_vec();

        let (_, four_cc) =
            four_cc::<()>(&data).map_err(|_| JsError::new("File has less than 4 bytes"))?;

        if four_cc == FourCC::SDOC {
            let heading = self.document.create_element("h2")?;
            heading.set_text_content(Some(name));
            self.output.append_child(&heading)?;

            let sdoc = self.parse_sdoc(&data)?;
            self.fc.reset();
            let dfci = self.fc.load(&self.fs, &sdoc.cset).await;
            let pd = match dfci.print_driver(None) {
                Some(pd) => pd,
                None => {
                    // FIXME: pick the "best" format?
                    log::warn!("Could not auto-select a font format, some fonts are not available");
                    FontKind::Printer(PrinterKind::Needle24)
                }
            };
            let images = sdoc
                .hcim
                .as_ref()
                .map(|hcim| hcim.decode_images())
                .unwrap_or_default();

            self.active = Some(ActiveDocument {
                sdoc: sdoc.into_owned(),
                di: DocumentInfo::new(dfci, images),
                pd,
                name: name.to_owned(),
            });
        } else if let Some(font_kind) = Option::<FontKind>::from(four_cc) {
            self.show_font(font_kind, name, &data).await?;
        } else {
            warn!("Unknown format: {}", four_cc);
            let heading = self.document.create_element("h2")?;
            heading.set_text_content(Some(name));
            self.output.append_child(&heading)?;
            let p = self.document.create_element("p")?;
            p.append_with_str_1("Unknown format: ")?;
            p.append_with_str_1(&four_cc.as_bstr().to_string())?;
            self.output.append_child(&p)?;
        }
        Ok(())
    }

    async fn show_home(&mut self) -> Result<(), JsValue> {
        let node = self.document.create_element("div")?;
        node.class_list()
            .add_4("p-5", "mb-4", "bg-body-tertiary", "rounded-3")?;
        let container = self.document.create_element("div")?;
        container.class_list().add_2("container-fluid", "py-5")?;
        let h1 = self.document.create_element("h1")?;
        h1.class_list().add_2("display-5", "fw-bold")?;
        h1.set_text_content(Some("Welcome to SDO Studio!"));
        container.append_child(&h1)?;

        let p = self.document.create_element("p")?;
        p.class_list().add_2("col-md-8", "fs-4")?;
        p.set_text_content(Some(
            "Please select a file (*.SDO, *.E24, *.P24, *.P09, *.L30)",
        ));
        container.append_child(&p)?;

        let file_list = js_input_file_list(&self.input)?;
        if file_list.length() > 0 {
            let button = self
                .document
                .create_element("a")?
                .unchecked_into::<HtmlAnchorElement>();
            button.class_list().add_3("btn", "btn-primary", "btn-lg")?;
            button.set_href("#/staged/");
            button.set_text_content(Some("See staged files"));
            container.append_child(&button)?;
        }

        node.append_child(&container)?;
        self.output.append_child(&node)?;
        Ok(())
    }

    #[wasm_bindgen]
    pub async fn open(&mut self, fragment: &str) -> Result<(), JsValue> {
        self.reset()?;
        if let Some(rest) = fragment.strip_prefix("#/staged/") {
            if rest.is_empty() {
                self.on_change().await?;
            } else {
                self.show_staged(rest).await?;
            }
        } else if matches!(fragment, "" | "#" | "#/") {
            self.show_home().await?;
        } else if let Some(rest) = fragment.strip_prefix("#/CHSETS/") {
            if rest.is_empty() {
                self.list_chsets().await?;
            } else {
                self.show_chset(rest).await?;
            }
        }
        Ok(())
    }

    async fn show_font(
        &mut self,
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
        Ok(())
    }

    async fn show_chset(&mut self, name: &str) -> Result<(), JsValue> {
        let chsets = self.fs.chset_dir().await?;
        let file_handle = js_directory_get_file_handle(&chsets, name).await?;
        let file = fs_file_handle_get_file(&file_handle).await?;
        let arr = js_file_data(&file).await?;
        let four_cc = js_four_cc(&arr).ok_or(js_sys::Error::new("No four-cc: file too short"))?;
        if let Some(font_kind) = Option::<FontKind>::from(four_cc) {
            let data = arr.to_vec();
            self.show_font(font_kind, name, &data).await?;
        }
        Ok(())
    }

    async fn list_chset_entry(&mut self, entry: &DirEntry) -> Result<(), JsValue> {
        let path = entry.path();
        let name = path.file_name().map(OsStr::to_string_lossy);
        let name = name.as_deref().unwrap_or("");

        let file = self.fs.open_dir_entry(entry).await?;
        let data = js_file_data(&file).await?.to_vec();

        let (_, four_cc) =
            four_cc::<()>(&data).map_err(|_| JsError::new("Failed to parse FourCC"))?;
        info!("Loading {} ({})", name, four_cc);
        let href = format!("#/CHSETS/{}", name);
        let card = self.card(name, four_cc, &href)?;
        if let Err(e) = self.card_preview(&card, name, four_cc, &data).await {
            console::error_3(
                &JsValue::from_str("Failed to generate preview"),
                &JsValue::from_str(name),
                &e,
            );
        }
        self.output.append_child(&card)?;
        Ok(())
    }

    async fn list_chsets(&mut self) -> Result<(), JsValue> {
        let mut iter = self.fs.read_dir(OriginPrivateFS::chsets_path()).await?;
        while let Some(next) = iter.next().await {
            let entry = next?;
            if self.fs.is_file_entry(&entry) {
                if let Err(e) = self.list_chset_entry(&entry).await {
                    let path = entry.path();
                    let path = path.to_string_lossy();
                    console::log_2(&JsValue::from_str(&path), &e);
                }
            }
        }
        info!("Done listing charsets");
        Ok(())
    }

    #[wasm_bindgen(js_name = onChange)]
    pub async fn on_change(&mut self) -> Result<(), JsValue> {
        self.reset()?;
        for file in js_input_files_iter(&self.input)? {
            let file = file?;
            let arr = js_file_data(&file).await?;
            self.stage(&file.name(), arr).await?;
        }
        Ok(())
    }

    fn card(&self, name: &str, four_cc: FourCC, href: &str) -> Result<Element, JsValue> {
        let card = self.document.create_element("a")?;
        let kind = decode_atari_str(four_cc.as_slice());
        card.class_list()
            .add_3("list-group-item", "list-group-item-action", kind.as_ref())?;
        card.set_attribute("href", href)?;
        self.card_body(&card, name, four_cc)?;
        Ok(card)
    }

    async fn card_preview(
        &mut self,
        card: &Element,
        name: &str,
        four_cc: FourCC,
        data: &[u8],
    ) -> Result<(), JsValue> {
        match four_cc {
            FourCC::SDOC => {
                let doc = self.parse_sdoc(data)?;
                self.sdoc_card(card, &doc).await?;
            }
            FourCC::ESET => {
                log::info!("{name}: Signum Editor Bitmap Font");
                let eset = self.parse_eset(data)?;
                log::info!("{name}: Parsed editor font");
                self.eset_card(card, &eset, name)?;
            }
            FourCC::PS24 | FourCC::PS09 | FourCC::LS30 => {
                log::info!("{name}: {}", four_cc.file_format_name().unwrap());
                let pset = self.parse_pset(data)?;
                log::info!("{name}: Parsed printer font");
                self.pset_card(card, &pset, name)?;
            }
            k => {
                log::warn!("Unknown File Format '{}'", k);
            }
        }
        Ok(())
    }

    async fn stage(&mut self, name: &str, arr: Uint8Array) -> Result<(), JsValue> {
        let data = arr.to_vec();
        info!("Parsing file '{}'", name);

        if let Ok((_, four_cc)) = four_cc::<()>(&data) {
            let href = format!("#/staged/{name}");
            let card = self.card(name, four_cc, &href)?;
            if let Err(e) = self.card_preview(&card, name, four_cc, &data).await {
                console::error_3(
                    &JsValue::from_str("Failed to generate preview"),
                    &JsValue::from_str(name),
                    &e,
                );
            }
            self.output.append_child(&card)?;
            Ok(())
        } else {
            log::warn!("File is less than 4 bytes long");
            Ok(())
        }
    }

    fn card_body(&self, card_body: &Element, name: &str, four_cc: FourCC) -> Result<(), JsValue> {
        let card_title = self.document.create_element("h5")?;
        card_title.class_list().add_1("card-title")?;
        card_title.set_text_content(Some(name));
        card_body.append_child(&card_title)?;
        let card_subtitle = self.document.create_element("h6")?;
        card_subtitle
            .class_list()
            .add_3("card-subtitle", "mb-2", "text-body-secondary")?;
        card_subtitle.set_text_content(Some(four_cc.file_format_name().unwrap_or("Unknown")));
        card_body.append_child(&card_subtitle)?;
        Ok(())
    }

    fn eset_card(
        &mut self,
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

    fn pset_card(
        &mut self,
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

    async fn sdoc_card(&mut self, list_item: &Element, doc: &SDoc<'_>) -> Result<(), JsValue> {
        let header_info = self.document.create_element("div")?;
        header_info.class_list().add_1("mb-2")?;
        let mut text = format!(
            "Created: {} | Modified: {} | Text Pages: {}",
            doc.header.ctime,
            doc.header.mtime,
            doc.tebu.pages.len()
        );
        if let Some(hcim) = &doc.hcim {
            write!(text, " | Embedded images: {}", hcim.header.img_count).unwrap();
        }
        header_info.set_text_content(Some(&text));
        list_item.append_child(&header_info)?;
        let chset_list = self.document.create_element("ol")?;
        chset_list
            .class_list()
            .add_2("list-group", "list-group-horizontal-md")?;
        info!("Loading charsets");
        for chset in doc.cset.names.iter().filter(|c| !c.is_empty()) {
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
        list_item.append_child(&chset_list)?;

        Ok(())
    }
}
