use std::{future::Future, marker::PhantomData, pin::Pin, task::Poll};

use js_sys::{Array, ArrayBuffer, Function, Reflect, Symbol, Uint8Array};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, BlobPropertyBag, FileList, FileSystemDirectoryHandle, HtmlInputElement, StorageManager,
};

// type JsResult<T> = std::result::Result<T, JsValue>;

pub(crate) mod fs;

pub struct JsTypedFuture<T> {
    inner: JsFuture,
    _type: PhantomData<fn() -> T>,
}

impl<T> JsTypedFuture<T> {
    fn project(self: Pin<&mut Self>) -> Pin<&mut JsFuture> {
        // SAFETY: inner does never change after construction
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }

    fn new(promise: js_sys::Promise) -> Self {
        Self {
            inner: JsFuture::from(promise),
            _type: PhantomData,
        }
    }
}

impl<T: JsCast> Future for JsTypedFuture<T> {
    type Output = Result<T, JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        match JsTypedFuture::project(self).poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(Ok(v.unchecked_into::<T>())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub(crate) async fn js_file_data(file: &web_sys::File) -> Result<Uint8Array, JsValue> {
    let buf = js_file_array_buffer(file).await?;
    Ok(Uint8Array::new(&buf))
}

pub(crate) async fn js_file_array_buffer(file: &web_sys::File) -> Result<ArrayBuffer, JsValue> {
    let array_buffer = JsFuture::from(file.array_buffer()).await?;
    Ok(array_buffer.unchecked_into::<ArrayBuffer>())
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

pub(crate) fn js_wrap_err(e: impl Into<JsValue>, message: &str) -> js_sys::Error {
    let err = js_sys::Error::new(message);
    err.set_cause(&e.into());
    err
}

pub(crate) async fn js_storage_manager_get_directory(
    storage: &StorageManager,
) -> Result<FileSystemDirectoryHandle, JsValue> {
    let val = JsFuture::from(storage.get_directory()).await?;
    Ok(FileSystemDirectoryHandle::unchecked_from_js(val))
}
