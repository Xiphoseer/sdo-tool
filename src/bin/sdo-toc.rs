use std::{ffi::OsStr, fs, io::BufWriter, path::PathBuf};

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use log::{info, trace};
use ron::ser::PrettyConfig;
use signum::{chsets::cache::ChsetCache, docs::tebu::Style, util::LocalFS};

use sdo_tool::cli::{
    self,
    opt::{DocScript, OutlineItem},
    sdoc::Document,
};

#[derive(Parser, Debug)]
/// Run a document script
pub struct RunOpts {
    /// A document script
    file: PathBuf,

    /// Output file (- is STDOUT)
    #[clap(default_value = "-")]
    out: PathBuf,
}

pub fn run(buffer: &[u8], opt: RunOpts) -> eyre::Result<()> {
    let script_str_res = std::str::from_utf8(buffer);
    let script_str = WrapErr::wrap_err(script_str_res, "Failed to parse as string")?;
    let script_res = ron::from_str(script_str);
    let script: DocScript = WrapErr::wrap_err(script_res, "Failed to parse DocScript")?;

    // Set-Up font cache
    let folder = opt.file.parent().unwrap();
    let chsets_folder = folder.join(&script.chsets);
    let chsets_folder: PathBuf = chsets_folder.canonicalize().wrap_err_with(|| {
        format!(
            "Failed to canonicalize CHSETS folder path `{}`",
            chsets_folder.display()
        )
    })?;
    let fs = LocalFS::new(chsets_folder);
    let mut fc = ChsetCache::new();

    let capacity = script.files.len();

    // Load documents
    let mut doc_files = Vec::with_capacity(capacity);
    for doc_path in &script.files {
        let doc_file = folder.join(doc_path);
        let doc_file = doc_file.canonicalize().wrap_err_with(|| {
            format!(
                "Failed to canonicalize document file path `{}`",
                doc_file.display()
            )
        })?;
        let input = std::fs::read(&doc_file)?;
        doc_files.push((doc_file, input));
    }

    let mut documents = Vec::with_capacity(capacity);
    for (doc_file, input) in &doc_files {
        let mut document = Document::new();

        info!("Loading document file '{}'", doc_file.display());
        let di = document.process_sdoc(input, &fs, &mut fc)?;
        documents.push((document, di));
    }

    let mut outline = Vec::<OutlineItem>::new();

    let mut pnum = 0;
    for (doc, di) in documents {
        for page in doc.text_pages() {
            let mut y = 0;
            let mut page_lines: Vec<(u8, u16, String)> = Vec::new();
            for (ydiff, line) in &page.content {
                y += *ydiff + 1;
                if let Some((st, _cset)) = line.line_style() {
                    if st == (Style::TALL | Style::WIDE) {
                        let text = line.text(&fc, &di.fonts);
                        if let Some((_, last_y, last_text)) = page_lines.last_mut() {
                            if y - *last_y < 40
                                && !text.chars().next().is_some_and(|c| c.is_ascii_digit())
                            {
                                if last_text.ends_with('-') || last_text.ends_with('~') {
                                    last_text.pop();
                                } else {
                                    last_text.push(' ');
                                }
                                last_text.push_str(&text);
                                continue;
                            }
                        }
                        let mut level = 0;
                        if let Some((_, rest)) = text.split_once('.') {
                            if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                                level = 1;
                            }
                        }
                        page_lines.push((level, y, text));
                    } else if st == Style::TALL {
                        let text = line.text(&fc, &di.fonts);
                        page_lines.push((2, y, text));
                    }
                }
            }
            for (level, y, text) in page_lines {
                let indent = &"             "[..level as usize];
                trace!("{indent} p{pnum:>3}+{y:>4} {text}");
                let out = recurse_outline_level(level, &mut outline);
                out.push(OutlineItem {
                    title: text,
                    dest: cli::opt::Destination::PageFitH(pnum, 800 - (y as usize * 4 / 3)),
                    children: vec![],
                });
            }
            pnum += 1;
        }
    }

    let cfg = PrettyConfig::new().with_separate_tuple_members(false);
    if opt.out.as_os_str() == OsStr::new("-") {
        ron::ser::to_writer_pretty(std::io::stdout(), &outline, cfg)?;
    } else {
        let file = std::fs::File::create_new(&opt.out)?;
        let writer = BufWriter::new(file);
        ron::ser::to_writer_pretty(writer, &outline, cfg)?;
        log::info!("Wrote '{}'", opt.out.display());
    }

    Ok(())
}

fn recurse_outline_level(level: u8, o: &mut Vec<OutlineItem>) -> &mut Vec<OutlineItem> {
    if level == 0 || o.is_empty() {
        return o;
    }
    let inner = o.last_mut().unwrap();
    recurse_outline_level(level - 1, &mut inner.children)
}

fn main() -> color_eyre::Result<()> {
    let opt: RunOpts = cli::init()?;

    let buffer = fs::read(&opt.file)
        .wrap_err_with(|| format!("Failed to open file: `{}`", opt.file.display()))?;

    run(&buffer, opt)
}
