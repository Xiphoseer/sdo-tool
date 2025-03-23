#![allow(non_snake_case)] // wasm_bindgen macro

use convert::page_to_blob;
use dom::blob_image_el;
use glue::{
    fs::{
        directory_handle_get_file_handle, directory_handle_get_file_handle_with_options,
        file_handle_create_writable, file_handle_get_file, writable_file_stream_close,
        writable_file_stream_write_with_js_u8_array,
    },
    js_error_with_cause, js_file_data, js_input_file_list, js_input_files_iter, js_wrap_err,
    slice_to_blob,
};
use js_sys::{Array, Uint8Array};
use log::{info, warn, Level};
use sdo_pdf::{generate_pdf, MetaInfo};
use signum::{
    chsets::{cache::ChsetCache, encoding::decode_atari_str, v2::TAG_CSET2, FontKind},
    docs::{
        four_cc, hcim::ImageSite, pbuf, tebu::PageText, v3::TAG_SDOC3, DocumentInfo,
        GenerationContext, Overrides, SDoc,
    },
    images::imc::parse_imc,
    raster::{self, render_doc_page},
    util::{
        AsyncIterator, FileFormatKind, FileFormatKindV1, FourCC, Signum1Format, Signum3Format,
        SignumFormat, VFS,
    },
};
use std::{cell::RefCell, ffi::OsStr, io::BufWriter, path::Path};
use vfs::{DirEntry, OriginPrivateFS};
use wasm_bindgen::prelude::*;
use web_sys::{
    console, window, Blob, Document, Element, Event, FileSystemGetFileOptions, HtmlAnchorElement,
    HtmlElement, HtmlInputElement,
};

