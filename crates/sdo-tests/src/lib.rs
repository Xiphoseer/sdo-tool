use std::path::{Path, PathBuf};

pub fn docs_path() -> PathBuf {
    let test_pkg = Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap();
    let crates = test_pkg.parent().unwrap();
    let workspace = crates.parent().unwrap();
    workspace.join("docs")
}

pub const PRODUCER: &str = "Signum! Document Toolbox";
