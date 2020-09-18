use std::{env::args, io::Write};

use pdf::{
    backend::Backend,
    file::Storage,
    file::Trailer,
    object::PlainRef,
    object::{Object, Resolve, Stream},
    primitive::Dictionary,
    primitive::Primitive,
};

#[derive(Clone)]
pub struct BitIter<'a> {
    state: u8,
    buffer: u8,
    inner: std::slice::Iter<'a, u8>,
}

impl<'a> BitIter<'a> {
    pub fn new(bytes: &'a [u8]) -> BitIter<'a> {
        BitIter {
            state: 0,
            buffer: 0,
            inner: bytes.iter(),
        }
    }

    pub fn next_2(&mut self) -> Option<(bool, bool)> {
        let a = self.next()?;
        let b = self.next()?;
        Some((a, b))
    }
}

impl Iterator for BitIter<'_> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == 0 {
            self.state = 7;
            if let Some(value) = self.inner.next() {
                self.buffer = *value;
            } else {
                return None;
            }
        } else {
            self.state -= 1;
        }
        let (next_buffer, carry) = self.buffer.overflowing_mul(2);
        self.buffer = next_buffer;
        Some(carry)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.inner.size_hint().0 * 8 + self.state as usize;
        (size, Some(size))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count() * 8 + self.state as usize
    }
}

