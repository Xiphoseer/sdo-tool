//! # (`hcim`) The hardcopy images

use std::borrow::Cow;

use nom::{
    bytes::{
        complete::{tag, take_until},
        streaming::take,
    },
    error::ParseError,
    multi::count,
    number::complete::{be_u16, be_u32},
    IResult,
};
use serde::Serialize;

use crate::{
    images::imc::{decode_imc, MonochromeScreen},
    util::{Buf, Bytes16, Bytes32, FourCC},
};

use super::{bytes16, bytes32, Chunk};

#[derive(Debug, Serialize)]
/// The header of a HCIM chunk
pub struct HcimHeader {
    /// The length of the site_table
    pub header_length: u32,
    /// The number of images stored as bitmaps
    pub img_count: u16,
    /// The number of image use-sites within the document
    pub site_count: u16,
    /// Unknown, probably padding
    pub c: Bytes32,
    /// Unknown, probably padding
    pub d: Bytes32,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
/// Information on an image site
///
/// This struct defines what part of an image is used at which position in the document
pub struct ImageSite {
    /// The index of the page that has this image
    pub page: u16,
    /// The site of the image
    pub site: ImageArea,
    /// Unknown
    pub _5: u16,
    /// The selection of the original image that is displayed
    pub sel: ImageArea,
    /// Unknown
    pub _A: u16,
    /// Unknown
    pub _B: u16,
    /// Unknown
    pub _C: u16,
    /// The index of the image that is used
    pub img: u16,
    /// Unknown
    pub _E: u16,
    /// Unknown
    pub _F: Bytes16,
}

#[derive(Debug, Copy, Clone, Serialize)]
/// The area of an image
pub struct ImageArea {
    /// The horizontal position of the left edge
    pub x: u16,
    /// The vertical position of the top edge
    pub y: u16,
    /// The horizontal dimension / width
    pub w: u16,
    /// The vertical dimension / height
    pub h: u16,
}

#[derive(Debug, Serialize)]
/// A partially parsed HCIM
pub struct Hcim<'a> {
    /// The header
    pub header: HcimHeader,
    /// The table of sites
    pub sites: Vec<ImageSite>,
    /// The table of images
    pub images: Vec<Buf<'a>>,
}

/// Parse an entry in the images table
pub fn parse_image_buf<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Buf<'a>, E> {
    let (input, length2) = be_u32(input)?;
    let (input, buf2) = take((length2 - 4) as usize)(input)?;
    Ok((input, Buf(buf2)))
}

#[derive(Debug)]
/// A parsed image
pub struct Image<'a> {
    /// The filename
    pub key: Cow<'a, str>,
    /// The (padding?) bytes after the name
    pub bytes: Buf<'a>,
    /// The uncompressed image
    pub image: MonochromeScreen,
}

const ZERO: &[u8] = &[0];

/// Parse an embedded image file
pub fn parse_image(input: &[u8]) -> IResult<&[u8], Image> {
    let (input, key_bytes) = take_until(ZERO)(input)?;
    let key = String::from_utf8_lossy(key_bytes);

    let (input, _) = tag(ZERO)(input)?;
    let (input, bytes) = take(27usize - key_bytes.len())(input)?;
    let (input, image) = decode_imc(input)?;

    let bytes = Buf(bytes);
    Ok((input, Image { key, bytes, image }))
}

/// Parse the `hcim` header
pub fn parse_hcim_header<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], HcimHeader, E> {
    let (input, header_length) = be_u32(input)?;
    let (input, img_count) = be_u16(input)?;
    let (input, site_count) = be_u16(input)?;
    let (input, c) = bytes32(input)?;
    let (input, d) = bytes32(input)?;

    Ok((
        input,
        HcimHeader {
            header_length,
            img_count,
            site_count,
            c,
            d,
        },
    ))
}

#[allow(non_snake_case, clippy::just_underscores_and_digits)]
/// Parse a site table entry
pub fn parse_hcim_img_ref<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], ImageSite, E> {
    let (input, page) = be_u16(input)?;
    let (input, site_x) = be_u16(input)?;
    let (input, site_y) = be_u16(input)?;
    let (input, site_w) = be_u16(input)?;
    let (input, site_h) = be_u16(input)?;
    let (input, _5) = be_u16(input)?;
    let (input, sel_x) = be_u16(input)?;
    let (input, sel_y) = be_u16(input)?;
    let (input, sel_w) = be_u16(input)?;
    let (input, sel_h) = be_u16(input)?;
    let (input, _A) = be_u16(input)?;
    let (input, _B) = be_u16(input)?;
    let (input, _C) = be_u16(input)?;
    let (input, img) = be_u16(input)?;
    let (input, _E) = be_u16(input)?;
    let (input, _F) = bytes16(input)?;
    Ok((
        input,
        ImageSite {
            page,
            site: ImageArea {
                x: site_x,
                y: site_y,
                w: site_w,
                h: site_h,
            },
            _5,
            sel: ImageArea {
                x: sel_x,
                y: sel_y,
                w: sel_w,
                h: sel_h,
            },
            _A,
            _B,
            _C,
            img,
            _E,
            _F,
        },
    ))
}

/// Parse a `hcim` chunk
pub fn parse_hcim<'a, E: ParseError<&'a [u8]>>(input: &'a [u8]) -> IResult<&'a [u8], Hcim, E> {
    let (input, header) = parse_hcim_header(input)?;
    let (input, buf) = take(header.header_length as usize)(input)?;
    let (_, sites) = count(parse_hcim_img_ref, header.site_count as usize)(buf)?;
    let (input, images) = count(parse_image_buf, header.img_count as usize)(input)?;

    Ok((
        input,
        Hcim {
            header,
            sites,
            images,
        },
    ))
}

impl<'a> Chunk<'a> for Hcim<'a> {
    const TAG: crate::util::FourCC = FourCC::_HCIM;

    fn parse<E>(input: &'a [u8]) -> IResult<&'a [u8], Self, E>
    where
        E: ParseError<&'a [u8]>,
    {
        parse_hcim(input)
    }
}
