//! # The `*.IMC` file format
//!
//! This is an custom ASH format for storing screenshots and images. The magic bytes
//! are `bimc0002`. Its full name likely somewhere between "bitmap in-memory copy"
//! and "black/white image compressed".

use crate::{
    docs::{bytes16, bytes32},
    util::{BitIter, Bytes16, Bytes32},
};
use core::fmt;
use nom::{
    bytes::complete::{tag, take},
    error::ErrorKind,
    number::complete::{be_u16, be_u32},
    Err, IResult,
};
use std::convert::TryInto;

#[allow(dead_code)]
/// IMC metadata
pub struct ImcHeader {
    /// total size
    size: u32,
    /// pixel width
    width: u16,
    /// pixel height
    height: u16,
    /// inner count
    hchunks: u16,
    /// outer_count
    vchunks: u16,
    /// data offset
    size_of_bits: u32,
    /// size of byte-stream
    size_of_data: u32,
    /// Final XOR
    final_xor: Bytes16,
    u4: Bytes16,
    u5: Bytes32,
    u6: Bytes32,
}

impl fmt::Debug for ImcHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImcHeader")
            .field("size", &self.size)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("hchunks", &self.hchunks)
            .field("vchunks", &self.vchunks)
            .field("size_of_bits", &self.size_of_bits)
            .field("size_of_data", &self.size_of_data)
            .field("final_xor", &self.final_xor)
            .finish()
    }
}

fn parse_imc_header(input: &[u8]) -> IResult<&[u8], ImcHeader> {
    let (input, size) = be_u32(input)?;
    let (input, width) = be_u16(input)?;
    let (input, height) = be_u16(input)?;

    let (input, hchunks) = be_u16(input)?;
    let (input, vchunks) = be_u16(input)?;
    let (input, size_of_bits) = be_u32(input)?;
    let (input, size_of_data) = be_u32(input)?;

    let (input, final_xor) = bytes16(input)?;
    let (input, u4) = bytes16(input)?;
    let (input, u5) = bytes32(input)?;
    let (input, u6) = bytes32(input)?;

    let header = ImcHeader {
        size,
        width,
        height,
        hchunks,
        vchunks,
        size_of_bits,
        size_of_data,
        final_xor,
        u4,
        u5,
        u6,
    };
    Ok((input, header))
}

fn load_chunk<'a>(src: &mut std::slice::Iter<'a, u8>, dest: &mut [u8]) -> IResult<&'a [u8], ()> {
    let mut mask = *src.next().ok_or(eof_at(src))?;

    for i in 0..8 {
        let (next_mask, bit_set) = mask.overflowing_mul(2);
        if bit_set {
            dest[2 * i] = *src.next().ok_or(eof_at(src))?;
        }
        mask = next_mask;
    }
    Ok((src.as_slice(), ()))
}

#[derive(Debug)]
/// A fixed size 600x420 pixel screen buffer
pub struct MonochromeScreen(Vec<u8>);

impl MonochromeScreen {
    /// Return the underlying buffer
    ///
    /// The buffer is ordered in scanlines, with one byte representing 8 consecutive pixels.
    ///
    /// Note: 0 means white (no ink) and 1 means black (ink)
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    /// Output the screen as a Portable Bitmap (PBM)
    pub fn write_as_pbm<W: std::fmt::Write>(&self, out: &mut W) -> std::fmt::Result {
        writeln!(out, "P1 640 400")?;
        for line in self.0.chunks(8) {
            for byte in line {
                write!(out, "{:08b}", byte)?;
            }
            writeln!(out)?;
        }
        Ok(())
    }
}

/// Parse a plain IMC file to a screen buffer.
///
/// This method checks for the magic bytes `bimc0002`
pub fn parse_imc(
    input: &[u8],
) -> Result<(ImcHeader, MonochromeScreen), Err<nom::error::Error<&[u8]>>> {
    let (input, _) = tag(b"bimc0002")(input)?;
    let (_input, (header, image)) = decode_imc(input)?;
    Ok((header, image))
}

fn eof_at<'a>(iter: &std::slice::Iter<'a, u8>) -> nom::Err<nom::error::Error<&'a [u8]>> {
    nom::Err::Error(nom::error::Error {
        input: iter.as_slice(),
        code: ErrorKind::Eof,
    })
}

macro_rules! next {
    ($bit_iter:ident, $state:ident) => {
        $bit_iter.next().ok_or(eof_at(&$state))
    };
}