fn fax_decode_h_black(bit_iter: &mut BitIter) -> Option<u16> {
    if bit_iter.next()? {
        if bit_iter.next()? {
            Some(2) // 11
        } else {
            Some(3) // 10
        }
    } else if bit_iter.next()? {
        // 01
        if bit_iter.next()? {
            Some(4) // 011
        } else {
            Some(1) // 010
        }
    } else if bit_iter.next()? {
        // 001
        if bit_iter.next()? {
            Some(5) // 0011
        } else {
            Some(6) // 0010
        }
    } else if bit_iter.next()? {
        // 0001
        if bit_iter.next()? {
            Some(7) // 00011
        } else if bit_iter.next()? {
            Some(8) // 000101
        } else {
            Some(9) // 000100
        }
    } else if bit_iter.next()? {
        // 00001
        if bit_iter.next()? {
            // 000011
            if bit_iter.next()? {
                Some(12) // 0000111
            } else if bit_iter.next()? {
                // 00001101
                if bit_iter.next()? {
                    // 000011011
                    if bit_iter.next()? {
                        Some(0) // 0000110111 => 0
                    } else if bit_iter.next()? {
                        if bit_iter.next()? {
                            Some(43) // 000011011011
                        } else {
                            Some(42) // 000011011010
                        }
                    } else {
                        Some(21) // 00001101100
                    }
                } else if bit_iter.next()? {
                    // 0000110101
                    match bit_iter.next_2()? {
                        (true, true) => Some(39),   // 000011010111
                        (true, false) => Some(38),  // 000011010110
                        (false, true) => Some(37),  // 000011010101
                        (false, false) => Some(36), // 000011010100
                    }
                } else if bit_iter.next()? {
                    // 00001101001
                    if bit_iter.next()? {
                        Some(35) // 000011010011
                    } else {
                        Some(34) // 000011010010
                    }
                } else {
                    Some(20) // 00001101000
                }
            } else if bit_iter.next()? {
                // 000011001
                if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(19) // 00001100111
                    } else if bit_iter.next()? {
                        Some(29) // 000011001101
                    } else {
                        Some(28) // 000011001100
                    }
                } else if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(27) // 000011001011
                    } else {
                        Some(26) // 000011001010
                    }
                } else if bit_iter.next()? {
                    Some(192) // 000011001001
                } else {
                    Some(128) // 000011001000
                }
            } else {
                Some(15) // 000011000
            }
        } else if bit_iter.next()? {
            Some(11) // 0000101
        } else {
            Some(10) // 0000100
        }
    } else if bit_iter.next()? {
        // 000001
        if bit_iter.next()? {
            // 0000011
            if bit_iter.next()? {
                Some(14) // 00000111
            } else if bit_iter.next()? {
                // 000001101
                if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(22) // 00000110111
                    } else if bit_iter.next()? {
                        Some(41) // 000001101101
                    } else {
                        Some(40) // 000001101100
                    }
                } else {
                    match bit_iter.next_2()? {
                        (true, true) => Some(33),   // 000001101011
                        (true, false) => Some(32),  // 000001101010
                        (false, true) => Some(31),  // 000001101001
                        (false, false) => Some(30), // 000001101000
                    }
                }
            } else if bit_iter.next()? {
                // 0000011001
                match bit_iter.next_2()? {
                    (true, true) => Some(63),   // 000001100111
                    (true, false) => Some(62),  // 000001100110
                    (false, true) => Some(49),  // 000001100101
                    (false, false) => Some(48), // 000001100100
                }
            } else {
                Some(17) // 0000011000
            }
        } else if bit_iter.next()? {
            // 00000101
            if bit_iter.next()? {
                if bit_iter.next()? {
                    Some(16) // 0000010111
                } else {
                    // 0000010110
                    match bit_iter.next_2()? {
                        (true, true) => Some(256),  // 000001011011
                        (true, false) => Some(61),  // 000001011010
                        (false, true) => Some(58),  // 000001011001
                        (false, false) => Some(57), // 000001011000
                    }
                }
            } else if bit_iter.next()? {
                match bit_iter.next_2()? {
                    (true, true) => Some(47),   // 000001010111
                    (true, false) => Some(46),  // 000001010110
                    (false, true) => Some(45),  // 000001010101
                    (false, false) => Some(44), // 000001010100
                }
            } else if bit_iter.next()? {
                if bit_iter.next()? {
                    Some(51) // 000001010011
                } else {
                    Some(50) // 000001010010
                }
            } else {
                Some(23) // 00000101000
            }
        } else {
            Some(13) // 00000100
        }
    } else if bit_iter.next()? {
        // 0000001
        if bit_iter.next()? {
            // 00000011
            if bit_iter.next()? {
                // 000000111
                if bit_iter.next()? {
                    Some(64) // 0000001111
                } else if bit_iter.next()? {
                    // 00000011101
                    match bit_iter.next_2()? {
                        (true, true) => Some(1216),   // 0000001110111
                        (true, false) => Some(1152),  // 0000001110110
                        (false, true) => Some(1088),  // 0000001110101
                        (false, false) => Some(1024), // 0000001110100
                    }
                } else if bit_iter.next()? {
                    // 000000111001
                    if bit_iter.next()? {
                        Some(960) // 0000001110011
                    } else {
                        Some(896) // 0000001110010
                    }
                } else {
                    Some(54) // 000000111000
                }
            } else if bit_iter.next()? {
                // 0000001101
                if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(53) // 000000110111
                    } else if bit_iter.next()? {
                        Some(576) // 0000001101101
                    } else {
                        Some(512) // 0000001101100
                    }
                } else if bit_iter.next()? {
                    Some(448) // 000000110101
                } else {
                    Some(384) // 000000110100
                }
            } else if bit_iter.next()? {
                // 00000011001
                if bit_iter.next()? {
                    Some(320) // 000000110011
                } else if bit_iter.next()? {
                    Some(1728) // 0000001100101
                } else {
                    Some(1664) // 0000001100100
                }
            } else {
                Some(25) // 00000011000
            }
        } else if bit_iter.next()? {
            // 000000101
            if bit_iter.next()? {
                // 0000001011
                if bit_iter.next()? {
                    Some(24) // 00000010111
                } else if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(1600) // 0000001011011
                    } else {
                        Some(1536) // 0000001011010
                    }
                } else {
                    Some(60) // 000000101100
                }
            } else if bit_iter.next()? {
                // 00000010101
                if bit_iter.next()? {
                    Some(59) // 000000101011
                } else if bit_iter.next()? {
                    Some(1472) // 0000001010101
                } else {
                    Some(1408) // 0000001010100
                }
            } else if bit_iter.next()? {
                // 000000101001
                if bit_iter.next()? {
                    Some(1344) // 0000001010011
                } else {
                    Some(1280) // 0000001010010
                }
            } else {
                Some(56) // 000000101000
            }
        } else if bit_iter.next()? {
            // 0000001001
            if bit_iter.next()? {
                // 00000010011
                if bit_iter.next()? {
                    Some(55) // 000000100111
                } else if bit_iter.next()? {
                    Some(832) // 0000001001101
                } else {
                    Some(768) // 0000001001100
                }
            } else if bit_iter.next()? {
                // 000000100101
                if bit_iter.next()? {
                    Some(704) // 0000001001011
                } else {
                    Some(640) // 0000001001010
                }
            } else {
                Some(52) // 000000100100
            }
        } else {
            Some(18) // 0000001000
        }
    } else {
        fax_decode_h_both(bit_iter)
    }
}

