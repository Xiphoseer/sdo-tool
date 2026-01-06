//! # Iterator-based decoder
//!
//! Implements CCITT T.6 (Group 4 2-dimensional) decoding based on [`BitIter`]
//! i.e. consuming bits from the input as a sequence of [`bool`] and matching
//! on them.

mod terminals;

use std::io;

use terminals::{black_terminal, fax_decode_h, white_terminal};

use crate::{ascii_art::BorderDrawing, bits::BitIter, g42d::FaxResult, ASCII};

pub struct FaxImage {
    width: usize,
    complete: Vec<bool>,
}

impl FaxImage {
    fn print_border(&self, b: &BorderDrawing) {
        print!("{}", b.left);
        for _ in 0..self.width {
            print!("{}", b.middle);
        }
        println!("{}", b.right);
    }

    pub fn print(&self) {
        let b = ASCII;
        self.print_border(&b.top);
        for row in self.complete.chunks_exact(self.width) {
            print!("{}", b.left);
            for bit in row {
                if *bit {
                    print!("{}", b.ink);
                } else {
                    print!("{}", b.no_ink);
                }
            }
            println!("{}", b.right);
        }
        self.print_border(&b.bottom);
    }

    pub fn write_pbm<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        let height = self.complete.len().div_ceil(self.width);
        writeln!(writer, "P1 {} {}", self.width, height)?;
        for row in self.complete.chunks_exact(self.width) {
            for bit in row {
                let v = if *bit { 0 } else { 1 };
                write!(writer, "{:b}", v)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

pub struct FaxDecode {
    complete: Vec<bool>,
    reference: Vec<bool>,
    current: Vec<bool>,
    width: usize,
    a0: usize,
    ink: bool,
    first: bool,
    debug: bool,
}

impl FaxDecode {
    pub fn new(width: usize) -> Self {
        let reference = vec![false; width];
        FaxDecode {
            complete: vec![],
            width,
            reference,
            current: vec![false; width],
            a0: 0,
            ink: false,
            first: true,
            debug: false,
        }
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn decode(mut self, bit_iter: &mut BitIter) -> FaxResult<FaxImage> {
        loop {
            let mut done = None;
            while self.a0 <= self.width {
                done = self.next(bit_iter);
                if done == Some(true) {
                    break;
                }
            }
            if self.debug {
                println!();
            }
            if done == Some(true) {
                break;
            }

            self.a0 = 0;
            self.ink = false;
            self.complete.extend_from_slice(&self.current);
            self.first = true;
            std::mem::swap(&mut self.current, &mut self.reference);
        }
        Ok(FaxImage {
            width: self.width,
            complete: self.complete,
        })
    }

    fn vertical(&mut self, new_a0: usize) {
        if self.debug {
            print!(" [{} v]", new_a0);
        }
        if new_a0 > self.width + 1 {
            if self.debug {
                println!("ERROR!");
            }
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
        if self.debug {
            print!("[{}]", self.a0);
        }
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
        if self.debug {
            print!("({},{})", b1, b2);
        }

        if bit_iter.next().unwrap() {
            if self.debug {
                print!(" 0 V");
            }
            // 1 --> V(0) --> a_1 just under b_1
            self.vertical(b1);
        } else if bit_iter.next().unwrap() {
            // 01
            if bit_iter.next().unwrap() {
                // 011 --> V_R(1) --> a_1 is 1 right of b_1
                if self.debug {
                    print!(" 1 VR");
                }
                self.vertical(b1 + 1);
            } else {
                // 010 --> V_L(1) --> a_1 is 1 left of b_1
                if self.debug {
                    print!(" 1 VL");
                }
                self.vertical(b1 - 1);
            }
        } else if bit_iter.next().unwrap() {
            // 001 --> horizontal writing mode
            let (a, b) = if self.ink {
                let a = fax_decode_h(bit_iter, black_terminal)?;
                let b = fax_decode_h(bit_iter, white_terminal)?;
                (a, b)
            } else {
                let a = fax_decode_h(bit_iter, white_terminal)?;
                let b = fax_decode_h(bit_iter, black_terminal)?;
                (a, b)
            };

            if self.debug {
                print!(" {} {} H", a, b);
            }
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
            if self.debug {
                print!(" P");
            }

            let start = if self.a0 == 0 { 1 } else { self.a0 };
            for i in start..b2 {
                self.current[i - 1] = self.ink;
            }
            self.a0 = b2;
        } else if bit_iter.next().unwrap() {
            // 00001
            if bit_iter.next().unwrap() {
                if self.debug {
                    print!(" 2 VR"); // 000011
                }
                self.vertical(b1 + 2);
            } else {
                if self.debug {
                    print!(" 2 VL"); // 000010
                }
                self.vertical(b1 - 2);
            }
        } else if bit_iter.next().unwrap() {
            // 000001
            if bit_iter.next().unwrap() {
                if self.debug {
                    print!(" 3 VR"); // 0000011
                }
                self.vertical(b1 + 3);
            } else {
                if self.debug {
                    print!(" 3 VL"); // 0000010
                }
                self.vertical(b1 - 3);
            }
        } else if bit_iter.next().unwrap() {
            // 0000001
            let a = bit_iter.next()?;
            let b = bit_iter.next()?;
            let c = bit_iter.next()?;
            let bit = |v: bool| if v { 1 } else { 0 };
            panic!("Extension {}{}{}", bit(a), bit(b), bit(c));
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
                if self.debug {
                    println!("Unknown");
                }
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
