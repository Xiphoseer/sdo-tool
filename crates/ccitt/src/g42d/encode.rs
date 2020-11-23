//! Encoder implementation

use crate::bit_iter::{BitIter, BitWriter};
use super::common::Color;

/// The encoder
pub struct Encoder<'a> {
    iter: BitIter<'a>,
    width: usize,
    reference: Vec<Color>,
    current: Vec<Color>,
    output: BitWriter,
    read_color: Color,
    read_pos: usize,
    ref_color: Color,
    ref_pos: usize,
    done: bool,
    /// How many bits of the source image to skip at the start of each line
    pub skip_lead: usize,
    /// How many bits of the source image to skip at the end of each line
    pub skip_tail: usize,
    /// Whether to print debug info
    pub debug: bool,
}

impl<'a> Encoder<'a> {
    /// Create a new encoder for the given input and bit width
    pub fn new(width: usize, input: &'a [u8]) -> Self {
        Self {
            iter: BitIter::new(input),
            width,
            reference: vec![Color::White; width],
            current: vec![Color::White; width],
            output: BitWriter::new(),
            read_color: Color::White,
            read_pos: 0,
            ref_color: Color::White,
            ref_pos: 0,
            done: false,
            skip_lead: 0,
            skip_tail: 0,
            debug: false,
        }
    }

    fn find_next_changing_element(&mut self) -> usize {
        loop {
            if self.read_pos >= self.width {
                break self.width + 1;
            }
            self.read_pos += 1;
            if let Some(next) = self.iter.next() {
                let color = Color::from(next);
                self.current[self.read_pos - 1] = color;
                if color != self.read_color {
                    // found changing element
                    self.read_color = color;
                    break self.read_pos;
                }
            } else {
                self.done = true;
                break self.width + 1;
            }
        }
    }

    fn find_next_changing_ref(&mut self) -> usize {
        loop {
            //print!("(rp/{})", self.ref_pos);
            if self.ref_pos >= self.width {
                //println!("(rp/{})", self.width + 1);
                break self.width + 1;
            }
            let color = self.reference[self.ref_pos];
            /*match color {
                Color::White => print!("[W]"),
                Color::Black => print!("[B]"),
            }*/
            self.ref_pos += 1;
            if color != self.ref_color {
                // found changing element
                self.ref_color = color;
                //println!("(rp/{})", self.ref_pos);
                break self.ref_pos;
            }
        }
    }

    fn find_next_opposite_ref(&mut self, color: Color) -> usize {
        //println!("(c/{:?})", color);
        //println!("(i/{:?})", self.ref_color);
        loop {
            let c = self.find_next_changing_ref();
            //print!("(r/{:?})", self.ref_color);
            //println!("(cp/{})", c);
            if self.ref_color != color || c > self.width {
                break c;
            }
        }
    }

    fn find_b1_b2_at(&mut self, a0: usize, color: Color, b1: &mut usize, b2: &mut usize) {
        self.ref_pos = a0;
        if a0 <= self.width {
            self.ref_color = self.reference[a0 - 1];
            *b1 = self.find_next_opposite_ref(color);
            *b2 = self.find_next_changing_ref();
        }
    }

