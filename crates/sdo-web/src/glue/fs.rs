use js_sys::{Object, Uint8Array};
use wasm_bindgen::JsValue;
use web_sys::{
    FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemGetFileOptions, FileSystemWritableFileStream,
};

use super::JsTypedFuture;

/// Return a JS `File` for a file handle
pub(crate) fn file_handle_get_file(
    file_handle: &FileSystemFileHandle,
) -> JsTypedFuture<web_sys::File> {
    JsTypedFuture::new(file_handle.get_file())
}

/// Return a writable stream for a file
pub(crate) fn file_handle_create_writable(
    file_handle: &FileSystemFileHandle,
) -> JsTypedFuture<FileSystemWritableFileStream> {
    JsTypedFuture::new(file_handle.create_writable())
}

#[allow(dead_code)]
pub(crate) fn writable_file_stream_write_with_buffer_source(
    stream: &FileSystemWritableFileStream,
    data: &Object,
) -> Result<JsTypedFuture<JsValue>, JsValue> {
    let promise = stream.write_with_buffer_source(data)?;
    Ok(JsTypedFuture::new(promise))
}

pub(crate) fn writable_file_stream_write_with_js_u8_array(
    stream: &FileSystemWritableFileStream,
    data: &Uint8Array,
) -> Result<JsTypedFuture<JsValue>, JsValue> {
    let promise = stream.write_with_js_u8_array(data)?;
    Ok(JsTypedFuture::new(promise))
}

pub(crate) fn writable_file_stream_close(
    stream: &FileSystemWritableFileStream,
) -> JsTypedFuture<JsValue> {
    JsTypedFuture::new(stream.close())
}

/// Return a handle for the named file
pub(crate) fn directory_handle_get_file_handle(
    dir: &FileSystemDirectoryHandle,
    s: &str,
) -> JsTypedFuture<FileSystemFileHandle> {
    JsTypedFuture::new(dir.get_file_handle(s))
}

/// Return a handle for the named file with options
pub(crate) fn directory_handle_get_file_handle_with_options(
    dir: &FileSystemDirectoryHandle,
    s: &str,
    opts: &FileSystemGetFileOptions,
) -> JsTypedFuture<FileSystemFileHandle> {
    JsTypedFuture::new(dir.get_file_handle_with_options(s, opts))
}

/// Return a handle for the named directory
pub(crate) fn directory_handle_get_directory_handle_with_options(
    dir: &FileSystemDirectoryHandle,
    s: &str,
    opt: &FileSystemGetDirectoryOptions,
) -> JsTypedFuture<FileSystemDirectoryHandle> {
    JsTypedFuture::new(dir.get_directory_handle_with_options(s, opt))
}
