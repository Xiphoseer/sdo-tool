use nom::{
    bytes::complete::{tag, take},
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::{count, length_count},
    number::complete::{be_i16, be_i32, be_u16, be_u32, le_i16, le_i32, le_u16, le_u32, le_u8},
    sequence::{preceded, tuple},
    IResult,
};

use crate::{
    BitMap, BitWidth, ByteOrder, PCFAccelerators, PCFBDFEncodings, PCFBitmaps, PCFGlyphNames,
    PCFHeader, PCFHeaderEntry, PCFMetricBounds, PCFMetrics, PCFProperties, PCFScalableWidths,
    PCFTableKind, PropVal, TableRef, XChar, XCharMetrics, PCF_ACCEL_W_INKBOUNDS,
    PCF_COMPRESSED_METRICS,
};

pub fn p_pcf_table_kind<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFTableKind, E> {
    map(le_u32, PCFTableKind)(input)
}

pub fn p_pcf_header_entry<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFHeaderEntry, E> {
    map(
        tuple((p_pcf_table_kind, le_u32, le_u32, le_u32)),
        |(kind, format, size, offset)| PCFHeaderEntry {
            kind,
            format,
            pos: TableRef { size, offset },
        },
    )(input)
}

pub fn p_pcf_header<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFHeader, E> {
    map(
        preceded(tag(b"\x01fcp"), length_count(le_u32, p_pcf_header_entry)),
        |tables| PCFHeader { tables },
    )(input)
}

pub fn p_pcf_glpyh_names<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFGlyphNames, E> {
    let (input, format) = le_u32(input)?;
    let p_u32 = if format & (1 << 2) == 0 {
        le_u32
    } else {
        be_u32
    };
    let (input, glyph_count) = p_u32(input)?;
    let (input, offsets) = count(p_u32, glyph_count as usize)(input)?;
    let (input, string_size) = p_u32(input)?;
    let (input, string): (&[u8], &[u8]) = take(string_size)(input)?;
    let mut names = vec![];
    for offset in offsets {
        let slice = &string[(offset as usize)..];
        let name = slice.split(|x| *x == 0).next();
        names.push(String::from_utf8_lossy(name.unwrap()).into_owned());
    }

    Ok((input, PCFGlyphNames { names }))
}

#[derive(Debug)]
struct Prop {
    /// Offset into the following string table
    name_offset: u32,
    /// `value` is an offset if this is true
    is_string_prop: u8,
    /// The value for integer props, the offset for string props
    value: u32,
}

fn p_pcf_prop<'a, E: ParseError<&'a [u8]>, F>(
    p_u32: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], Prop, E>
where
    F: Copy + FnMut(&'a [u8]) -> IResult<&'a [u8], u32, E>,
{
    map(
        tuple((p_u32, le_u8, p_u32)),
        |(name_offset, is_string_prop, value)| Prop {
            name_offset,
            is_string_prop,
            value,
        },
    )
}

pub fn p_pcf_properties<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFProperties, E> {
    let (input, format) = le_u32(input)?;
    let p_u32 = if format & (1 << 2) == 0 {
        le_u32
    } else {
        be_u32
    };
    let (input, nprops) = p_u32(input)?;
    let (input, prop_headers): (&[u8], Vec<Prop>) =
        count(p_pcf_prop(p_u32), nprops as usize)(input)?;

    let padlen = if (nprops & 3) == 0 {
        0
    } else {
        4 - (nprops & 3)
    };
    let (input, _) = take(padlen)(input)?;

    let (input, string_size) = p_u32(input)?;
    let (input, string): (&[u8], &[u8]) = take(string_size)(input)?;
    let mut props = PCFProperties::default();
    for prop in prop_headers {
        let slice = &string[(prop.name_offset as usize)..];
        let name = slice.split(|x| *x == 0).next();
        let name = String::from_utf8_lossy(name.unwrap());

        let value = if prop.is_string_prop != 0 {
            let slice = &string[(prop.value as usize)..];
            let value = slice.split(|x| *x == 0).next();
            let value = String::from_utf8_lossy(value.unwrap()).into_owned();
            PropVal::String(value)
        } else {
            PropVal::Int(prop.value)
        };

        match name.as_ref() {
            "AVERAGE_WIDTH" => props.average_width = value,
            "CAP_HEIGHT" => props.cap_height = value,
            "CHARSET_COLLECTIONS" => props.charset_collections = value,
            "CHARSET_ENCODING" => props.charset_encoding = value,
            "CHARSET_REGISTRY" => props.charset_registry = value,
            "COPYRIGHT" => props.copyright = value,
            "FAMILY_NAME" => props.family_name = value,
            "FOUNDRY" => props.foundry = value,
            "FONT" => props.font = value,
            "FONTNAME_REGISTRY" => props.fontname_registry = value,
            "FULL_NAME" => props.full_name = value,
            "PIXEL_SIZE" => props.pixel_size = value,
            "POINT_SIZE" => props.point_size = value,
            "QUAD_WIDTH" => props.quad_width = value,
            "RESOLUTION" => props.resolution = value,
            "RESOLUTION_X" => props.resolution_x = value,
            "RESOLUTION_Y" => props.resolution_y = value,
            "SETWIDTH_NAME" => props.setwidth_name = value,
            "SLANT" => props.setwidth_name = value,
            "SPACING" => props.spacing = value,
            "WEIGHT" => props.weight = value,
            "WEIGHT_NAME" => props.weight_name = value,
            "X_HEIGHT" => props.x_height = value,
            _ => {
                props.misc.insert(name.into_owned(), value);
            }
        }
    }
    Ok((input, props))
}

pub fn p_pcf_swidths<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFScalableWidths, E> {
    let (input, format) = le_u32(input)?;
    let (p_i32, p_u32): (FnI32<'a, E>, FnU32<'a, E>) = if format & (1 << 2) == 0 {
        (le_i32, le_u32)
    } else {
        (be_i32, be_u32)
    };
    let (input, swidths) = length_count(p_u32, p_i32)(input)?;
    Ok((input, PCFScalableWidths { swidths }))
}

pub fn p_pcf_bdf_encodings<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFBDFEncodings, E> {
    let (input, format) = le_u32(input)?;
    let p_i16 = if format & (1 << 2) == 0 {
        le_i16
    } else {
        be_i16
    };
    let (input, (min_char_or_byte2, max_char_or_byte2, min_byte1, max_byte1, default_char)) =
        tuple((p_i16, p_i16, p_i16, p_i16, p_i16))(input)?;
    let index_count = (max_char_or_byte2 - min_char_or_byte2 + 1) * (max_byte1 - min_byte1 + 1);
    let (input, glyphindeces) = count(p_i16, index_count as usize)(input)?;
    //int16 [];
    //                                /* Gives the glyph index that corresponds to each encoding value */
    //                                /* a value of 0xffff means no glyph for that encoding */
    Ok((
        input,
        PCFBDFEncodings {
            min_char_or_byte2,
            max_char_or_byte2,
            min_byte1,
            max_byte1,
            default_char,
            glyphindeces,
        },
    ))
}

pub fn p_pcf_i16_compressed<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], i16, E> {
    map(le_u8, |val| i16::from(val) - 0x80)(input)
}

