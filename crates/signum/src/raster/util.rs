use crate::util::data::BIT_STRING;

pub fn print(bytes_per_line: u32, width: u32, buffer: &[u8]) {
    let _final_bits = width % 8;
    let border = || {
        print!("+");
        for _ in 0..bytes_per_line {
            print!("--------");
        }
        println!("+");
    };

    border();
    for line in buffer.chunks_exact(bytes_per_line as usize) {
        print!("|");
        for byte in line.iter().copied() {
            print!("{}", &BIT_STRING[byte as usize]);
        }
        println!("|");
    }
    border();
}
