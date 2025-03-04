use color_eyre::eyre;
use signum::{
    docs::tebu::{Char, Line},
    util::Pos,
};

use super::Document;

fn print_char_cmds(data: &[Char], x: &mut u16, y: u16) {
    for te in data {
        *x += te.offset;
        println!("({}, {}, {},  {}),", *x, y, te.cval, te.cset);
    }
}

fn print_line_cmds(line: &Line, skip: u16, pos: &mut Pos) {
    pos.x = 0;
    pos.y += (skip + 1) * 2;

    print_char_cmds(&line.data, &mut pos.x, pos.y);
}

pub fn output_pdraw(doc: &Document) -> eyre::Result<()> {
    for page_text in &doc.tebu.pages {
        let mut pos = Pos::new(0, 0);
        for (skip, line) in &page_text.content {
            print_line_cmds(line, *skip, &mut pos);
        }
    }
    Ok(())
}
