use crate::{docs::hcim::ImageArea, util::bit_iter::BitIter};

use super::Page;

pub struct VScaler<'a> {
    w: usize,
    h: usize,
    sel_h: usize,
    sel_w: usize,
    image: &'a Page,
    pixel_h_len: usize,
    pixel_v_len: usize,
    vpixel_count: usize,
    last_vcount: usize,
    iubpl: usize,
    ibyte_index: usize,
    skip_bits: u16,
    ivpixel_rem: usize,
    vpxl: bool,
}

impl<'a> VScaler<'a> {
    pub(crate) fn new(image: &'a Page, w: usize, h: usize, sel: ImageArea) -> Self {
        let iubpl = image.bytes_per_line() as usize;
        let pixel_v_len = h / (sel.h as usize);
        let ivpixel_rem = 0;
        let ibyte_index = (sel.y as usize) * iubpl + (sel.x as usize) / 8;
        let skip_bits = sel.x % 8;
        let vpxl = false;
        Self {
            sel_h: sel.h as usize,
            sel_w: sel.w as usize,
            w,
            h,
            image,
            pixel_h_len: w / (sel.w as usize),
            pixel_v_len,
            vpixel_count: 0,
            last_vcount: 0,
            iubpl,
            ibyte_index,
            skip_bits,
            ivpixel_rem,
            vpxl,
        }
    }

    pub(crate) fn next_line<'b>(&'b mut self) -> HScaler<'a, 'b> {
        let mut ibit_iter = self.image.bits(self.ibyte_index..);
        let hpixel_count = 0;
        let last_hcount = 0;
        let hpxl = self.vpxl;
        let ipixel_rem = 0;
        for _ in 0..self.skip_bits {
            let _ = ibit_iter.next();
        }
        let icurr = ibit_iter.next().unwrap_or(true);
        HScaler {
            vscaler: self,
            ibit_iter,
            hpixel_count,
            last_hcount,
            hpxl,
            ipixel_rem,
            icurr,
        }
    }
}

pub struct HScaler<'a, 'b> {
    vscaler: &'b mut VScaler<'a>,
    ibit_iter: BitIter<'a>,
    hpixel_count: usize,
    last_hcount: usize,
    hpxl: bool,
    ipixel_rem: usize,
    icurr: bool,
}

impl HScaler<'_, '_> {
    pub(crate) fn next(&mut self) -> bool {
        if self.vscaler.pixel_h_len == 0 {
            while self.last_hcount < self.hpixel_count * self.vscaler.sel_w / self.vscaler.w {
                if self.ipixel_rem == 7 {
                    self.hpxl = !self.hpxl;
                    //self.icurr = self.hpxl;
                    self.ipixel_rem = 0;
                } else {
                    self.ipixel_rem += 1;
                }
                self.icurr = self.ibit_iter.next().unwrap();
                self.last_hcount += 1;
            }
        } else {
            let hcount = self.hpixel_count * self.vscaler.sel_w / self.vscaler.w;
            if self.last_hcount < hcount {
                if self.ipixel_rem == 7 {
                    self.hpxl = !self.hpxl;
                    //self.icurr = self.hpxl;
                    self.ipixel_rem = 0;
                } else {
                    self.ipixel_rem += 1;
                }
                self.icurr = self.ibit_iter.next().unwrap();
                self.last_hcount += 1;
            }
        }
        self.hpixel_count += 1;
        self.icurr
    }

    pub(crate) fn end(self) {
        let vs = self.vscaler;
        /*println!(
            "lhcount: {:4}, hpixel: {:4}, sel_w: {:4}, w: {:4}",
            self.last_hcount, self.hpixel_count, vs.sel_w, vs.w
        );
        println!(
            "lvcount: {:4}, vpixel: {:4}, sel_h: {:4}, h: {:4}",
            vs.last_vcount, vs.vpixel_count, vs.sel_h, vs.h
        );*/

        //vs.ivpixel_rem = vs.pixel_v_len;
        if vs.pixel_v_len == 0 {
            while vs.last_vcount < vs.vpixel_count * vs.sel_h / vs.h {
                vs.last_vcount += 1;
                vs.ibyte_index += vs.iubpl;
                if vs.ivpixel_rem == 7 {
                    vs.ivpixel_rem = 0;
                    vs.vpxl = !vs.vpxl;
                } else {
                    vs.ivpixel_rem += 1;
                }
            }
        } else {
            let vcount = vs.vpixel_count * vs.sel_h / vs.h;
            if vs.last_vcount < vcount {
                vs.ibyte_index += vs.iubpl;
                if vs.ivpixel_rem == 7 {
                    vs.ivpixel_rem = 0;
                    vs.vpxl = !vs.vpxl;
                } else {
                    vs.ivpixel_rem += 1;
                }
                vs.last_vcount += 1;
            }
        }

        vs.vpixel_count += 1;
    }
}
