use wasm_bindgen::JsValue;
use web_sys::{Blob, HtmlImageElement, Url};

pub(super) fn blob_image_el(blob: &Blob) -> Result<HtmlImageElement, JsValue> {
    let url = Url::create_object_url_with_blob(blob)?;
    let el_image = HtmlImageElement::new()?;
    el_image.set_src(&url);
    Ok(el_image)
}
