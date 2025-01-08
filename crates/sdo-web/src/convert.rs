use std::io::Cursor;

use image::ImageOutputFormat;
use js_sys::{Array, Uint8Array};
use signum::raster;
use wasm_bindgen::JsValue;
use web_sys::{Blob, BlobPropertyBag};

pub(super) fn page_as_blob(page: &raster::Page) -> Result<Blob, JsValue> {
    let mut buffer = Cursor::new(Vec::<u8>::new());
    page.to_alpha_image()
        .write_to(&mut buffer, ImageOutputFormat::Png)
        .unwrap();
    Blob::new_with_u8_array_sequence_and_options(
        &Array::from_iter([Uint8Array::from(buffer.get_ref().as_slice())]),
        &{
            let bag = BlobPropertyBag::new();
            bag.set_type("image/png");
            bag
        },
    )
}