fn fax_decode_h_white(bit_iter: &mut BitIter) -> Option<u16> {
    if bit_iter.next()? {
        // 1..
        if bit_iter.next()? {
            // 11...
            if bit_iter.next()? {
                // 111...
                if bit_iter.next()? {
                    Some(7) // 1111
                } else {
                    Some(6) // 1110
                }
            } else if bit_iter.next()? {
                // 1101...
                if bit_iter.next()? {
                    Some(64) // 11011
                } else if bit_iter.next()? {
                    Some(15) // 110101
                } else {
                    Some(14) // 110100
                }
            } else {
                Some(5) // 1100
            }
        } else if bit_iter.next()? {
            // 101
            if bit_iter.next()? {
                Some(4) // 1011
            } else if bit_iter.next()? {
                // 10101...
                if bit_iter.next()? {
                    Some(17) // 101011
                } else {
                    Some(16) // 101010
                }
            } else {
                Some(9) //10100
            }
        } else if bit_iter.next()? {
            // 1001
            if bit_iter.next()? {
                Some(8) // 10011
            } else {
                Some(128) // 10010
            }
        } else {
            Some(3) // 1000
        }
    } else if bit_iter.next()? {
        // 01
        if bit_iter.next()? {
            // 011
            if bit_iter.next()? {
                Some(2) // 0111
            } else {
                // 0110
                if bit_iter.next()? {
                    // 01101
                    if bit_iter.next()? {
                        // 011011
                        if bit_iter.next()? {
                            Some(256) // 0110111
                        } else {
                            // 0110110..
                            match bit_iter.next_2()? {
                                (false, false) => Some(1216),
                                (false, true) => Some(1280),
                                (true, false) => Some(1344),
                                (true, true) => Some(1408),
                            }
                        }
                    } else if bit_iter.next()? {
                        // 0110101
                        match bit_iter.next_2()? {
                            (false, false) => Some(960),
                            (false, true) => Some(1024),
                            (true, false) => Some(1088),
                            (true, true) => Some(1152),
                        }
                    } else if bit_iter.next()? {
                        // 01101001..
                        if bit_iter.next()? {
                            Some(896) // 011010011
                        } else {
                            Some(832) // 011010010
                        }
                    } else {
                        Some(576) // 01101000
                    }
                } else if bit_iter.next()? {
                    // 011001
                    if bit_iter.next()? {
                        // 0110011
                        if bit_iter.next()? {
                            Some(640) // 01100111
                        } else if bit_iter.next()? {
                            Some(768) // 011001101
                        } else {
                            Some(704) // 011001100
                        }
                    } else if bit_iter.next()? {
                        Some(512) // 01100101
                    } else {
                        Some(448) // 01100100
                    }
                } else {
                    Some(1664) // 011000
                }
            }
        } else if bit_iter.next()? {
            // 0101
            if bit_iter.next()? {
                // 01011
                if bit_iter.next()? {
                    Some(192) // 010111
                } else {
                    // 010110..
                    match bit_iter.next_2()? {
                        (false, false) => Some(55),
                        (false, true) => Some(56),
                        (true, false) => Some(57),
                        (true, true) => Some(58),
                    }
                }
            } else {
                // 01010
                match bit_iter.next_2()? {
                    (false, false) => Some(24), // 0101000
                    (false, true) => {
                        if bit_iter.next()? {
                            Some(50) // 01010011
                        } else {
                            Some(49) // 01010010
                        }
                    }
                    (true, false) => {
                        if bit_iter.next()? {
                            Some(52) // 01010101
                        } else {
                            Some(51) // 01010100
                        }
                    }
                    (true, true) => Some(25), // 0101011
                }
            }
        } else if bit_iter.next()? {
            // 01001
            let a = bit_iter.next()?;
            let b = bit_iter.next()?;
            match (a, b) {
                (false, false) => Some(27), // 0100100
                (false, true) => {
                    if bit_iter.next()? {
                        Some(60) // 01001011
                    } else {
                        Some(59) // 01001010
                    }
                }
                (true, false) => {
                    match bit_iter.next_2()? {
                        (false, false) => Some(1472), // 010011000
                        (false, true) => Some(1536),  // 010011001
                        (true, false) => Some(1600),  // 010011010
                        (true, true) => Some(1728),   // 010011011
                    }
                }
                (true, true) => Some(18), // 0100111
            }
        } else {
            Some(11) // 01000
        }
    } else if bit_iter.next()? {
        if bit_iter.next()? {
            if bit_iter.next()? {
                Some(10) // 00111
            } else if bit_iter.next()? {
                // 001101
                match bit_iter.next_2()? {
                    (false, false) => Some(63), // 00110100
                    (false, true) => Some(0),   // 00110101
                    (true, false) => Some(320), // 00110110
                    (true, true) => Some(384),  // 00110111
                }
            } else if bit_iter.next()? {
                // 0011001
                if bit_iter.next()? {
                    Some(62) // 00110011
                } else {
                    Some(61) // 00110010
                }
            } else {
                Some(28) // 0011000
            }
        } else if bit_iter.next()? {
            // 00101
            if bit_iter.next()? {
                if bit_iter.next()? {
                    Some(21) // 0010111
                } else if bit_iter.next()? {
                    Some(44) // 00101101
                } else {
                    Some(43) // 00101100
                }
            } else {
                // 001010
                match bit_iter.next_2()? {
                    (false, false) => Some(39), // 00101000
                    (false, true) => Some(40),  // 00101001
                    (true, false) => Some(41),  // 00101010
                    (true, true) => Some(42),   // 00101011
                }
            }
        } else if bit_iter.next()? {
            // 001001
            if bit_iter.next()? {
                Some(26) // 0010011
            } else if bit_iter.next()? {
                Some(54) // 00100101
            } else {
                Some(53) // 00100100
            }
        } else {
            Some(12) // 001000
        }
    } else if bit_iter.next()? {
        // 0001
        if bit_iter.next()? {
            if bit_iter.next()? {
                Some(1) // 000111
            } else if bit_iter.next()? {
                // 0001101
                if bit_iter.next()? {
                    Some(32) // 00011011
                } else {
                    Some(31) // 00011010
                }
            } else {
                Some(19) // 0001100
            }
        } else {
            // 00010
            match bit_iter.next_2()? {
                (false, false) => Some(20), // 0001000
                (false, true) => {
                    if bit_iter.next()? {
                        Some(34) // 00010011
                    } else {
                        Some(33) // 00010010
                    }
                }
                (true, false) => {
                    if bit_iter.next()? {
                        Some(36) // 00010101
                    } else {
                        Some(35) // 00010100
                    }
                }
                (true, true) => {
                    if bit_iter.next()? {
                        Some(38) // 00010111
                    } else {
                        Some(37) // 00010110
                    }
                }
            }
        }
    } else if bit_iter.next()? {
        // 00001
        if bit_iter.next()? {
            Some(13) // 000011
        } else if bit_iter.next()? {
            // 0000101
            if bit_iter.next()? {
                Some(48) // 00001011
            } else {
                Some(47) // 00001010
            }
        } else {
            Some(23) // 0000100
        }
    } else if bit_iter.next()? {
        // 000001
        if bit_iter.next()? {
            Some(22) // 0000011
        } else if bit_iter.next()? {
            Some(46) // 00000101
        } else {
            Some(45) // 00000100
        }
    } else if bit_iter.next()? {
        // 0000001
        if bit_iter.next()? {
            Some(30) // 00000011
        } else {
            Some(29) // 00000010
        }
    } else {
        // 0000000
        fax_decode_h_both(bit_iter)
    }
}

