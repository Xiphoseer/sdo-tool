use std::{
    borrow::Cow,
    fs,
    future::Future,
    io,
    path::{Path, PathBuf},
};

/// Async Iterator trait
pub trait AsyncIterator {
    /// Single item
    type Item;

    /// Next method
    fn next(&mut self) -> impl Future<Output = Option<Self::Item>>;
}

/// # Virtual File System
///
/// This virtual file system is used for loading fonts into
/// the [crate::chsets::cache::ChsetCache].
pub trait VFS {
    /// Error type
    type Error: std::fmt::Display;
    /// Directory iterator
    type DirIter: AsyncIterator<Item = Result<Self::DirEntry, Self::Error>>;
    /// Directory entry
    type DirEntry;
    /// Open file
    type File;

    /// Return the root path of the VFS
    fn root(&self) -> impl Future<Output = PathBuf> + 'static;

    /// Check whether the path is a file
    fn is_file(&self, path: &Path) -> impl Future<Output = bool>;

    /// Check whether the path is a directory
    fn is_dir(&self, path: &Path) -> impl Future<Output = bool>;

    /// Read a directory
    fn read_dir(&self, path: &Path) -> impl Future<Output = Result<Self::DirIter, Self::Error>>;

    /// Open a file
    fn open(&self, path: &Path) -> impl Future<Output = Result<Self::File, Self::Error>>;

    /// Check whether the directory entry is a file
    fn dir_entry_is_file(&self, entry: &Self::DirEntry) -> bool;

    /// Check whether the directory entry is a directory
    fn dir_entry_is_dir(&self, entry: &Self::DirEntry) -> bool;

    /// Get the path of a directory entry
    fn dir_entry_path<'a>(&self, entry: &'a Self::DirEntry) -> Cow<'a, Path>;

    /// Open a file
    fn dir_entry_to_file(
        &self,
        dir_entry: &Self::DirEntry,
    ) -> impl Future<Output = Result<Self::File, Self::Error>>;

    /// Read a file
    fn read(&self, path: &Path) -> impl Future<Output = Result<Vec<u8>, Self::Error>>;

    /// Check whether the error is a 'NotFound'
    fn is_file_not_found(_e: &Self::Error) -> bool {
        false
    }
}

/// VFS for the Local File System ([`std::fs`])
pub struct LocalFS {
    chsets_folder: PathBuf,
}

impl LocalFS {
    /// Create a new instance rooted at `chsets_folder`
    pub fn new(chsets_folder: PathBuf) -> Self {
        Self { chsets_folder }
    }
}

impl VFS for LocalFS {
    fn root(&self) -> impl Future<Output = PathBuf> + 'static {
        std::future::ready(self.chsets_folder.to_owned())
    }

    fn is_file(&self, path: &Path) -> impl Future<Output = bool> {
        std::future::ready(path.is_file())
    }

    fn dir_entry_is_file(&self, entry: &Self::DirEntry) -> bool {
        entry.path().is_file()
    }

    fn is_dir(&self, path: &Path) -> impl Future<Output = bool> {
        std::future::ready(path.is_dir())
    }

    fn dir_entry_is_dir(&self, entry: &Self::DirEntry) -> bool {
        entry.path().is_dir()
    }

    async fn read_dir(&self, path: &Path) -> Result<Self::DirIter, Self::Error> {
        std::fs::read_dir(path)
    }

    type Error = io::Error;

    type DirIter = fs::ReadDir;

    type DirEntry = fs::DirEntry;

    type File = fs::File;

    fn open(&self, path: &Path) -> impl Future<Output = Result<Self::File, Self::Error>> {
        std::future::ready(std::fs::File::open(path))
    }

    fn read(&self, path: &Path) -> impl Future<Output = Result<Vec<u8>, Self::Error>> {
        std::future::ready(std::fs::read(path)) // FIXME: async
    }

    async fn dir_entry_to_file(
        &self,
        dir_entry: &Self::DirEntry,
    ) -> Result<Self::File, Self::Error> {
        let path = dir_entry.path();
        let file = self.open(&path).await?;
        Ok(file)
    }

    fn dir_entry_path<'a>(&self, entry: &'a Self::DirEntry) -> Cow<'a, Path> {
        Cow::Owned(entry.path())
    }

    fn is_file_not_found(e: &Self::Error) -> bool {
        e.kind() == io::ErrorKind::NotFound
    }
}

impl AsyncIterator for fs::ReadDir {
    type Item = Result<fs::DirEntry, io::Error>;

    async fn next(&mut self) -> Option<Self::Item> {
        <Self as Iterator>::next(self)
    }
}