pub fn p_pcf_xchar_compressed<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], XCharMetrics, E> {
    map(
        tuple((
            p_pcf_i16_compressed,
            p_pcf_i16_compressed,
            p_pcf_i16_compressed,
            p_pcf_i16_compressed,
            p_pcf_i16_compressed,
        )),
        |(
            left_sided_bearing,
            right_side_bearing,
            character_width,
            character_ascent,
            character_descent,
        )| XCharMetrics {
            left_sided_bearing,
            right_side_bearing,
            character_width,
            character_ascent,
            character_descent,
            character_attributes: 0,
        },
    )(input)
}

pub type FnI16<'a, E> = fn(&'a [u8]) -> IResult<&'a [u8], i16, E>;
pub type FnU16<'a, E> = fn(&'a [u8]) -> IResult<&'a [u8], u16, E>;
pub type FnI32<'a, E> = fn(&'a [u8]) -> IResult<&'a [u8], i32, E>;
pub type FnU32<'a, E> = fn(&'a [u8]) -> IResult<&'a [u8], u32, E>;

pub fn p_pcf_xchar_uncompressed<'a, E: ParseError<&'a [u8]>>(
    p_i16: FnI16<'a, E>,
    p_u16: FnU16<'a, E>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], XCharMetrics, E> {
    move |input: &'a [u8]| {
        map(
            tuple((p_i16, p_i16, p_i16, p_i16, p_i16, p_u16)),
            |(
                left_sided_bearing,
                right_side_bearing,
                character_width,
                character_ascent,
                character_descent,
                character_attributes,
            )| XCharMetrics {
                left_sided_bearing,
                right_side_bearing,
                character_width,
                character_ascent,
                character_descent,
                character_attributes,
            },
        )(input)
    }
}

