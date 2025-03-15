//! # Implementation of a charset cache

use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use bstr::BStr;
use log::{info, warn};

use crate::{
    chsets::{
        editor::{ESet, OwnedESet},
        encoding::{p_mapping_file, Mapping},
        printer::{OwnedPSet, PSet, PrinterKind},
        LoadError,
    },
    docs::cset,
    util::{AsyncIterator, FileFormatKind, VFS},
};

use super::{encoding::decode_atari_str, FontKind};

#[allow(dead_code)]
type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

#[allow(clippy::manual_flatten)]
async fn find_font_file<FS: VFS>(
    fs: &FS,
    cset_folder: &Path,
    name: &str,
    extension: &str,
) -> Option<PathBuf> {
    info!("Searching font {} in {:?}", name, cset_folder.display());
    let cset_file_base = cset_folder.join(name);
    let cset_file = cset_file_base.with_extension(extension);

    if fs.is_file(&cset_file).await {
        return Some(cset_file);
    }

    let mut dir_iter = match fs.read_dir(cset_folder).await {
        Ok(i) => i,
        Err(e) => {
            warn!("Could not find CHSET folder: {}", e);
            return None;
        }
    };

    while let Some(entry) = dir_iter.next().await {
        if let Ok(de) = entry {
            let subfolder = fs.dir_entry_path(&de);
            if fs.is_dir(&subfolder).await {
                if subfolder.file_name().is_some_and(|p| p == ".git") {
                    continue;
                }
                // Note: need to box the future here because this is async recursion.
                let fut = Box::pin(find_font_file(fs, &subfolder, name, extension));
                if let Some(path) = fut.await {
                    return Some(path);
                }
            }
        }
    }
    None
}

async fn load_printer_font<FS: VFS>(
    fs: &FS,
    cset_file: &Path,
    pk: PrinterKind,
) -> Option<OwnedPSet> {
    let extension = pk.extension();
    let printer_cset_file = cset_file.with_extension(extension);
    let buffer = fs
        .read(&printer_cset_file)
        .await
        .inspect_err(|e| {
            if !FS::is_file_not_found(e) {
                warn!("Failed to load {:?}: {}", printer_cset_file, e);
            }
        })
        .ok()?;
    match OwnedPSet::load_from_buffer(buffer, pk) {
        Ok(pset) => {
            info!("Loaded printer font file '{}'", printer_cset_file.display());
            Some(pset)
        }
        Err(LoadError::Io(_)) => None,
        Err(LoadError::Parse(f)) => {
            warn!("Failed to parse {} printer font: {}", extension, f);
            None
        }
    }
}

fn load_mapping_file(editor_cset_file: &Path) -> Option<Mapping> {
    let cset_mapping_file = editor_cset_file.with_extension("TXT");
    if cset_mapping_file.is_file() {
        let input = std::fs::read_to_string(&cset_mapping_file).unwrap();
        match p_mapping_file(&input) {
            Ok(mapping) => {
                info!("Loaded cset mapping file '{}'", cset_mapping_file.display());
                Some(mapping)
            }
            Err(err) => {
                warn!(
                    "Failed to parse mapping file '{}': {}",
                    cset_mapping_file.display(),
                    err
                );
                None
            }
        }
    } else {
        warn!(
            "Missing mapping for font '{}'",
            editor_cset_file.file_stem().unwrap().to_string_lossy()
        );
        None
    }
}

async fn load_editor_font<FS: VFS>(fs: &FS, editor_cset_file: &Path) -> Option<OwnedESet> {
    let buffer = fs
        .read(editor_cset_file)
        .await
        .map_err(|e| warn!("Failed to load {:?}: {}", editor_cset_file, e))
        .ok()?;
    match OwnedESet::load_from_buf(buffer) {
        Ok(eset) => {
            info!("Loaded editor font file '{}'", editor_cset_file.display());
            Some(eset)
        }
        Err(LoadError::Parse(e)) => {
            info!(
                "Failed to parse editor font file {}
                Are you sure this is a valid Signum! editor font?
                Error: {}",
                editor_cset_file.display(),
                e
            );
            None
        }
        Err(LoadError::Io(e)) => {
            warn!(
                "Failed to load editor font file {}
                Error: {}",
                editor_cset_file.display(),
                e
            );
            None
        }
    }
}

/// Holds variants of a charset
///
/// This structure holds different representations (e.g. Bitmaps for different printer kinds) of the same character set.
pub struct CSet {
    name: String,
    l30: Option<OwnedPSet>,
    p24: Option<OwnedPSet>,
    p09: Option<OwnedPSet>,
    e24: Option<OwnedESet>,
    map: Option<Mapping>,
}