    /// Encode the bitmap
    pub fn encode(mut self) -> Vec<u8> {
        let mut color = Color::White;

        loop {
            if self.debug {
                let mut cl = self.iter.clone();
                print!("|");
                for _ in 0..self.skip_lead {
                    match cl.next() {
                        Some(true) => print!("#"),
                        Some(false) => print!("_"),
                        None => print!(" "),
                    }
                }
                print!("|");
                for _ in 0..self.width {
                    match cl.next() {
                        Some(true) => print!("#"),
                        Some(false) => print!("_"),
                        None => print!(" "),
                    }
                }
                print!("|");
                for _ in 0..self.skip_tail {
                    match cl.next() {
                        Some(true) => print!("#"),
                        Some(false) => print!("_"),
                        None => print!(" "),
                    }
                }
                println!("|");
            }

            for _ in 0..self.skip_lead {
                self.done |= self.iter.next().is_none();
            }

            let mut a0 = 0;
            let mut a1 = self.find_next_changing_element();

            if self.done {
                break;
            }
            let mut a2 = self.find_next_changing_element();

            let mut b1 = self.find_next_opposite_ref(color);
            let mut b2 = self.find_next_changing_ref();

            loop {
                //println!("a0: {}, a1: {}, a2: {}, b1: {}, b2: {}", a0, a1, a2, b1, b2);

                if a0 == 0 {
                    a0 = 1;
                }

                if b2 < a1 {
                    // pass mode
                    if self.debug {
                        print!("P({})", b2);
                    }
                    self.output.write_bits(0b0001, 4);
                    //println!("\n-----------------------------");
                    a0 = b2;

                    self.find_b1_b2_at(a0, color, &mut b1, &mut b2);
                //println!("a0: {}, b1: {}, b2: {}", a0, b1, b2);
                } else {
                    let d = (a1 as isize) - (b1 as isize);
                    //print!("(d/{})", d);
                    let v = match d {
                        -3 => {
                            if self.debug {
                                print!("VL3");
                            }
                            self.output.write_bits(0b0000010, 7);
                            true
                        }
                        -2 => {
                            if self.debug {
                                print!("VL2");
                            }
                            self.output.write_bits(0b000010, 6);
                            true
                        }
                        -1 => {
                            if self.debug {
                                print!("VL1");
                            }
                            self.output.write_bits(0b010, 3);
                            true
                        }
                        0 => {
                            if self.debug {
                                print!("V0");
                            }
                            self.output.write_bits(0b1, 1);
                            true
                        }
                        1 => {
                            if self.debug {
                                print!("VR1");
                            }
                            self.output.write_bits(0b011, 3);
                            true
                        }
                        2 => {
                            if self.debug {
                                print!("VR2");
                            }
                            self.output.write_bits(0b000011, 6);
                            true
                        }
                        3 => {
                            if self.debug {
                                print!("VR3");
                            }
                            self.output.write_bits(0b0000011, 7);
                            true
                        }
                        _ => false,
                    };

                    if v {
                        //println!("\n-----------------------------");
                        if self.debug {
                            print!("({})", b1);
                        }
                        a0 = a1;
                        a1 = a2;
                        a2 = self.find_next_changing_element();
                        color.invert();
                        self.find_b1_b2_at(a0, color, &mut b1, &mut b2);
                    } else {
                        // horizontal mode
                        let a0a1 = a1 - a0;
                        let a1a2 = a2 - a1;
                        if self.debug {
                            print!("H({},{})", a0a1, a1a2);
                        }
                        self.output.write_bits(0b001, 3);
                        match color {
                            Color::White => {
                                write_white_len(&mut self.output, a0a1);
                                write_black_len(&mut self.output, a1a2);
                            }
                            Color::Black => {
                                write_black_len(&mut self.output, a0a1);
                                write_white_len(&mut self.output, a1a2);
                            }
                        }
                        //println!("\n-----------------------------");

                        a0 = a2;
                        a1 = self.find_next_changing_element();
                        a2 = self.find_next_changing_element();
                        self.find_b1_b2_at(a0, color, &mut b1, &mut b2);
                    }
                }

                //println!();
                //println!("{} {} {} | {} {}", a0, a1, a2, b1, b2);

                if a0 > self.width {
                    if self.debug {
                        println!("#");
                    }
                    break;
                }
            }

            color = Color::White;
            self.read_color = Color::White;
            self.ref_color = Color::White;
            self.read_pos = 0;
            self.ref_pos = 0;
            std::mem::swap(&mut self.reference, &mut self.current);

            for _ in 0..self.skip_tail {
                self.iter.next();
            }

            //Color::_print_row(&self.reference);

            if self.done {
                break;
            }
        }

        self.output.write_bits(0b000000000001000000000001, 24);
        self.output.done()
    }
}

