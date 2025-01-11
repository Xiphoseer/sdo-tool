use std::io::Cursor;

use image::ImageOutputFormat;
use signum::raster;
use wasm_bindgen::JsValue;
use web_sys::Blob;

use crate::glue::{js_error_with_cause, slice_to_blob};

/// Convert a raster page to a PNG image blob
pub(super) fn page_to_blob(page: &raster::Page) -> Result<Blob, JsValue> {
    let mut buffer = Cursor::new(Vec::<u8>::new());
    page.to_alpha_image()
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .map_err(|e| js_error_with_cause(e, "Failed to encode image as PNG"))?;
    let bytes: &[u8] = buffer.get_ref();
    slice_to_blob(bytes, "image/png")
}