#[rustfmt::skip]
impl<'a> CSet {
    /// The the name of the character set
    pub fn name(&self) -> &str { &self.name }
    /// Get the unicode mapping
    pub fn map(&self) -> Option<&Mapping> { self.map.as_ref() }
    /// Get the laser printer bitmaps
    pub fn l30(&'a self) -> Option<&'a PSet<'a>> { self.l30.as_ref().map(OwnedPSet::borrowed) }
    /// Get the 24-needle printer bitmaps
    pub fn p24(&'a self) -> Option<&'a PSet<'a>> { self.p24.as_ref().map(OwnedPSet::borrowed) }
    /// Get the 9-needle printer bitmaps
    pub fn p09(&'a self) -> Option<&'a PSet<'a>> { self.p09.as_ref().map(OwnedPSet::borrowed) }
    /// Get the editor bitmaps
    pub fn e24(&self) -> Option<&ESet<'static>> { self.e24.as_deref() }
    /// Get the bitmaps for the specified printer kind
    pub fn printer(&'a self, pk: PrinterKind) -> Option<&'a PSet<'a>> {
        match pk {
            PrinterKind::Needle9 => self.p09.as_ref().map(OwnedPSet::borrowed),
            PrinterKind::Needle24 => self.p24.as_ref().map(OwnedPSet::borrowed),
            PrinterKind::Laser30 => self.l30.as_ref().map(OwnedPSet::borrowed),
        }
    }

    /// Override the stored character mapping
    pub fn set_mapping(&mut self, mapping: Option<Mapping>) {
        self.map = mapping;
    }
}

/// A simple cache for charsets
pub struct ChsetCache {
    chsets: Vec<CSet>,
    names: HashMap<String, usize>,
}

impl ChsetCache {
    /// Create a new instance
    pub fn new() -> Self {
        ChsetCache {
            chsets: Vec::with_capacity(8),
            names: HashMap::new(),
        }
    }

    /// Get the slice of all charsets
    pub fn chsets(&self) -> &[CSet] {
        &self.chsets
    }

    /// Get the slice of all charsets
    pub fn chsets_mut(&mut self) -> &mut [CSet] {
        &mut self.chsets
    }

    /// Get a specific printer charset
    pub fn pset(&self, pk: PrinterKind, index: usize) -> Option<&PSet<'_>> {
        self.cset(index).and_then(|cset| cset.printer(pk))
    }

    /// Get a specific editor charset
    pub fn eset(&self, index: usize) -> Option<&ESet<'static>> {
        self.cset(index).and_then(CSet::e24)
    }

    /// Get a specific charset
    pub fn cset(&self, index: usize) -> Option<&CSet> {
        self.chsets.get(index)
    }

    /// Load a CSET section into the font cache, returning a document specific info struct.
    pub async fn load<FS: VFS>(&mut self, fs: &FS, cset: &cset::CSet<'_>) -> DocumentFontCacheInfo {
        let mut all_eset = true;
        let mut all_p24 = true;
        let mut all_l30 = true;
        let mut all_p09 = true;

        let mut chsets = [FontCacheInfo::EMPTY; 8];

        for (index, name) in cset.names.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            chsets[index].name = Some(name.to_string());
            let cset_cache_index = self.load_cset(fs, name).await;
            let cset = self
                .cset(cset_cache_index)
                .expect("invalid index returned by load_cset");
            chsets[index].index = Some(cset_cache_index);
            all_eset &= cset.e24().is_some();
            all_p24 &= cset.p24().is_some();
            all_l30 &= cset.l30().is_some();
            all_p09 &= cset.p09().is_some();
        }
        DocumentFontCacheInfo {
            all_eset,
            all_l30,
            all_p24,
            all_p09,
            chsets,
        }
    }

    /// Load a character set
    pub async fn load_cset<FS: VFS>(&mut self, fs: &FS, name: &BStr) -> usize {
        let name = decode_atari_str(name.as_ref()).into_owned();
        if let Some(index) = self.names.get(&name) {
            return *index;
        }

        let cset = match find_font_file(fs, &fs.root().await, &name, "E24").await {
            Some(editor_cset_file) => {
                // Load all font files
                CSet {
                    name: name.clone(),
                    e24: load_editor_font(fs, &editor_cset_file).await,
                    p09: load_printer_font(fs, &editor_cset_file, PrinterKind::Needle9).await,
                    p24: load_printer_font(fs, &editor_cset_file, PrinterKind::Needle24).await,
                    l30: load_printer_font(fs, &editor_cset_file, PrinterKind::Laser30).await,
                    map: load_mapping_file(&editor_cset_file),
                }
            }
            None => {
                warn!("Editor font for `{}` not found!", name);
                CSet {
                    name: name.clone(),
                    e24: None,
                    p09: None,
                    p24: None,
                    l30: None,
                    map: None,
                }
            }
        };

        // Get index and push
        let new_index = self.chsets.len();
        self.chsets.push(cset);

        // Add lookup and return
        self.names.insert(name, new_index);
        new_index
    }

    /// Reset the font cache
    pub fn reset(&mut self) {
        let _ = std::mem::replace(self, ChsetCache::new());
    }
}

