use std::path::PathBuf;

use nom::{Finish, IResult};
use nom_supreme::{
    error::ErrorTree,
    final_parser::{ByteOffset, ExtractContext},
};
use pcf::{
    parser::{
        p_pcf_accelerators, p_pcf_bdf_encodings, p_pcf_bitmaps, p_pcf_glpyh_names, p_pcf_header,
        p_pcf_metrics, p_pcf_properties, p_pcf_swidths,
    },
    PCFProperties, PCFTableKind, XChar,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
/// Prints information about a X11 PCF file
struct Opts {
    file: PathBuf,
}

fn to_err_tree<'a>(
    original_input: &'a [u8],
) -> impl FnOnce(ErrorTree<&'a [u8]>) -> ErrorTree<usize> {
    move |t| {
        let t2: ErrorTree<ByteOffset> = t.extract_context(original_input);
        let t3: ErrorTree<usize> = t2.map_locations(|o| o.0);
        t3
    }
}

fn load<'a, F, T>(fun: F, input: &'a [u8]) -> Result<T, ErrorTree<usize>>
where
    F: FnOnce(&'a [u8]) -> IResult<&'a [u8], T, ErrorTree<&'a [u8]>>,
{
    let (_, result) = fun(input).finish().map_err(to_err_tree(input))?;
    Ok(result)
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let opts: Opts = Opts::from_args();
    let buffer = std::fs::read(&opts.file)?;
    let (_, header) = p_pcf_header(&buffer)
        .finish()
        .map_err(to_err_tree(&buffer[..]))?;

    let mut props = PCFProperties::default();

    if let Some(prop_ref) = header.get(PCFTableKind::PROPERTIES) {
        if let Some(prop_buf) = prop_ref.of(&buffer) {
            props = load(p_pcf_properties, prop_buf)?;
        }
    }

    let mut accs = None;
    if let Some(accs_ref) = header.get(PCFTableKind::ACCELERATORS) {
        if let Some(accs_buf) = accs_ref.of(&buffer) {
            accs = Some(load(p_pcf_accelerators, accs_buf)?);
        }
    }

    let mut glyphs = Vec::new();
    if let Some(mets_ref) = header.get(PCFTableKind::METRICS) {
        if let Some(mets_buf) = mets_ref.of(&buffer) {
            let mets = load(p_pcf_metrics, mets_buf)?;
            glyphs = Vec::with_capacity(mets.metrics.len());
            for metrics in mets.metrics {
                glyphs.push(XChar {
                    metrics,
                    bitmap: None,
                    swidth: None,
                    name: None,
                })
            }
        }
    }

    if let Some(bitmaps_ref) = header.get(PCFTableKind::BITMAPS) {
        if let Some(bitmaps_buf) = bitmaps_ref.of(&buffer) {
            load(p_pcf_bitmaps(&mut glyphs), bitmaps_buf)?;
        }
    }

    if let Some(swidth_ref) = header.get(PCFTableKind::SWIDTHS) {
        if let Some(swidth_buf) = swidth_ref.of(&buffer) {
            let swidth_tbl = load(p_pcf_swidths, swidth_buf)?;
            assert_eq!(swidth_tbl.swidths.len(), glyphs.len());
            for (i, swidth) in swidth_tbl.swidths.into_iter().enumerate() {
                glyphs[i].swidth = Some(swidth);
            }
        }
    }

    if let Some(names_ref) = header.get(PCFTableKind::GLYPH_NAMES) {
        if let Some(names_buf) = names_ref.of(&buffer) {
            let names_tbl = load(p_pcf_glpyh_names, names_buf)?;
            assert_eq!(names_tbl.names.len(), glyphs.len());
            for (i, name) in names_tbl.names.into_iter().enumerate() {
                glyphs[i].name = Some(name);
            }
        }
    }

    let mut encodings = None;
    if let Some(encodings_ref) = header.get(PCFTableKind::BDF_ENCODINGS) {
        if let Some(encodings_buf) = encodings_ref.of(&buffer) {
            encodings = Some(load(p_pcf_bdf_encodings, encodings_buf)?);
        }
    }

    println!("{:#?}", props);
    println!("{:#?}", accs);
    println!("{:?}", encodings);
    println!("{:#?}", glyphs);

    for table in header.tables {
        match table.kind {
            PCFTableKind::PROPERTIES => println!("- PROPERTIES"),
            PCFTableKind::ACCELERATORS => println!("- ACCELERATORS"),
            PCFTableKind::METRICS => println!("- METRICS"),
            PCFTableKind::BITMAPS => println!("- BITMAPS"),
            PCFTableKind::INK_METRICS => println!("- INK_METRICS"),
            PCFTableKind::BDF_ENCODINGS => println!("- BDF_ENCODINGS"),
            PCFTableKind::SWIDTHS => println!("- SWIDTHS"),
            PCFTableKind::GLYPH_NAMES => println!("- GLYPH_NAMES"),
            PCFTableKind::BDF_ACCELERATORS => println!("- BDF_ACCELERATORS"),
            _ => println!("Unknown table kind: {}", table.kind.0),
        }
        if table.pos.of(&buffer).is_some() {
            println!("Table OK");
        } else {
            println!(
                "Table out of bounds: {}+{} of {}",
                table.pos.offset,
                table.pos.size,
                buffer.len()
            );
        }
    }

    Ok(())
}
