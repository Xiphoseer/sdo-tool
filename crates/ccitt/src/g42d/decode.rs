//! Decoder implementation

use crate::bit_iter::BitWriter;
use thiserror::Error;

use super::common::Color;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
enum Bits {
    B = 1,
    B0,
    B1,
    B00,
    B01,
    B10,
    B11,
    B000,
    B001,
    B010,
    B011,
    B100,
    B101,
    B110,
    B111,
}

impl Bits {
    pub fn from_u8(input: u8) -> [Bits; 3] {
        TABLE[input as usize]
    }

    fn off(state: u16, rem: Rem) -> Self {
        let (bit, mask) = match rem {
            Rem::R0 => (0b00000001, 0x0000),
            Rem::R1 => (0b00000010, 0x0001),
            Rem::R2 => (0b00000100, 0x0003),
        };
        unsafe { std::mem::transmute((state & mask) as u8 | bit) }
    }

    fn push(&self, state: &mut u16, off: &mut u8) {
        use Bits::*;
        match self {
            B => {}
            B0 | B1 => {
                *state = (*state << 1) | ((*self as u8) & 0b00000001) as u16;
                *off += 1;
            }
            B00 | B01 | B10 | B11 => {
                *state = (*state << 2) | ((*self as u8) & 0b00000011) as u16;
                *off += 2;
            }
            B000 | B001 | B010 | B011 | B100 | B101 | B110 | B111 => {
                *state = (*state << 3) | ((*self as u8) & 0b00000111) as u16;
                *off += 3;
            }
        }
    }
}

#[rustfmt::skip]
const TABLE: [[Bits; 3]; 256] = {
    use Bits::*;
    [
        [B000, B000, B00], [B000, B000, B01], [B000, B000, B10], [B000, B000, B11],
        [B000, B001, B00], [B000, B001, B01], [B000, B001, B10], [B000, B001, B11],
        [B000, B010, B00], [B000, B010, B01], [B000, B010, B10], [B000, B010, B11],
        [B000, B011, B00], [B000, B011, B01], [B000, B011, B10], [B000, B011, B11],
        [B000, B100, B00], [B000, B100, B01], [B000, B100, B10], [B000, B100, B11],
        [B000, B101, B00], [B000, B101, B01], [B000, B101, B10], [B000, B101, B11],
        [B000, B110, B00], [B000, B110, B01], [B000, B110, B10], [B000, B110, B11],
        [B000, B111, B00], [B000, B111, B01], [B000, B111, B10], [B000, B111, B11],
        [B001, B000, B00], [B001, B000, B01], [B001, B000, B10], [B001, B000, B11],
        [B001, B001, B00], [B001, B001, B01], [B001, B001, B10], [B001, B001, B11],
        [B001, B010, B00], [B001, B010, B01], [B001, B010, B10], [B001, B010, B11],
        [B001, B011, B00], [B001, B011, B01], [B001, B011, B10], [B001, B011, B11],
        [B001, B100, B00], [B001, B100, B01], [B001, B100, B10], [B001, B100, B11],
        [B001, B101, B00], [B001, B101, B01], [B001, B101, B10], [B001, B101, B11],
        [B001, B110, B00], [B001, B110, B01], [B001, B110, B10], [B001, B110, B11],
        [B001, B111, B00], [B001, B111, B01], [B001, B111, B10], [B001, B111, B11],
        [B010, B000, B00], [B010, B000, B01], [B010, B000, B10], [B010, B000, B11],
        [B010, B001, B00], [B010, B001, B01], [B010, B001, B10], [B010, B001, B11],
        [B010, B010, B00], [B010, B010, B01], [B010, B010, B10], [B010, B010, B11],
        [B010, B011, B00], [B010, B011, B01], [B010, B011, B10], [B010, B011, B11],
        [B010, B100, B00], [B010, B100, B01], [B010, B100, B10], [B010, B100, B11],
        [B010, B101, B00], [B010, B101, B01], [B010, B101, B10], [B010, B101, B11],
        [B010, B110, B00], [B010, B110, B01], [B010, B110, B10], [B010, B110, B11],
        [B010, B111, B00], [B010, B111, B01], [B010, B111, B10], [B010, B111, B11],
        [B011, B000, B00], [B011, B000, B01], [B011, B000, B10], [B011, B000, B11],
        [B011, B001, B00], [B011, B001, B01], [B011, B001, B10], [B011, B001, B11],
        [B011, B010, B00], [B011, B010, B01], [B011, B010, B10], [B011, B010, B11],
        [B011, B011, B00], [B011, B011, B01], [B011, B011, B10], [B011, B011, B11],
        [B011, B100, B00], [B011, B100, B01], [B011, B100, B10], [B011, B100, B11],
        [B011, B101, B00], [B011, B101, B01], [B011, B101, B10], [B011, B101, B11],
        [B011, B110, B00], [B011, B110, B01], [B011, B110, B10], [B011, B110, B11],
        [B011, B111, B00], [B011, B111, B01], [B011, B111, B10], [B011, B111, B11],
        [B100, B000, B00], [B100, B000, B01], [B100, B000, B10], [B100, B000, B11],
        [B100, B001, B00], [B100, B001, B01], [B100, B001, B10], [B100, B001, B11],
        [B100, B010, B00], [B100, B010, B01], [B100, B010, B10], [B100, B010, B11],
        [B100, B011, B00], [B100, B011, B01], [B100, B011, B10], [B100, B011, B11],
        [B100, B100, B00], [B100, B100, B01], [B100, B100, B10], [B100, B100, B11],
        [B100, B101, B00], [B100, B101, B01], [B100, B101, B10], [B100, B101, B11],
        [B100, B110, B00], [B100, B110, B01], [B100, B110, B10], [B100, B110, B11],
        [B100, B111, B00], [B100, B111, B01], [B100, B111, B10], [B100, B111, B11],
        [B101, B000, B00], [B101, B000, B01], [B101, B000, B10], [B101, B000, B11],
        [B101, B001, B00], [B101, B001, B01], [B101, B001, B10], [B101, B001, B11],
        [B101, B010, B00], [B101, B010, B01], [B101, B010, B10], [B101, B010, B11],
        [B101, B011, B00], [B101, B011, B01], [B101, B011, B10], [B101, B011, B11],
        [B101, B100, B00], [B101, B100, B01], [B101, B100, B10], [B101, B100, B11],
        [B101, B101, B00], [B101, B101, B01], [B101, B101, B10], [B101, B101, B11],
        [B101, B110, B00], [B101, B110, B01], [B101, B110, B10], [B101, B110, B11],
        [B101, B111, B00], [B101, B111, B01], [B101, B111, B10], [B101, B111, B11],
        [B110, B000, B00], [B110, B000, B01], [B110, B000, B10], [B110, B000, B11],
        [B110, B001, B00], [B110, B001, B01], [B110, B001, B10], [B110, B001, B11],
        [B110, B010, B00], [B110, B010, B01], [B110, B010, B10], [B110, B010, B11],
        [B110, B011, B00], [B110, B011, B01], [B110, B011, B10], [B110, B011, B11],
        [B110, B100, B00], [B110, B100, B01], [B110, B100, B10], [B110, B100, B11],
        [B110, B101, B00], [B110, B101, B01], [B110, B101, B10], [B110, B101, B11],
        [B110, B110, B00], [B110, B110, B01], [B110, B110, B10], [B110, B110, B11],
        [B110, B111, B00], [B110, B111, B01], [B110, B111, B10], [B110, B111, B11],
        [B111, B000, B00], [B111, B000, B01], [B111, B000, B10], [B111, B000, B11],
        [B111, B001, B00], [B111, B001, B01], [B111, B001, B10], [B111, B001, B11],
        [B111, B010, B00], [B111, B010, B01], [B111, B010, B10], [B111, B010, B11],
        [B111, B011, B00], [B111, B011, B01], [B111, B011, B10], [B111, B011, B11],
        [B111, B100, B00], [B111, B100, B01], [B111, B100, B10], [B111, B100, B11],
        [B111, B101, B00], [B111, B101, B01], [B111, B101, B10], [B111, B101, B11],
        [B111, B110, B00], [B111, B110, B01], [B111, B110, B10], [B111, B110, B11],
        [B111, B111, B00], [B111, B111, B01], [B111, B111, B10], [B111, B111, B11],
    ]
};

/// This struct can represents a scanline
pub trait ColorLine {
    /// Get the color at index i
    fn color_at(&self, i: usize) -> Color;
    /// Set the color at index i
    fn set_color(&mut self, i: usize, color: Color);
}

/// This struct can store a bitmap
pub trait Store {
    /// The type of scanline
    type Row: ColorLine;

    /// Create a new struct
    fn new() -> Self;
    /// Create the next row
    fn new_row(width: usize) -> Self::Row;
    /// Add a row to the bitmap
    fn extend(&mut self, row: &Self::Row);
}

impl Store for BitWriter {
    type Row = Vec<Color>;

    fn new() -> Self {
        BitWriter::new()
    }

    fn new_row(width: usize) -> Self::Row {
        vec![Color::White; width]
    }

    fn extend(&mut self, row: &Self::Row) {
        for color in row {
            self.write(*color == Color::Black);
        }
        // TODO: flush here?
    }
}

impl ColorLine for Vec<Color> {
    fn color_at(&self, i: usize) -> Color {
        if i == 0 {
            Color::White
        } else {
            self[i - 1]
        }
    }

    fn set_color(&mut self, i: usize, color: Color) {
        if i == 0 {
            //println!("WARN: trying to assign color to index 0")
        } else {
            self[i - 1] = color;
        }
    }
}

impl Store for Vec<Color> {
    type Row = Vec<Color>;

