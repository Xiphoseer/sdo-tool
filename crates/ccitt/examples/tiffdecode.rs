use std::{
    fmt,
    io::{BufWriter, Cursor},
    path::PathBuf,
};

use ccitt_t4_t6::{
    bits::{BitWriter, FillOrder},
    g42d::{fax_decode, FaxOptions, G4Decoder},
    pbm_to_io_writer,
};
use color_eyre::eyre::{self, eyre};
use tiff::{
    decoder::{ifd::Value, Decoder as TiffDecoder},
    tags::{CompressionMethod, PhotometricInterpretation, ResolutionUnit, Tag},
};

#[derive(Debug)]
struct UnknownDecoder;

impl fmt::Display for UnknownDecoder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

enum DecoderImpl {
    BitIter,
    StateMachine,
}

impl std::str::FromStr for DecoderImpl {
    type Err = UnknownDecoder;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "b" | "bi" | "bit-iter" => Ok(Self::BitIter),
            "s" | "sm" | "state-machine" => Ok(Self::StateMachine),
            _ => Err(UnknownDecoder),
        }
    }
}

#[derive(argh::FromArgs)]
/// load a Group 4 encoded file and write it to console
struct Options {
    #[argh(positional)]
    /// path to input file
    file: PathBuf,

    #[argh(option, short = 'o')]
    /// path to output file
    output: Option<PathBuf>,

    #[argh(option, short = 'b')]
    /// write a PBM file from the decoded input
    pbm: Option<PathBuf>,

    #[argh(option, short = 'd', default = "DecoderImpl::BitIter")]
    /// write a PBM file from the decoded input
    decoder: DecoderImpl,

    /// invert black and white
    #[argh(switch, short = 'i')]
    invert: bool,

    /// print a bitmap after decoding
    #[argh(switch, short = 'p')]
    print: bool,

    /// print a bitmap after decoding
    #[argh(switch, short = 'v')]
    debug: bool,
}

fn value_into_rational(v: Value) -> Option<f64> {
    match v {
        Value::Rational(num, denom) => Some(num as f64 / denom as f64),
        Value::RationalBig(num, denom) => Some(num as f64 / denom as f64),
        Value::SRational(num, denom) => Some(num as f64 / denom as f64),
        Value::SRationalBig(num, denom) => Some(num as f64 / denom as f64),
        _ => None,
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let opt: Options = argh::from_env();
    let file = std::fs::read(&opt.file)?;

    let mut tiff_decoder = TiffDecoder::new(Cursor::new(file))?;
    let compression = tiff_decoder.get_tag(Tag::Compression)?.into_u16()?;
    let compression = CompressionMethod::from_u16_exhaustive(compression);
    dbg!(compression);
    match compression {
        CompressionMethod::Fax3 => {
            // Group 4
            todo!("Group 3 coding");
        }
        CompressionMethod::Fax4 => {
            // Group 4
            let photometric_interpretation = tiff_decoder
                .find_tag(Tag::PhotometricInterpretation)?
                .map(Value::into_u16)
                .transpose()?
                .and_then(PhotometricInterpretation::from_u16)
                .unwrap_or(PhotometricInterpretation::WhiteIsZero);
            dbg!(photometric_interpretation);
            let width = tiff_decoder.get_tag(Tag::ImageWidth)?.into_u16()?;
            let length = tiff_decoder.get_tag(Tag::ImageLength)?.into_u16()?;
            let f = tiff_decoder
                .find_tag(Tag::FillOrder)?
                .map(Value::into_u16)
                .transpose()?;
            let fill_order = match f {
                Some(1) => Some(FillOrder::MsbToLsb),
                Some(2) => Some(FillOrder::LsbToMsb),
                Some(i) => return Err(eyre!("Unknown fill order: {i}")),
                None => None,
            }
            .unwrap_or(FillOrder::MsbToLsb);

            let resolution_unit = tiff_decoder
                .find_tag(Tag::ResolutionUnit)?
                .map(Value::into_u16)
                .transpose()?
                .and_then(ResolutionUnit::from_u16);
            let xres = tiff_decoder
                .find_tag(Tag::XResolution)?
                .and_then(value_into_rational);
            let yres = tiff_decoder
                .find_tag(Tag::YResolution)?
                .and_then(value_into_rational);
            let res = xres.zip(yres);
            let mut dbl = false;
            if let Some(resolution) = res {
                dbg!(resolution);
                let aspect_ratio = resolution.0 / resolution.1;
                dbg!(aspect_ratio);
                if aspect_ratio.round() == 2.0 {
                    dbl = true;
                }
            }
            dbg!(resolution_unit);

            dbg!(width, length);
            dbg!(fill_order);
            dbg!(tiff_decoder.byte_order());
            let offsets = tiff_decoder.get_tag_u64_vec(Tag::StripOffsets)?;
            let byte_counts = tiff_decoder.get_tag_u64_vec(Tag::StripByteCounts)?;
            let iter = offsets.into_iter().zip(byte_counts.into_iter());
            //dbg!(tiff_decoder.chunk_data_dimensions(0));
            for (offset, byte_count) in iter {
                dbg!(offset, byte_count);
                tiff_decoder.goto_offset_u64(offset)?;
                let inner = tiff_decoder.inner();
                let pos = inner.position();
                let bytes = inner.get_ref().as_slice();
                let bytes = &bytes[pos as usize..][..byte_count as usize];

                assert_eq!(bytes.len(), byte_count as usize);

                if let Some(out) = &opt.output {
                    std::fs::write(out, bytes)?;
                }

                match opt.decoder {
                    DecoderImpl::BitIter => {
                        let mut fax_options = FaxOptions::default();
                        fax_options.fill_order = fill_order;
                        fax_options.width = width.into();
                        fax_options.debug = opt.debug;
                        let image = fax_decode(bytes, fax_options).expect("fax_decode");
                        println!("DONE fax_decode");

                        if let Some(out) = &opt.pbm {
                            let file = std::fs::File::create(&out)?;
                            let mut buf_writer = BufWriter::new(file);
                            image.write_pbm(&mut buf_writer, dbl, opt.invert)?;
                        }

                        if opt.print {
                            image.print(opt.invert);
                        }
                    }
                    DecoderImpl::StateMachine => {
                        let mut decoder = G4Decoder::<BitWriter>::new(width.into());
                        decoder.set_fill_order(fill_order);
                        decoder.decode(&bytes)?;
                        let store = decoder.into_store();

                        let bitmap = store.done();
                        println!("DONE G4Decoder");

                        if let Some(out) = &opt.pbm {
                            let file = std::fs::File::create(&out)?;
                            let mut buf_writer = BufWriter::new(file);
                            pbm_to_io_writer(
                                &mut buf_writer,
                                &bitmap,
                                width as usize,
                                opt.invert,
                                dbl,
                            )?;
                        }

                        if opt.print {
                            let mut string = String::new();
                            ccitt_t4_t6::ascii_art(
                                &mut string,
                                &bitmap,
                                width as usize,
                                opt.invert,
                            )
                            .unwrap();
                            print!("{}", string);
                        }
                    }
                }
            }
            Ok(())
        }
        _ => Err(eyre!("Compression not supported: {:?}", compression)),
    }
}
