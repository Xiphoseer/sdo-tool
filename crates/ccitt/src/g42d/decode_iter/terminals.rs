use crate::bits::BitIter;

pub fn fax_decode_h(
    bit_iter: &mut BitIter,
    terminal: fn(bit_iter: &mut BitIter) -> Option<u16>,
) -> Option<u16> {
    let mut sum = 0;
    loop {
        let v = terminal(bit_iter)?;
        sum += v;
        if v < 64 {
            break Some(sum);
        }
    }
}

pub fn black_terminal(bit_iter: &mut BitIter) -> Option<u16> {
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

pub fn white_terminal(bit_iter: &mut BitIter) -> Option<u16> {
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

fn fax_decode_h_both(bit_iter: &mut BitIter) -> Option<u16> {
    if bit_iter.next()? {
        // 00000001
        if bit_iter.next()? {
            // 000000011
            if bit_iter.next()? {
                if bit_iter.next()? {
                    if bit_iter.next()? {
                        Some(2560) // 000000011111
                    } else {
                        Some(2496) // 000000011110
                    }
                } else if bit_iter.next()? {
                    Some(2432) // 000000011101
                } else {
                    Some(2368) // 000000011100
                }
            } else if bit_iter.next()? {
                Some(1920) // 00000001101
            } else {
                Some(1856) // 00000001100
            }
        } else if bit_iter.next()? {
            if bit_iter.next()? {
                if bit_iter.next()? {
                    Some(2304) // 000000010111
                } else {
                    Some(2240) // 000000010110
                }
            } else if bit_iter.next()? {
                Some(2176) // 000000010101
            } else {
                Some(2112) // 000000010100
            }
        } else if bit_iter.next()? {
            if bit_iter.next()? {
                Some(2048) // 000000010011
            } else {
                Some(1984) // 000000010010
            }
        } else {
            Some(1792) // 00000001000
        }
    } else {
        panic!("Invalid Code");
    }
}