fn fax_decode_h_both(_bit_iter: &mut BitIter) -> Option<u16> {
    todo!("")
}

struct FaxDecode {
    complete: Vec<bool>,
    reference: Vec<bool>,
    current: Vec<bool>,
    width: usize,
    a0: usize,
    ink: bool,
    first: bool,
}

fn fax_decode(glyph_data: &[u8], width: usize) {
    let mut bit_iter = BitIter::new(glyph_data);
    let mut fax_decode = FaxDecode::new(width);

    loop {
        let mut done = None;
        while fax_decode.a0 <= fax_decode.width {
            done = fax_decode.next(&mut bit_iter);
            if done == Some(true) {
                break;
            }
        }
        println!();
        if done == Some(true) {
            break;
        }

        fax_decode.a0 = 0;
        fax_decode.ink = false;
        fax_decode.complete.extend_from_slice(&fax_decode.current);
        fax_decode.first = true;
        std::mem::swap(&mut fax_decode.current, &mut fax_decode.reference);
    }

    println!("----------------------------------------------");
    println!();

    for row in fax_decode.complete.chunks_exact(fax_decode.width) {
        for bit in row {
            if *bit {
                print!("_");
            } else {
                print!("#");
            }
        }
        println!();
    }
}

impl FaxDecode {
    fn new(width: usize) -> Self {
        let reference = vec![false; width];
        FaxDecode {
            complete: vec![],
            width,
            reference,
            current: vec![false; width],
            a0: 0,
            ink: false,
            first: true,
        }
    }