    fn new() -> Self {
        vec![]
    }

    fn new_row(width: usize) -> Self::Row {
        vec![Color::White; width]
    }

    fn extend(&mut self, row: &Self::Row) {
        self.extend_from_slice(row);
    }
}

#[derive(Debug)]
struct Stack(Cmd, Cmd, Cmd, Cmd);

impl Stack {
    fn peek(&self) -> Cmd {
        self.0
    }

    fn replace(&mut self, cmd: Cmd) -> Cmd {
        std::mem::replace(&mut self.0, cmd)
    }

    fn pop(&mut self) -> Cmd {
        let x = self.0;
        self.0 = self.1;
        self.1 = self.2;
        self.2 = self.3;
        self.3 = Cmd::X;
        x
    }
}

macro_rules! st {
    ($a:expr) => {
        Ok(Stack($a, Cmd::X, Cmd::X, Cmd::X))
    };
    ($a:expr,$b:expr) => {
        Ok(Stack($a, $b, Cmd::X, Cmd::X))
    };
    ($a:expr,$b:expr,$c:expr) => {
        Ok(Stack($a, $b, $c, Cmd::X))
    };
    ($a:expr,$b:expr,$c:expr,$d:expr) => {
        Ok(Stack($a, $b, $c, $d))
    };
}

enum NextBits {
    None,
    A1(Bits),
    A2(Bits, Bits),
}

/// The decoder
pub struct Decoder<S: Store> {
    /// Whether to print debug info
    pub debug: bool,
    stack: Stack,
    next_bits: NextBits,
    store: S,
    width: usize,
    reference: S::Row,
    current: S::Row,
    color: Color,
    a0: usize,
}

impl<S: Store> Decoder<S> {
    /// Create a new decoder instance
    pub fn new(width: usize) -> Self {
        use Cmd::*;
        Decoder {
            stack: Stack(MP(ModePrefix::M), X, X, X),
            next_bits: NextBits::None,
            width,
            store: S::new(),
            reference: S::new_row(width),
            current: S::new_row(width),
            color: Color::White,
            a0: 0,
            debug: false,
        }
    }

