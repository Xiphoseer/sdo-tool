use js_sys::Array;
use js_sys::ArrayBuffer;
use js_sys::Uint8Array;
use log::Level;
use signum::chsets::encoding::decode_atari_str;
use signum::docs::container::parse_sdoc0001_container;
use signum::docs::hcim::parse_image;
use signum::docs::SDoc;
use wasm_bindgen::prelude::*;
use web_sys::Document;
use web_sys::HtmlElement;
use web_sys::{Event, HtmlInputElement};

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
pub fn setup(name: &str) -> Result<Module, JsValue> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    let input_element = document
        .get_element_by_id(name)
        .ok_or(JsValue::from_str("Could not find input element"))?
        .dyn_into::<HtmlInputElement>()?;

    let on_data = Closure::new(move |_v: JsValue| -> () {
        let _arr = _v.dyn_into::<ArrayBuffer>().unwrap();
        let _by = Uint8Array::new(&_arr);
        let _data = _by.to_vec();

        fn do_stuff(document: &Document, _data: &[u8]) -> Result<(), JsValue> {
            let _body = document.body().expect("document should have a body");
            let el_header = document
                .get_element_by_id("0001")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            let el_cset = document
                .get_element_by_id("cset")
                .unwrap()
                .dyn_into::<HtmlElement>()
                .unwrap();
            log("Has data");

            match &_data[..4] {
                b"sdoc" => match parse_sdoc0001_container(&_data) {
                    Ok((_rest, container)) => {
                        log("Has Container");

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

                        // 0001
                        log::info!("Created: {}", &doc.header.ctime);
                        log::info!("Modified: {}", &doc.header.mtime);
                        let text = format!(
                            "Created: {}<br>Modified: {}",
                            doc.header.ctime, doc.header.mtime
                        );
                        el_header.set_inner_html(&text);

                        // cset
                        let ar = Array::new();
                        let mut html = "<ol>".to_string();
                        for chr in doc.charsets {
                            let name = decode_atari_str(chr.as_ref());
                            html.push_str("<li>");
                            ar.push(js_sys::JsString::from(name.as_ref()).as_ref());
                            html.push_str(&name);
                            html.push_str("</li>");
                        }
                        html.push_str("</ol>");
                        log_array("cset", ar);
                        el_cset.set_inner_html(&html);

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
                            let el_hcim = document
                                .get_element_by_id("hcim")
                                .unwrap()
                                .dyn_into::<HtmlElement>()
                                .unwrap();
                            let mut html = "<h3>Embedded Images</h3>".to_string();
                            for (i, _im) in hcim.images.iter().enumerate() {
                                match parse_image(_im.0) {
                                    Ok((_rest, image)) => {
                                        html.push_str(&format!("<p>{}</p>", image.key));
                                    }
                                    Err(e) => {
                                        log::error!("Failed to parse image {}: {}", i, e);
                                    }
                                }
                            }
                            if let Ok(tebu) = serde_wasm_bindgen::to_value(hcim) {
                                log_val("hcim", &tebu);
                            }
                            el_hcim.set_inner_html(&html);
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
                },
                k => {
                    log_val(
                        "Unknown File Format",
                        &std::str::from_utf8(k).unwrap_or("").into(),
                    );
                }
            }
            Ok(())
        }
        if let Err(e) = do_stuff(&document, &_data) {
            error_val(e);
        }
    });
    let callback = Closure::<(dyn FnMut(Event) + 'static)>::new(move |_ev: Event| {
        let el = _ev
            .target()
            .expect("event to have a target")
            .dyn_into::<HtmlInputElement>()
            .expect("target to stay <input>");
        if let Some(files) = el.files() {
            for i in 0..files.length() {
                let file = files.item(i).expect("file to exist");
                let _ = file.array_buffer().then(&on_data);

                log_val("file", file.as_ref());
            }
        }
    });
    input_element.add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())?;

    Ok(Module { callback })
}