/// Decode a Signum! .IMC image
pub fn decode_imc(src: &[u8]) -> IResult<&[u8], (ImcHeader, MonochromeScreen)> {
    let mut buffer = vec![0; 32000];
    let mut dest = &mut buffer[..]; // state[A] moving destination address

    let (rest, header) = parse_imc_header(src).unwrap();

    // println!("{:#?}", header);

    // Is this really 80, or should this be hchunks * 2?
    let bytes_per_line = header.hchunks * 2;

    // this should be sign extend instead of from, but 0x50 is always positive

    // because one chunk is 32 bytes, and bytes_per_line is hchunks * 2
    // and 16 * 2 = 32, this is probably bytes_per_group
    let bytes_per_group: usize = usize::from(bytes_per_line) * 16;

    let (data, bits) = take(header.size_of_bits as usize)(rest)?;

    let mut bit_iter = BitIter::new(bits);

    let mut byte_iter = data.iter();

    let mut temp: [u8; 32];

    for _ in 0..header.vchunks {
        if next!(bit_iter, byte_iter)? {
            // subroutine C
            for j in 0..header.hchunks {
                if next!(bit_iter, byte_iter)? {
                    // subroutine D
                    let mut d3 = 0;
                    if next!(bit_iter, byte_iter)? {
                        d3 += 2;
                    }
                    if next!(bit_iter, byte_iter)? {
                        d3 += 1;
                    }
                    //print!("{}", d3);

                    if d3 == 3 {
                        // subroutine E
                        let (rest, a) = take(32usize)(byte_iter.as_slice())?;
                        temp = a.try_into().unwrap();
                        byte_iter = rest.iter();
                    } else {
                        temp = [0u8; 32]; // subroutine G
                        let (first, second) = temp.split_at_mut(16);

                        // first half of temp
                        if next!(bit_iter, byte_iter)? {
                            load_chunk(&mut byte_iter, first)?;
                        }
                        if next!(bit_iter, byte_iter)? {
                            load_chunk(&mut byte_iter, &mut first[1..])?;
                        }

                        // second half of temp
                        if next!(bit_iter, byte_iter)? {
                            load_chunk(&mut byte_iter, second)?;
                        }
                        if next!(bit_iter, byte_iter)? {
                            load_chunk(&mut byte_iter, &mut second[1..])?;
                        }

                        if d3 == 1 {
                            // subroutine I
                            let (mut d00, mut d01) = (0, 0);
                            for row in temp.chunks_exact_mut(2) {
                                d00 ^= row[0];
                                d01 ^= row[1];
                                row[0] = d00;
                                row[1] = d01;
                            }
                        } else if d3 == 2 {
                            // subroutine J
                            let (mut d00, mut d01, mut d02, mut d03) = (0, 0, 0, 0);
                            for row in temp.chunks_exact_mut(4) {
                                d00 ^= row[0];
                                d01 ^= row[1];
                                d02 ^= row[2];
                                d03 ^= row[3];

                                row[0] = d00;
                                row[1] = d01;
                                row[2] = d02;
                                row[3] = d03;
                            }
                        }
                    }

                    for i in 0..16 {
                        // subroutine F
                        let offset = (j as usize) * 2 + i * (bytes_per_line as usize);
                        dest[offset] = temp[i * 2];
                        dest[offset + 1] = temp[i * 2 + 1];
                    }
                } else {
                    //print!("_");
                }
            }
        //println!();
        } else {
            // TODO: this assumes that hchunks is always 40
            //println!("________________________________________");
        }
        dest = &mut dest[bytes_per_group..];
    }

    if header.final_xor.0 != 0 {
        let [a, b] = header.final_xor.to_bytes();
        /*// subroutine K

        state.proc_l(a, &mut buffer[..], header.s08, header.s0a);
        state.proc_l(b, &mut buffer[80..], header.s08, header.s0a);*/

        let mut a0 = &mut buffer[..];
        for _ in 0..(header.vchunks * 8) {
            //println!("rem: {}, bytes_per_line: {}", a0.len(), bytes_per_line * 2);
            let (mut a1, new_a0) = a0.split_at_mut(bytes_per_line as usize);
            for _ in 0..header.hchunks {
                a1[0] ^= a;
                a1[1] ^= a;
                a1 = &mut a1[2..];
            }
            let (mut a1, new_a0) = new_a0.split_at_mut(bytes_per_line as usize);
            for _ in 0..header.hchunks {
                a1[0] ^= b;
                a1[1] ^= b;
                a1 = &mut a1[2..];
            }
            a0 = new_a0; // 80 * 2 = 160
        }
    }

    Ok((byte_iter.as_slice(), (header, MonochromeScreen(buffer))))
}

#[cfg(test)]
mod tests {
    const DATA: &[u8] = include_bytes!("../../res/TEST.IMC");

    #[test]
    fn test_decode() {
        super::parse_imc(DATA).unwrap();
    }
}