fn write_black_len(output: &mut BitWriter, mut len: usize) {
    while len >= 2560 {
        len -= 2560;
        output.write_bits(0b000000011111, 12);
    }

    match len / 64 {
        0 => {}
        1 => output.write_bits(0b0000001111, 10),
        2 => output.write_bits(0b000011001000, 12),
        3 => output.write_bits(0b000011001001, 12),
        4 => output.write_bits(0b000001011011, 12),
        5 => output.write_bits(0b000000110011, 12),
        6 => output.write_bits(0b000000110100, 12),
        7 => output.write_bits(0b000000110101, 12),
        8 => output.write_bits(0b0000001101100, 13),
        9 => output.write_bits(0b0000001101101, 13),
        10 => output.write_bits(0b0000001001010, 13),
        11 => output.write_bits(0b0000001001011, 13),
        12 => output.write_bits(0b0000001001100, 13),
        13 => output.write_bits(0b0000001001101, 13),
        14 => output.write_bits(0b0000001110010, 13),
        15 => output.write_bits(0b0000001110011, 13),
        16 => output.write_bits(0b0000001110100, 13),
        17 => output.write_bits(0b0000001110101, 13),
        18 => output.write_bits(0b0000001110110, 13),
        19 => output.write_bits(0b0000001110111, 13),
        20 => output.write_bits(0b0000001010010, 13),
        21 => output.write_bits(0b0000001010011, 13),
        22 => output.write_bits(0b0000001010100, 13),
        23 => output.write_bits(0b0000001010101, 13),
        24 => output.write_bits(0b0000001011010, 13),
        25 => output.write_bits(0b0000001011011, 13),
        26 => output.write_bits(0b0000001100100, 13),
        27 => output.write_bits(0b0000001100101, 13),
        28 => output.write_bits(0b00000001000, 11),
        29 => output.write_bits(0b00000001100, 11),
        30 => output.write_bits(0b00000001101, 11),
        31 => output.write_bits(0b000000010010, 12),
        32 => output.write_bits(0b000000010011, 12),
        33 => output.write_bits(0b000000010100, 12),
        34 => output.write_bits(0b000000010101, 12),
        35 => output.write_bits(0b000000010110, 12),
        36 => output.write_bits(0b000000010111, 12),
        37 => output.write_bits(0b000000011100, 12),
        38 => output.write_bits(0b000000011101, 12),
        39 => output.write_bits(0b000000011110, 12),
        _ => unreachable!(),
    }

    match len % 64 {
        0 => output.write_bits(0b0000110111, 10),
        1 => output.write_bits(0b010, 3),
        2 => output.write_bits(0b11, 2),
        3 => output.write_bits(0b10, 2),
        4 => output.write_bits(0b011, 3),
        5 => output.write_bits(0b0011, 4),
        6 => output.write_bits(0b0010, 4),
        7 => output.write_bits(0b00011, 5),
        8 => output.write_bits(0b000101, 6),
        9 => output.write_bits(0b000100, 6),
        10 => output.write_bits(0b0000100, 7),
        11 => output.write_bits(0b0000101, 7),
        12 => output.write_bits(0b0000111, 7),
        13 => output.write_bits(0b00000100, 8),
        14 => output.write_bits(0b00000111, 8),
        15 => output.write_bits(0b000011000, 9),
        16 => output.write_bits(0b0000010111, 10),
        17 => output.write_bits(0b0000011000, 10),
        18 => output.write_bits(0b0000001000, 10),
        19 => output.write_bits(0b00001100111, 11),
        20 => output.write_bits(0b00001101000, 11),
        21 => output.write_bits(0b00001101100, 11),
        22 => output.write_bits(0b00000110111, 11),
        23 => output.write_bits(0b00000101000, 11),
        24 => output.write_bits(0b00000010111, 11),
        25 => output.write_bits(0b00000011000, 11),
        26 => output.write_bits(0b000011001010, 12),
        27 => output.write_bits(0b000011001011, 12),
        28 => output.write_bits(0b000011001100, 12),
        29 => output.write_bits(0b000011001101, 12),
        30 => output.write_bits(0b000001101000, 12),
        31 => output.write_bits(0b000001101001, 12),
        32 => output.write_bits(0b000001101010, 12),
        33 => output.write_bits(0b000001101011, 12),
        34 => output.write_bits(0b000011010010, 12),
        35 => output.write_bits(0b000011010011, 12),
        36 => output.write_bits(0b000011010100, 12),
        37 => output.write_bits(0b000011010101, 12),
        38 => output.write_bits(0b000011010110, 12),
        39 => output.write_bits(0b000011010111, 12),
        40 => output.write_bits(0b000001101100, 12),
        41 => output.write_bits(0b000001101101, 12),
        42 => output.write_bits(0b000011011010, 12),
        43 => output.write_bits(0b000011011011, 12),
        44 => output.write_bits(0b000001010100, 12),
        45 => output.write_bits(0b000001010101, 12),
        46 => output.write_bits(0b000001010110, 12),
        47 => output.write_bits(0b000001010111, 12),
        48 => output.write_bits(0b000001100100, 12),
        49 => output.write_bits(0b000001100101, 12),
        50 => output.write_bits(0b000001010010, 12),
        51 => output.write_bits(0b000001010011, 12),
        52 => output.write_bits(0b000000100100, 12),
        53 => output.write_bits(0b000000110111, 12),
        54 => output.write_bits(0b000000111000, 12),
        55 => output.write_bits(0b000000100111, 12),
        56 => output.write_bits(0b000000101000, 12),
        57 => output.write_bits(0b000001011000, 12),
        58 => output.write_bits(0b000001011001, 12),
        59 => output.write_bits(0b000000101011, 12),
        60 => output.write_bits(0b000000101100, 12),
        61 => output.write_bits(0b000001011010, 12),
        62 => output.write_bits(0b000001100110, 12),
        63 => output.write_bits(0b000001100111, 12),
        _ => unreachable!(),
    }
}