    fn next_bits<'a>(&mut self, input: &'a [u8]) -> Result<(&'a [u8], Bits), Err> {
        match self.next_bits {
            NextBits::None => {
                if let Some((first, rest)) = input.split_first() {
                    let [b1, b2, b3] = Bits::from_u8(*first);
                    self.next_bits = NextBits::A2(b2, b3);
                    Ok((rest, b1))
                } else {
                    Err(Err::EOS)
                }
            }
            NextBits::A1(b1) => {
                self.next_bits = NextBits::None;
                Ok((input, b1))
            }
            NextBits::A2(b1, b2) => {
                self.next_bits = NextBits::A1(b2);
                Ok((input, b1))
            }
        }
    }

    fn find_b1(&self) -> usize {
        let mut ref_color = self.reference.color_at(self.a0);
        for i in (self.a0 + 1)..=self.width {
            let i_color = self.reference.color_at(i);
            if i_color != ref_color {
                // changing element
                if i_color != self.color {
                    return i;
                } else {
                    ref_color = i_color;
                }
            }
        }
        self.width + 1
    }

    fn find_b2(&self) -> usize {
        let b1 = self.find_b1();
        if b1 > self.width {
            return b1;
        }
        let ref_color = self.reference.color_at(b1);
        for i in (b1 + 1)..=self.width {
            if self.reference.color_at(i) != ref_color {
                // changing element
                return i;
            }
        }
        self.width + 1
    }

    fn pass_mode(&mut self) {
        let b2 = self.find_b2();
        if self.debug {
            print!("P({})", b2);
        }
        for i in self.a0..b2 {
            self.current.set_color(i, self.color);
        }
        self.a0 = b2;
        self.stack.pop();
    }

    fn vertical_mode(&mut self, a1: usize) -> Result<(), Err> {
        if a1 > self.width + 1 {
            return Err(Err::OutOfBounds(a1));
        }
        let a0 = self.a0.max(1);
        for i in a0..a1 {
            self.current.set_color(i, self.color);
        }
        self.color.invert();
        self.a0 = a1;
        self.stack.pop();
        Ok(())
    }

    fn len<'a>(
        &mut self,
        func: impl Fn(u16, u8) -> Option<(u16, Rem)>,
        bits: Bits,
        mut input: &'a [u8],
    ) -> Result<(&'a [u8], u16, Bits), Err> {
        let mut base = 0;
        let mut off = 0;
        let mut state = 0;
        bits.push(&mut state, &mut off);
        loop {
            if let Some((val, rem)) = func(state, off) {
                if val >= 1792 {
                    // That was a make-up code: store the result and reset the state
                    base += val;
                    rem.reset(&mut state, &mut off);
                } else {
                    let bits = Bits::off(state, rem);
                    return Ok((input, val + base, bits));
                }
            } else {
                let (rest, bits) = self.next_bits(input)?;
                bits.push(&mut state, &mut off);
                input = rest;
            }
        }
    }

    fn horizontal<'a>(&mut self, bits: Bits, input: &'a [u8]) -> Result<(&'a [u8], Bits), Err> {
        let (input, a, b, bits) = match self.color {
            Color::White => {
                let (input, white_len, bits) = self.len(white, bits, input)?;
                let (input, black_len, bits) = self.len(black, bits, input)?;
                (input, white_len, black_len, bits)
            }
            Color::Black => {
                let (input, black_len, bits) = self.len(black, bits, input)?;
                let (input, white_len, bits) = self.len(white, bits, input)?;
                (input, black_len, white_len, bits)
            }
        };
        if self.debug {
            print!("H({},{})", a, b);
        }
        if self.a0 == 0 {
            self.a0 = 1;
        }
        let a0 = self.a0;
        let a1 = a0 + a as usize;
        if a1 > self.width + 1 {
            return Err(Err::OutOfBounds(a1));
        }
        for i in a0..a1 {
            self.current.set_color(i, self.color);
        }
        self.color.invert();
        let a2 = a1 + b as usize;
        if a2 > self.width + 1 {
            return Err(Err::OutOfBounds(a2));
        }
        for i in a1..a2 {
            self.current.set_color(i, self.color);
        }
        self.color.invert();
        self.a0 = a2;

        Ok((input, bits))
    }

    fn eofbp2<'a>(&mut self, input: &'a [u8], k: u8) -> Result<&'a [u8], Err> {
        let (rest, bits) = self.next_bits(input)?;
        let _x: Cmd;
        match (k, bits) {
            (_, Bits::B) => {}
            (0, Bits::B1) => _x = self.stack.replace(Cmd::EOFB),
            (0, Bits::B10) => _x = self.stack.replace(Cmd::EOFB),
            (0, Bits::B100) => _x = self.stack.replace(Cmd::EOFB),

            (1, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP2(0)),
            (1, Bits::B01) => _x = self.stack.replace(Cmd::EOFB),
            (1, Bits::B010) => _x = self.stack.replace(Cmd::EOFB),

            (2, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP2(1)),
            (2, Bits::B00) => _x = self.stack.replace(Cmd::EOFBP2(0)),
            (2, Bits::B001) => _x = self.stack.replace(Cmd::EOFB),

            (_, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP2(k - 1)),
            (_, Bits::B00) => _x = self.stack.replace(Cmd::EOFBP2(k - 2)),
            (_, Bits::B000) => _x = self.stack.replace(Cmd::EOFBP2(k - 3)),
            (_, _) => return Err(Err::EOFB(k)),
        }
        Ok(rest)
    }

    fn eofbp1<'a>(&mut self, input: &'a [u8], k: u8) -> Result<&'a [u8], Err> {
        let (rest, bits) = self.next_bits(input)?;
        let _x: Cmd;
        match (k, bits) {
            (_, Bits::B) => {}
            (0, Bits::B1) => _x = self.stack.replace(Cmd::EOFBP2(11)),
            (0, Bits::B10) => _x = self.stack.replace(Cmd::EOFBP2(10)),
            (0, Bits::B100) => _x = self.stack.replace(Cmd::EOFBP2(9)),
            (0, _) => return Err(Err::EOFB(12)),
            (1, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP1(0)),
            (1, Bits::B01) => _x = self.stack.replace(Cmd::EOFBP2(11)),
            (1, Bits::B010) => _x = self.stack.replace(Cmd::EOFBP2(10)),
            (1, _) => return Err(Err::EOFB(13)),
            (2, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP1(1)),
            (2, Bits::B00) => _x = self.stack.replace(Cmd::EOFBP1(0)),
            (2, Bits::B001) => _x = self.stack.replace(Cmd::EOFBP2(11)),
            (2, _) => return Err(Err::EOFB(14)),
            (_, Bits::B0) => _x = self.stack.replace(Cmd::EOFBP1(k - 1)),
            (_, Bits::B00) => _x = self.stack.replace(Cmd::EOFBP1(k - 2)),
            (_, Bits::B000) => _x = self.stack.replace(Cmd::EOFBP1(k - 3)),
            (_, _) => return Err(Err::EOFB(12 + k)),
        }
        Ok(rest)
    }

    /// Turn the decoder into it's result store
    pub fn into_store(self) -> S {
        self.store
    }

    /// Decode some input
    pub fn decode(&mut self, mut input: &[u8]) -> Result<(), Err> {
        loop {
            let cmd = self.stack.peek();
            match cmd {
                Cmd::X => panic!("CCITTFaxDecode: X"),
                Cmd::MP(mp) => {
                    let (rest, bits) = self.next_bits(input)?;
                    self.stack = mp.next(bits)?;
                    input = rest;
                }
                Cmd::EX(_) => break Err(Err::ExtNotSupported),
                Cmd::EOFBP1(k) => input = self.eofbp1(input, k)?,
                Cmd::EOFBP2(k) => input = self.eofbp2(input, k)?,
                Cmd::EOFB => break Ok(()),
                Cmd::P => {
                    self.pass_mode();
                }
                Cmd::VL3 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VL3({})", b1);
                    }
                    self.vertical_mode(b1 - 3)?;
                }
                Cmd::VL2 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VL2({})", b1);
                    }
                    self.vertical_mode(b1 - 2)?;
                }
                Cmd::VL1 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VL1({})", b1);
                    }
                    self.vertical_mode(b1 - 1)?;
                }
                Cmd::V0 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("V0({})", b1);
                    }
                    self.vertical_mode(b1)?;
                }
                Cmd::VR1 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VR1({})", b1);
                    }
                    self.vertical_mode(b1 + 1)?;
                }
                Cmd::VR2 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VR2({})", b1);
                    }
                    self.vertical_mode(b1 + 2)?;
                }
                Cmd::VR3 => {
                    let b1 = self.find_b1();
                    if self.debug {
                        print!("VR2({})", b1);
                    }
                    self.vertical_mode(b1 + 3)?;
                }
                Cmd::H(bits) => {
                    let (rest, bits) = self.horizontal(bits, input)?;
                    self.stack = ModePrefix::M.next(bits)?;
                    input = rest;
                }
            }
            if self.a0 > self.width {
                // line end
                if self.debug {
                    println!("#");
                }
                self.store.extend(&self.current);
                self.a0 = 0;
                self.color = Color::White;
                std::mem::swap(&mut self.reference, &mut self.current);
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum ModePrefix {
    M,
    M0,
    M00,
    M01,
    M000,
    M0000,
    M00000,
    M00001,
    M000000,
    M000001,
}

/// The error struct
#[allow(clippy::clippy::upper_case_acronyms)]
#[derive(Debug, Error)]
pub enum Err {
    /// Out of bounds ({0})
    #[error("Out of bounds ({0})")]
    OutOfBounds(usize),

    /// End of Stream
    #[error("End of Stream")]
    EOS,

    /// Invalid EOFB, at -{0}
    #[error("Invalid EOFB, at -{0}")]
    EOFB(u8),

    /// Extensions are not supported
    #[error("Extensions are not supported")]
    ExtNotSupported,
}

#[allow(clippy::clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone)]
enum Cmd {
    X,
    MP(ModePrefix),
    EX(Bits),
    /// 1. Prefix of EOFB, needs .0 more `0` bits
    EOFBP1(u8),
    /// 2. Prefix of EOFB, needs .0 more `0` bits
    EOFBP2(u8),
    /// End of Facsimile Block
    EOFB,
    /// Pass Mode
    P,
    /// Vertical Mode; Offset -3
    VL3,
    /// Vertical Mode; Offset -2
    VL2,
    /// Vertical Mode; Offset -1
    VL1,
    /// Vertical Mode; Offset 0
    V0,
    /// Vertical Mode; Offset 1
    VR1,
    /// Vertical Mode; Offset 2
    VR2,
    /// Vertical Mode; Offset 3
    VR3,
    /// Horizontal Mode
    H(Bits),
}

impl ModePrefix {
    fn next(&self, bits: Bits) -> Result<Stack, Err> {
        use {Bits::*, Cmd::*, ModePrefix::*};
        match self {
            M => match bits {
                B => st!(MP(M)),
                B0 => st!(MP(M0)),
                B00 => st!(MP(M00)),
                B000 => st!(MP(M000)),
                B001 => st!(H(B)),
                B01 => st!(MP(M01)),
                B010 => st!(VL1, MP(M)),
                B011 => st!(VR1, MP(M)),
                B1 => st!(V0, MP(M)),
                B10 => st!(V0, MP(M0)),
                B100 => st!(V0, MP(M00)),
                B101 => st!(V0, MP(M01)),
                B11 => st!(V0, V0, MP(M)),
                B110 => st!(V0, V0, MP(M0)),
                B111 => st!(V0, V0, V0, MP(M)),
            },
            M0 => match bits {
                B => st!(MP(M0)),
                B0 => st!(MP(M00)),
                B00 => st!(MP(M000)),
                B000 => st!(MP(M0000)),
                B001 => st!(P, MP(M)),
                B01 => st!(H(B)),
                B010 => st!(H(B0)),
                B011 => st!(H(B1)),
                B1 => st!(MP(M01)),
                B10 => st!(VL1, MP(M)),
                B100 => st!(VL1, MP(M0)),
                B101 => st!(VL1, V0, MP(M)),
                B11 => st!(VR1, MP(M)),
                B110 => st!(VR1, MP(M0)),
                B111 => st!(VR1, V0, MP(M)),
            },
            M00 => match bits {
                B => st!(MP(M00)),
                B0 => st!(MP(M000)),
                B00 => st!(MP(M0000)),
                B000 => st!(MP(M00000)),
                B001 => st!(MP(M00001)),
                B01 => st!(P, MP(M)),
                B010 => st!(P, MP(M0)),
                B011 => st!(P, V0, MP(M)),
                B1 => st!(H(B)),
                B10 => st!(H(B0)),
                B100 => st!(H(B00)),
                B101 => st!(H(B01)),
                B11 => st!(H(B1)),
                B110 => st!(H(B10)),
                B111 => st!(H(B11)),
            },
            M01 => match bits {
                B => st!(MP(M01)),
                B0 => st!(VL1, MP(M)),
                B00 => st!(VL1, MP(M0)),
                B000 => st!(VL1, MP(M00)),
                B001 => st!(VL1, MP(M01)),
                B01 => st!(VL1, V0, MP(M)),
                B010 => st!(VL1, V0, MP(M0)),
                B011 => st!(VL1, V0, V0, MP(M)),
                B1 => st!(VR1, MP(M)),
                B10 => st!(VR1, MP(M0)),
                B100 => st!(VR1, MP(M00)),
                B101 => st!(VR1, MP(M01)),
                B11 => st!(VR1, V0, MP(M)),
                B110 => st!(VR1, V0, MP(M0)),
                B111 => st!(VR1, V0, V0, MP(M)),
            },
            M000 => match bits {
                B => st!(MP(M000)),
                B0 => st!(MP(M0000)),
                B00 => st!(MP(M00000)),
                B000 => st!(MP(M000000)),
                B001 => st!(MP(M000001)),
                B01 => st!(MP(M00001)),
                B010 => st!(VL2, MP(M)),
                B011 => st!(VR2, MP(M)),
                B1 => st!(P, MP(M)),
                B10 => st!(P, MP(M0)),
                B100 => st!(P, MP(M00)),
                B101 => st!(P, MP(M01)),
                B11 => st!(P, V0, MP(M)),
                B110 => st!(P, V0, MP(M0)),
                B111 => st!(P, V0, V0, MP(M)),
            },
            M0000 => match bits {
                B => st!(MP(M0000)),
                B0 => st!(MP(M00000)),
                B00 => st!(MP(M000000)),
                B000 => st!(EOFBP1(4)),
                B001 => st!(EX(B)),
                B01 => st!(MP(M000001)),
                B010 => st!(VL3, MP(M)),
                B011 => st!(VR3, MP(M)),
                B1 => st!(MP(M00001)),
                B10 => st!(VL2, MP(M)),
                B100 => st!(VL2, MP(M0)),
                B101 => st!(VL2, V0, MP(M)),
                B11 => st!(VR2, MP(M)),
                B110 => st!(VR2, MP(M0)),
                B111 => st!(VR2, V0, MP(M)),
            },
            M00000 => match bits {
                B => st!(MP(M00000)),
                B0 => st!(MP(M000000)),
                B00 => st!(EOFBP1(4)),
                B000 => st!(EOFBP1(3)),
                B001 => Err(Err::EOFB(17)),
                B01 => st!(EX(B)),
                B010 => st!(EX(B0)),
                B011 => st!(EX(B1)),
                B1 => st!(MP(M000001)),
                B10 => st!(VL3, MP(M)),
                B100 => st!(VL3, MP(M0)),
                B101 => st!(VL3, V0, MP(M)),
                B11 => st!(VR3, MP(M)),
                B110 => st!(VR3, MP(M0)),
                B111 => st!(VR3, V0, MP(M)),
            },
            M00001 => match bits {
                B => st!(MP(M00001)),
                B0 => st!(VL2, MP(M)),
                B00 => st!(VL2, MP(M0)),
                B000 => st!(VL2, MP(M00)),
                B001 => st!(VL2, MP(M01)),
                B01 => st!(VL2, V0, MP(M)),
                B010 => st!(VL2, V0, MP(M0)),
                B011 => st!(VL2, V0, V0, MP(M)),
                B1 => st!(VR2, MP(M)),
                B10 => st!(VR2, MP(M0)),
                B100 => st!(VR2, MP(M00)),
                B101 => st!(VR2, MP(M01)),
                B11 => st!(VR2, V0, MP(M)),
                B110 => st!(VR2, V0, MP(M0)),
                B111 => st!(VR2, V0, V0, MP(M)),
            },
            M000000 => match bits {
                B => st!(MP(M000000)),
                B0 => st!(EOFBP1(4)),
                B00 => st!(EOFBP1(3)),
                B000 => st!(EOFBP1(2)),
                B001 => Err(Err::EOFB(16)),
                B01 | B010 | B011 => Err(Err::EOFB(17)),
                B1 => st!(EX(B)),
                B10 => st!(EX(B0)),
                B100 => st!(EX(B00)),
                B101 => st!(EX(B01)),
                B11 => st!(EX(B1)),
                B110 => st!(EX(B10)),
                B111 => st!(EX(B11)),
            },
            M000001 => match bits {
                B => st!(MP(M000001)),
                B0 => st!(VL3, MP(M)),
                B00 => st!(VL3, MP(M0)),
                B000 => st!(VL3, MP(M00)),
                B001 => st!(VL3, MP(M01)),
                B01 => st!(VL3, V0, MP(M)),
                B010 => st!(VL3, V0, MP(M0)),
                B011 => st!(VL3, V0, V0, MP(M)),
                B1 => st!(VR3, MP(M)),
                B10 => st!(VR3, MP(M0)),
                B100 => st!(VR3, MP(M00)),
                B101 => st!(VR3, MP(M01)),
                B11 => st!(VR3, V0, MP(M)),
                B110 => st!(VR3, V0, MP(M0)),
                B111 => st!(VR3, V0, V0, MP(M)),
            },
        }
    }
}

#[derive(Debug)]
enum Rem {
    R0,
    R1,
    R2,
}

impl Rem {
    fn reset(&self, state: &mut u16, off: &mut u8) {
        match self {
            Rem::R0 => {
                *state = 0;
                *off = 0;
            }
            Rem::R1 => {
                *state &= 0b1;
                *off = 1;
            }
            Rem::R2 => {
                *state &= 0b11;
                *off = 2;
            }
        }
    }
}

fn black(state: u16, off: u8) -> Option<(u16, Rem)> {
    match off {
        0 | 1 => None,
        2 => black2(state),
        3 => black3(state),
        4 => black4(state),
        5 => black5(state),
        6 => black6(state),
        7 => black7(state),
        8 => black8(state),
        9 => black9(state),
        10 => black10(state),
        11 => black11(state),
        12 => black12(state),
        13 => black13(state),
        14 => black14(state),
        15 => black15(state),
        _ => todo!(),
    }
}

fn black2(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 2 bit
        0b10 => Some((3, Rem::R0)),
        0b11 => Some((2, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black3(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 2 bit
        0b100 | 0b101 => Some((3, Rem::R1)),
        0b110 | 0b111 => Some((2, Rem::R1)),
        // 3 bit
        0b010 => Some((1, Rem::R0)),
        0b011 => Some((4, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black4(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 2 bit
        0b1000..=0b1011 => Some((3, Rem::R2)),
        0b1100..=0b1111 => Some((2, Rem::R2)),
        // 3 bit
        0b0100 | 0b0101 => Some((1, Rem::R1)),
        0b0110 | 0b0111 => Some((4, Rem::R1)),
        // 4 bit
        0b0010 => Some((6, Rem::R0)),
        0b0011 => Some((5, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black5(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 3 bit
        0b01000..=0b01011 => Some((1, Rem::R2)),
        0b01100..=0b01111 => Some((4, Rem::R2)),
        // 4 bit
        0b00100 | 0b00101 => Some((6, Rem::R1)),
        0b00110 | 0b00111 => Some((5, Rem::R1)),
        // 5 bit
        0b00011 => Some((7, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black6(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 4 bit
        0b001000..=0b001011 => Some((6, Rem::R2)),
        0b001100..=0b001111 => Some((5, Rem::R2)),
        // 5 bit
        0b000110 | 0b000111 => Some((7, Rem::R1)),
        // 6 bit
        0b000100 => Some((9, Rem::R0)),
        0b000101 => Some((8, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black7(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 5 bit
        0b0001100..=0b0001111 => Some((7, Rem::R2)),
        // 6 bit
        0b0001000 | 0b0001001 => Some((9, Rem::R1)),
        0b0001010 | 0b0001011 => Some((8, Rem::R1)),
        // 7 bit
        0b0000100 => Some((10, Rem::R0)),
        0b0000101 => Some((11, Rem::R0)),
        0b0000111 => Some((12, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black8(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 6 bit
        0b00010000..=0b00010011 => Some((9, Rem::R2)),
        0b00010100..=0b00010111 => Some((8, Rem::R2)),
        // 7 bit
        0b00001000 | 0b00001001 => Some((10, Rem::R1)),
        0b00001010 | 0b00001011 => Some((11, Rem::R1)),
        0b00001110 | 0b00001111 => Some((12, Rem::R1)),
        // 8 bit
        0b00000100 => Some((13, Rem::R0)),
        0b00000111 => Some((14, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black9(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 7 bit
        0b000010000..=0b000010011 => Some((10, Rem::R2)),
        0b000010100..=0b000010111 => Some((11, Rem::R2)),
        0b000011100..=0b000011111 => Some((12, Rem::R2)),
        // 8 bit
        0b000001000 | 0b000001001 => Some((13, Rem::R1)),
        0b000001110 | 0b000001111 => Some((14, Rem::R1)),
        // 9 bit
        0b000011000 => Some((15, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black10(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 8 bit
        0b0000010000..=0b0000010011 => Some((13, Rem::R2)),
        0b0000011100..=0b0000011111 => Some((14, Rem::R2)),
        // 9 bit
        0b0000110000 | 0b0000110001 => Some((15, Rem::R1)),
        // 10 bit
        0b0000001000 => Some((18, Rem::R0)),
        0b0000001111 => Some((64, Rem::R0)),
        0b0000010111 => Some((16, Rem::R0)),
        0b0000011000 => Some((17, Rem::R0)),
        0b0000110111 => Some((0, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black11(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 9 bit
        0b00001100000..=0b00001100011 => Some((15, Rem::R2)),
        // 10 bit
        0b00000010000 | 0b00000010001 => Some((18, Rem::R1)),
        0b00000011110 | 0b00000011111 => Some((64, Rem::R1)),
        0b00000101110 | 0b00000101111 => Some((16, Rem::R1)),
        0b00000110000 | 0b00000110001 => Some((17, Rem::R1)),
        0b00001101110 | 0b00001101111 => Some((0, Rem::R1)),
        // 11 bit
        0b00000001000 => Some((1792, Rem::R0)),
        0b00000001100 => Some((1856, Rem::R0)),
        0b00000001101 => Some((1920, Rem::R0)),
        0b00000010111 => Some((24, Rem::R0)),
        0b00000011000 => Some((25, Rem::R0)),
        0b00000101000 => Some((23, Rem::R0)),
        0b00000110111 => Some((22, Rem::R0)),
        0b00001100111 => Some((19, Rem::R0)),
        0b00001101000 => Some((20, Rem::R0)),
        0b00001101100 => Some((21, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black12(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 10 bit
        0b000000100000..=0b000000100011 => Some((18, Rem::R2)),
        0b000000111100..=0b000000111111 => Some((64, Rem::R2)),
        0b000001011100..=0b000001011111 => Some((16, Rem::R2)),
        0b000001100000..=0b000001100011 => Some((17, Rem::R2)),
        0b000011011100..=0b000011011111 => Some((0, Rem::R2)),
        // 11 bit
        0b000000010000 | 0b000000010001 => Some((1792, Rem::R1)),
        0b000000011000 | 0b000000011001 => Some((1856, Rem::R1)),
        0b000000011010 | 0b000000011011 => Some((1920, Rem::R1)),
        0b000000101110 | 0b000000101111 => Some((24, Rem::R1)),
        0b000000110000 | 0b000000110001 => Some((25, Rem::R1)),
        0b000001010000 | 0b000001010001 => Some((23, Rem::R1)),
        0b000001101110 | 0b000001101111 => Some((22, Rem::R1)),
        0b000011001110 | 0b000011001111 => Some((19, Rem::R1)),
        0b000011010000 | 0b000011010001 => Some((20, Rem::R1)),
        0b000011011000 | 0b000011011001 => Some((21, Rem::R1)),
        // 12 bit
        0b000000010010 => Some((1984, Rem::R0)),
        0b000000010011 => Some((2048, Rem::R0)),
        0b000000010100 => Some((2112, Rem::R0)),
        0b000000010101 => Some((2176, Rem::R0)),
        0b000000010110 => Some((2240, Rem::R0)),
        0b000000010111 => Some((2304, Rem::R0)),
        0b000000011100 => Some((2368, Rem::R0)),
        0b000000011101 => Some((2432, Rem::R0)),
        0b000000011110 => Some((2496, Rem::R0)),
        0b000000011111 => Some((2560, Rem::R0)),
        0b000000100100 => Some((52, Rem::R0)),
        0b000000100111 => Some((55, Rem::R0)),
        0b000000101000 => Some((56, Rem::R0)),
        0b000000101011 => Some((59, Rem::R0)),
        0b000000101100 => Some((60, Rem::R0)),
        0b000000110011 => Some((320, Rem::R0)),
        0b000000110100 => Some((384, Rem::R0)),
        0b000000110101 => Some((448, Rem::R0)),
        0b000000110111 => Some((53, Rem::R0)),
        0b000000111000 => Some((54, Rem::R0)),
        0b000001010010 => Some((50, Rem::R0)),
        0b000001010011 => Some((51, Rem::R0)),
        0b000001010100 => Some((44, Rem::R0)),
        0b000001010101 => Some((45, Rem::R0)),
        0b000001010110 => Some((46, Rem::R0)),
        0b000001010111 => Some((47, Rem::R0)),
        0b000001011000 => Some((57, Rem::R0)),
        0b000001011001 => Some((58, Rem::R0)),
        0b000001011010 => Some((61, Rem::R0)),
        0b000001011011 => Some((256, Rem::R0)),
        0b000001100100 => Some((48, Rem::R0)),
        0b000001100101 => Some((49, Rem::R0)),
        0b000001100110 => Some((62, Rem::R0)),
        0b000001100111 => Some((63, Rem::R0)),
        0b000001101000 => Some((30, Rem::R0)),
        0b000001101001 => Some((31, Rem::R0)),
        0b000001101010 => Some((32, Rem::R0)),
        0b000001101011 => Some((33, Rem::R0)),
        0b000001101100 => Some((40, Rem::R0)),
        0b000001101101 => Some((41, Rem::R0)),
        0b000011001000 => Some((128, Rem::R0)),
        0b000011001001 => Some((192, Rem::R0)),
        0b000011001010 => Some((26, Rem::R0)),
        0b000011001011 => Some((27, Rem::R0)),
        0b000011001100 => Some((28, Rem::R0)),
        0b000011001101 => Some((29, Rem::R0)),
        0b000011010010 => Some((34, Rem::R0)),
        0b000011010011 => Some((35, Rem::R0)),
        0b000011010100 => Some((36, Rem::R0)),
        0b000011010101 => Some((37, Rem::R0)),
        0b000011010110 => Some((38, Rem::R0)),
        0b000011010111 => Some((39, Rem::R0)),
        0b000011011010 => Some((42, Rem::R0)),
        0b000011011011 => Some((43, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black13(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 11 bit
        0b0000000100000..=0b0000000100011 => Some((1792, Rem::R2)),
        0b0000000110000..=0b0000000110011 => Some((1856, Rem::R2)),
        0b0000000110100..=0b0000000110111 => Some((1920, Rem::R2)),
        0b0000001011100..=0b0000001011111 => Some((24, Rem::R2)),
        0b0000001100000..=0b0000001100011 => Some((25, Rem::R2)),
        0b0000010100000..=0b0000010100011 => Some((23, Rem::R2)),
        0b0000011011100..=0b0000011011111 => Some((22, Rem::R2)),
        0b0000110011100..=0b0000110011111 => Some((19, Rem::R2)),
        0b0000110100000..=0b0000110100011 => Some((20, Rem::R2)),
        0b0000110110000..=0b0000110110011 => Some((21, Rem::R2)),
        // 12 bit
        0b0000000100100 | 0b0000000100101 => Some((1984, Rem::R1)),
        0b0000000100110 | 0b0000000100111 => Some((2048, Rem::R1)),
        0b0000000101000 | 0b0000000101001 => Some((2112, Rem::R1)),
        0b0000000101010 | 0b0000000101011 => Some((2176, Rem::R1)),
        0b0000000101100 | 0b0000000101101 => Some((2240, Rem::R1)),
        0b0000000101110 | 0b0000000101111 => Some((2304, Rem::R1)),
        0b0000000111000 | 0b0000000111001 => Some((2368, Rem::R1)),
        0b0000000111010 | 0b0000000111011 => Some((2432, Rem::R1)),
        0b0000000111100 | 0b0000000111101 => Some((2496, Rem::R1)),
        0b0000000111110 | 0b0000000111111 => Some((2560, Rem::R1)),
        0b0000001001000 | 0b0000001001001 => Some((52, Rem::R1)),
        0b0000001001110 | 0b0000001001111 => Some((55, Rem::R1)),
        0b0000001010000 | 0b0000001010001 => Some((56, Rem::R1)),
        0b0000001010110 | 0b0000001010111 => Some((59, Rem::R1)),
        0b0000001011000 | 0b0000001011001 => Some((60, Rem::R1)),
        0b0000001100110 | 0b0000001100111 => Some((320, Rem::R1)),
        0b0000001101000 | 0b0000001101001 => Some((384, Rem::R1)),
        0b0000001101010 | 0b0000001101011 => Some((448, Rem::R1)),
        0b0000001101110 | 0b0000001101111 => Some((53, Rem::R1)),
        0b0000001110000 | 0b0000001110001 => Some((54, Rem::R1)),
        0b0000010100100 | 0b0000010100101 => Some((50, Rem::R1)),
        0b0000010100110 | 0b0000010100111 => Some((51, Rem::R1)),
        0b0000010101000 | 0b0000010101001 => Some((44, Rem::R1)),
        0b0000010101010 | 0b0000010101011 => Some((45, Rem::R1)),
        0b0000010101100 | 0b0000010101101 => Some((46, Rem::R1)),
        0b0000010101110 | 0b0000010101111 => Some((47, Rem::R1)),
        0b0000010110000 | 0b0000010110001 => Some((57, Rem::R1)),
        0b0000010110010 | 0b0000010110011 => Some((58, Rem::R1)),
        0b0000010110100 | 0b0000010110101 => Some((61, Rem::R1)),
        0b0000010110110 | 0b0000010110111 => Some((256, Rem::R1)),
        0b0000011001000 | 0b0000011001001 => Some((48, Rem::R1)),
        0b0000011001010 | 0b0000011001011 => Some((49, Rem::R1)),
        0b0000011001100 | 0b0000011001101 => Some((62, Rem::R1)),
        0b0000011001110 | 0b0000011001111 => Some((63, Rem::R1)),
        0b0000011010000 | 0b0000011010001 => Some((30, Rem::R1)),
        0b0000011010010 | 0b0000011010011 => Some((31, Rem::R1)),
        0b0000011010100 | 0b0000011010101 => Some((32, Rem::R1)),
        0b0000011010110 | 0b0000011010111 => Some((33, Rem::R1)),
        0b0000011011000 | 0b0000011011001 => Some((40, Rem::R1)),
        0b0000011011010 | 0b0000011011011 => Some((41, Rem::R1)),
        0b0000110010000 | 0b0000110010001 => Some((128, Rem::R1)),
        0b0000110010010 | 0b0000110010011 => Some((192, Rem::R1)),
        0b0000110010100 | 0b0000110010101 => Some((26, Rem::R1)),
        0b0000110010110 | 0b0000110010111 => Some((27, Rem::R1)),
        0b0000110011000 | 0b0000110011001 => Some((28, Rem::R1)),
        0b0000110011010 | 0b0000110011011 => Some((29, Rem::R1)),
        0b0000110100100 | 0b0000110100101 => Some((34, Rem::R1)),
        0b0000110100110 | 0b0000110100111 => Some((35, Rem::R1)),
        0b0000110101000 | 0b0000110101001 => Some((36, Rem::R1)),
        0b0000110101010 | 0b0000110101011 => Some((37, Rem::R1)),
        0b0000110101100 | 0b0000110101101 => Some((38, Rem::R1)),
        0b0000110101110 | 0b0000110101111 => Some((39, Rem::R1)),
        0b0000110110100 | 0b0000110110101 => Some((42, Rem::R1)),
        0b0000110110110 | 0b0000110110111 => Some((43, Rem::R1)),
        // 13 bit
        0b0000001001010 => Some((640, Rem::R0)),
        0b0000001001011 => Some((704, Rem::R0)),
        0b0000001001100 => Some((768, Rem::R0)),
        0b0000001001101 => Some((832, Rem::R0)),
        0b0000001010010 => Some((1280, Rem::R0)),
        0b0000001010011 => Some((1344, Rem::R0)),
        0b0000001010100 => Some((1408, Rem::R0)),
        0b0000001010101 => Some((1472, Rem::R0)),
        0b0000001011010 => Some((1536, Rem::R0)),
        0b0000001011011 => Some((1600, Rem::R0)),
        0b0000001100100 => Some((1664, Rem::R0)),
        0b0000001100101 => Some((1728, Rem::R0)),
        0b0000001101100 => Some((512, Rem::R0)),
        0b0000001101101 => Some((576, Rem::R0)),
        0b0000001110010 => Some((896, Rem::R0)),
        0b0000001110011 => Some((960, Rem::R0)),
        0b0000001110100 => Some((1024, Rem::R0)),
        0b0000001110101 => Some((1088, Rem::R0)),
        0b0000001110110 => Some((1152, Rem::R0)),
        0b0000001110111 => Some((1216, Rem::R0)),
        // rest
        _ => None,
    }
}

fn black14(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 12 bit
        0b00000001001000..=0b00000001001011 => Some((1984, Rem::R2)),
        0b00000001001100..=0b00000001001111 => Some((2048, Rem::R2)),
        0b00000001010000..=0b00000001010011 => Some((2112, Rem::R2)),
        0b00000001010100..=0b00000001010111 => Some((2176, Rem::R2)),
        0b00000001011000..=0b00000001011011 => Some((2240, Rem::R2)),
        0b00000001011100..=0b00000001011111 => Some((2304, Rem::R2)),
        0b00000001110000..=0b00000001110011 => Some((2368, Rem::R2)),
        0b00000001110100..=0b00000001110111 => Some((2432, Rem::R2)),
        0b00000001111000..=0b00000001111011 => Some((2496, Rem::R2)),
        0b00000001111100..=0b00000001111111 => Some((2560, Rem::R2)),
        0b00000010010000..=0b00000010010011 => Some((52, Rem::R2)),
        0b00000010011100..=0b00000010011111 => Some((55, Rem::R2)),
        0b00000010100000..=0b00000010100011 => Some((56, Rem::R2)),
        0b00000010101100..=0b00000010101111 => Some((59, Rem::R2)),
        0b00000010110000..=0b00000010110011 => Some((60, Rem::R2)),
        0b00000011001100..=0b00000011001111 => Some((320, Rem::R2)),
        0b00000011010000..=0b00000011010011 => Some((384, Rem::R2)),
        0b00000011010100..=0b00000011010111 => Some((448, Rem::R2)),
        0b00000011011100..=0b00000011011111 => Some((53, Rem::R2)),
        0b00000011100000..=0b00000011100011 => Some((54, Rem::R2)),
        0b00000101001000..=0b00000101001011 => Some((50, Rem::R2)),
        0b00000101001100..=0b00000101001111 => Some((51, Rem::R2)),
        0b00000101010000..=0b00000101010011 => Some((44, Rem::R2)),
        0b00000101010100..=0b00000101010111 => Some((45, Rem::R2)),
        0b00000101011000..=0b00000101011011 => Some((46, Rem::R2)),
        0b00000101011100..=0b00000101011111 => Some((47, Rem::R2)),
        0b00000101100000..=0b00000101100011 => Some((57, Rem::R2)),
        0b00000101100100..=0b00000101100111 => Some((58, Rem::R2)),
        0b00000101101000..=0b00000101101011 => Some((61, Rem::R2)),
        0b00000101101100..=0b00000101101111 => Some((256, Rem::R2)),
        0b00000110010000..=0b00000110010011 => Some((48, Rem::R2)),
        0b00000110010100..=0b00000110010111 => Some((49, Rem::R2)),
        0b00000110011000..=0b00000110011011 => Some((62, Rem::R2)),
        0b00000110011100..=0b00000110011111 => Some((63, Rem::R2)),
        0b00000110100000..=0b00000110100011 => Some((30, Rem::R2)),
        0b00000110100100..=0b00000110100111 => Some((31, Rem::R2)),
        0b00000110101000..=0b00000110101011 => Some((32, Rem::R2)),
        0b00000110101100..=0b00000110101111 => Some((33, Rem::R2)),
        0b00000110110000..=0b00000110110011 => Some((40, Rem::R2)),
        0b00000110110100..=0b00000110110111 => Some((41, Rem::R2)),
        0b00001100100000..=0b00001100100011 => Some((128, Rem::R2)),
        0b00001100100100..=0b00001100100111 => Some((192, Rem::R2)),
        0b00001100101000..=0b00001100101011 => Some((26, Rem::R2)),
        0b00001100101100..=0b00001100101111 => Some((27, Rem::R2)),
        0b00001100110000..=0b00001100110011 => Some((28, Rem::R2)),
        0b00001100110100..=0b00001100110111 => Some((29, Rem::R2)),
        0b00001101001000..=0b00001101001011 => Some((34, Rem::R2)),
        0b00001101001100..=0b00001101001111 => Some((35, Rem::R2)),
        0b00001101010000..=0b00001101010011 => Some((36, Rem::R2)),
        0b00001101010100..=0b00001101010111 => Some((37, Rem::R2)),
        0b00001101011000..=0b00001101011011 => Some((38, Rem::R2)),
        0b00001101011100..=0b00001101011111 => Some((39, Rem::R2)),
        0b00001101101000..=0b00001101101011 => Some((42, Rem::R2)),
        0b00001101101100..=0b00001101101111 => Some((43, Rem::R2)),
        // 13 bit
        0b00000010010100 | 0b00000010010101 => Some((640, Rem::R1)),
        0b00000010010110 | 0b00000010010111 => Some((704, Rem::R1)),
        0b00000010011000 | 0b00000010011001 => Some((768, Rem::R1)),
        0b00000010011010 | 0b00000010011011 => Some((832, Rem::R1)),
        0b00000010100100 | 0b00000010100101 => Some((1280, Rem::R1)),
        0b00000010100110 | 0b00000010100111 => Some((1344, Rem::R1)),
        0b00000010101000 | 0b00000010101001 => Some((1408, Rem::R1)),
        0b00000010101010 | 0b00000010101011 => Some((1472, Rem::R1)),
        0b00000010110100 | 0b00000010110101 => Some((1536, Rem::R1)),
        0b00000010110110 | 0b00000010110111 => Some((1600, Rem::R1)),
        0b00000011001000 | 0b00000011001001 => Some((1664, Rem::R1)),
        0b00000011001010 | 0b00000011001011 => Some((1728, Rem::R1)),
        0b00000011011000 | 0b00000011011001 => Some((512, Rem::R1)),
        0b00000011011010 | 0b00000011011011 => Some((576, Rem::R1)),
        0b00000011100100 | 0b00000011100101 => Some((896, Rem::R1)),
        0b00000011100110 | 0b00000011100111 => Some((960, Rem::R1)),
        0b00000011101000 | 0b00000011101001 => Some((1024, Rem::R1)),
        0b00000011101010 | 0b00000011101011 => Some((1088, Rem::R1)),
        0b00000011101100 | 0b00000011101101 => Some((1152, Rem::R1)),
        0b00000011101110 | 0b00000011101111 => Some((1216, Rem::R1)),
        // rest
        _ => None,
    }
}

fn black15(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 13 bit
        0b000000100101000..=0b000000100101011 => Some((640, Rem::R1)),
        0b000000100101100..=0b000000100101111 => Some((704, Rem::R1)),
        0b000000100110000..=0b000000100110011 => Some((768, Rem::R1)),
        0b000000100110100..=0b000000100110111 => Some((832, Rem::R1)),
        0b000000101001000..=0b000000101001011 => Some((1280, Rem::R1)),
        0b000000101001100..=0b000000101001111 => Some((1344, Rem::R1)),
        0b000000101010000..=0b000000101010011 => Some((1408, Rem::R1)),
        0b000000101010100..=0b000000101010111 => Some((1472, Rem::R1)),
        0b000000101101000..=0b000000101101011 => Some((1536, Rem::R1)),
        0b000000101101100..=0b000000101101111 => Some((1600, Rem::R1)),
        0b000000110010000..=0b000000110010011 => Some((1664, Rem::R1)),
        0b000000110010100..=0b000000110010111 => Some((1728, Rem::R1)),
        0b000000110110000..=0b000000110110011 => Some((512, Rem::R1)),
        0b000000110110100..=0b000000110110111 => Some((576, Rem::R1)),
        0b000000111001000..=0b000000111001011 => Some((896, Rem::R1)),
        0b000000111001100..=0b000000111001111 => Some((960, Rem::R1)),
        0b000000111010000..=0b000000111010011 => Some((1024, Rem::R1)),
        0b000000111010100..=0b000000111010111 => Some((1088, Rem::R1)),
        0b000000111011000..=0b000000111011011 => Some((1152, Rem::R1)),
        0b000000111011100..=0b000000111011111 => Some((1216, Rem::R1)),
        // rest
        _ => None,
    }
}

fn white(state: u16, off: u8) -> Option<(u16, Rem)> {
    match off {
        0 | 1 | 2 | 3 => None,
        4 => white4(state),
        5 => white5(state),
        6 => white6(state),
        7 => white7(state),
        8 => white8(state),
        9 => white9(state),
        10 => white10(state),
        11 => white11(state),
        12 => white12(state),
        13 => white13(state),
        14 => white14(state),
        _ => todo!("{:016b} ({})", state, off),
    }
}

/// After collecting 4 bits
fn white4(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 4 bit
        0b0111 => Some((2, Rem::R0)),
        0b1000 => Some((3, Rem::R0)),
        0b1011 => Some((4, Rem::R0)),
        0b1100 => Some((5, Rem::R0)),
        0b1110 => Some((6, Rem::R0)),
        0b1111 => Some((7, Rem::R0)),
        _ => None,
    }
}

fn white5(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 4 bit
        0b01110 | 0b01111 => Some((2, Rem::R1)),
        0b10000 | 0b10001 => Some((3, Rem::R1)),
        0b10110 | 0b10111 => Some((4, Rem::R1)),
        0b11000 | 0b11001 => Some((5, Rem::R1)),
        0b11100 | 0b11101 => Some((6, Rem::R1)),
        0b11110 | 0b11111 => Some((7, Rem::R1)),
        // 5 bit
        0b00111 => Some((10, Rem::R0)),
        0b01000 => Some((11, Rem::R0)),
        0b10010 => Some((128, Rem::R0)),
        0b10011 => Some((8, Rem::R0)),
        0b10100 => Some((9, Rem::R0)),
        0b11011 => Some((64, Rem::R0)),
        _ => None,
    }
}

fn white6(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 4 bit
        0b011100..=0b011111 => Some((2, Rem::R2)),
        0b100000..=0b100011 => Some((3, Rem::R2)),
        0b101100..=0b101111 => Some((4, Rem::R2)),
        0b110000..=0b110011 => Some((5, Rem::R2)),
        0b111000..=0b111011 => Some((6, Rem::R2)),
        0b111100..=0b111111 => Some((7, Rem::R2)),
        // 5 bit
        0b001110 | 0b001111 => Some((10, Rem::R1)),
        0b010000 | 0b010001 => Some((11, Rem::R1)),
        0b100100 | 0b100101 => Some((128, Rem::R1)),
        0b100110 | 0b100111 => Some((8, Rem::R1)),
        0b101000 | 0b101001 => Some((9, Rem::R1)),
        0b110110 | 0b110111 => Some((64, Rem::R1)),
        // 6 bit
        0b000011 => Some((13, Rem::R0)),
        0b000111 => Some((1, Rem::R0)),
        0b001000 => Some((12, Rem::R0)),
        0b010111 => Some((192, Rem::R0)),
        0b011000 => Some((1664, Rem::R0)),
        0b101010 => Some((16, Rem::R0)),
        0b101011 => Some((17, Rem::R0)),
        0b110100 => Some((14, Rem::R0)),
        0b110101 => Some((15, Rem::R0)),
        // rest
        _ => None,
    }
}

fn white7(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 5 bit
        0b0011100..=0b0011111 => Some((10, Rem::R2)),
        0b0100000..=0b0100011 => Some((11, Rem::R2)),
        0b1001000..=0b1001011 => Some((128, Rem::R2)),
        0b1001100..=0b1001111 => Some((8, Rem::R2)),
        0b1010000..=0b1010011 => Some((9, Rem::R2)),
        0b1101100..=0b1101111 => Some((64, Rem::R2)),
        // 6 bit
        0b0000110 | 0b0000111 => Some((13, Rem::R1)),
        0b0001110 | 0b0001111 => Some((1, Rem::R1)),
        0b0010000 | 0b0010001 => Some((12, Rem::R1)),
        0b0101110 | 0b0101111 => Some((192, Rem::R1)),
        0b0110000 | 0b0110001 => Some((1664, Rem::R1)),
        0b1010100 | 0b1010101 => Some((16, Rem::R1)),
        0b1010110 | 0b1010111 => Some((17, Rem::R1)),
        0b1101000 | 0b1101001 => Some((14, Rem::R1)),
        0b1101010 | 0b1101011 => Some((15, Rem::R1)),
        // 7 bit
        0b0000011 => Some((22, Rem::R0)),
        0b0000100 => Some((23, Rem::R0)),
        0b0001000 => Some((20, Rem::R0)),
        0b0001100 => Some((19, Rem::R0)),
        0b0010011 => Some((26, Rem::R0)),
        0b0010111 => Some((21, Rem::R0)),
        0b0011000 => Some((28, Rem::R0)),
        0b0100100 => Some((27, Rem::R0)),
        0b0100111 => Some((18, Rem::R0)),
        0b0101000 => Some((24, Rem::R0)),
        0b0101011 => Some((25, Rem::R0)),
        0b0110111 => Some((256, Rem::R0)),
        // rest
        _ => None,
    }
}

fn white8(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 6 bit
        0b00001100..=0b00001111 => Some((13, Rem::R2)),
        0b00011100..=0b00011111 => Some((1, Rem::R2)),
        0b00100000..=0b00100011 => Some((12, Rem::R2)),
        0b01011100..=0b01011111 => Some((192, Rem::R2)),
        0b01100000..=0b01100011 => Some((1664, Rem::R2)),
        0b10101000..=0b10101011 => Some((16, Rem::R2)),
        0b10101100..=0b10101111 => Some((17, Rem::R2)),
        0b11010000..=0b11010011 => Some((14, Rem::R2)),
        0b11010100..=0b11010111 => Some((15, Rem::R2)),
        // 7 bit
        0b00000110 | 0b00000111 => Some((22, Rem::R1)),
        0b00001000 | 0b00001001 => Some((23, Rem::R1)),
        0b00010000 | 0b00010001 => Some((20, Rem::R1)),
        0b00011000 | 0b00011001 => Some((19, Rem::R1)),
        0b00100110 | 0b00100111 => Some((26, Rem::R1)),
        0b00101110 | 0b00101111 => Some((21, Rem::R1)),
        0b00110000 | 0b00110001 => Some((28, Rem::R1)),
        0b01001000 | 0b01001001 => Some((27, Rem::R1)),
        0b01001110 | 0b01001111 => Some((18, Rem::R1)),
        0b01010000 | 0b01010001 => Some((24, Rem::R1)),
        0b01010110 | 0b01010111 => Some((25, Rem::R1)),
        0b01101110 | 0b01101111 => Some((256, Rem::R1)),
        // 8 bit
        0b00000011 => Some((30, Rem::R0)),
        0b00000100 => Some((45, Rem::R0)),
        0b00000101 => Some((46, Rem::R0)),
        0b00001010 => Some((47, Rem::R0)),
        0b00001011 => Some((48, Rem::R0)),
        0b00010010 => Some((33, Rem::R0)),
        0b00010011 => Some((34, Rem::R0)),
        0b00010100 => Some((35, Rem::R0)),
        0b00010101 => Some((36, Rem::R0)),
        0b00010110 => Some((37, Rem::R0)),
        0b00010111 => Some((38, Rem::R0)),
        0b00011010 => Some((31, Rem::R0)),
        0b00011011 => Some((32, Rem::R0)),
        0b00100100 => Some((53, Rem::R0)),
        0b00100101 => Some((54, Rem::R0)),
        0b00101000 => Some((39, Rem::R0)),
        0b00101001 => Some((40, Rem::R0)),
        0b00101010 => Some((41, Rem::R0)),
        0b00101011 => Some((42, Rem::R0)),
        0b00101100 => Some((43, Rem::R0)),
        0b00101101 => Some((44, Rem::R0)),
        0b00110010 => Some((61, Rem::R0)),
        0b00110011 => Some((62, Rem::R0)),
        0b00110100 => Some((63, Rem::R0)),
        0b00110101 => Some((0, Rem::R0)),
        0b00110110 => Some((320, Rem::R0)),
        0b00110111 => Some((384, Rem::R0)),
        0b01001010 => Some((59, Rem::R0)),
        0b01001011 => Some((60, Rem::R0)),
        0b01010010 => Some((49, Rem::R0)),
        0b01010011 => Some((50, Rem::R0)),
        0b01010100 => Some((51, Rem::R0)),
        0b01010101 => Some((52, Rem::R0)),
        0b01011000 => Some((55, Rem::R0)),
        0b01011001 => Some((56, Rem::R0)),
        0b01011010 => Some((57, Rem::R0)),
        0b01011011 => Some((58, Rem::R0)),
        0b01100100 => Some((448, Rem::R0)),
        0b01100101 => Some((512, Rem::R0)),
        0b01100111 => Some((640, Rem::R0)),
        0b01101000 => Some((576, Rem::R0)),
        // rest
        _ => None,
    }
}

fn white9(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 7 bit
        0b000001100..=0b000001111 => Some((22, Rem::R2)),
        0b000010000..=0b000010011 => Some((23, Rem::R2)),
        0b000100000..=0b000100011 => Some((20, Rem::R2)),
        0b000110000..=0b000110011 => Some((19, Rem::R2)),
        0b001001100..=0b001001111 => Some((26, Rem::R2)),
        0b001011100..=0b001011111 => Some((21, Rem::R2)),
        0b001100000..=0b001100011 => Some((28, Rem::R2)),
        0b010010000..=0b010010011 => Some((27, Rem::R2)),
        0b010011100..=0b010011111 => Some((18, Rem::R2)),
        0b010100000..=0b010100011 => Some((24, Rem::R2)),
        0b010101100..=0b010101111 => Some((25, Rem::R2)),
        0b011011100..=0b011011111 => Some((256, Rem::R2)),
        // 8 bit
        0b000000110 | 0b000000111 => Some((30, Rem::R1)),
        0b000001000 | 0b000001001 => Some((45, Rem::R1)),
        0b000001010 | 0b000001011 => Some((46, Rem::R1)),
        0b000010100 | 0b000010101 => Some((47, Rem::R1)),
        0b000010110 | 0b000010111 => Some((48, Rem::R1)),
        0b000100100 | 0b000100101 => Some((33, Rem::R1)),
        0b000100110 | 0b000100111 => Some((34, Rem::R1)),
        0b000101000 | 0b000101001 => Some((35, Rem::R1)),
        0b000101010 | 0b000101011 => Some((36, Rem::R1)),
        0b000101100 | 0b000101101 => Some((37, Rem::R1)),
        0b000101110 | 0b000101111 => Some((38, Rem::R1)),
        0b000110100 | 0b000110101 => Some((31, Rem::R1)),
        0b000110110 | 0b000110111 => Some((32, Rem::R1)),
        0b001001000 | 0b001001001 => Some((53, Rem::R1)),
        0b001001010 | 0b001001011 => Some((54, Rem::R1)),
        0b001010000 | 0b001010001 => Some((39, Rem::R1)),
        0b001010010 | 0b001010011 => Some((40, Rem::R1)),
        0b001010100 | 0b001010101 => Some((41, Rem::R1)),
        0b001010110 | 0b001010111 => Some((42, Rem::R1)),
        0b001011000 | 0b001011001 => Some((43, Rem::R1)),
        0b001011010 | 0b001011011 => Some((44, Rem::R1)),
        0b001100100 | 0b001100101 => Some((61, Rem::R1)),
        0b001100110 | 0b001100111 => Some((62, Rem::R1)),
        0b001101000 | 0b001101001 => Some((63, Rem::R1)),
        0b001101010 | 0b001101011 => Some((0, Rem::R1)),
        0b001101100 | 0b001101101 => Some((320, Rem::R1)),
        0b001101110 | 0b001101111 => Some((384, Rem::R1)),
        0b010010100 | 0b010010101 => Some((59, Rem::R1)),
        0b010010110 | 0b010010111 => Some((60, Rem::R1)),
        0b010100100 | 0b010100101 => Some((49, Rem::R1)),
        0b010100110 | 0b010100111 => Some((50, Rem::R1)),
        0b010101000 | 0b010101001 => Some((51, Rem::R1)),
        0b010101010 | 0b010101011 => Some((52, Rem::R1)),
        0b010110000 | 0b010110001 => Some((55, Rem::R1)),
        0b010110010 | 0b010110011 => Some((56, Rem::R1)),
        0b010110100 | 0b010110101 => Some((57, Rem::R1)),
        0b010110110 | 0b010110111 => Some((58, Rem::R1)),
        0b011001000 | 0b011001001 => Some((448, Rem::R1)),
        0b011001010 | 0b011001011 => Some((512, Rem::R1)),
        0b011001110 | 0b011001111 => Some((640, Rem::R1)),
        0b011010000 | 0b011010001 => Some((576, Rem::R1)),
        // 9 bit
        0b010011000 => Some((1472, Rem::R0)),
        0b010011001 => Some((1536, Rem::R0)),
        0b010011010 => Some((1600, Rem::R0)),
        0b010011011 => Some((1728, Rem::R0)),
        0b011001100 => Some((704, Rem::R0)),
        0b011001101 => Some((768, Rem::R0)),
        0b011010010 => Some((832, Rem::R0)),
        0b011010011 => Some((896, Rem::R0)),
        0b011010100 => Some((960, Rem::R0)),
        0b011010101 => Some((1024, Rem::R0)),
        0b011010110 => Some((1088, Rem::R0)),
        0b011010111 => Some((1152, Rem::R0)),
        0b011011000 => Some((1216, Rem::R0)),
        0b011011001 => Some((1280, Rem::R0)),
        0b011011010 => Some((1344, Rem::R0)),
        0b011011011 => Some((1408, Rem::R0)),
        // rest
        _ => None,
    }
}

fn white10(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 8 bit
        0b0000001100..=0b0000001111 => Some((30, Rem::R2)),
        0b0000010000..=0b0000010011 => Some((45, Rem::R2)),
        0b0000010100..=0b0000010111 => Some((46, Rem::R2)),
        0b0000101000..=0b0000101011 => Some((47, Rem::R2)),
        0b0000101100..=0b0000101111 => Some((48, Rem::R2)),
        0b0001001000..=0b0001001011 => Some((33, Rem::R2)),
        0b0001001100..=0b0001001111 => Some((34, Rem::R2)),
        0b0001010000..=0b0001010011 => Some((35, Rem::R2)),
        0b0001010100..=0b0001010111 => Some((36, Rem::R2)),
        0b0001011000..=0b0001011011 => Some((37, Rem::R2)),
        0b0001011100..=0b0001011111 => Some((38, Rem::R2)),
        0b0001101000..=0b0001101011 => Some((31, Rem::R2)),
        0b0001101100..=0b0001101111 => Some((32, Rem::R2)),
        0b0010010000..=0b0010010011 => Some((53, Rem::R2)),
        0b0010010100..=0b0010010111 => Some((54, Rem::R2)),
        0b0010100000..=0b0010100011 => Some((39, Rem::R2)),
        0b0010100100..=0b0010100111 => Some((40, Rem::R2)),
        0b0010101000..=0b0010101011 => Some((41, Rem::R2)),
        0b0010101100..=0b0010101111 => Some((42, Rem::R2)),
        0b0010110000..=0b0010110011 => Some((43, Rem::R2)),
        0b0010110100..=0b0010110111 => Some((44, Rem::R2)),
        0b0011001000..=0b0011001011 => Some((61, Rem::R2)),
        0b0011001100..=0b0011001111 => Some((62, Rem::R2)),
        0b0011010000..=0b0011010011 => Some((63, Rem::R2)),
        0b0011010100..=0b0011010111 => Some((0, Rem::R2)),
        0b0011011000..=0b0011011011 => Some((320, Rem::R2)),
        0b0011011100..=0b0011011111 => Some((384, Rem::R2)),
        0b0100101000..=0b0100101011 => Some((59, Rem::R2)),
        0b0100101100..=0b0100101111 => Some((60, Rem::R2)),
        0b0101001000..=0b0101001011 => Some((49, Rem::R2)),
        0b0101001100..=0b0101001111 => Some((50, Rem::R2)),
        0b0101010000..=0b0101010011 => Some((51, Rem::R2)),
        0b0101010100..=0b0101010111 => Some((52, Rem::R2)),
        0b0101100000..=0b0101100011 => Some((55, Rem::R2)),
        0b0101100100..=0b0101100111 => Some((56, Rem::R2)),
        0b0101101000..=0b0101101011 => Some((57, Rem::R2)),
        0b0101101100..=0b0101101111 => Some((58, Rem::R2)),
        0b0110010000..=0b0110010011 => Some((448, Rem::R2)),
        0b0110010100..=0b0110010111 => Some((512, Rem::R2)),
        0b0110011100..=0b0110011111 => Some((640, Rem::R2)),
        0b0110100000..=0b0110100011 => Some((576, Rem::R2)),
        // 9 bit
        0b0100110000 | 0b0100110001 => Some((1472, Rem::R1)),
        0b0100110010 | 0b0100110011 => Some((1536, Rem::R1)),
        0b0100110100 | 0b0100110101 => Some((1600, Rem::R1)),
        0b0100110110 | 0b0100110111 => Some((1728, Rem::R1)),
        0b0110011000 | 0b0110011001 => Some((704, Rem::R1)),
        0b0110011010 | 0b0110011011 => Some((768, Rem::R1)),
        0b0110100100 | 0b0110100101 => Some((832, Rem::R1)),
        0b0110100110 | 0b0110100111 => Some((896, Rem::R1)),
        0b0110101000 | 0b0110101001 => Some((960, Rem::R1)),
        0b0110101010 | 0b0110101011 => Some((1024, Rem::R1)),
        0b0110101100 | 0b0110101101 => Some((1088, Rem::R1)),
        0b0110101110 | 0b0110101111 => Some((1152, Rem::R1)),
        0b0110110000 | 0b0110110001 => Some((1216, Rem::R1)),
        0b0110110010 | 0b0110110011 => Some((1280, Rem::R1)),
        0b0110110100 | 0b0110110101 => Some((1344, Rem::R1)),
        0b0110110110 | 0b0110110111 => Some((1408, Rem::R1)),
        // rest
        _ => None,
    }
}

fn white11(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 9 bit
        0b01001100000..=0b01001100011 => Some((1472, Rem::R2)),
        0b01001100100..=0b01001100111 => Some((1536, Rem::R2)),
        0b01001101000..=0b01001101011 => Some((1600, Rem::R2)),
        0b01001101100..=0b01001101111 => Some((1728, Rem::R2)),
        0b01100110000..=0b01100110011 => Some((704, Rem::R2)),
        0b01100110100..=0b01100110111 => Some((768, Rem::R2)),
        0b01101001000..=0b01101001011 => Some((832, Rem::R2)),
        0b01101001100..=0b01101001111 => Some((896, Rem::R2)),
        0b01101010000..=0b01101010011 => Some((960, Rem::R2)),
        0b01101010100..=0b01101010111 => Some((1024, Rem::R2)),
        0b01101011000..=0b01101011011 => Some((1088, Rem::R2)),
        0b01101011100..=0b01101011111 => Some((1152, Rem::R2)),
        0b01101100000..=0b01101100011 => Some((1216, Rem::R2)),
        0b01101100100..=0b01101100111 => Some((1280, Rem::R2)),
        0b01101101000..=0b01101101011 => Some((1344, Rem::R2)),
        0b01101101100..=0b01101101111 => Some((1408, Rem::R2)),
        // 11 bit
        0b00000001000 => Some((1792, Rem::R0)),
        0b00000001100 => Some((1856, Rem::R0)),
        0b00000001101 => Some((1920, Rem::R0)),
        // rest
        _ => None,
    }
}

fn white12(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 11 bit
        0b000000010000 | 0b000000010001 => Some((1792, Rem::R1)),
        0b000000011000 | 0b000000011001 => Some((1856, Rem::R1)),
        0b000000011010 | 0b000000011011 => Some((1920, Rem::R1)),
        // 12 bit
        0b000000010010 => Some((1984, Rem::R0)),
        0b000000010011 => Some((2048, Rem::R0)),
        0b000000010100 => Some((2112, Rem::R0)),
        0b000000010101 => Some((2176, Rem::R0)),
        0b000000010110 => Some((2240, Rem::R0)),
        0b000000010111 => Some((2304, Rem::R0)),
        0b000000011100 => Some((2368, Rem::R0)),
        0b000000011101 => Some((2432, Rem::R0)),
        0b000000011110 => Some((2496, Rem::R0)),
        0b000000011111 => Some((2560, Rem::R0)),
        _ => None,
    }
}

fn white13(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 11 bit
        0b0000000100000..=0b0000000100011 => Some((1792, Rem::R2)),
        0b0000000110000..=0b0000000110011 => Some((1856, Rem::R2)),
        0b0000000110100..=0b0000000110111 => Some((1920, Rem::R2)),
        // 12 bit
        0b0000000100100 | 0b0000000100101 => Some((1984, Rem::R1)),
        0b0000000100110 | 0b0000000100111 => Some((2048, Rem::R1)),
        0b0000000101000 | 0b0000000101001 => Some((2112, Rem::R1)),
        0b0000000101010 | 0b0000000101011 => Some((2176, Rem::R1)),
        0b0000000101100 | 0b0000000101101 => Some((2240, Rem::R1)),
        0b0000000101110 | 0b0000000101111 => Some((2304, Rem::R1)),
        0b0000000111000 | 0b0000000111001 => Some((2368, Rem::R1)),
        0b0000000111010 | 0b0000000111011 => Some((2432, Rem::R1)),
        0b0000000111100 | 0b0000000111101 => Some((2496, Rem::R1)),
        0b0000000111110 | 0b0000000111111 => Some((2560, Rem::R1)),
        _ => None,
    }
}

fn white14(state: u16) -> Option<(u16, Rem)> {
    match state {
        // 12 bit
        0b00000001001000..=0b00000001001011 => Some((1984, Rem::R2)),
        0b00000001001100..=0b00000001001111 => Some((2048, Rem::R2)),
        0b00000001010000..=0b00000001010011 => Some((2112, Rem::R2)),
        0b00000001010100..=0b00000001010111 => Some((2176, Rem::R2)),
        0b00000001011000..=0b00000001011011 => Some((2240, Rem::R2)),
        0b00000001011100..=0b00000001011111 => Some((2304, Rem::R2)),
        0b00000001110000..=0b00000001110011 => Some((2368, Rem::R2)),
        0b00000001110100..=0b00000001110111 => Some((2432, Rem::R2)),
        0b00000001111000..=0b00000001111011 => Some((2496, Rem::R2)),
        0b00000001111100..=0b00000001111111 => Some((2560, Rem::R2)),
        _ => None,
    }
}