pub fn p_pcf_accelerators<'a, E: ParseError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], PCFAccelerators, E> {
    let (input, format) = le_u32(input)?;
    let with_inkbounds = format & 0xFFFFFF00 == PCF_ACCEL_W_INKBOUNDS;
    let (p_i16, p_u16, p_i32): (FnI16<'a, E>, FnU16<'a, E>, FnI32<'a, E>) =
        if format & (1 << 2) == 0 {
            (le_i16, le_u16, le_i32)
        } else {
            (be_i16, be_u16, be_i32)
        };

    let (input, no_overlap) = le_u8(input)?;
    let (input, constant_metrics) = le_u8(input)?;
    let (input, terminal_font) = le_u8(input)?;
    let (input, constant_width) = le_u8(input)?;
    let (input, ink_inside) = le_u8(input)?;
    let (input, ink_metrics) = le_u8(input)?;
    let (input, draw_direction) = le_u8(input)?;
    let (input, _padding) = le_u8(input)?;

    let (input, font_ascent) = p_i32(input)?;
    let (input, font_descent) = p_i32(input)?;
    let (input, max_overlap) = p_i32(input)?;

    let (input, minbounds) = p_pcf_xchar_uncompressed(p_i16, p_u16)(input)?;
    let (input, maxbounds) = p_pcf_xchar_uncompressed(p_i16, p_u16)(input)?;

    let (input, ink_bounds) = if with_inkbounds {
        let (input, ink_minbounds) = p_pcf_xchar_uncompressed(p_i16, p_u16)(input)?;
        let (input, ink_maxbounds) = p_pcf_xchar_uncompressed(p_i16, p_u16)(input)?;
        (
            input,
            Some(PCFMetricBounds {
                min: ink_minbounds,
                max: ink_maxbounds,
            }),
        )
    } else {
        (input, None)
    };

    Ok((
        input,
        PCFAccelerators {
            no_overlap,
            constant_metrics,
            terminal_font,
            constant_width,
            ink_inside,
            ink_metrics,
            draw_direction,
            font_ascent,
            font_descent,
            max_overlap,
            bounds: PCFMetricBounds {
                min: minbounds,
                max: maxbounds,
            },
            ink_bounds,
        },
    ))
}

pub fn p_pcf_metrics<'a, E>(input: &'a [u8]) -> IResult<&'a [u8], PCFMetrics, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    let (input, format) = le_u32(input)?;
    let compressed = format & 0xFFFFFF00 == PCF_COMPRESSED_METRICS;
    let (p_i16, p_u16, p_i32): (FnI16<'a, E>, FnU16<'a, E>, FnI32<'a, E>) =
        if format & (1 << 2) == 0 {
            (le_i16, le_u16, le_i32)
        } else {
            (be_i16, be_u16, be_i32)
        };

    let (input, metrics) = if compressed {
        let (input, metrics_count) = p_i16(input)?;
        let (input, metrics) = context(
            "metrics compressed",
            count(p_pcf_xchar_compressed, metrics_count as usize),
        )(input)?;
        (input, metrics)
    } else {
        let (input, metrics_count) = p_i32(input)?;
        let (input, metrics) = context(
            "metrics uncompressed",
            count(
                p_pcf_xchar_uncompressed(p_i16, p_u16),
                metrics_count as usize,
            ),
        )(input)?;
        (input, metrics)
    };

    Ok((input, PCFMetrics { metrics }))
}

pub fn p_pcf_bitmaps<'a, E>(
    glyphs: &'a mut [XChar],
) -> impl FnOnce(&'a [u8]) -> IResult<&'a [u8], PCFBitmaps, E>
where
    E: ParseError<&'a [u8]> + ContextError<&'a [u8]>,
{
    move |input: &'a [u8]| {
        let (input, format) = le_u32(input)?;
        let pad_width = BitWidth::from_bits((format & 0x03) as u8).unwrap();
        let store_width = BitWidth::from_bits(((format & 0x30) >> 4) as u8).unwrap();
        let order = ByteOrder::from_bits(((format & 0b1100) >> 2) as u8);
        let p_i32: FnI32<'a, E> = if format & (1 << 2) == 0 {
            le_i32
        } else {
            be_i32
        };
        let (input, glyph_count) = p_i32(input)?; // should be the same as metric count
        assert_eq!(glyph_count as usize, glyphs.len());
        let (input, mut offset_data) = take((glyph_count << 2) as usize)(input)?;
        //let (input, offsets) = count(p_i32, glyph_count as usize)(input)?; // byte offsets to bitmap data
        let (input, bitmap_sizes): (&[u8], [i32; 4]) =
            map(tuple((p_i32, p_i32, p_i32, p_i32)), |(a, b, c, d)| {
                [a, b, c, d]
            })(input)?;
        let length = bitmap_sizes[(format & 3) as usize];
        let (input, bitmap_data): (&[u8], &[u8]) = take(length as usize)(input)?;

        let clen = match pad_width {
            BitWidth::Bytes => 1,
            BitWidth::Shorts => 2,
            BitWidth::Ints => 4,
        };

        for glyph in glyphs {
            let (input, pos) = p_i32(offset_data)?;
            let ink_width = glyph.metrics.right_side_bearing - glyph.metrics.left_sided_bearing;
            let byte_width = ink_width / 8 + if ink_width % 8 > 0 { 1 } else { 0 };
            let partial_count = byte_width % clen;
            let width = byte_width
                + if partial_count > 0 {
                    clen - partial_count
                } else {
                    0
                };
            let height = glyph.metrics.character_ascent + glyph.metrics.character_descent;
            let start = pos as usize;
            let end = start + (width as usize * height as usize);
            let bytes = &bitmap_data[start..end];

            glyph.bitmap = Some(BitMap(bytes.to_owned(), width as u32));
            offset_data = input;
        }

        Ok((
            input,
            PCFBitmaps {
                order,
                pad_width,
                store_width,
            },
        ))
    }
}
