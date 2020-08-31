use crate::{
    sdoc::{bytes16, bytes32},
    util::{Buf, Bytes16, Bytes32},
};
use nom::{
    bytes::complete::tag,
    error::ErrorKind,
    number::complete::{be_u16, be_u32},
    Err, IResult,
};
use std::{convert::TryInto, slice::Iter};

#[derive(Debug)]
struct IMCHeader {
    u1: Bytes32,
    u2: Bytes32,
    /// ???
    s08: u16,
    /// outer_count
    s0a: u16,
    /// data offset
    s0c: u32,
    //u1: Bytes16,
    u3: Bytes32,
    /// ???
    s14: u16,
    u4: Bytes16,
    u5: Bytes32,
    u6: Bytes32,
}

fn parse_imc_header(input: &[u8]) -> IResult<&[u8], IMCHeader> {
    let (input, u1) = bytes32(input)?;
    let (input, u2) = bytes32(input)?;

    let (input, s08) = be_u16(input)?;
    let (input, s0a) = be_u16(input)?;
    let (input, s0c) = be_u32(input)?;

    let (input, u3) = bytes32(input)?;
    let (input, s14) = be_u16(input)?;
    let (input, u4) = bytes16(input)?;
    let (input, u5) = bytes32(input)?;
    let (input, u6) = bytes32(input)?;

    let header = IMCHeader {
        u1,
        u2,
        s08,
        s0a,
        s0c,
        u3,
        s14,
        u4,
        u5,
        u6,
    };
    Ok((input, header))
}

struct IMCState<'src> {
    d4: u32,
    d5: u8,
    d6: u8,
    d7: u16,
    a5: &'src [u8],
    a6: Iter<'src, u8>,
}

impl<'src> IMCState<'src> {
    /// subprocedure B
    fn next_bit(&mut self) -> bool {
        if self.d5 == 0 {
            self.d5 = 7;
            match self.a6.next() {
                Some(v) => {
                    self.d6 = *v;
                }
                None => {
                    panic!("Could not fetch next bit: {:#?}", Buf(self.a6.as_slice()));
                }
            }
        } else {
            self.d5 -= 1;
        }
        let (n, b) = self.d6.overflowing_add(self.d6);
        self.d6 = n;
        b
    }

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

    fn proc_l(&mut self, d0: u8, mut a0: &mut [u8], s08: u16, s0a: u16) {
        let d1 = s0a << 3;
        for _ in 0..d1 {
            let (mut a1, new_a0) = a0.split_at_mut((self.d7 * 2) as usize);
            for _ in 0..s08 {
                a1[0] ^= d0;
                a1[1] ^= d0;
                a1 = &mut a1[2..];
            }
            a0 = new_a0; // 80 * 2 = 160
        }
    }
}

pub fn parse_imc(input: &[u8]) -> Result<Vec<u8>, Err<(&[u8], ErrorKind)>> {
    let (input, _) = tag(b"bimc0002")(input)?;
    Ok(decode_imc(input))
}

/// Decode a Signum! .IMC image
pub fn decode_imc(src: &[u8]) -> Vec<u8> {
    let mut buffer = vec![0; 32000];
    let mut dest = &mut buffer[..]; // state[A] moving destination address

    let (rest, header) = parse_imc_header(src).unwrap();

    println!("{:#?}", header);

    let d7 = 0x50;
    // this should be sign extend instead of from, but 0x50 is always positive
    let d4 = u32::from(d7) << 4;

    let (bits, data) = rest.split_at(header.s0c as usize);

    let mut state = IMCState {
        d4,
        d5: 0x00,
        d6: 0x08,
        d7,
        a5: data,
        a6: bits.iter(),
    };

    for _ in 0..header.s0a {
        if state.next_bit() {
            // subroutine C
            for j in 0..header.s08 {
                if state.next_bit() {
                    // subroutine D
                    let mut d3 = 0;
                    if state.next_bit() {
                        d3 += 2;
                    }
                    if state.next_bit() {
                        d3 += 1;
                    }

                    print!("{}", d3);

                    let mut temp: [u8; 32];
                    if d3 == 3 {
                        // subroutine E
                        let (a, b) = state.a5.split_at(32);
                        temp = a.try_into().unwrap();
                        state.a5 = b;
                    } else {
                        temp = [0u8; 32]; // subroutine G
                        let mut a3 = &mut temp[..];
                        // first half of temp
                        if state.next_bit() {
                            state.proc_h(a3);
                        }
                        a3 = &mut a3[1..];
                        if state.next_bit() {
                            state.proc_h(a3);
                        }
                        a3 = &mut a3[0xf..];
                        // second half of temp
                        if state.next_bit() {
                            state.proc_h(a3);
                        }
                        a3 = &mut a3[1..];
                        if state.next_bit() {
                            state.proc_h(a3);
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
                        let offset = (j as usize) * 2 + (i * 80);
                        dest[offset] = temp[i * 2];
                        dest[offset + 1] = temp[i * 2 + 1];
                    }
                } else {
                    print!("_");
                }
            }
            println!();
        } else {
            println!("________________________________________");
        }
        dest = &mut dest[(state.d4 as usize)..];
    }

    if header.s14 != 0 {
        // subroutine K
        let [a, b] = header.s14.to_be_bytes();
        state.proc_l(a, &mut dest[..], header.s08, header.s0a);
        state.proc_l(b, &mut dest[80..], header.s08, header.s0a);
    }

    buffer
}
