//! # Implementation of a charset cache

use std::{collections::HashMap, fs::DirEntry, path::Path, path::PathBuf};

use log::{info, warn};

use crate::chsets::{
    editor::ESet,
    editor::OwnedESet,
    encoding::{p_mapping_file, Mapping},
    printer::OwnedPSet,
    printer::PSet,
    printer::PrinterKind,
    LoadError,
};

fn find_font_file(cset_folder: &Path, name: &str, extension: &str) -> Option<PathBuf> {
    let cset_file = cset_folder.join(name);
    let editor_cset_file = cset_file.with_extension(extension);

    if editor_cset_file.exists() && editor_cset_file.is_file() {
        return Some(editor_cset_file);
    }

    let mut dir_iter = match std::fs::read_dir(cset_folder) {
        Ok(i) => i,
        Err(e) => {
            warn!("Could not find CHSET folder: {}", e);
            return None;
        }
    };

    dir_iter.find_map(|entry| {
        entry
            .ok()
            .as_ref()
            .map(DirEntry::path)
            .filter(|p| p.is_dir())
            .and_then(|cset_folder| find_font_file(&cset_folder, name, extension))
    })
}

fn load_printer_font(editor_cset_file: &Path, pk: PrinterKind) -> Option<OwnedPSet> {
    let extension = pk.extension();
    let printer_cset_file = editor_cset_file.with_extension(extension);
    match OwnedPSet::load(&printer_cset_file, pk) {
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

fn load_editor_font(editor_cset_file: &Path) -> Option<OwnedESet> {
    match OwnedESet::load(editor_cset_file) {
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
impl CSet {
    /// The the name of the character set
    pub fn name(&self) -> &str { &self.name }
    /// Get the unicode mapping
    pub fn map(&self) -> Option<&Mapping> { self.map.as_ref() }
    /// Get the laser printer bitmaps
    pub fn l30(&self) -> Option<&PSet<'static>> { self.l30.as_deref() }
    /// Get the 24-needle printer bitmaps
    pub fn p24(&self) -> Option<&PSet<'static>> { self.p24.as_deref() }
    /// Get the 9-needle printer bitmaps
    pub fn p09(&self) -> Option<&PSet<'static>> { self.p09.as_deref() }
    /// Get the editor bitmaps
    pub fn e24(&self) -> Option<&ESet<'static>> { self.e24.as_deref() }
    /// Get the bitmaps for the specified printer kind
    pub fn printer(&self, pk: PrinterKind) -> Option<&PSet<'static>> {
        match pk {
            PrinterKind::Needle9 => self.p09.as_deref(),
            PrinterKind::Needle24 => self.p24.as_deref(),
            PrinterKind::Laser30 => self.l30.as_deref(),
        }
    }
}

/// A simple cache for charsets
pub struct ChsetCache {
    chsets_folder: PathBuf,
    chsets: Vec<CSet>,
    names: HashMap<String, usize>,
}

impl ChsetCache {
    /// Create a new instance
    pub fn new(chsets_folder: PathBuf) -> Self {
        ChsetCache {
            chsets: Vec::with_capacity(8),
            names: HashMap::new(),
            chsets_folder,
        }
    }

    /// Get the slice of all charsets
    pub fn chsets(&self) -> &[CSet] {
        &self.chsets
    }

    /// Get a specific printer charset
    pub fn pset(&self, pk: PrinterKind, index: usize) -> Option<&PSet<'static>> {
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

    /// Load a character set
    pub fn load_cset(&mut self, name: &str) -> Option<usize> {
        if let Some(index) = self.names.get(name) {
            return Some(*index);
        }

        let cset = match find_font_file(&self.chsets_folder, name, "E24") {
            Some(editor_cset_file) => {
                // Load all font files
                CSet {
                    name: name.to_owned(),
                    e24: load_editor_font(&editor_cset_file),
                    p09: load_printer_font(&editor_cset_file, PrinterKind::Needle9),
                    p24: load_printer_font(&editor_cset_file, PrinterKind::Needle24),
                    l30: load_printer_font(&editor_cset_file, PrinterKind::Laser30),
                    map: load_mapping_file(&editor_cset_file),
                }
            }
            None => {
                warn!("Editor font for `{}` not found!", name);
                CSet {
                    name: name.to_owned(),
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
        self.names.insert(name.to_owned(), new_index);
        Some(new_index)
    }
}
