//! # Implementation of a charset cache

use std::{collections::HashMap, fs::DirEntry, path::Path, path::PathBuf};

use bstr::BStr;
use log::{info, warn};

use crate::{
    chsets::{
        editor::ESet,
        editor::OwnedESet,
        encoding::{p_mapping_file, Mapping},
        printer::OwnedPSet,
        printer::PSet,
        printer::PrinterKind,
        LoadError,
    },
    docs::cset,
};

use super::{encoding::decode_atari_str, FontKind};

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
impl<'a> CSet {
    /// The the name of the character set
    pub fn name(&self) -> &str { &self.name }
    /// Get the unicode mapping
    pub fn map(&self) -> Option<&Mapping> { self.map.as_ref() }
    /// Get the laser printer bitmaps
    pub fn l30(&'a self) -> Option<&PSet<'a>> { self.l30.as_ref().map(OwnedPSet::borrowed) }
    /// Get the 24-needle printer bitmaps
    pub fn p24(&'a self) -> Option<&PSet<'a>> { self.p24.as_ref().map(OwnedPSet::borrowed) }
    /// Get the 9-needle printer bitmaps
    pub fn p09(&'a self) -> Option<&PSet<'a>> { self.p09.as_ref().map(OwnedPSet::borrowed) }
    /// Get the editor bitmaps
    pub fn e24(&self) -> Option<&ESet<'static>> { self.e24.as_deref() }
    /// Get the bitmaps for the specified printer kind
    pub fn printer(&'a self, pk: PrinterKind) -> Option<&PSet<'a>> {
        match pk {
            PrinterKind::Needle9 => self.p09.as_ref().map(OwnedPSet::borrowed),
            PrinterKind::Needle24 => self.p24.as_ref().map(OwnedPSet::borrowed),
            PrinterKind::Laser30 => self.l30.as_ref().map(OwnedPSet::borrowed),
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
    pub fn pset<'a>(&'a self, pk: PrinterKind, index: usize) -> Option<&PSet<'a>> {
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
    pub fn load_cset(&mut self, name: &BStr) -> Option<usize> {
        let name = decode_atari_str(name.as_ref()).into_owned();
        if let Some(index) = self.names.get(&name) {
            return Some(*index);
        }

        let cset = match find_font_file(&self.chsets_folder, &name, "E24") {
            Some(editor_cset_file) => {
                // Load all font files
                CSet {
                    name: name.clone(),
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
        Some(new_index)
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
    /// Chosen Printer Driver
    print_driver: Option<FontKind>,
    /// Character sets used by this document
    pub chsets: [FontCacheInfo; 8],
}

impl DocumentFontCacheInfo {
    /// Get the preferred print driver
    pub fn print_driver(&self) -> Option<FontKind> {
        self.print_driver
    }

    /*pub fn from_cache<'a>(&self, fc: &'a ChsetCache) -> [Option<&'a FontInfo>; 8] {

    }*/

    /// Get print options for a CSet
    pub fn of<'a>(
        cset: &cset::CSet<'a>,
        fc: &mut ChsetCache,
        mut print_driver: Option<FontKind>,
    ) -> Self {
        let mut all_eset = true;
        let mut all_p24 = true;
        let mut all_l30 = true;
        let mut all_p09 = true;

        let mut chsets = [FontCacheInfo::EMPTY; 8];

        for (index, &name) in cset.names.iter().enumerate() {
            if name.is_empty() {
                continue;
            }
            chsets[index].name = Some(name.to_string());
            if let Some(cset_cache_index) = fc.load_cset(name) {
                let cset = fc
                    .cset(cset_cache_index)
                    .expect("invalid index returned by load_cset");
                chsets[index].index = Some(cset_cache_index);
                all_eset &= cset.e24().is_some();
                all_p24 &= cset.p24().is_some();
                all_l30 &= cset.l30().is_some();
                all_p09 &= cset.p09().is_some();
            }
        }
        // Print info on which sets are available
        if all_eset {
            info!("Editor fonts available for all character sets");
        }
        if all_p24 {
            info!("Printer fonts (24-needle) available for all character sets");
        }
        if all_l30 {
            info!("Printer fonts (laser/30) available for all character sets");
        }
        if all_p09 {
            info!("Printer fonts (9-needle) available for all character sets");
        }

        // If none was set, choose one strategy
        if let Some(pd) = print_driver {
            match pd {
                FontKind::Editor if !all_eset => {
                    warn!("Explicitly chosen editor print-driver but not all fonts are available");
                }
                FontKind::Printer(PrinterKind::Needle24) if !all_p24 => {
                    warn!(
                        "Explicitly chosen 24-needle print-driver but not all fonts are available"
                    );
                }
                FontKind::Printer(PrinterKind::Needle9) if !all_p09 => {
                    warn!(
                        "Explicitly chosen 9-needle print-driver but not all fonts are available"
                    );
                }
                FontKind::Printer(PrinterKind::Laser30) if !all_l30 => {
                    warn!(
                        "Explicitly chosen laser/30 print-driver but not all fonts are available"
                    );
                }
                _ => {
                    // All fonts available
                }
            }
        } else if all_l30 {
            print_driver = Some(FontKind::Printer(PrinterKind::Laser30));
        } else if all_p24 {
            print_driver = Some(FontKind::Printer(PrinterKind::Needle24));
        } else if all_p09 {
            print_driver = Some(FontKind::Printer(PrinterKind::Needle9));
        } else if all_eset {
            print_driver = Some(FontKind::Editor);
        } else {
            warn!("No print-driver has all fonts available.");
        }
        Self {
            print_driver,
            chsets,
        }
    }

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
}
