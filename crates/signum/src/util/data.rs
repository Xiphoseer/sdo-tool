//! # Static lookup tables
//!
//! This module contains static lookup tables that make printing black/white
//! bitmaps like glyphs in a charset a little easier.

/// Helper macro that generates arrays with 256 elements where the value is a
/// mapping of the bit pattern of the index based on using $a for 0 and $b for 1.
///
/// If $x is `b`, the elements will be byte arrays, if $x is `str`, the elements
/// will be `&'static str`
macro_rules! bits {
    [$($x:ident)? $a:literal $b:literal] => {
        // Initial step
        bits![$($x)? $a $b => 0 0 0 0 0 0 0 0 => []]
    };
    [$($x:ident)? $a:literal $b:literal => $c0:literal $($c:literal)* => $([$($d:literal),*]),*] => {
        // Recursion with two output arrays per input array
        bits![$($x)? $a $b => $($c)* => $([$($d,)* $a], [$($d,)* $b]),*]
    };
    [str $a:literal $b:literal => => $([$($d:literal),*]),*] => {
        // Recursion anchor with conversion to `&str`
        [$(unsafe { std::str::from_utf8_unchecked(&[$($d),*])}),*]
    };
    [$a:literal $b:literal => => $([$($d:literal),*]),*] => {
        // Simple recursion anchor
        [$([$($d),*]),*]
    };
}

/// Lookup table for bytes as a string of ` ` (0) and `#` (1)
pub const BIT_STRING: [&str; 256] = bits![str b' ' b'#'];

/// Lookup table for bytes as a byte array of `0xFF` / white / no-ink (0) and `0x00` / black / ink (1)
pub const BIT_PROJECTION: [[u8; 8]; 256] = bits![0xFF 0x00];

#[cfg(test)]
mod tests {
    use crate::util::data::{BIT_PROJECTION, BIT_STRING};

    #[test]
    fn test_pattern() {
        assert_eq!(
            BIT_PROJECTION[0b00010111],
            [0xFF, 0xFF, 0xFF, 0x00, 0xFF, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            BIT_PROJECTION[0b11111111],
            [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            BIT_PROJECTION[0b01010101],
            [0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00]
        );

        assert_eq!(BIT_STRING[0b00010111], "   # ###");
        assert_eq!(BIT_STRING[0b11111111], "########");
        assert_eq!(BIT_STRING[0b01010101], " # # # #");
    }
}
