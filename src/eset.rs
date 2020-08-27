use crate::Buf;
use nom::{
    bytes::complete::{tag, take},
    multi::count,
    number::complete::{be_u32, be_u8},
    IResult,
};

const BORDER: [&str; 17] = [
    "++",
    "+-+",
    "+--+",
    "+---+",
    "+----+",
    "+-----+",
    "+------+",
    "+-------+",
    "+--------+",
    "+---------+",
    "+----------+",
    "+-----------+",
    "+------------+",
    "+-------------+",
    "+--------------+",
    "+---------------+",
    "+----------------+",
];

const BIT_STRING: [&str; 256] = [
    "        ", "       #", "      # ", "      ##", "     #  ", "     # #", "     ## ", "     ###",
    "    #   ", "    #  #", "    # # ", "    # ##", "    ##  ", "    ## #", "    ### ", "    ####",
    "   #    ", "   #   #", "   #  # ", "   #  ##", "   # #  ", "   # # #", "   # ## ", "   # ###",
    "   ##   ", "   ##  #", "   ## # ", "   ## ##", "   ###  ", "   ### #", "   #### ", "   #####",
    "  #     ", "  #    #", "  #   # ", "  #   ##", "  #  #  ", "  #  # #", "  #  ## ", "  #  ###",
    "  # #   ", "  # #  #", "  # # # ", "  # # ##", "  # ##  ", "  # ## #", "  # ### ", "  # ####",
    "  ##    ", "  ##   #", "  ##  # ", "  ##  ##", "  ## #  ", "  ## # #", "  ## ## ", "  ## ###",
    "  ###   ", "  ###  #", "  ### # ", "  ### ##", "  ####  ", "  #### #", "  ##### ", "  ######",
    " #      ", " #     #", " #    # ", " #    ##", " #   #  ", " #   # #", " #   ## ", " #   ###",
    " #  #   ", " #  #  #", " #  # # ", " #  # ##", " #  ##  ", " #  ## #", " #  ### ", " #  ####",
    " # #    ", " # #   #", " # #  # ", " # #  ##", " # # #  ", " # # # #", " # # ## ", " # # ###",
    " # ##   ", " # ##  #", " # ## # ", " # ## ##", " # ###  ", " # ### #", " # #### ", " # #####",
    " ##     ", " ##    #", " ##   # ", " ##   ##", " ##  #  ", " ##  # #", " ##  ## ", " ##  ###",
    " ## #   ", " ## #  #", " ## # # ", " ## # ##", " ## ##  ", " ## ## #", " ## ### ", " ## ####",
    " ###    ", " ###   #", " ###  # ", " ###  ##", " ### #  ", " ### # #", " ### ## ", " ### ###",
    " ####   ", " ####  #", " #### # ", " #### ##", " #####  ", " ##### #", " ###### ", " #######",
    "#       ", "#      #", "#     # ", "#     ##", "#    #  ", "#    # #", "#    ## ", "#    ###",
    "#   #   ", "#   #  #", "#   # # ", "#   # ##", "#   ##  ", "#   ## #", "#   ### ", "#   ####",
    "#  #    ", "#  #   #", "#  #  # ", "#  #  ##", "#  # #  ", "#  # # #", "#  # ## ", "#  # ###",
    "#  ##   ", "#  ##  #", "#  ## # ", "#  ## ##", "#  ###  ", "#  ### #", "#  #### ", "#  #####",
    "# #     ", "# #    #", "# #   # ", "# #   ##", "# #  #  ", "# #  # #", "# #  ## ", "# #  ###",
    "# # #   ", "# # #  #", "# # # # ", "# # # ##", "# # ##  ", "# # ## #", "# # ### ", "# # ####",
    "# ##    ", "# ##   #", "# ##  # ", "# ##  ##", "# ## #  ", "# ## # #", "# ## ## ", "# ## ###",
    "# ###   ", "# ###  #", "# ### # ", "# ### ##", "# ####  ", "# #### #", "# ##### ", "# ######",
    "##      ", "##     #", "##    # ", "##    ##", "##   #  ", "##   # #", "##   ## ", "##   ###",
    "##  #   ", "##  #  #", "##  # # ", "##  # ##", "##  ##  ", "##  ## #", "##  ### ", "##  ####",
    "## #    ", "## #   #", "## #  # ", "## #  ##", "## # #  ", "## # # #", "## # ## ", "## # ###",
    "## ##   ", "## ##  #", "## ## # ", "## ## ##", "## ###  ", "## ### #", "## #### ", "## #####",
    "###     ", "###    #", "###   # ", "###   ##", "###  #  ", "###  # #", "###  ## ", "###  ###",
    "### #   ", "### #  #", "### # # ", "### # ##", "### ##  ", "### ## #", "### ### ", "### ####",
    "####    ", "####   #", "####  # ", "####  ##", "#### #  ", "#### # #", "#### ## ", "#### ###",
    "#####   ", "#####  #", "##### # ", "##### ##", "######  ", "###### #", "####### ", "########",
];

