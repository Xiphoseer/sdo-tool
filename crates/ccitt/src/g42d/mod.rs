//! CCITT Group 4 2D-encoding

use terminals::{fax_decode_h_black, fax_decode_h_white};

use super::bits::BitIter;

mod decode;
mod encode;
mod terminals;

pub use decode::Decoder;
pub use encode::Encoder;

struct FaxDecode {
    complete: Vec<bool>,
    reference: Vec<bool>,
    current: Vec<bool>,
    width: usize,
    a0: usize,
    ink: bool,
    first: bool,
}

/// Decode a bitmap and print it to the console
///
/// **Note**: This does not use [`Decoder`]!
pub fn fax_decode(glyph_data: &[u8], width: usize) {
    let mut bit_iter = BitIter::new(glyph_data);
    let mut fax_decode = FaxDecode::new(width);

    fax_decode.decode(&mut bit_iter);
    fax_decode.print();
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

    fn decode(&mut self, bit_iter: &mut BitIter) {
        loop {
            let mut done = None;
            while self.a0 <= self.width {
                done = self.next(bit_iter);
                if done == Some(true) {
                    break;
                }
            }
            println!();
            if done == Some(true) {
                break;
            }

            self.a0 = 0;
            self.ink = false;
            self.complete.extend_from_slice(&self.current);
            self.first = true;
            std::mem::swap(&mut self.current, &mut self.reference);
        }
    }

    fn print_border(&self) {
        print!("+");
        for _ in 0..self.width {
            print!("-");
        }
        println!("+");
    }

    fn print(&self) {
        self.print_border();
        for row in self.complete.chunks_exact(self.width) {
            print!("|");
            for bit in row {
                if *bit {
                    print!(" ");
                } else {
                    print!("#");
                }
            }
            println!("|");
        }
        self.print_border();
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
