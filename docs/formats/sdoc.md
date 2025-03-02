# The Signum! file format (SDO)

The Signum! word processor was a text editing application from the german software publisher
"Application Systems Heidelberg" (ASH), written for the ATARI ST. It was one of the most
popular word processors available for that system. This document is as far as I know the only
description of that file format that is available online.

All of the code snippets in this document are simplified pseudo-code, even though they
are inspired by the `nom` parser-combinator library. For general information on Signum!
documents, have a look at [this page]({{'/signum/documents' | relative_url }}).

## The container

Every SDO file starts with the bytes `73 64 6f 63`, that is `sdoc` in most ASCII-compatible
encodings, including UTF-8 and the [ATARI ST Character Set](https://en.wikipedia.org/wiki/Atari_ST_character_set).

Following this is a sequence of sub-files, that is a 4 byte lowercase alphanumeric name,
a 32bit big-endian length-specifier and then a binary section of that length. In pseudo-code,
that is

```rust
tag("sdoc");
while len > 0 {
  key = take(4);
  len = be_u32();
  bytes = take(len);
}
```

## The individual parts

The first section in all the files that I have available has the name `0001`. I assume that is
intended to be a file format version number at the same time.

The sections are:

### Version 1 Header `0001`

This section is mostly zeros, with the creation date at offset 72 (`$48`) and the modified
date at offset 76 (`$4c`). Both are given as two `WORD`s (i.e. `u16` / 2 bytes) representing
date and time respectively, that have the same layout as returned by the GEMDOS functions
[`Tgetdate`] and [`Tgettime`].

[`Tgetdate`]: https://freemint.github.io/tos.hyp/en/gemdos_datetime.html#Tgetdate
[`Tgettime`]: https://freemint.github.io/tos.hyp/en/gemdos_datetime.html#Tgettime

```rust
take(72)
created.date = be_u16()
created.time = be_u16()
modified.date = be_u16()
modified.time = be_u16()
```

This section is usually 128 bytes long

### Character Sets `cset`

This section is an array of 8 times 10 bytes, each holding a zero-terminated
character-set name. I've found some documents where the first slot is empty,
so you alway

```rust
for i in 0..8 {
  let bytes = take(10);
  chsets[i] = zt_string(bytes);
}
```

This section is usually 80 bytes long

### System parameters `sysp`

This section contains information on default page parameters
as well as general formatting options.

```rust
take(50); // unknown

space_width    = be_u16();  // Leerzeichenbreite
letter_spacing = be_u16();  // Sperrung
line_distance  = be_u16();  // Hauptzeilenabstand
index_distance = be_u16();  // Indexabstand
margin_left    = be_u16();  // Linker Rand (0)
margin_right   = be_u16();  // Rechter Rand (6.5 * 90)
header         = be_u16();  // Kopfzeilen (0.1 * 54)
footer         = be_u16();  // Fußzeilen (0.1 * 54)
page_length    = be_u16();  // Seitenlänge (10.4 * 54)

page_numbering = bytes16(); // 0x5800 == keine Seitennummerierung
format_options = bytes16(); // 0b10011 == format. optionen

bytes16();                  // 0x302 == trennen
bytes16();                  // 0 == Randausgleiche und Sperren
bytes32();                  // 1 == nicht einrücken, Absatzabstand mitkorrigieren
```

This section is usually 110 bytes long

### Page Buffer `pbuf`

This section contains information on the pages in the document. It contains the number of pages,
two unknown values (or some other 8 bytes), five times the tag `unde` in ASCII, which may or may
not be related to my documents using the german language and 34 bytes of information for every
page.

```rust
page_count = be_u32();
be_u32(); // called "kl" in some places, possibly length of each entry
first_page_nr = be_u32();
for i in 0..5 {
  tag("unde");
}
for p in 0..page_count {
  index = be_u16();
  physical_page_nr = be_u16();
  logical_page_nr = be_u16();
  
  take(2);

  margin_left = be_u16();
  margin_right = be_u16(); // from the left
  margin_top = be_u16();
  margin_bottom = be_u16();

  take(18);
}
```

The length of this section depends on the content

### Text Buffer `tebu`

This section contains the bulk of the document content. It is made up of *lines*, which
correspond to the vertical alignment from top to bottom. It starts with one u32, which
is supposed to be the total line count or total height of the document (?).

The rest of this section is a sequence of lines, with the following layout:

```rust
vskip = be_u16();
length = be_u16();
content = take(length);
```

#### Lines

Each `content` starts with a 16 bit identifier, that is probably a bitfield:

- 0x0001: prefixed with a 16 bit value, possibly `hskip`
- 0x0080: prefixed with 16 bit page number
- 0x0400: standard line (Hauptzeile)
- 0x0800: paragraph 
- 0x1000: non-text content
- 0x2000: page-end
- 0x4000: page-start
- 0x8000: page-command (always set for start and end)

These are the only combinations I have seen used in documents:
0x0000, 0x0400, 0x0401, 0x0800, 0x0C00, 0x0C01,
0x1000, 0x1400, 0x1C00, 0xA000, 0xA080, 0xC000, 0xC080

#### Characters

Every non-page-command can be followed by some amount of *characters*. Note that there
is no space *character*, instead the offset between characters is longer, wherever a space
character would be used in other encodings. Characters are 2 bytes wide by default and
use the following encoding:

If the first bit is set, then the command is a standard character and the next 6 bits
encode the offset from the previous drawing position. The last bit has some other function,
possibly related to the charset used.

If the first bit is not set, the command is 4 bytes long and the last two bytes encode
the offset value in big endian. If the second bit is set, the character is underlined.

The second byte is always the character. The highest bit of the character is the lower
bit of the selected charset. The last bit of the first byte is the high bit of the
selected charset.


**Normal character**:
```
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+  
| 1 |         OFFSET        | CHSET |          CHARACTER        |  
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
```

**Extended character**
```
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+  
| 0 | U | V | W | X | Y |   CHSET   |          CHARACTER        |  
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+  
| B | F | C | G | K |                 OFFSET                    |  
+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
```

They map to the [font-modifiers](../signum/font-modifiers.md) as follows:

|---|---|---|---|
| `B` | wide | `U` | underlined |
| `F` | bold | `V` | *unknown* (mark 1 ?) |
| `C` | italic | `W` | *unknown* (mark 2 ?) |
| `G` | tall | `X` | *unknown* (mark 3 ?) |
| `K` | small | `Y` | footnote |

### Hardcopy Images `hcim`

This sections contains information on the images embedded in the document.

```rust
site_tbl_len = be_u32(); // == offset to image table
img_count = be_u16();
site_count = be_u16();
take(8)
for i in 0..site_count {
  site[i].page = be_u16();
  site[i].pos_x = be_u16();
  site[i].pos_y = be_u16();
  site[i].site_w = be_u16();
  site[i].site_h = be_u16();
  be_u16();
  site[i].sel_x = be_u16();
  site[i].sel_y = be_u16();
  site[i].sel_w = be_u16();
  site[i].sel_h = be_u16();
  be_u16();
  be_u16();
  be_u16();
  site[i].img = be_u16();
  be_u16();
  bytes16();
}
for i in 0..img_count {
  buf_len = be_u32()
  name_bytes = take(28);
  img[i].name = zt_string(name_bytes);
  img[i].bytes = take(buf_len - 32)
}
```

The `bytes` of an image correspond to a `bimc` encoded file without the leading `bimc0002`
magic bytes.

This section seems optional

This section is usually 16 bytes long

### Unknown `pl01`

This section seems optional

This section is usually 0 bytes long

### Unknown `syp2`

This section seems optional

This section is usually 64 bytes long

## References

- <https://github.com/ggnkua/Atari_ST_Sources/tree/master/C/Sig_unit>
- <http://stcarchiv.de/stc1989/02/von-1stword-zu-signum2>

## Potentially useful links

- <https://en.wikipedia.org/wiki/Atari_ST_character_set>
- <http://www.atarimania.com/st/files/scarabus_purix.pdf>
- <http://xchem.de/publication/atari/info_e.html>
- <http://www.stcarchiv.de/stc1989/04/signum-fontutilities>
- <http://www.tug.org/tetex/html/fontfaq/cf_89.html>