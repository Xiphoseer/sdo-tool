use std::{fs::File, io::BufWriter, io::Write, path::Path};

use color_eyre::eyre::{self, eyre};
use sdo::{font::printer::PrintDriver, ps::PSWriter};

use crate::cli::font::ps::write_ls30_ps_bitmap;

use super::{ps_proc::prog_dict, Document};

fn output_ps_writer(doc: &Document, pw: &mut PSWriter<impl Write>) -> eyre::Result<()> {
    let pd = doc
        .print_driver
        .ok_or_else(|| eyre!("No printer type selected"))?;
    let (hdpi, vdpi) = pd.resolution();

    pw.write_magic()?;
    pw.write_meta_field("Creator", "Signum! Document Toolbox v0.3")?;
    let file_name = doc.file.file_name().unwrap().to_string_lossy();
    pw.write_meta_field("Title", file_name.as_ref())?;
    //pw.write_meta_field("CreationDate", "Sun Sep 13 23:55:06 2020")?;
    pw.write_meta_field("Pages", &format!("{}", doc.page_count))?;
    pw.write_meta_field("PageOrder", "Ascend")?;
    pw.write_meta_field("BoundingBox", "0 0 596 842")?;
    pw.write_meta_field("DocumentPaperSizes", "a4")?;
    pw.write_meta("EndComments")?;

    pw.write_meta_field("BeginProcSet", "signum.pro")?;
    pw.write_header_end()?;

    const DICT: &str = "SignumDict";
    const FONTS: [&str; 8] = ["Fa", "Fb", "Fc", "Fd", "Fe", "Ff", "Fg", "Fh"];
    prog_dict(pw, DICT)?;

    pw.write_meta("EndProcSet")?;
    pw.name(DICT)?;

    let mut use_matrix: [[usize; 128]; 8] = [[0; 128]; 8];

    for page in &doc.tebu {
        for (_, line) in &page.content {
            for tw in &line.data {
                use_matrix[tw.cset as usize][tw.cval as usize] += 1;
            }
        }
    }

    pw.begin(|pw| {
        pw.isize(39158280)?;
        pw.isize(55380996)?;
        pw.isize(1000)?;
        pw.isize(hdpi)?;
        pw.isize(vdpi)?;
        pw.bytes(b"hello.dvi")?;
        pw.crlf()?;
        pw.name("@start")?;
        for (i, use_matrix) in use_matrix.iter().enumerate() {
            match pd {
                PrintDriver::Printer24 => {
                    if let Some(pset) = &doc.chsets_p24[i] {
                        let name = &doc.chsets[i];
                        pw.write_comment(&format!("SignumBitmapFont: {}", name))?;
                        write_ls30_ps_bitmap(FONTS[i], name, pw, pset, Some(use_matrix))?;
                        pw.write_comment("EndSignumBitmapFont")?;
                    }
                }
                PrintDriver::Laser30 => {
                    if let Some(pset) = &doc.chsets_l30[i] {
                        let name = &doc.chsets[i];
                        pw.write_comment(&format!("SignumBitmapFont: {}", name))?;
                        write_ls30_ps_bitmap(FONTS[i], name, pw, pset, Some(use_matrix))?;
                        pw.write_comment("EndSignumBitmapFont")?;
                    }
                }
                _ => {
                    println!("Print-Driver {:?} not yet supported", pd);
                }
            }
        }

        Ok(())
    })?;
    pw.write_meta("EndProlog")?;

    pw.write_meta("BeginSetup")?;
    let feature = format!("*Resolution {}dpi", hdpi);
    pw.write_meta_field("Feature", &feature)?;

    pw.name(DICT)?;
    pw.begin(|pw| {
        pw.write_meta_field("BeginPaperSize", "a4")?;
        pw.lit("setpagedevice")?;
        pw.ps_where()?;
        pw.crlf()?;
        pw.seq(|pw| {
            pw.ps_pop()?;
            pw.dict(|pw| {
                pw.lit("PageSize")?;
                pw.arr(|pw| {
                    pw.isize(595)?;
                    pw.isize(842)
                })
            })?;
            pw.ps_setpagedevice()
        })?;
        pw.crlf()?;
        pw.seq(|pw| {
            pw.lit("a4")?;
            pw.ps_where()?;
            pw.seq(|pw| {
                pw.ps_pop()?;
                pw.name("a4")
            })?;
            pw.ps_if()
        })?;
        pw.crlf()?;
        pw.ps_ifelse()?;
        pw.write_meta("EndPaperSize")?;
        Ok(())
    })?;
    pw.write_meta("EndSetup")?;

    let x_offset = doc.opt.xoffset.unwrap_or(0);

    for (index, page) in doc.tebu.iter().enumerate() {
        let page_info = doc.pages[page.index as usize].as_ref().unwrap();
        let page_comment = format!("{} {}", page_info.log_pnr, page_info.phys_pnr);
        pw.write_meta_field("Page", &page_comment)?;

        pw.name(DICT)?;
        pw.begin(|pw| {
            let mut x: u16;
            let mut y: u16 = 0;
            let mut cset = 10;

            pw.isize(page_info.log_pnr as isize)?;
            pw.isize(index as isize)?;
            pw.name("bop")?;

            for (skip, line) in &page.content {
                y += 1 + *skip;
                x = 0;

                let y_val = pd.scale_y(y) as isize;
                for chr in &line.data {
                    // moveto
                    x += chr.offset;

                    if cset != chr.cset {
                        // select font a
                        cset = chr.cset;
                        pw.name(FONTS[chr.cset as usize])?;
                    }

                    let x_val = pd.scale_x(x) as isize + x_offset;
                    pw.isize(x_val)?;
                    pw.isize(y_val)?;
                    pw.name("a")?;

                    pw.bytes(&[chr.cval])?;
                    pw.name("p")?;
                }
            }

            pw.name("eop")?;
            Ok(())
        })?;
    }
    pw.write_meta("Trailer")?;

    pw.ps_userdict()?;
    pw.lit("end-hook")?;
    pw.ps_known()?;
    pw.seq(|pw| pw.name("end-hook"))?;
    pw.ps_if()?;

    pw.write_meta("EOF")?;
    Ok(())
}

pub fn output_postscript(doc: &Document) -> eyre::Result<()> {
    if doc.opt.out == Path::new("-") {
        println!("----------------------------- PostScript -----------------------------");
        let mut pw = PSWriter::new();
        output_ps_writer(doc, &mut pw)?;
        println!("----------------------------------------------------------------------");
        Ok(())
    } else {
        let file = doc.file.file_stem().unwrap();
        let out = {
            let mut buf = doc.opt.out.join(file);
            buf.set_extension("ps");
            buf
        };
        let out_file = File::create(&out)?;
        let out_buf = BufWriter::new(out_file);
        let mut pw = PSWriter::from(out_buf);
        print!("Writing `{}` ...", out.display());
        output_ps_writer(doc, &mut pw)?;
        println!(" Done!");
        Ok(())
    }
}