impl Default for ChsetCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Printer options for a single font
#[derive(Default, Clone)]
pub struct FontCacheInfo {
    index: Option<usize>,
    name: Option<String>,
}

impl FontCacheInfo {
    /// Get the index
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Get the name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl FontCacheInfo {
    const EMPTY: Self = Self {
        index: None,
        name: None,
    };
}

/// Print Options for a Document
#[derive(Default)]
pub struct DocumentFontCacheInfo {
    // /// Chosen Printer Driver
    // print_driver: Option<FontKind>,
    all_eset: bool,
    all_p24: bool,
    all_l30: bool,
    all_p09: bool,

    /// Character sets used by this document
    chsets: [FontCacheInfo; 8],
}

impl DocumentFontCacheInfo {
    /// Get the preferred print driver
    pub fn print_driver(&self, mut print_driver: Option<FontKind>) -> Option<FontKind> {
        // Print info on which sets are available
        if self.all_eset {
            info!("Editor fonts available for all character sets");
        }
        if self.all_p24 {
            info!("Printer fonts (24-needle) available for all character sets");
        }
        if self.all_l30 {
            info!("Printer fonts (laser/30) available for all character sets");
        }
        if self.all_p09 {
            info!("Printer fonts (9-needle) available for all character sets");
        }

        // If none was set, choose one strategy
        if let Some(pd) = print_driver {
            match pd {
                FontKind::Editor if !self.all_eset => {
                    warn!("Explicitly chosen editor print-driver but not all fonts are available");
                }
                FontKind::Printer(PrinterKind::Needle24) if !self.all_p24 => {
                    warn!(
                        "Explicitly chosen 24-needle print-driver but not all fonts are available"
                    );
                }
                FontKind::Printer(PrinterKind::Needle9) if !self.all_p09 => {
                    warn!(
                        "Explicitly chosen 9-needle print-driver but not all fonts are available"
                    );
                }
                FontKind::Printer(PrinterKind::Laser30) if !self.all_l30 => {
                    warn!(
                        "Explicitly chosen laser/30 print-driver but not all fonts are available"
                    );
                }
                _ => {
                    // All fonts available
                }
            }
        } else if self.all_l30 {
            print_driver = Some(FontKind::Printer(PrinterKind::Laser30));
        } else if self.all_p24 {
            print_driver = Some(FontKind::Printer(PrinterKind::Needle24));
        } else if self.all_p09 {
            print_driver = Some(FontKind::Printer(PrinterKind::Needle9));
        } else if self.all_eset {
            print_driver = Some(FontKind::Editor);
        } else {
            warn!("No print-driver has all fonts available.");
        }
        print_driver
    }

    /*pub fn from_cache<'a>(&self, fc: &'a ChsetCache) -> [Option<&'a FontInfo>; 8] {

    }*/

    /// Get the editor charset by index
    pub fn eset<'f>(&self, fc: &'f ChsetCache, cset: u8) -> Option<&'f ESet<'f>> {
        self.cset(fc, cset).and_then(CSet::e24)
    }

    /// Get the printer character set by index
    pub fn pset<'f>(&self, fc: &'f ChsetCache, cset: u8, pk: PrinterKind) -> Option<&'f PSet<'f>> {
        self.cset(fc, cset).and_then(match pk {
            PrinterKind::Needle24 => CSet::p24,
            PrinterKind::Needle9 => CSet::p09,
            PrinterKind::Laser30 => CSet::l30,
        })
    }

    /// Get the `cache::CSet` by index
    pub fn cset<'f>(&self, fc: &'f ChsetCache, cset: u8) -> Option<&'f CSet> {
        self.chsets[cset as usize]
            .index
            .and_then(|index| fc.cset(index))
    }

    /// Get the [FontCacheInfo] by (document) index
    pub fn font_cache_info_at(&self, cset: usize) -> Option<&FontCacheInfo> {
        self.chsets.get(cset)
    }

    /// Get all [FontCacheInfo]s
    pub fn font_cache_info(&self) -> &[FontCacheInfo; 8] {
        &self.chsets
    }

    /// Get all [FontCacheInfo]s
    pub fn cset_name(&self, cset: u8) -> Option<&str> {
        self.chsets.get(cset as usize).and_then(FontCacheInfo::name)
    }
}
