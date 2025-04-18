use core::fmt;
use js_sys::Object;
use signum::util::{AsyncIterator, VFS};
use std::{
    borrow::Cow,
    cell::RefCell,
    future::Future,
    path::{Path, PathBuf},
};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemHandle, FileSystemHandleKind, StorageManager,
};

use crate::glue::{
    fs::{
        directory_handle_get_directory_handle_with_options, directory_handle_get_file_handle,
        file_handle_get_file,
    },
    js_error_with_cause, js_file_data, js_storage_manager_get_directory, try_iter_async,
};

/// Browser Origin Private File System
pub struct OriginPrivateFS {
    storage: StorageManager,
    root: RefCell<Option<FileSystemDirectoryHandle>>,
}

impl OriginPrivateFS {
    pub fn root_dir(&self) -> Result<FileSystemDirectoryHandle, JsValue> {
        let root_ref = self
            .root
            .try_borrow()
            .map_err(|e| js_error_with_cause(e, "OPFS: concurrent modification"))?;
        let root = root_ref
            .as_ref()
            .ok_or_else(|| JsError::new("OPFS not initialized"))?;
        Ok(root.clone())
    }

    pub fn chsets_path() -> &'static Path {
        Path::new("CHSETS")
    }

    pub async fn chset_dir(&self) -> Result<FileSystemDirectoryHandle, JsValue> {
        let root = self.root_dir()?;
        let dir = resolve_dir(&root, Self::chsets_path(), true).await?;
        Ok(dir)
    }
}

#[derive(Debug)]
pub struct Error(pub JsValue);

impl From<Error> for JsValue {
    fn from(value: Error) -> Self {
        value.0
    }
}

impl From<js_sys::Error> for Error {
    fn from(value: js_sys::Error) -> Self {
        Self(value.into())
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        Self(value)
    }
}

impl From<JsError> for Error {
    fn from(value: JsError) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_ref::<Object>().unwrap().to_string().fmt(f)
    }
}

impl AsyncIterator for DirIter {
    type Item = Result<DirEntry, Error>;

    async fn next(&mut self) -> Option<Self::Item> {
        let promise = self.0.next().ok()?;
        let next = JsFuture::from(promise).await.ok()?;
        let next = next.unchecked_ref::<js_sys::IteratorNext>();
        if next.done() {
            None
        } else {
            let val = next.value();
            let pair = val.unchecked_ref::<js_sys::Array>();
            let key = pair.at(0).as_string().unwrap();
            let value = pair.at(1).unchecked_into::<_>();
            Some(Ok(DirEntry(value, self.1.join(key))))
        }
    }
}

pub struct Directory {
    inner: FileSystemDirectoryHandle,
    path: PathBuf,
}

impl Directory {
    pub async fn read_dir(&self) -> Result<DirIter, Error> {
        let iter =
            try_iter_async(&self.inner)?.ok_or_else(|| JsError::new("Not async iterable"))?;
        Ok(DirIter(iter, self.path.clone()))
    }
}

pub struct DirEntry(FileSystemHandle, PathBuf);

impl fmt::Display for DirEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.name().fmt(f)
    }
}

impl fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DirEntry")
            .field("name", &self.0.name())
            .field("kind", &self.0.kind())
            .field("path", &self.1)
            .finish()
    }
}

pub struct DirIter(pub js_sys::AsyncIterator, PathBuf);

async fn resolve_dir(
    h: &FileSystemDirectoryHandle,
    path: &Path,
    create: bool,
) -> Result<FileSystemDirectoryHandle, js_sys::Error> {
    let mut curr = h.clone();
    let opt = FileSystemGetDirectoryOptions::new();
    if create {
        opt.set_create(true); // bug?
    }
    for p in path {
        if let Some(s) = p.to_str() {
            curr = directory_handle_get_directory_handle_with_options(&curr, s, &opt)
                .await
                .map_err(|e| {
                    let err_message = format!("Directory not found: {}", path.display());
                    let err = js_sys::Error::new(&err_message);
                    err.set_name("SDOWebError");
                    err.set_cause(&e);
                    err
                })?;
        } else {
            let err_message = format!(
                "Failed to resolve directory: Malformed path {}",
                path.display()
            );
            let err = js_sys::Error::new(&err_message);
            err.set_name("SDOWebError");
            return Err(err);
        }
    }
    Ok(curr)
}

