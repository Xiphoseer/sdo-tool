use js_sys::{Array, JsString};
use signum::{
    chsets::{cache::ChsetCache, encoding::decode_atari_str, printer::PrinterKind, FontKind},
    docs::{
        container::parse_sdoc0001_container,
        hcim::{parse_image, Hcim},
        header, DocumentInfo, SDoc,
    },
};
use std::fmt::Write;
use wasm_bindgen::{JsError, JsValue};
use web_sys::Element;

use crate::blob_image_el;

use super::{log, page_to_blob, ActiveDocument, Handle};

impl Handle {
    pub(super) fn parse_sdoc<'a>(&self, data: &'a [u8]) -> Result<SDoc<'a>, JsValue> {
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

    pub(super) async fn show_sdoc(&self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        let heading = self.document.create_element("h2")?;
        heading.set_text_content(Some(name));
        self.output.append_child(&heading)?;

        let sdoc = self.parse_sdoc(data)?;
        let mut fc = ChsetCache::new();
        let dfci = fc.load(&self.fs, &sdoc.cset).await;
        for cset in fc.chsets_mut() {
            if cset.map().is_none() {
                if let Some(mapping) = sdo_fonts::mappings::lookup(cset.name()) {
                    log::info!("Using built-in unicode mapping for {}", cset.name());
                    cset.set_mapping(Some(mapping.clone()));
                }
            }
        }
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

        *self.active.borrow_mut() = Some(ActiveDocument {
            sdoc: sdoc.into_owned(),
            di: DocumentInfo::new(dfci, images),
            pd,
            fc,
            name: name.to_owned(),
        });
        Ok(())
    }

    pub(super) async fn sdoc_card(
        &self,
        list_item: &Element,
        doc: &SDoc<'_>,
    ) -> Result<(), JsValue> {
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
        log::info!("Loading charsets");
        let mut fc = ChsetCache::new();
        for chset in doc.cset.names.iter().filter(|c| !c.is_empty()) {
            log::info!("Loading {}", chset);
            let (cls, tooltip) = {
                let cset_index = fc.load_cset(&self.fs, chset).await;
                super::console::log_2(&"Font Index".into(), &cset_index.into());
                let cset = fc.cset(cset_index).unwrap();
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
        log::info!("Done Loading charsets");
        list_item.append_child(&chset_list)?;

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
        crate::log_array("cset", ar);
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
            crate::log_val("hcim", &tebu);
        }
        self.output.append_child(&el_hcim)?;
        Ok(())
    }
}
