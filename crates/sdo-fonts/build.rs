use std::{
    env,
    io::{self, BufWriter, Write},
    path::Path,
};

use signum::chsets::encoding::p_mapping_file;

fn main() -> io::Result<()> {
    let mappings_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .join("mappings");
    println!("cargo::rerun-if-changed={}", mappings_path.display());
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("mappings.rs");
    let out_file = std::fs::File::create(&out_path).unwrap();
    let mut writer = BufWriter::new(out_file);

    let mut names = vec![];

    for entry in std::fs::read_dir(&mappings_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let _mapping = p_mapping_file(&text).unwrap();

        let mut code = String::new();
        signum::chsets::code::write_map(&_mapping, &mut code, name).unwrap();
        writer.write_all(code.as_bytes())?;

        names.push(name.to_owned());
    }

    writeln!(writer, "use signum::chsets::encoding::Mapping;")?;
    writeln!(
        writer,
        "pub fn lookup(name: &str) -> Option<&'static Mapping> {{"
    )?;
    writeln!(writer, "    match name {{")?;
    for name in names {
        writeln!(
            writer,
            "        {name:?} => Some(&Mapping {{ chars: {name} }}),"
        )?;
    }
    writeln!(writer, "        _ => None")?;
    writeln!(writer, "    }}")?;
    writeln!(writer, "}}")?;

    Ok(())
}