#[derive(Debug)]
pub struct ESet<'a> {
    pub buf1: Buf<'a>,
    pub buf2: Buf<'a>,
    pub offsets: Vec<u32>,
}

#[derive(Debug)]
pub struct EChar<'a> {
    width: u8,
    height: u8,
    a: u8,
    d: u8,
    buf: &'a [u8],
}

impl<'a> ESet<'a> {
    pub fn print(&self) {
        let capacity = self.offsets.len();
        let mut widths = Vec::with_capacity(capacity);
        let mut skips = Vec::with_capacity(capacity);
        for off in &self.offsets {
            println!("{}", off);
            let ou = *off as usize;
            let (_, ch) = parse_echar(&self.buf2.0[ou..]).unwrap();
            let wu = ch.width as usize;
            let hu = ch.height as usize;
            widths.push(ch.width);
            skips.push(ch.a);
            println!("{}, {}x{}, {}", ch.a, wu, hu, ch.d);
            if ch.width > 8 {
                let border = BORDER[wu];
                let width = wu - 8;
                println!("{}", border);
                for i in 0..hu {
                    let left = ch.buf[2 * i] as usize;
                    let right = ch.buf[2 * i + 1] as usize;
                    println!("|{}{}|", &BIT_STRING[left], &BIT_STRING[right][..width]);
                }
                println!("{}", border);
            } else {
                let border = BORDER[wu];
                println!("{}", border);
                for i in 0..hu {
                    let byte = ch.buf[2 * i] as usize;
                    println!("|{}|", &BIT_STRING[byte][..wu]);
                }
                println!("{}", border);
            }
        }
        println!();
        println!("pub const WIDTH: [u8; 128] = [");
        print!("  0, ");
        for (i, w) in widths.iter().cloned().enumerate() {
            if i % 16 == 15 {
                println!();
            }
            print!("{:3},", w);
            if i % 16 != 14 {
                print!(" ");
            }
        }
        println!();
        println!("];");
        println!("pub const SKIP: [u8; 128] = [");
        print!("  0, ");
        for (i, s) in skips.iter().cloned().enumerate() {
            if i % 16 == 15 {
                println!();
            }
            print!("{:3},", s);
            if i % 16 != 14 {
                print!(" ");
            }
        }
        println!();
        println!("];");
    }
}

pub fn parse_echar(input: &[u8]) -> IResult<&[u8], EChar> {
    let (input, a) = be_u8(input)?;
    let (input, height) = be_u8(input)?;
    let (input, width) = be_u8(input)?;
    let (input, d) = be_u8(input)?;
    let (input, buf) = take((height * 2) as usize)(input)?;
    Ok((
        input,
        EChar {
            width,
            height,
            a,
            d,
            buf,
        },
    ))
}

pub fn parse_eset(input: &[u8]) -> IResult<&[u8], ESet> {
    let (input, _) = tag(b"eset")(input)?;
    let (input, _) = tag(b"0001")(input)?;
    let (input, skip) = be_u32(input)?;

    let (input, buf1) = take(skip as usize)(input)?;
    //let (input, cnt) = be_u32(input)?;

    let (input, len) = be_u32(input)?;
    let (input, offsets) = count(be_u32, (skip - 1) as usize)(input)?;
    let (input, buf2) = take(len as usize)(input)?;

    Ok((
        input,
        ESet {
            buf1: Buf(buf1),
            buf2: Buf(buf2),
            offsets,
        },
    ))
}