    fn vertical(&mut self, new_a0: usize) {
        print!(" [{} v]", new_a0);
        if new_a0 > self.width + 1 {
            println!("ERROR!");
            self.a0 = self.width + 1;
            return;
        }
        for i in (self.a0 + 1)..new_a0 {
            self.current[i - 1] = self.ink;
        }
        self.ink = !self.ink;
        self.a0 = new_a0;
    }

    fn next(&mut self, bit_iter: &mut BitIter) -> Option<bool> {
        print!("[{}]", self.a0);
        let mut ref_ink = if self.a0 == 0 {
            false
        } else {
            self.reference[self.a0 - 1]
        };
        let (b1, b2) = {
            let mut b1 = None;
            let mut b2 = None;
            for i in self.a0..self.width {
                let ink = self.reference[i];
                if ink != ref_ink {
                    // changing element
                    if b1.is_some() {
                        b2 = Some(i + 1);
                        break;
                    }

                    if ink != self.ink {
                        // with a color other than a0
                        b1 = Some(i + 1);
                    }

                    // update color
                    ref_ink = ink;
                }
            }
            let b1 = b1.unwrap_or(self.width + 1);
            let b2 = b2.unwrap_or(self.width + 1);
            (b1, b2)
        };
        print!("({},{})", b1, b2);

        if bit_iter.next().unwrap() {
            print!(" 0 V");
            // 1 --> V(0) --> a_1 just under b_1
            self.vertical(b1);
        } else if bit_iter.next().unwrap() {
            // 01
            if bit_iter.next().unwrap() {
                // 011 --> V_R(1) --> a_1 is 1 right of b_1
                print!(" 1 VR");
                self.vertical(b1 + 1);
            } else {
                // 010 --> V_L(1) --> a_1 is 1 left of b_1
                print!(" 1 VL");
                self.vertical(b1 - 1);
            }
        } else if bit_iter.next().unwrap() {
            // 001 --> horizontal writing mode
            let (a, b) = if self.ink {
                let a = fax_decode_h_black(bit_iter)?;
                let b = fax_decode_h_white(bit_iter)?;
                (a, b)
            } else {
                let a = fax_decode_h_white(bit_iter)?;
                let b = fax_decode_h_black(bit_iter)?;
                (a, b)
            };

            print!(" {} {} H", a, b);
            let start = if self.first { 0 } else { 1 };
            for _ in start..a {
                self.current[self.a0] = self.ink;
                self.a0 += 1;
            }
            for _ in 0..b {
                self.current[self.a0] = !self.ink;
                self.a0 += 1;
            }
            self.a0 += 1;

        } else if bit_iter.next().unwrap() {
            // 0001 -> passtrough
            print!(" P");

            let start = if self.a0 == 0 { 1 } else { self.a0 };
            for i in start..b2 {
                self.current[i - 1] = self.ink;
            }
            self.a0 = b2;
        } else if bit_iter.next().unwrap() {
            // 00001
            if bit_iter.next().unwrap() {
                print!(" 2 VR"); // 000011
                self.vertical(b1 + 2);
            } else {
                print!(" 2 VL"); // 000010
                self.vertical(b1 - 2);
            }
        } else if bit_iter.next().unwrap() {
            // 000001
            if bit_iter.next().unwrap() {
                print!(" 3 VR"); // 0000011
                self.vertical(b1 + 3);
            } else {
                print!(" 3 VL"); // 0000010
                self.vertical(b1 - 3);
            }
        } else if bit_iter.next().unwrap() {
            // 0000001
            panic!("Extension");
        } else {
            // 0000000
            let bi2 = bit_iter.clone();
            let rest: Vec<bool> = bi2.take(17).collect();
            if rest
                == [
                    false, false, false, false, true, false, false, false, false, false, false,
                    false, false, false, false, false, true,
                ]
            {
                return Some(true);
            } else {
                println!("Unknown");
                return Some(true);
            }
        }
        if self.a0 <= self.width {
            self.current[self.a0 - 1] = self.ink;
        }
        self.first = false;
        Some(false)
    }
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|substr| substr == needle)
}