fn write_white_len(output: &mut BitWriter, mut len: usize) {
    while len >= 2560 {
        len -= 2560;
        output.write_bits(0b000000011111, 12);
    }
    match len / 64 {
        0 => {}
        1 => output.write_bits(0b11011, 5),
        2 => output.write_bits(0b10010, 5),
        3 => output.write_bits(0b010111, 6),
        4 => output.write_bits(0b0110111, 7),
        5 => output.write_bits(0b00110110, 8),
        6 => output.write_bits(0b00110111, 8),
        7 => output.write_bits(0b01100100, 8),
        8 => output.write_bits(0b01100101, 8),
        9 => output.write_bits(0b01101000, 8),
        10 => output.write_bits(0b01100111, 8),
        11 => output.write_bits(0b011001100, 9),
        12 => output.write_bits(0b011001101, 9),
        13 => output.write_bits(0b011010010, 9),
        14 => output.write_bits(0b011010011, 9),
        15 => output.write_bits(0b011010100, 9),
        16 => output.write_bits(0b011010101, 9),
        17 => output.write_bits(0b011010110, 9),
        18 => output.write_bits(0b011010111, 9),
        19 => output.write_bits(0b011011000, 9),
        20 => output.write_bits(0b011011001, 9),
        21 => output.write_bits(0b011011010, 9),
        22 => output.write_bits(0b011011011, 9),
        23 => output.write_bits(0b010011000, 9),
        24 => output.write_bits(0b010011001, 9),
        25 => output.write_bits(0b010011010, 9),
        26 => output.write_bits(0b011000, 6),
        27 => output.write_bits(0b010011011, 9),
        28 => output.write_bits(0b00000001000, 11),
        29 => output.write_bits(0b00000001100, 11),
        30 => output.write_bits(0b00000001101, 11),
        31 => output.write_bits(0b000000010010, 12),
        32 => output.write_bits(0b000000010011, 12),
        33 => output.write_bits(0b000000010100, 12),
        34 => output.write_bits(0b000000010101, 12),
        35 => output.write_bits(0b000000010110, 12),
        36 => output.write_bits(0b000000010111, 12),
        37 => output.write_bits(0b000000011100, 12),
        38 => output.write_bits(0b000000011101, 12),
        39 => output.write_bits(0b000000011110, 12),
        _ => unreachable!(),
    }

    match len % 64 {
        0 => output.write_bits(0b00110101, 8),
        1 => output.write_bits(0b000111, 6),
        2 => output.write_bits(0b0111, 4),
        3 => output.write_bits(0b1000, 4),
        4 => output.write_bits(0b1011, 4),
        5 => output.write_bits(0b1100, 4),
        6 => output.write_bits(0b1110, 4),
        7 => output.write_bits(0b1111, 4),
        8 => output.write_bits(0b10011, 5),
        9 => output.write_bits(0b10100, 5),
        10 => output.write_bits(0b00111, 5),
        11 => output.write_bits(0b01000, 5),
        12 => output.write_bits(0b001000, 6),
        13 => output.write_bits(0b000011, 6),
        14 => output.write_bits(0b110100, 6),
        15 => output.write_bits(0b110101, 6),
        16 => output.write_bits(0b101010, 6),
        17 => output.write_bits(0b101011, 6),
        18 => output.write_bits(0b0100111, 7),
        19 => output.write_bits(0b0001100, 7),
        20 => output.write_bits(0b0001000, 7),
        21 => output.write_bits(0b0010111, 7),
        22 => output.write_bits(0b0000011, 7),
        23 => output.write_bits(0b0000100, 7),
        24 => output.write_bits(0b0101000, 7),
        25 => output.write_bits(0b0101011, 7),
        26 => output.write_bits(0b0010011, 7),
        27 => output.write_bits(0b0100100, 7),
        28 => output.write_bits(0b0011000, 7),
        29 => output.write_bits(0b00000010, 8),
        30 => output.write_bits(0b00000011, 8),
        31 => output.write_bits(0b00011010, 8),
        32 => output.write_bits(0b00011011, 8),
        33 => output.write_bits(0b00010010, 8),
        34 => output.write_bits(0b00010011, 8),
        35 => output.write_bits(0b00010100, 8),
        36 => output.write_bits(0b00010101, 8),
        37 => output.write_bits(0b00010110, 8),
        38 => output.write_bits(0b00010111, 8),
        39 => output.write_bits(0b00101000, 8),
        40 => output.write_bits(0b00101001, 8),
        41 => output.write_bits(0b00101010, 8),
        42 => output.write_bits(0b00101011, 8),
        43 => output.write_bits(0b00101100, 8),
        44 => output.write_bits(0b00101101, 8),
        45 => output.write_bits(0b00000100, 8),
        46 => output.write_bits(0b00000101, 8),
        47 => output.write_bits(0b00001010, 8),
        48 => output.write_bits(0b00001011, 8),
        49 => output.write_bits(0b01010010, 8),
        50 => output.write_bits(0b01010011, 8),
        51 => output.write_bits(0b01010100, 8),
        52 => output.write_bits(0b01010101, 8),
        53 => output.write_bits(0b00100100, 8),
        54 => output.write_bits(0b00100101, 8),
        55 => output.write_bits(0b01011000, 8),
        56 => output.write_bits(0b01011001, 8),
        57 => output.write_bits(0b01011010, 8),
        58 => output.write_bits(0b01011011, 8),
        59 => output.write_bits(0b01001010, 8),
        60 => output.write_bits(0b01001011, 8),
        61 => output.write_bits(0b00110010, 8),
        62 => output.write_bits(0b00110011, 8),
        63 => output.write_bits(0b00110100, 8),
        _ => unreachable!(),
    }
}
