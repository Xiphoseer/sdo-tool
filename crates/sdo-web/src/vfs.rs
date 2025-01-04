use core::fmt;
use js_sys::{Function, Object, Reflect, Symbol};
use signum::chsets::cache::{AsyncIterator, VFS};
use std::{
    future::Future,
    path::{Path, PathBuf},
};
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemHandle, FileSystemHandleKind, StorageManager,
};

/// Browser Origin Private File System
pub struct OriginPrivateFS {
    storage: StorageManager,
    root: Option<FileSystemDirectoryHandle>,
}

impl OriginPrivateFS {
    pub fn root_dir(&self) -> Result<&FileSystemDirectoryHandle, JsValue> {
        let root = self
            .root
            .as_ref()
            .ok_or_else(|| JsError::new("OPFS not initialized"))?;
        Ok(root)
    }

    pub async fn chset_dir(&self) -> Result<FileSystemDirectoryHandle, JsValue> {
        let root = self.root_dir()?;
        let dir = resolve_dir(root, Path::new("CHSETS"), true).await?;
        Ok(dir)
    }
}

#[derive(Debug)]
pub struct Error(pub JsValue);

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

pub struct DirEntry(pub FileSystemHandle, pub PathBuf);

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
) -> Result<FileSystemDirectoryHandle, JsValue> {
    let mut curr = h.clone();
    let opt = FileSystemGetDirectoryOptions::new();
    if create {
        opt.set_create(true); // bug?
    }
    for p in path {
        if let Some(s) = p.to_str() {
            curr = JsFuture::from(curr.get_directory_handle_with_options(s, &opt))
                .await?
                .unchecked_into::<FileSystemDirectoryHandle>();
        } else {
            return Err(JsError::new("Not Found").into());
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
            let result = JsFuture::from(dir.get_file_handle(s)).await?;
            let file = result.unchecked_into::<FileSystemFileHandle>();
            return Ok(file);
        }
    }
    Err(JsError::new("Not Found").into())
}

impl VFS for OriginPrivateFS {
    type Error = Error;

    type DirIter = DirIter;

    type DirEntry = DirEntry;

    fn root(&self) -> impl Future<Output = PathBuf> + 'static {
        std::future::ready(PathBuf::from(self.root.as_deref().unwrap().name()))
    }

    async fn is_file(&self, path: &Path) -> bool {
        let root = self.root.as_ref().expect("Uninitialized OPFS");
        resolve_file(root, path)
            .await
            .map(|f| f.kind() == FileSystemHandleKind::File)
            .unwrap_or(false)
    }

    async fn is_dir(&self, path: &Path) -> bool {
        let root = self.root.as_ref().expect("Uninitialized OPFS");
        resolve_dir(root, path, false)
            .await
            .map(|f| f.kind() == FileSystemHandleKind::Directory)
            .unwrap_or(false)
    }

    async fn read_dir(&self, path: &Path) -> Result<Self::DirIter, Self::Error> {
        let root = self.root.as_ref().expect("Uninitialized OPFS");
        let dir = resolve_dir(root, path, false).await?;
        let iter =
            try_iter_async(dir.as_ref())?.ok_or_else(|| JsError::new("Not async iterable"))?;
        Ok(DirIter(iter, path.to_owned()))
    }

    fn dir_entry_path(&self, entry: &Self::DirEntry) -> PathBuf {
        entry.1.clone()
    }

    async fn read(&self, path: &Path) -> Result<Vec<u8>, Self::Error> {
        let root = self.root_dir()?;
        let file_handle = resolve_file(root, path).await?;
        let file = JsFuture::from(file_handle.get_file())
            .await?
            .unchecked_into::<web_sys::File>();
        let array_buffer = JsFuture::from(file.array_buffer())
            .await?
            .unchecked_into::<js_sys::ArrayBuffer>();
        let uint8_buf = js_sys::Uint8Array::new(&array_buffer);
        Ok(uint8_buf.to_vec())
    }
}

pub fn try_iter_async(val: &JsValue) -> Result<Option<js_sys::AsyncIterator>, JsValue> {
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

impl OriginPrivateFS {
    pub fn new() -> Self {
        let window = window().unwrap();
        let _navigator = window.navigator();
        let storage = _navigator.storage();

        Self {
            storage,
            root: None,
        }
    }

    pub async fn init(&mut self) -> Result<(), JsValue> {
        let dir_handle = FileSystemDirectoryHandle::unchecked_from_js(
            JsFuture::from(self.storage.get_directory()).await?,
        );
        self.root = Some(dir_handle);
        Ok(())
    }
}