fn main() {
    let path = args().nth(1).expect("Usage: sdo-pdf <FILE>");
    println!("Reading: {}", path);

    let data = std::fs::read(path).expect("could not open file");
    let (xref_tab, trailer) = data.read_xref_table_and_trailer().unwrap();
    let storage = pdf::file::Storage::new(data, xref_tab);

    let trailer: Trailer = Trailer::from_dict(trailer, &storage).expect("Expect `Trailer`");
    println!("{}", trailer.highest_id);

    for id in 0..=(trailer.highest_id as u64) {
        let r = storage.resolve(PlainRef { id, gen: 0 });
        if let Ok(prim) = r {
            println!("{} 0 obj", id);
            if let Primitive::Stream(pdf_stream) = prim {
                println!("Stream({:?})", pdf_stream.info);
                let stream =
                    Stream::<()>::from_stream(pdf_stream, &storage).expect("Expected Stream");
                let decoded = stream.decode().expect("Expected valid stream");
                let bytes = decoded.as_ref();
                if let Some(pos_a) = bytes.windows(3).position(|slice| slice == b"ID ") {
                    if let Some(pos_b) = bytes.windows(3).position(|slice| slice == b"\nEI") {
                        let pos_c = find(bytes, b"/Columns ").expect("Expect Columns") + 9;
                        let pos_d = find(&bytes[pos_c..], b">>").expect("Expect >>") + pos_c;
                        let col_bytes = &bytes[pos_c..pos_d];
                        let col_str = std::str::from_utf8(col_bytes).unwrap();
                        let width = usize::from_str_radix(col_str, 10).unwrap();
                        println!("width: {}", width);

                        let start = pos_a + 3;
                        let glyph_data = &bytes[start..pos_b];
                        println!("offset: {}..{}", start, pos_b);
                        println!("{:?}", glyph_data);

                        print!("byte:");
                        for byte in glyph_data {
                            print!(" {:08b}", *byte);
                        }
                        println!();
                        fax_decode(glyph_data, width);
                    }
                }

                let mut stdout = std::io::stdout();
                println!("```stream");
                stdout.write_all(&decoded).unwrap();
                println!("```");
            } else {
                println!("{:?}", prim);
            }
        }
    }
}