async fn resolve_file(
    root: &FileSystemDirectoryHandle,
    path: &Path,
) -> Result<FileSystemFileHandle, JsValue> {
    let dir = if let Some(parent) = path.parent() {
        resolve_dir(root, parent, false).await?
    } else {
        root.clone()
    };
    if let Some(name) = path.file_name() {
        if let Some(s) = name.to_str() {
            return directory_handle_get_file_handle(&dir, s).await;
        }
    }
    Err(JsError::new("Not Found").into())
}

impl VFS for OriginPrivateFS {
    type Error = Error;

    type DirIter = DirIter;

    type DirEntry = DirEntry;

    type File = web_sys::File;

    fn root(&self) -> impl Future<Output = PathBuf> + 'static {
        std::future::ready(PathBuf::from(self.root_dir().as_deref().unwrap().name()))
    }

    async fn is_file(&self, path: &Path) -> bool {
        let root = self.root_dir().unwrap();
        resolve_file(&root, path)
            .await
            .map(|f| f.kind() == FileSystemHandleKind::File)
            .unwrap_or(false)
    }

    fn dir_entry_is_file(&self, entry: &Self::DirEntry) -> bool {
        entry.0.kind() == FileSystemHandleKind::File
    }

    async fn is_dir(&self, path: &Path) -> bool {
        let root = self.root_dir().unwrap();
        resolve_dir(&root, path, false)
            .await
            .map(|f| f.kind() == FileSystemHandleKind::Directory)
            .unwrap_or(false)
    }

    fn dir_entry_is_dir(&self, entry: &Self::DirEntry) -> bool {
        entry.0.kind() == FileSystemHandleKind::Directory
    }

    async fn read_dir(&self, path: &Path) -> Result<Self::DirIter, Self::Error> {
        let dir = self.directory(path, false).await?;
        dir.read_dir().await
    }

    async fn open(&self, path: &Path) -> Result<Self::File, Self::Error> {
        let root = self.root_dir()?;
        let file_handle = resolve_file(&root, path).await?;
        let file = file_handle_get_file(&file_handle).await?;
        Ok(file)
    }

    async fn read(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        let file = self.open(path).await?;
        let uint8_buf = js_file_data(&file).await?;
        Ok(uint8_buf.to_vec())
    }

    async fn dir_entry_to_file(
        &self,
        dir_entry: &Self::DirEntry,
    ) -> Result<Self::File, Self::Error> {
        let file_handle = dir_entry
            .0
            .dyn_ref::<FileSystemFileHandle>()
            .ok_or_else(|| JsError::new("not a file"))?;
        let file = file_handle_get_file(file_handle).await?;
        Ok(file)
    }

    fn dir_entry_path<'a>(&self, entry: &'a Self::DirEntry) -> Cow<'a, Path> {
        Cow::Borrowed(&entry.1)
    }
}

impl OriginPrivateFS {
    pub fn new() -> Self {
        let window = window().unwrap();
        let _navigator = window.navigator();
        let storage = _navigator.storage();

        Self {
            storage,
            root: RefCell::new(None),
        }
    }

    pub(crate) async fn directory(&self, path: &Path, create: bool) -> Result<Directory, Error> {
        let root = self.root_dir().unwrap();
        let inner = resolve_dir(&root, path, create).await?;
        Ok(Directory {
            inner,
            path: path.to_owned(),
        })
    }

    pub async fn init(&self) -> Result<(), JsValue> {
        let dir_handle = js_storage_manager_get_directory(&self.storage).await?;
        *self.root.borrow_mut() = Some(dir_handle);
        Ok(())
    }
}