mod chset;
mod convert;
mod dom;
mod glue;
mod sdoc;
mod staged;
mod vfs;

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
    fc: ChsetCache,
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
    // fc: ChsetCache,
    active: RefCell<Option<ActiveDocument>>,
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
            closures: Vec::new(),
            input,
            active: RefCell::new(None),
        };
        log::info!("New handle created!");
        Ok(h)
    }

    #[wasm_bindgen]
    pub async fn init(&self) -> Result<(), JsValue> {
        self.fs.init().await?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reset(&self) -> Result<(), JsValue> {
        self.output.set_inner_html("");
        *self.active.borrow_mut() = None;
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
        //self.fc.reset();
        let root_dir = self.fs.root_dir()?.clone();
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
                _ => &root_dir,
            };
            let r = directory_handle_get_file_handle_with_options(dir, &name, &opts).await?;
            let w = file_handle_create_writable(&r).await?;
            let o = writable_file_stream_write_with_js_u8_array(&w, &data)?.await?;
            assert_eq!(o, JsValue::UNDEFINED);
            let o = writable_file_stream_close(&w).await?;
            assert_eq!(o, JsValue::UNDEFINED);
            console::info_3(&"Added".into(), &name.into(), &"to collection!".into());
            count += 1;
        }
        Ok(count)
    }

    #[wasm_bindgen(js_name = exportToPdf)]
    pub async fn export_to_pdf(&mut self) -> Result<Blob, JsError> {
        let active_doc_ref = self.active.borrow();
        let active_doc = active_doc_ref
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
        let pdf = generate_pdf(&active_doc.fc, pk, &meta, &overrides, active_doc)?;
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
        let active_doc_ref = self.active.borrow();
        if let Some(ActiveDocument {
            sdoc, di, pd, fc, ..
        }) = &*active_doc_ref
        {
            if let Some(page_text) = sdoc.tebu.pages.get(requested_index) {
                let index = page_text.index as usize;
                log::info!("Rendering page {} ({})", requested_index, index);
                if let Some((pbuf_entry, _)) = sdoc.pbuf.pages[index].as_ref() {
                    let page =
                        render_doc_page(page_text, pbuf_entry, sdoc.image_sites(), di, *pd, fc);

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
        self.active.borrow().is_some()
    }

    #[wasm_bindgen(js_name = activePageCount)]
    pub fn active_page_count(&self) -> Option<usize> {
        self.active
            .borrow()
            .as_ref()
            .map(|active| active.sdoc.tebu.pages.len())
    }

    async fn show_staged(&self, name: &str) -> Result<(), JsValue> {
        let file = self.input_file(name)?;
        let data = js_file_data(&file).await?.to_vec();

        let (_, four_cc) =
            four_cc::<()>(&data).map_err(|_| JsError::new("File has less than 4 bytes"))?;

        if four_cc == FourCC::SDOC {
            self.show_sdoc(name, &data).await?;
        } else if let Some(font_kind) = Option::<FontKind>::from(four_cc) {
            self.show_font(font_kind, name, &data).await?;
        } else if four_cc == FourCC::BIMC {
            self.show_image(name, &data).await?;
        } else if data.starts_with(TAG_SDOC3) {
            self.show_sdoc3(name, &data).await?;
        } else if data.starts_with(TAG_CSET2) {
            self.show_cset2(name, &data).await?;
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

    async fn show_home(&self) -> Result<(), JsValue> {
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
            "Please select a file (*.SDO, *.E24, *.P24, *.P09, *.L30, *.IMC)",
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

        self.list_docs().await?;

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn open(&self, fragment: &str) -> Result<(), JsValue> {
        info!("opening {:?}", fragment);
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
                self.list_chsets()
                    .await
                    .map_err(|e| js_wrap_err(e, "Failed to list CHSETS"))?;
            } else {
                self.show_chset(rest)
                    .await
                    .map_err(|e| js_wrap_err(e, "Failed to show CHSET"))?;
            }
        } else if let Some(name) = fragment.strip_prefix("#/") {
            self.show_file(name).await?;
        }
        Ok(())
    }

    async fn show_image(&self, _name: &str, data: &[u8]) -> Result<(), JsValue> {
        let (_header, decoded) =
            parse_imc(data).map_err(|err| js_error_with_cause(err, "Failed to parse IMC image"))?;
        let page = raster::Page::from(decoded);
        let blob = page_to_blob(&page)?;
        let img = blob_image_el(&blob)?;
        img.class_list().add_1("bimc")?;
        self.output.append_child(&img)?;
        Ok(())
    }

    async fn show_chset(&self, name: &str) -> Result<(), JsValue> {
        let chsets = self.fs.chset_dir().await?;
        let file_handle = directory_handle_get_file_handle(&chsets, name).await?;
        let file = file_handle_get_file(&file_handle).await?;
        let arr = js_file_data(&file).await?;
        let four_cc = js_four_cc(&arr).ok_or(js_sys::Error::new("No four-cc: file too short"))?;
        if let Some(font_kind) = Option::<FontKind>::from(four_cc) {
            let data = arr.to_vec();
            self.show_font(font_kind, name, &data).await?;
        }
        Ok(())
    }

    async fn show_file(&self, name: &str) -> Result<(), JsValue> {
        let root = self.fs.root_dir()?;
        let file_handle = directory_handle_get_file_handle(&root, name).await?;
        let file = file_handle_get_file(&file_handle).await?;
        let arr = js_file_data(&file).await?;
        let four_cc = js_four_cc(&arr).ok_or(js_sys::Error::new("No four-cc: file too short"))?;
        if four_cc == FourCC::SDOC {
            let data = arr.to_vec();
            self.show_sdoc(name, &data).await?;
        }
        Ok(())
    }

    async fn list_doc_entry(&self, entry: &DirEntry) -> Result<(), JsValue> {
        let path = self.fs.dir_entry_path(entry);
        let name = path.file_name().map(OsStr::to_string_lossy);
        let name = name.as_deref().unwrap_or("");

        let file = self.fs.dir_entry_to_file(entry).await?;
        let data = js_file_data(&file).await?.to_vec();

        let format = SignumFormat::detect(&data);
        info!("Loading {} ({:?})", name, format);
        let href = format!("#/{}", path.display());
        let card = self.card(name, format, &href)?;
        if let Some(format) = format {
            if let Err(e) = self.card_preview(&card, name, format, &data).await {
                console::error_3(
                    &JsValue::from_str("Failed to generate preview"),
                    &JsValue::from_str(name),
                    &e,
                );
            }
        }
        self.output.append_child(&card)?;
        Ok(())
    }

    async fn list_docs(&self) -> Result<(), JsValue> {
        let mut iter = self.fs.read_dir(Path::new("")).await?;
        while let Some(next) = iter.next().await {
            let entry = next?;
            if self.fs.dir_entry_is_file(&entry) {
                if let Err(e) = self.list_doc_entry(&entry).await {
                    let path = self.fs.dir_entry_path(&entry);
                    let path = path.to_string_lossy();
                    console::log_2(&JsValue::from_str(&path), &e);
                }
            }
        }
        info!("Done listing documents");
        Ok(())
    }

    async fn list_chset_entry(&self, entry: &DirEntry) -> Result<(), JsValue> {
        let path = self.fs.dir_entry_path(entry);
        let name = path.file_name().map(OsStr::to_string_lossy);
        let name = name.as_deref().unwrap_or("");

        let file = self.fs.dir_entry_to_file(entry).await?;
        let data = js_file_data(&file).await?.to_vec();

        let format = SignumFormat::detect(&data);
        info!("Loading {} ({:?})", name, format);
        let href = format!("#/CHSETS/{}", name);
        let card = self.card(name, format, &href)?;
        if let Some(format) = format {
            if let Err(e) = self.card_preview(&card, name, format, &data).await {
                console::error_3(
                    &JsValue::from_str("Failed to generate preview"),
                    &JsValue::from_str(name),
                    &e,
                );
            }
        }
        self.output.append_child(&card)?;
        Ok(())
    }

    async fn list_chsets(&self) -> Result<(), JsValue> {
        let path = OriginPrivateFS::chsets_path();
        let chset = self.fs.directory(path, true).await?;
        let mut iter = chset.read_dir().await?;
        while let Some(next) = iter.next().await {
            let entry = next?;
            if self.fs.dir_entry_is_file(&entry) {
                if let Err(e) = self.list_chset_entry(&entry).await {
                    let path = self.fs.dir_entry_path(&entry);
                    let path = path.to_string_lossy();
                    console::log_2(&JsValue::from_str(&path), &e);
                }
            }
        }
        info!("Done listing charsets");
        Ok(())
    }

    #[wasm_bindgen(js_name = onChange)]
    pub async fn on_change(&self) -> Result<(), JsValue> {
        self.reset()?;
        info!("Showing all input files");
        for file in js_input_files_iter(&self.input)? {
            let file = file?;
            let arr = js_file_data(&file).await?;
            self.stage(&file.name(), arr).await?;
        }
        Ok(())
    }

    fn card(
        &self,
        name: &str,
        format: Option<SignumFormat>,
        href: &str,
    ) -> Result<Element, JsValue> {
        let card = self.document.create_element("a")?;
        let class = card.class_list();
        class.add_2("list-group-item", "list-group-item-action")?;
        if let Some(f) = format {
            match f {
                SignumFormat::Signum1(sig1) => {
                    let m = sig1.magic();
                    let kind = decode_atari_str(m.as_slice());
                    class.add_1(kind.as_ref())?;
                }
                SignumFormat::Signum3(sig3) => {
                    let kind = match sig3 {
                        Signum3Format::Document => "s3doc",
                        Signum3Format::Font => "s3fnt",
                    };
                    class.add_1(kind.as_ref())?;
                }
            }
        }
        card.set_attribute("href", href)?;
        self.card_body(
            &card,
            name,
            format.as_ref().map(SignumFormat::file_format_name),
        )?;
        Ok(card)
    }

    async fn card_preview(
        &self,
        card: &Element,
        name: &str,
        format: SignumFormat,
        data: &[u8],
    ) -> Result<(), JsValue> {
        match format {
            SignumFormat::Signum1(Signum1Format::Document) => {
                let doc = self.parse_sdoc(data)?;
                self.sdoc_card(card, &doc).await?;
            }
            SignumFormat::Signum1(Signum1Format::Font(FontKind::Editor)) => {
                log::info!("{name}: Signum Editor Bitmap Font");
                let eset = self.parse_eset(data)?;
                log::info!("{name}: Parsed editor font");
                self.eset_card(card, &eset, name)?;
            }
            SignumFormat::Signum1(Signum1Format::Font(FontKind::Printer(_))) => {
                log::info!("{name}: {}", format.file_format_name());
                let pset = self.parse_pset(data)?;
                log::info!("{name}: Parsed printer font");
                self.pset_card(card, &pset, name)?;
            }
            SignumFormat::Signum1(Signum1Format::HardcopyImage) => {
                // TODO: preview
            }
            _ => {
                log::warn!("Unimplemented '{:?}'", format);
            }
        }
        Ok(())
    }

    async fn stage(&self, name: &str, arr: Uint8Array) -> Result<(), JsValue> {
        let data = arr.to_vec();
        info!("Parsing file '{}'", name);

        let format = SignumFormat::detect(&data);
        let href = format!("#/staged/{name}");
        let card = self.card(name, format, &href)?;
        if let Some(format) = format {
            if let Err(e) = self.card_preview(&card, name, format, &data).await {
                console::error_3(
                    &JsValue::from_str("Failed to generate preview"),
                    &JsValue::from_str(name),
                    &e,
                );
            }
        }
        self.output.append_child(&card)?;
        Ok(())
    }

    fn card_body(
        &self,
        card_body: &Element,
        name: &str,
        file_format_name: Option<&str>,
    ) -> Result<(), JsValue> {
        let card_title = self.document.create_element("h5")?;
        card_title.class_list().add_1("card-title")?;
        card_title.set_text_content(Some(name));
        card_body.append_child(&card_title)?;
        let card_subtitle = self.document.create_element("h6")?;
        card_subtitle
            .class_list()
            .add_3("card-subtitle", "mb-2", "text-body-secondary")?;
        card_subtitle.set_text_content(Some(file_format_name.unwrap_or("Unknown")));
        card_body.append_child(&card_subtitle)?;
        Ok(())
    }
}
