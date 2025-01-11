use js_sys::{Array, ArrayBuffer, Function, Reflect, Symbol, Uint8Array};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, BlobPropertyBag, FileList, FileSystemFileHandle, HtmlInputElement};

pub(crate) async fn fs_file_handle_get_file(
    file_handle: &FileSystemFileHandle,
) -> Result<web_sys::File, JsValue> {
    let file = JsFuture::from(file_handle.get_file())
        .await?
        .unchecked_into::<web_sys::File>();
    Ok(file)
}

pub(crate) async fn js_file_data(file: &web_sys::File) -> Result<Uint8Array, JsValue> {
    let buf = js_file_array_buffer(file).await?;
    Ok(Uint8Array::new(&buf))
}

pub(crate) async fn js_file_array_buffer(file: &web_sys::File) -> Result<ArrayBuffer, JsValue> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await?
        .unchecked_into::<ArrayBuffer>();
    Ok(array_buffer)
}

/// Return the [FileList] for the given input element
pub(crate) fn js_input_file_list(input: &HtmlInputElement) -> Result<FileList, JsValue> {
    let files = input
        .files()
        .ok_or_else(|| JsError::new("Not a file input"))?;
    Ok(files)
}

/// Return an iterator over the [web_sys::File]s in the given input element
pub(crate) fn js_input_files_iter(
    input: &HtmlInputElement,
) -> Result<impl Iterator<Item = Result<web_sys::File, JsValue>>, JsValue> {
    let files = js_input_file_list(input)?;
    let file_iter = js_sys::try_iter(&files)?.ok_or_else(|| JsError::new("Not a file iterator"))?;
    Ok(file_iter.map(|res| res.map(|file| file.unchecked_into::<web_sys::File>())))
}

pub(crate) fn try_iter_async(val: &JsValue) -> Result<Option<js_sys::AsyncIterator>, JsValue> {
    let async_iter_sym = Symbol::async_iterator();
    let iter_fn = Reflect::get(val, async_iter_sym.as_ref())?;

    let iter_fn: Function = match iter_fn.dyn_into() {
        Ok(iter_fn) => iter_fn,
        Err(_) => return Ok(None),
    };

    let it: js_sys::AsyncIterator = match iter_fn.call0(val)?.dyn_into() {
        Ok(it) => it,
        Err(_) => return Ok(None),
    };

    Ok(Some(it))
}

pub(crate) fn slice_to_blob(bytes: &[u8], mime_type: &str) -> Result<Blob, JsValue> {
    // SAFETY: the UInt8Array is used to initialize the blob but does not leave this function
    let parts = Array::from_iter([unsafe { Uint8Array::view(bytes) }]);
    Blob::new_with_u8_array_sequence_and_options(&parts, &{
        let bag = BlobPropertyBag::new();
        bag.set_type(mime_type);
        bag
    })
}

pub(crate) fn js_error_with_cause<E: std::error::Error>(e: E, message: &str) -> js_sys::Error {
    let err = js_sys::Error::new(message);
    err.set_cause(&JsError::from(e).into());
    err
}
