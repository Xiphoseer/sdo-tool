use std::{
    io::{self, BufReader},
    path::{Path, PathBuf},
    process::exit,
};

use esc_p::{Command, EscPDecoder, Escape};
use image::{GrayImage, Luma};

#[derive(clap::Parser)]
struct Opts {
    /// Path to the printer file
    pub path: PathBuf,

    #[clap(long)]
    /// Create the file with mkfifo (1) before opening
    pub fifo: bool,

    #[clap(long)]
    /// Fill the gaps signum leaves for adjacent points
    pub fill: bool,

    #[clap(long)]
    /// Save bitmaps into the specified folder
    pub img_out: Option<PathBuf>,

    #[clap(long, default_value = "0")]
    /// Initial counter value for image output file names
    pub img_out_counter: usize,

    #[clap(long, default_value = "out")]
    /// Prefix for the image output
    pub img_out_prefix: String,

    #[clap(short, long, default_value = "2988")]
    /// Width of a single page / paper
    pub width: u32,

    #[clap(short, long, default_value = "4212")]
    /// Height of a single page / paper
    pub height: u32,
}

fn main() -> io::Result<()> {
    let opts: Opts = clap::Parser::parse();
    let path = &opts.path;

    #[cfg(unix)]
    if opts.fifo {
        let to_remove = path.to_owned();
        ctrlc::set_handler(move || {
            println!("received Ctrl+C!");
            std::fs::remove_file(&to_remove).unwrap();
            exit(0);
        })
        .expect("Error setting Ctrl-C handler");

        make_fifo(path)?;
    }

    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut decoder = EscPDecoder::new(reader);
    #[cfg(unix)]
    decoder.set_eof_on_zero(!opts.fifo);
    let mut counter = opts.img_out_counter;

    let output_width = opts.width;
    let output_height = opts.height;
    let output_pixel = Luma([0xFF]);

    let mut output = GrayImage::from_pixel(output_width, output_height, output_pixel);
    let (mut x, mut y) = (0u32, 0u32);
    let mut line_height: u32 = 0;

    let xunit = 360 / 60;

    loop {
        match decoder.advance() {
            Ok(command) => {
                println!("{:?}", command);
                match command {
                    Command::Eof => break Ok(()),
                    Command::LineFeed => y += line_height,
                    Command::CarriageReturn => x = 0,
                    Command::Byte(_byte) => {}
                    Command::Esc(Escape::SelectBitImage { m: _, n, data }) => {
                        let width = n as u32;
                        let k = data.as_bytes().len();
                        let bytes_per_col = k as u32 / width;
                        let bpc_usize = bytes_per_col as usize;
                        for y0 in 0..bytes_per_col {
                            for y1 in 0..8 {
                                let mut prev_ink = false;
                                for (x0, c) in data.as_bytes().chunks(bpc_usize).enumerate() {
                                    let byte = c[y0 as usize];
                                    let val = (byte >> (7 - y1)) & 1;
                                    let ink = val != 0;
                                    let mut y_local = (y0 << 3) + y1;
                                    y_local <<= 1;
                                    let y_out = y + y_local;
                                    let x_out = x + x0 as u32;
                                    if ink {
                                        *output.get_pixel_mut(x_out, y_out) = Luma([0x00]);
                                    }
                                    if opts.fill && !prev_ink && ink && x_out > 0 {
                                        *output.get_pixel_mut(x_out - 1, y_out) = Luma([0x00]);
                                    }
                                    prev_ink = ink;
                                }
                            }
                        }
                    }
                    Command::Esc(Escape::LineSpacing { n }) => line_height = u32::from(n) << 1,
                    Command::Esc(Escape::LineSpacing360 { n }) => line_height = u32::from(n),
                    Command::Esc(Escape::XPos { n }) => x = u32::from(n) * xunit,
                    Command::Esc(Escape::Unknown(_)) => {}
                    Command::Esc(_) => unimplemented!(),
                    Command::FormFeed => {
                        if let Some(out) = opts.img_out.as_deref() {
                            let filename = format!("{}{counter:03}.png", opts.img_out_prefix);
                            counter += 1;
                            let path = out.join(&filename);
                            output.save(&path).unwrap();
                            output =
                                GrayImage::from_pixel(output_width, output_height, output_pixel);
                        }
                    }
                }
            }
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}

#[cfg(unix)]
fn make_fifo<P: AsRef<Path>>(path: &P) -> nix::Result<()> {
    use nix::sys::stat::Mode;
    nix::unistd::mkfifo(
        path.as_ref(),
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IROTH,
    )
}
