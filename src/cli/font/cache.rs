use std::{collections::HashMap, fs::DirEntry, path::Path, path::PathBuf};

use signum::chsets::{
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
            println!("Could not find CHSET folder: {}", e);
            return None;
        }
    };

    let file = dir_iter.find_map(|entry| {
        entry
            .ok()
            .as_ref()
            .map(DirEntry::path)
            .filter(|p| p.is_dir())
            .and_then(|cset_folder| find_font_file(&cset_folder, name, extension))
    });

    if let Some(file) = file {
        Some(file)
    } else {
        None
    }
}

fn load_printer_font(editor_cset_file: &Path, pk: PrinterKind) -> Option<OwnedPSet> {
    let extension = pk.extension();
    let printer_cset_file = editor_cset_file.with_extension(extension);
    match OwnedPSet::load(&printer_cset_file, pk) {
        Ok(pset) => {
            println!("Loaded printer font file '{}'", printer_cset_file.display());
            Some(pset)
        }
        Err(LoadError::Io(_)) => None,
        Err(LoadError::Parse(f)) => {
            println!("Failed to parse {} printer font: {}", extension, f);
            None
        }
    }
}

fn load_mapping_file(editor_cset_file: &Path) -> Option<Mapping> {
    let cset_mapping_file = editor_cset_file.with_extension("txt");
    if cset_mapping_file.is_file() {
        let input = std::fs::read_to_string(&cset_mapping_file).unwrap();
        match p_mapping_file(&input) {
            Ok(mapping) => {
                eprintln!("Loaded cset mapping file '{}'", cset_mapping_file.display());
                Some(mapping)
            }
            Err(err) => {
                eprintln!(
                    "[cli::font::cache] Failed to parse mapping file '{}': {}",
                    cset_mapping_file.display(),
                    err
                );
                None
            }
        }
    } else {
        eprintln!(
            "[cli::font::cache] missing mapping for font '{}",
            editor_cset_file.file_stem().unwrap().to_string_lossy()
        );
        None
    }
}

fn load_editor_font(editor_cset_file: &Path) -> Option<OwnedESet> {
    match OwnedESet::load(&editor_cset_file) {
        Ok(eset) => {
            println!("Loaded editor font file '{}'", editor_cset_file.display());
            Some(eset)
        }
        Err(LoadError::Parse(e)) => {
            println!(
                "Failed to parse editor font file {}",
                editor_cset_file.display()
            );
            println!("Are you sure this is a valid Signum! editor font?");
            println!("Error: {}", e);
            None
        }
        Err(LoadError::Io(e)) => {
            println!(
                "Failed to load editor font file {}",
                editor_cset_file.display()
            );
            println!("Error: {}", e);
            None
        }
    }
}

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
    pub fn name(&self) -> &str { &self.name }
    pub fn map(&self) -> Option<&Mapping> { self.map.as_ref() }
    pub fn l30(&self) -> Option<&PSet<'static>> { self.l30.as_deref() }
    pub fn p24(&self) -> Option<&PSet<'static>> { self.p24.as_deref() }
    pub fn p09(&self) -> Option<&PSet<'static>> { self.p09.as_deref() }
    pub fn e24(&self) -> Option<&ESet<'static>> { self.e24.as_deref() }
    pub fn printer(&self, pk: PrinterKind) -> Option<&PSet<'static>> {
        match pk {
            PrinterKind::Needle9 => self.p09.as_deref(),
            PrinterKind::Needle24 => self.p24.as_deref(),
            PrinterKind::Laser30 => self.l30.as_deref(),
        }
    }
}

pub struct FontCache {
    chsets_folder: PathBuf,
    chsets: Vec<CSet>,
    names: HashMap<String, usize>,
}

impl FontCache {
    pub fn new(chsets_folder: PathBuf) -> Self {
        FontCache {
            chsets: Vec::with_capacity(8),
            names: HashMap::new(),
            chsets_folder,
        }
    }

    pub fn chsets(&self) -> &[CSet] {
        &self.chsets
    }

    pub fn pset(&self, pk: PrinterKind, index: usize) -> Option<&PSet<'static>> {
        self.cset(index).and_then(|cset| cset.printer(pk))
    }

    pub fn eset(&self, index: usize) -> Option<&ESet<'static>> {
        self.cset(index).and_then(CSet::e24)
    }

    pub fn cset(&self, index: usize) -> Option<&CSet> {
        self.chsets.get(index)
    }

    pub fn load_cset(&mut self, name: &str) -> Option<usize> {
        if let Some(index) = self.names.get(name) {
            return Some(*index);
        }

        let editor_cset_file = match find_font_file(&self.chsets_folder, name, "E24") {
            Some(f) => f,
            None => {
                println!("Editor font for `{}` not found!", name);
                return None;
            }
        };

        // Load all font files
        let cset = CSet {
            name: name.to_owned(),
            e24: load_editor_font(&editor_cset_file),
            p09: load_printer_font(&editor_cset_file, PrinterKind::Needle9),
            p24: load_printer_font(&editor_cset_file, PrinterKind::Needle24),
            l30: load_printer_font(&editor_cset_file, PrinterKind::Laser30),
            map: load_mapping_file(&editor_cset_file),
        };

        // Get index and push
        let new_index = self.chsets.len();
        self.chsets.push(cset);

        // Add lookup and return
        self.names.insert(name.to_owned(), new_index);
        Some(new_index)
    }
}
