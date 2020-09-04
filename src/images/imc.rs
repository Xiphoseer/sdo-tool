use super::BitIter;
use crate::{
    sdoc::{bytes16, bytes32},
    util::{Bytes16, Bytes32},
};
use nom::{
    bytes::complete::{tag, take},
    error::ErrorKind,
    number::complete::{be_u16, be_u32},
    Err, IResult,
};
use std::convert::TryInto;

#[derive(Debug)]
struct IMCHeader {
    size: u32,

    width: u16,
    height: u16,
    /// ???
    hchunks: u16,
    /// outer_count
    vchunks: u16,
    /// data offset
    size_of_bits: u32,
    //u1: Bytes16,
    size_of_data: u32,
    /// ???
    s14: u16,
    u4: Bytes16,
    u5: Bytes32,
    u6: Bytes32,
}

fn parse_imc_header(input: &[u8]) -> IResult<&[u8], IMCHeader> {
    let (input, size) = be_u32(input)?;
    let (input, width) = be_u16(input)?;
    let (input, height) = be_u16(input)?;

    let (input, hchunks) = be_u16(input)?;
    let (input, vchunks) = be_u16(input)?;
    let (input, size_of_bits) = be_u32(input)?;
    let (input, size_of_data) = be_u32(input)?;

    let (input, s14) = be_u16(input)?;
    let (input, u4) = bytes16(input)?;
    let (input, u5) = bytes32(input)?;
    let (input, u6) = bytes32(input)?;

    let header = IMCHeader {
        size,
        width,
        height,
        hchunks,
        vchunks,
        size_of_bits,
        size_of_data,
        s14,
        u4,
        u5,
        u6,
    };
    Ok((input, header))
}

struct IMCState<'src> {
    a5: &'src [u8],
}

impl<'src> IMCState<'src> {
    fn proc_h(&mut self, a0: &mut [u8]) {
        let mut d1 = self.a5[0];
        self.a5 = &self.a5[1..];

        for i in 0..8 {
            let (new_d1, carry) = d1.overflowing_mul(2);
            if carry {
                a0[2 * i] = self.a5[0];
                self.a5 = &self.a5[1..];
            }
            d1 = new_d1;
        }
    }
}

#[derive(Debug)]
pub struct MonochromeScreen(Vec<u8>);

impl MonochromeScreen {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

pub fn parse_imc(input: &[u8]) -> Result<MonochromeScreen, Err<(&[u8], ErrorKind)>> {
    let (input, _) = tag(b"bimc0002")(input)?;
    let (_input, image) = decode_imc(input)?;
    Ok(image)
}

macro_rules! next {
    ($bit_iter:ident, $state:ident) => {
        $bit_iter
            .next()
            .ok_or(nom::Err::Error(($state.a5, ErrorKind::Eof)))
    };
}

/// Decode a Signum! .IMC image
pub fn decode_imc(src: &[u8]) -> IResult<&[u8], MonochromeScreen> {
    let mut buffer = vec![0; 32000];
    let mut dest = &mut buffer[..]; // state[A] moving destination address

    let (rest, header) = parse_imc_header(src).unwrap();

    println!("{:#?}", header);

    // Is this really 80, or should this be hchunks * 2?
    let bytes_per_line = header.hchunks * 2;

    // this should be sign extend instead of from, but 0x50 is always positive

    // because one chunk is 32 bytes, and bytes_per_line is hchunks * 2
    // and 16 * 2 = 32, this is probably bytes_per_group
    let bytes_per_group: usize = usize::from(bytes_per_line) * 16;

    let (data, bits) = take(header.size_of_bits as usize)(rest)?;

    let mut bit_iter = BitIter::new(bits);

    let mut state = IMCState { a5: data };

    let mut temp: [u8; 32];

    for _ in 0..header.vchunks {
        if next!(bit_iter, state)? {
            // subroutine C
            for j in 0..header.hchunks {
                if next!(bit_iter, state)? {
                    // subroutine D
                    let mut d3 = 0;
                    if next!(bit_iter, state)? {
                        d3 += 2;
                    }
                    if next!(bit_iter, state)? {
                        d3 += 1;
                    }
                    print!("{}", d3);

                    if d3 == 3 {
                        // subroutine E
                        let (rest, a) = take(32usize)(state.a5)?;
                        temp = a.try_into().unwrap();
                        state.a5 = rest;
                    } else {
                        temp = [0u8; 32]; // subroutine G
                        let (first, second) = temp.split_at_mut(16);

                        // first half of temp
                        if next!(bit_iter, state)? {
                            state.proc_h(first);
                        }
                        if next!(bit_iter, state)? {
                            state.proc_h(&mut first[1..]);
                        }

                        // second half of temp
                        if next!(bit_iter, state)? {
                            state.proc_h(second);
                        }
                        if next!(bit_iter, state)? {
                            state.proc_h(&mut second[1..]);
                        }

                        if d3 == 1 {
                            // subroutine I
                            let a0 = &mut temp[..];
                            let (mut d00, mut d01) = (a0[0], a0[1]);
                            for i in 1..16 {
                                d00 ^= a0[i * 2];
                                d01 ^= a0[i * 2 + 1];

                                a0[i * 2] = d00;
                                a0[i * 2 + 1] = d01;
                            }
                        } else if d3 == 2 {
                            // subroutine J
                            let a0 = &mut temp[..];
                            let (mut d00, mut d01, mut d02, mut d03) = (a0[0], a0[1], a0[2], a0[3]);
                            for i in 1..8 {
                                d00 ^= a0[i * 4];
                                d01 ^= a0[i * 4 + 1];
                                d02 ^= a0[i * 4 + 2];
                                d03 ^= a0[i * 4 + 3];

                                a0[i * 4] = d00;
                                a0[i * 4 + 1] = d01;
                                a0[i * 4 + 2] = d02;
                                a0[i * 4 + 3] = d03;
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
                    print!("_");
                }
            }
            println!();
        } else {
            // TODO: this assumes that hchunks is always 40
            println!("________________________________________");
        }
        dest = &mut dest[bytes_per_group..];
    }

    if header.s14 != 0 {
        let [a, b] = header.s14.to_be_bytes();
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

    Ok((state.a5, MonochromeScreen(buffer)))
}
