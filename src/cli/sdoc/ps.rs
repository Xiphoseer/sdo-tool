use std::io;

use io::Write;

use crate::ps::PSWriter;

pub fn prog_dict<W: Write>(pw: &mut PSWriter<W>, dict: &str) -> io::Result<()> {
    pw.lit(dict)?;
    pw.write_usize(300)?;
    pw.name("dict")?;
    pw.name("def")?;

    pw.name(dict)?;
    pw.name("begin")?;

    // alias N = `def`
    pw.lit("N")?;
    pw.seq(|pw| pw.name("def"))?;
    pw.name("def")?;

    // alias B = `bind def`
    pw.lit("B")?;
    pw.seq(|pw| {
        pw.name("bind")?;
        pw.name("def")
    })?;
    pw.name("N")?;

    // alias S = `exch`
    pw.lit("S")?;
    pw.seq(|pw| pw.name("exch"))?;
    pw.name("N")?;

    // alias X = `exch def`
    pw.lit("X")?;
    pw.seq(|pw| {
        pw.name("S")?;
        pw.name("N")
    })?;
    pw.name("B")?;

    // alias A = `dup`
    pw.lit("A")?;
    pw.seq(|pw| pw.name("dup"))?;
    pw.name("B")?;

    // alias TR = `translate`
    pw.lit("TR")?;
    pw.seq(|pw| pw.name("translate"))?;
    pw.name("N")?;

    // set isls = false
    pw.lit("isls")?;
    pw.bool(false)?;
    pw.name("N")?;

    // vsize = 11 * 72
    pw.lit("vsize")?;
    pw.write_usize(11)?;
    pw.write_usize(72)?;
    pw.ps_mul()?;
    pw.name("N")?;

    // hsize = 8.5 * 72
    pw.lit("hsize")?;
    pw.double(8.5)?;
    pw.write_usize(72)?;
    pw.ps_mul()?;
    pw.name("N")?;

    pw.lit("landplus90")?;
    pw.seq(|pw| pw.bool(false))?;
    pw.name("def")?; // TODO -> N?

    pw.lit("@rigin")?;
    pw.seq(|pw| {
        pw.name("isls")?;
        pw.seq(|pw| {
            // [0 landplus90{1 -1}{-1 1}ifelse 0 0 0]
            pw.arr(|pw| {
                pw.write_usize(0)?;
                pw.name("landplus90")?;
                pw.seq(|pw| {
                    pw.isize(1)?;
                    pw.isize(-1)
                })?;
                pw.seq(|pw| {
                    pw.isize(-1)?;
                    pw.isize(1)
                })?;
                pw.ps_ifelse()?;
                pw.isize(0)?;
                pw.isize(0)?;
                pw.isize(0)
            })?;
            pw.name("concat")
        })?;
        pw.ps_if()?;

        pw.write_usize(72)?;
        pw.name("Resolution")?;
        pw.ps_div()?;
        pw.write_usize(72)?;
        pw.name("VResolution")?;
        pw.ps_div()?;
        pw.ps_neg()?;
        pw.ps_scale()?;

        pw.name("isls")?;
        pw.seq(|pw| {
            pw.name("landplus90")?;
            pw.seq(|pw| {
                pw.name("VResolution")?;
                pw.isize(72)?;
                pw.ps_div()?;
                pw.name("vsize")?;
                pw.ps_mul()?;
                pw.isize(0)?;
                pw.ps_exch()
            })?;
            // {Resolution -72 div hsize mul 0}
            pw.seq(|pw| {
                pw.name("Resolution")?;
                pw.isize(-72)?;
                pw.ps_div()?;
                pw.name("hsize")?;
                pw.ps_mul()?;
                pw.isize(0)
            })?;
            pw.ps_ifelse()?;
            pw.name("TR")
        })?;
        pw.name("if")?;

        pw.name("Resolution")?;
        pw.name("VResolution")?;
        pw.name("vsize")?;
        pw.isize(-72)?;
        pw.name("div")?;
        pw.write_usize(1)?;
        pw.name("add")?;
        pw.name("mul")?;

        pw.name("TR")?;
        pw.arr(|pw| {
            pw.name("matrix")?;
            pw.name("currentmatrix")?;

            // {A A round sub abs 0.00001 lt{round}if}
            pw.seq(|pw| {
                pw.name("A")?;
                pw.name("A")?;
                pw.name("round")?;
                pw.ps_sub()?;
                pw.ps_abs()?;
                pw.double(0.00001)?;
                pw.ps_lt()?;
                pw.seq(|pw| pw.name("round"))?;
                pw.ps_if()
            })?;
            pw.name("forall")?;
            pw.name("round")?;
            pw.ps_exch()?;
            pw.name("round")?;
            pw.ps_exch()
        })?;
        pw.name("setmatrix")
    })?;
    pw.name("N")?;

    // @landscape
    pw.lit("@landscape")?;
    pw.seq(|pw| {
        pw.lit("isls")?;
        pw.bool(true)?;
        pw.name("N")
    })?;
    pw.name("B")?;

    // /@manualfeed{statusdict/manualfeed true put}B
    pw.lit("@manualfeed")?;
    pw.seq(|pw| {
        pw.name("statusdict")?;
        pw.lit("manualfeed")?;
        pw.bool(true)?;
        pw.ps_put()
    })?;
    pw.name("B")?;

    // /@copies{/#copies X}B
    pw.lit("@copies")?;
    pw.seq(|pw| {
        pw.lit("#copies")?;
        pw.name("X")
    })?;
    pw.name("B")?;

    // /FMat[1 0 0 -1 0 0]N
    pw.lit("FMat")?;
    pw.arr(|pw| {
        pw.isize(1)?;
        pw.isize(0)?;
        pw.isize(0)?;
        pw.isize(-1)?;
        pw.isize(0)?;
        pw.isize(0)
    })?;
    pw.name("N")?;

    // /FBB[0 0 0 0]N
    pw.lit("FBB")?;
    pw.arr(|pw| {
        pw.isize(0)?;
        pw.isize(0)?;
        pw.isize(0)?;
        pw.isize(0)
    })?;
    pw.name("N")?;

    // /nn 0 N
    pw.lit("nn")?;
    pw.isize(0)?;
    pw.name("N")?;

    // /IEn 0 N
    pw.lit("IEn")?;
    pw.isize(0)?;
    pw.name("N")?;

    // /ctr 0 N
    pw.lit("ctr")?;
    pw.isize(0)?;
    pw.name("N")?;

    pw.lit("df-tail")?;
    pw.seq(|pw| {
        // create the font dict
        pw.lit("nn")?;
        pw.isize(9)?;
        pw.ps_dict()?;
        pw.name("N")?;

        // populate the font dict
        pw.name("nn")?;
        pw.begin(|pw| {
            pw.lit("FontType")?;
            pw.isize(3)?;
            pw.name("N")?;

            pw.lit("FontMatrix")?;
            pw.name("fntrx")?;
            pw.name("N")?;

            pw.lit("FontBBox")?;
            pw.name("FBB")?;
            pw.name("N")?;

            pw.ps_string()?;
            pw.lit("base")?;
            pw.name("X")?;

            pw.ps_array()?;
            pw.lit("BitMaps")?;
            pw.name("X")?;

            pw.name("A")?;
            pw.lit("FontName")?;
            pw.name("X")?;

            pw.isize(2)?;
            pw.ps_dict()?;
            pw.name("A")?; // duplicate dict
            pw.begin(|pw| {
                pw.name("S")?; // get name from beneath dict
                pw.name("A")?;
                pw.lit("FamilyName")?;
                pw.name("X")?;
                pw.lit("FullName")?;
                pw.name("X")
            })?;
            pw.lit("FontInfo")?;
            pw.name("X")?;

            pw.lit("BuildChar")?;
            pw.seq(|pw| pw.name("CharBuilder"))?;
            pw.name("N")?;

            pw.lit("Encoding")?;
            pw.name("IEn")?;
            pw.name("N")
        })?;
        pw.name("A")?;

        pw.seq(|pw| {
            pw.lit("foo")?;
            pw.ps_setfont()
        })?;
        pw.isize(2)?;
        pw.ps_array()?;
        pw.ps_copy()?;
        pw.name("cvx")?;
        pw.name("N")?;
        pw.ps_load()?;
        pw.isize(0)?;
        pw.name("nn")?;
        pw.ps_put()?;
        pw.lit("ctr")?;
        pw.isize(0)?;
        pw.name("N")?;
        pw.arr_open()
    })?;
    pw.name("B")?;

    // /sf 0 N
    pw.lit("sf")?;
    pw.isize(0)?;
    pw.name("N")?;

    // /df{/sf 1 N/fntrx FMat N df-tail}B
    pw.lit("df")?;
    pw.seq(|pw| {
        pw.lit("sf")?;
        pw.isize(1)?;
        pw.name("N")?;

        pw.lit("fntrx")?;
        pw.name("FMat")?;
        pw.name("N")?;

        pw.name("df-tail")
    })?;
    pw.name("B")?;

    // /dfs{div/sf X/fntrx[sf 0 0 sf neg 0 0]N df-tail}B
    pw.lit("dfs")?;
    pw.seq(|pw| {
        pw.ps_div()?;
        pw.lit("sf")?;
        pw.name("X")?;

        pw.lit("fntrx")?;
        pw.arr(|pw| {
            pw.name("sf")?;
            pw.isize(0)?;
            pw.isize(0)?;
            pw.name("sf")?;
            pw.ps_neg()?;
            pw.isize(0)?;
            pw.isize(0)
        })?;
        pw.name("N")?;
        pw.name("df-tail")
    })?;
    pw.name("B")?;

    // /E{pop nn A definefont setfont}B
    pw.lit("E")?;
    pw.seq(|pw| {
        pw.ps_pop()?;
        pw.name("nn")?;
        pw.name("A")?;
        pw.lit("FontName")?;
        pw.ps_get()?;
        pw.name("S")?;
        pw.ps_definefont()?;
        pw.ps_setfont()
    })?;
    pw.name("B")?;

    // /Cw{Cd A length 5 sub get}B
    pw.lit("Cw")?;
    pw.seq(|pw| {
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_length()?;
        pw.isize(5)?;
        pw.ps_sub()?;
        pw.ps_get()
    })?;
    pw.name("B")?;

    // /Ch{Cd A length 4 sub get}B
    pw.lit("Ch")?;
    pw.seq(|pw| {
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_length()?;
        pw.isize(4)?;
        pw.ps_sub()?;
        pw.ps_get()
    })?;
    pw.name("B")?;

    // /Cx{128 Cd A length 3 sub get sub}B
    pw.lit("Cx")?;
    pw.seq(|pw| {
        pw.isize(128)?;
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_length()?;
        pw.isize(3)?;
        pw.ps_sub()?;
        pw.ps_get()?;
        pw.ps_sub()
    })?;
    pw.name("B")?;

    // /Cy{Cd A length 2 sub get 127 sub}B
    pw.lit("Cy")?;
    pw.seq(|pw| {
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_length()?;
        pw.isize(2)?;
        pw.ps_sub()?;
        pw.ps_get()?;
        pw.isize(127)?;
        pw.ps_sub()
    })?;
    pw.name("B")?;

    // /Cdx{Cd A length 1 sub get}B
    pw.lit("Cdx")?;
    pw.seq(|pw| {
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_length()?;
        pw.isize(1)?;
        pw.ps_sub()?;
        pw.ps_get()
    })?;
    pw.name("B")?;

    // /Ci{Cd A type/stringtype ne{ctr get/ctr ctr 1 add N}if}B
    pw.lit("Ci")?;
    pw.seq(|pw| {
        pw.name("Cd")?;
        pw.name("A")?;
        pw.ps_type()?;
        pw.lit("stringtype")?;
        pw.ps_ne()?;
        pw.seq(|pw| {
            pw.name("ctr")?;
            pw.ps_get()?;
            pw.lit("ctr")?;
            pw.name("ctr")?;
            pw.isize(1)?;
            pw.ps_add()?;
            pw.name("N")
        })?;
        pw.ps_if()
    })?;
    pw.name("B")?;

    /* /CharBuilder{save 3 1 roll S A/base get 2 index get S
    /BitMaps get S get/Cd X pop/ctr 0 N Cdx 0 Cx Cy Ch sub Cx Cw add Cy
    setcachedevice Cw Ch true[1 0 0 -1 -.1 Cx sub Cy .1 sub]{Ci}imagemask
    restore}B*/
    pw.lit("CharBuilder")?;
    pw.seq(|pw| {
        pw.ps_save()?;
        pw.isize(3)?;
        pw.isize(1)?;
        pw.ps_roll()?;
        pw.name("S")?;
        pw.name("A")?;
        pw.lit("base")?;
        pw.ps_get()?;
        pw.isize(2)?;
        pw.ps_index()?;
        pw.ps_get()?;
        pw.name("S")?;
        pw.lit("BitMaps")?;
        pw.ps_get()?;
        pw.name("S")?;
        pw.ps_get()?;
        pw.lit("Cd")?;
        pw.name("X")?;
        pw.ps_pop()?;
        pw.lit("ctr")?;
        pw.isize(0)?;
        pw.name("N")?;
        pw.name("Cdx")?;
        pw.isize(0)?;
        pw.name("Cx")?;
        pw.name("Cy")?;
        pw.name("Ch")?;
        pw.ps_sub()?;
        pw.name("Cx")?;
        pw.name("Cw")?;
        pw.ps_add()?;
        pw.name("Cy")?;
        pw.ps_setcachedevice()?;
        pw.name("Cw")?;
        pw.name("Ch")?;
        pw.bool(true)?;

        // [1 0 0 -1 -.1 Cx sub Cy .1 sub]
        pw.arr(|pw| {
            pw.isize(1)?;
            pw.isize(0)?;
            pw.isize(0)?;
            pw.isize(-1)?;
            pw.double(-0.1)?;
            pw.name("Cx")?;
            pw.ps_sub()?;
            pw.name("Cy")?;
            pw.double(0.1)?;
            pw.ps_sub()
        })?;
        // {Ci}
        pw.seq(|pw| pw.name("Ci"))?;
        pw.ps_imagemask()?;
        pw.ps_restore()
    })?;
    pw.name("B")?;

    /*/D{/cc X A type/stringtype ne{]}if nn/base get cc ctr put nn
    /BitMaps get S ctr S sf 1 ne{A A length 1 sub A 2 index S get sf div put
    }if put/ctr ctr 1 add N}B */
    pw.lit("D")?;
    pw.seq(|pw| {
        pw.lit("cc")?;
        pw.name("X")?;
        pw.name("A")?;
        pw.ps_type()?;
        pw.lit("stringtype")?;
        pw.ps_ne()?;
        pw.seq(|pw| pw.arr_close())?;
        pw.ps_if()?;
        pw.name("nn")?;
        pw.lit("base")?;
        pw.ps_get()?;
        pw.name("cc")?;
        pw.name("ctr")?;
        pw.ps_put()?;
        pw.name("nn")?;
        pw.lit("BitMaps")?;
        pw.ps_get()?;
        pw.name("S")?;
        pw.name("ctr")?;
        pw.name("S")?;
        pw.name("sf")?;
        pw.isize(1)?;
        pw.ps_ne()?;
        pw.seq(|pw| {
            pw.name("A")?;
            pw.name("A")?;
            pw.ps_length()?;
            pw.isize(1)?;
            pw.ps_sub()?;
            pw.name("A")?;
            pw.isize(2)?;
            pw.ps_index()?;
            pw.name("S")?;
            pw.ps_get()?;
            pw.name("sf")?;
            pw.ps_div()?;
            pw.ps_put()
        })?;
        pw.ps_if()?;
        pw.ps_put()?;
        pw.lit("ctr")?;
        pw.name("ctr")?;
        pw.isize(1)?;
        pw.ps_add()?;
        pw.name("N")
    })?;
    pw.name("B")?;

    // /I{cc 1 add D}B
    pw.lit("I")?;
    pw.seq(|pw| {
        pw.name("cc")?;
        pw.isize(1)?;
        pw.ps_add()?;
        pw.name("D")
    })?;
    pw.name("B")?;

    /* Beginning of a page
    /bop{userdict/bop-hook known{bop-hook}if/SI save N @rigin 0 0 moveto/V matrix currentmatrix A 1 get A
    mul exch 0 get A mul add .99 lt{/QV}{/RV}ifelse load def pop pop}N
    */
    pw.lit("bop")?;
    pw.seq(|pw| {
        // Call the bop-hook (if present)
        pw.ps_userdict()?;
        pw.lit("bop-hook")?;
        pw.ps_known()?;
        pw.seq(|pw| pw.name("bop-hook"))?;
        pw.ps_if()?;

        // Save VM state to /SI
        pw.lit("SI")?;
        pw.ps_save()?;
        pw.name("N")?;

        pw.name("@rigin")?;
        pw.isize(0)?;
        pw.isize(0)?;
        pw.ps_moveto()?;
        pw.lit("V")?;
        pw.ps_matrix()?;
        pw.ps_currentmatrix()?;
        pw.name("A")?;
        pw.isize(1)?;
        pw.ps_get()?;
        pw.name("A")?;
        pw.ps_mul()?;
        pw.ps_exch()?;
        pw.isize(0)?;
        pw.ps_get()?;
        pw.name("A")?;
        pw.ps_mul()?;
        pw.ps_add()?;
        pw.double(0.99)?;
        pw.ps_lt()?;
        pw.seq(|pw| pw.lit("QV"))?;
        pw.seq(|pw| pw.lit("RV"))?;
        pw.ps_ifelse()?;
        pw.ps_load()?;
        pw.ps_def()?;
        pw.ps_pop()?;
        pw.ps_pop()
    })?;
    pw.name("N")?;

    // /eop{SI restore userdict/eop-hook known{eop-hook}if showpage}N
    pw.lit("eop")?;
    pw.seq(|pw| {
        pw.name("SI")?;
        pw.ps_restore()?;
        pw.ps_userdict()?;
        pw.lit("eop-hook")?;
        pw.ps_known()?;
        pw.seq(|pw| pw.name("eop-hook"))?;
        pw.ps_if()?;
        pw.ps_showpage()
    })?;
    pw.name("N")?;

    /*
    /@start{userdict/start-hook known{start-hook}if pop/VResolution X/Resolution X
    1000 div/DVImag X/IEn 256 array N 2 string 0 1 255{IEn S A 360 add 36 4
    index cvrs cvn put}for pop 65781.76 div/vsize X 65781.76 div/hsize X}N
    */
    pw.lit("@start")?;
    pw.seq(|pw| {
        // Call start-hook (if present)
        pw.ps_userdict()?;
        pw.lit("start-hook")?;
        pw.ps_known()?;
        pw.seq(|pw| pw.name("start-hook"))?;
        pw.ps_if()?;

        pw.ps_pop()?;
        pw.lit("VResolution")?;
        pw.name("X")?;
        pw.lit("Resolution")?;
        pw.name("X")?;

        pw.isize(1000)?;
        pw.ps_div()?;
        pw.lit("DVImag")?;
        pw.name("X")?;
        pw.lit("IEn")?;
        pw.isize(256)?;
        pw.ps_array()?;
        pw.name("N")?;
        pw.isize(2)?;
        pw.ps_string()?;
        pw.isize(0)?;
        pw.isize(1)?;
        pw.isize(255)?;
        pw.seq(|pw| {
            pw.name("IEn")?;
            pw.name("S")?;
            pw.name("A")?;
            pw.isize(360)?;
            pw.ps_add()?;
            pw.isize(36)?;
            pw.isize(4)?;
            pw.ps_index()?;
            pw.name("cvrs")?;
            pw.name("cvn")?;
            pw.ps_put()
        })?;
        pw.ps_for()?;
        pw.ps_pop()?;
        pw.double(65781.76)?;
        pw.ps_div()?;
        pw.lit("vsize")?;
        pw.name("X")?;
        pw.double(65781.76)?;
        pw.ps_div()?;
        pw.lit("hsize")?;
        pw.name("X")
    })?;
    pw.name("N")?;

    // /dir 0 def
    pw.lit("dir")?;
    pw.isize(0)?;
    pw.ps_def()?;

    // /dyy{/dir 0 def}B
    pw.lit("dyy")?;
    pw.seq(|pw| {
        pw.lit("dir")?;
        pw.isize(0)?;
        pw.ps_def()
    })?;
    pw.name("B")?;

    // /dyt{/dir 1 def}B
    pw.lit("dyt")?;
    pw.seq(|pw| {
        pw.lit("dir")?;
        pw.isize(1)?;
        pw.ps_def()
    })?;
    pw.name("B")?;

    // /dty{/dir 2 def}B
    pw.lit("dty")?;
    pw.seq(|pw| {
        pw.lit("dir")?;
        pw.isize(2)?;
        pw.ps_def()
    })?;
    pw.name("B")?;

    // /dtt{/dir 3 def}B
    pw.lit("dtt")?;
    pw.seq(|pw| {
        pw.lit("dir")?;
        pw.isize(3)?;
        pw.ps_def()
    })?;
    pw.name("B")?;

    // /p{dir 2 eq{-90 rotate show 90 rotate}{dir 3 eq{-90 rotate show 90 rotate}{show}ifelse}ifelse}N
    pw.lit("p")?;
    pw.seq(|pw| {
        pw.name("dir")?;
        pw.isize(2)?;
        pw.ps_eq()?;
        pw.seq(|pw| {
            pw.isize(-90)?;
            pw.ps_rotate()?;
            pw.ps_show()?;
            pw.isize(90)?;
            pw.ps_rotate()
        })?;
        pw.seq(|pw| {
            pw.name("dir")?;
            pw.isize(3)?;
            pw.ps_eq()?;
            pw.seq(|pw| {
                pw.isize(-90)?;
                pw.ps_rotate()?;
                pw.ps_show()?;
                pw.isize(90)?;
                pw.ps_rotate()
            })?;
            pw.seq(|pw| pw.ps_show())?;
            pw.ps_ifelse()
        })?;
        pw.ps_ifelse()
    })?;
    pw.name("N")?;

    // /RMat[1 0 0 -1 0 0]N
    pw.lit("RMat")?;
    pw.arr(|pw| {
        pw.isize(1)?;
        pw.isize(0)?;
        pw.isize(0)?;
        pw.isize(-1)?;
        pw.isize(0)?;
        pw.isize(0)
    })?;
    pw.name("N")?;

    pw.lit("BDot")?;
    pw.isize(260)?;
    pw.ps_string()?;
    pw.name("N")?;

    pw.lit("Rx")?;
    pw.isize(0)?;
    pw.name("N")?;

    pw.lit("Ry")?;
    pw.isize(0)?;
    pw.name("N")?;

    pw.lit("V")?;
    pw.seq(|_| Ok(()))?;
    pw.name("B")?;

    // /RV
    pw.lit("RV")?;

    // /v{/Ry X/Rx X V}B
    pw.lit("v")?;
    pw.seq(|pw| {
        pw.lit("Ry")?;
        pw.name("X")?;
        pw.lit("Rx")?;
        pw.name("X")?;
        pw.name("V")
    })?;
    pw.name("B")?;

    /* statusdict begin
    /product where{
        pop
        false
        [(Display)(NeXT)(LaserWriter 16/600)]
        {A length product length le{
            A
            length product exch 0 exch getinterval
            eq{pop true exit}if
        }{pop}ifelse}
    forall}{false}ifelse
    end*/
    pw.ps_statusdict()?;
    pw.begin(|pw| {
        pw.lit("product")?;
        pw.ps_where()?;
        pw.seq(|pw| {
            pw.ps_pop()?;
            pw.bool(false)?;
            pw.arr(|pw| {
                pw.bytes(b"Display")?;
                pw.bytes(b"NeXT")?;
                pw.bytes(b"LaserWriter 16/600")
            })?;
            pw.seq(|pw| {
                pw.name("A")?;
                pw.ps_length()?;
                pw.name("product")?;
                pw.ps_length()?;
                pw.ps_le()?;
                pw.seq(|pw| {
                    pw.name("A")?;
                    pw.ps_length()?;
                    pw.name("product")?;
                    pw.ps_exch()?;
                    pw.isize(0)?;
                    pw.ps_exch()?;
                    pw.ps_getinterval()?;
                    pw.ps_eq()?;
                    pw.seq(|pw| {
                        pw.ps_pop()?;
                        pw.bool(true)?;
                        pw.ps_exit()
                    })?;
                    pw.ps_if()
                })?;
                pw.seq(|pw| pw.ps_pop())?;
                pw.ps_ifelse()
            })?;
            pw.ps_forall()
        })?;
        pw.seq(|pw| pw.bool(false))?;
        pw.ps_ifelse()
    })?;

    /*
    {{gsave TR -.1 .1 TR 1 1 scale Rx Ry false RMat{BDot}imagemask grestore}}
    {{gsave TR -.1 .1 TR Rx Ry scale 1 1 false RMat{BDot}imagemask grestore}}ifelse B
    */
    pw.seq(|pw| {
        pw.seq(|pw| {
            pw.ps_gsave()?;
            pw.name("TR")?;
            pw.double(-0.1)?;
            pw.double(0.1)?;
            pw.name("TR")?;
            pw.isize(1)?;
            pw.isize(1)?;
            pw.ps_scale()?;
            pw.name("Rx")?;
            pw.name("Ry")?;
            pw.bool(false)?;
            pw.name("RMat")?;
            pw.seq(|pw| pw.name("BDot"))?;
            pw.ps_imagemask()?;
            pw.ps_grestore()
        })
    })?;
    pw.seq(|pw| {
        pw.seq(|pw| {
            pw.ps_gsave()?;
            pw.name("TR")?;
            pw.double(-0.1)?;
            pw.double(0.1)?;
            pw.name("TR")?;
            pw.name("Rx")?;
            pw.name("Ry")?;
            pw.ps_scale()?;
            pw.isize(1)?;
            pw.isize(1)?;
            pw.bool(false)?;
            pw.name("RMat")?;
            pw.seq(|pw| pw.name("BDot"))?;
            pw.ps_imagemask()?;
            pw.ps_grestore()
        })
    })?;
    pw.ps_ifelse()?;
    pw.name("B")?;

    /*
    /QV{
        gsave newpath transform round exch round exch itransform moveto
        Rx 0 rlineto 0 Ry neg rlineto Rx neg 0 rlineto fill grestore
    }B
    */
    pw.lit("QV")?;
    pw.seq(|pw| {
        pw.ps_gsave()?;
        pw.ps_newpath()?;
        pw.ps_transform()?;
        pw.ps_round()?;
        pw.ps_exch()?;
        pw.ps_round()?;
        pw.ps_exch()?;
        pw.ps_itransform()?;
        pw.ps_moveto()?;
        pw.name("Rx")?;
        pw.isize(0)?;
        pw.ps_rlineto()?;
        pw.isize(0)?;
        pw.name("Ry")?;
        pw.ps_neg()?;
        pw.ps_rlineto()?;
        pw.name("Rx")?;
        pw.ps_neg()?;
        pw.isize(0)?;
        pw.ps_rlineto()?;
        pw.ps_fill()?;
        pw.ps_grestore()
    })?;
    pw.name("B")?;

    // /a{moveto}B
    pw.lit("a")?;
    pw.seq(|pw| pw.ps_moveto())?; // moveto
    pw.name("B")?;

    // /delta 0 N
    pw.lit("delta")?;
    pw.isize(0)?;
    pw.name("N")?;

    // /tail{A/delta X 0 rmoveto}B
    pw.lit("tail")?;
    pw.seq(|pw| {
        pw.name("A")?; // dup
        pw.lit("delta")?; // /delta
        pw.name("X")?; // exch def
        pw.isize(0)?; // move right
        pw.ps_rmoveto()
    })?;
    pw.name("B")?;

    // /M{S p delta add tail}B
    pw.lit("M")?;
    pw.seq(|pw| {
        pw.name("S")?;
        pw.name("p")?;
        pw.name("delta")?;
        pw.ps_add()?;
        pw.name("tail")
    })?;
    pw.name("B")?;

    // /b{S p tail}B
    pw.lit("b")?;
    pw.seq(|pw| {
        pw.name("S")?; // exch
        pw.name("p")?; // show (rotate if dir set)
        pw.name("tail")
    })?;
    pw.name("B")?;

    let mut make_kern = |c, val| {
        pw.lit(c)?;
        pw.seq(|pw| {
            pw.isize(val)?;
            pw.name("M")
        })?;
        pw.name("B")
    };

    make_kern("c", -4)?; // /c{-4 M}B
    make_kern("d", -3)?; // /d{-3 M}B
    make_kern("e", -2)?; // /e{-2 M}B
    make_kern("f", -1)?; // /f{-1 M}B
    make_kern("g", 0)?; // /g{0 M}B
    make_kern("h", 1)?; // /h{1 M}B
    make_kern("i", 2)?; // /i{2 M}B
    make_kern("j", 3)?; // /j{3 M}B
    make_kern("k", 4)?; // /k{4 M}B

    // /w{0 rmoveto}B
    pw.lit("w")?;
    pw.seq(|pw| {
        pw.isize(0)?;
        pw.ps_rmoveto()
    })?;
    pw.name("B")?;

    let mut make_skip = |c, val| {
        pw.lit(c)?;
        pw.seq(|pw| {
            pw.name("p")?;
            pw.isize(val)?;
            pw.name("w")
        })?;
        pw.name("B")
    };

    make_skip("l", -4)?; // /l{p -4 w}B
    make_skip("m", -3)?; // /m{p -3 w}B
    make_skip("n", -2)?; // /n{p -2 w}B
    make_skip("o", -1)?; // /o{p -1 w}B
    make_skip("q", 1)?; // /q{p 1 w}B
    make_skip("r", 2)?; // /r{p 2 w}B
    make_skip("s", 3)?; // /s{p 3 w}B
    make_skip("t", 4)?; // /t{p 4 w}B

    // /x{0 S rmoveto}B
    pw.lit("x")?;
    pw.seq(|pw| {
        pw.isize(0)?;
        pw.name("S")?;
        pw.ps_rmoveto()
    })?;
    pw.name("B")?;

    // /y{3 2 roll p a}B
    pw.lit("y")?;
    pw.seq(|pw| {
        // move string up
        pw.isize(3)?;
        pw.isize(2)?;
        pw.ps_roll()?;

        pw.name("p")?; // print (show)
        pw.name("a") // moveto
    })?;
    pw.name("B")?;

    // /bos{/SS save N}B
    pw.lit("bos")?;
    pw.seq(|pw| {
        pw.lit("SS")?;
        pw.ps_save()?;
        pw.name("N")
    })?;
    pw.name("B")?;

    // /eos{SS restore}B
    pw.lit("eos")?;
    pw.seq(|pw| {
        pw.name("SS")?;
        pw.ps_restore()
    })?;
    pw.name("B")?;

    pw.name("end")?;
    Ok(())
}