fn _test(trailer: Dictionary, storage: Storage<Vec<u8>>) {
    println!("Trailer");
    let mut root_ref = None;
    let mut info_ref = None;

    for (key, value) in &trailer {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Root" => {
                root_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Root` to be reference"),
                );
            }
            "Info" => {
                info_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expect `Info` to be reference"),
                );
            }
            _ => {}
        }
    }
    let root_ref = root_ref.expect("Expected `Root` in trailer");
    let info_ref = info_ref.expect("Expected `Info` in trailer");
    println!("root_ref: {:?}", root_ref);
    println!("info_ref: {:?}", info_ref);

    let root = storage
        .resolve(root_ref)
        .expect("Expected `Root` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Root` to be a dictionary");
    let info = storage
        .resolve(info_ref)
        .expect("Expected `Info` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Info` to be a dictionary");
    println!("root: {:?}", root);
    println!("info: {:?}", info);

    let mut pages_ref = None;
    let mut metadata_ref = None;
    for (key, value) in &root {
        println!("{}: {}", key, value);
        match key.as_str() {
            "Pages" => {
                pages_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Pages` to be a reference"),
                );
            }
            "Metadata" => {
                metadata_ref = Some(
                    value
                        .clone()
                        .to_reference()
                        .expect("Expected `Metadata` to be a reference"),
                );
            }
            _ => {}
        }
    }

    let pages_ref = pages_ref.expect("Expected `Pages` in `Root`");
    let metadata_ref = metadata_ref.expect("Expected `Metadata` in `Root");
    println!("{:?}", pages_ref);
    println!("{:?}", metadata_ref);

    let pages = storage
        .resolve(pages_ref)
        .expect("Expected `Pages` reference to be valid")
        .to_dictionary(&storage)
        .expect("Expected `Pages` to be a dictionary");

    let metadata = storage
        .resolve(metadata_ref)
        .expect("Expected `Metadata` reference to be valid")
        .to_stream(&storage)
        .expect("Expected `Metadata` to be a dictionary");

    println!("metadata: {:?}", &metadata.info);
    println!(
        "```metadata\n{}\n```",
        std::str::from_utf8(&metadata.data).expect("Expect `Metadata` to be a valid utf-8 stream")
    );
    println!("pages: {:?}", pages);

    let mut pages_kids = None;
    for (key, value) in &pages {
        if key.as_str() == "Kids" {
            pages_kids = Some(
                value
                    .clone()
                    .to_array(&storage)
                    .expect("Expect `Pages`.`Kids` to be an array"),
            );
        }
    }

    let pages_kids = pages_kids.expect("Expect `Pages.Kids` to exist");
    for kid_ref in pages_kids {
        println!("{:?}", kid_ref);
        let kid = kid_ref
            .to_dictionary(&storage)
            .expect("Expect `Kids` entry to be a dictionary");

        println!("{:?}", kid);

        let mut contents_ref = None;
        let mut resources = None;
        for (key, value) in kid.iter() {
            match key.as_str() {
                "Contents" => {
                    contents_ref = Some(
                        value
                            .clone()
                            .to_reference()
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "Resources" => {
                    resources = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let resources = resources.expect("Expected `Resources` in `Page`");
        let contents_ref = contents_ref.expect("Expected `Contents` in `Page`");

        println!("resources: {:?}", resources);

        let mut ext_g_state = None;
        let mut font = None;
        for (key, value) in &resources {
            match key.as_str() {
                "Font" => {
                    font = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expect `Contents` to be a reference"),
                    );
                }
                "ExtGState" => {
                    ext_g_state = Some(
                        value
                            .clone()
                            .to_dictionary(&storage)
                            .expect("Expected `Metadata` to be a reference"),
                    );
                }
                _ => {}
            }
        }

        let ext_g_state = ext_g_state.expect("Expected `Page`.`ExtGState`");
        let font = font.expect("Expected `Page`.`Font`");

        println!("font: {}", font);

        for (key, value_ref) in &font {
            let value = value_ref
                .clone()
                .to_dictionary(&storage)
                .expect("Expect `Font` entry to be dictionary");
            println!("{}: {:#?}", key, value);

            let mut encoding = None;
            let mut to_unicode_ref = None;
            let mut char_procs = None;
            for (key, value) in &value {
                match key.as_str() {
                    "Encoding" => {
                        encoding = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expect `Encoding` to be a dictionary"),
                        );
                    }
                    "ToUnicode" => {
                        to_unicode_ref = Some(
                            value
                                .clone()
                                .to_reference()
                                .expect("Expected `ToUnicode` to be a reference"),
                        );
                    }
                    "CharProcs" => {
                        char_procs = Some(
                            value
                                .clone()
                                .to_dictionary(&storage)
                                .expect("Expected `CharProcs` to be a dictionary"),
                        );
                    }
                    _ => {}
                }
            }

            println!("to_unicode_ref: {:?}", to_unicode_ref);
            println!("char_procs: {:?}", char_procs);
            println!("encoding: {:?}", encoding);
        }

        println!("ext-g-state: {}", ext_g_state);

        println!("contents_ref: {:?}", contents_ref);

        let contents = storage
            .resolve(contents_ref)
            .expect("Expect `Contents` ref to be valid");
        let contents = contents
            .to_stream(&storage)
            .expect("Expected `Contents` to be stream");
        println!("contents.info{:?}", &contents.info);

        let content_stream =
            Stream::<()>::from_stream(contents, &storage).expect("Expect `Contents` to be valid");
        let decoded = content_stream
            .decode()
            .expect("Expect `Contents` decode to work");
        let decoded_text =
            std::str::from_utf8(&decoded).expect("Expect `Contents` to be valid utf-8");
        println!("decoded_text: {}", decoded_text);
    }
}
