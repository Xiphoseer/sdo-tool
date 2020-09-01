# The Signum! file format (SDO)

The Signum! word processor was a text editing application from the german software publisher
"Application Systems Heidelberg" (ASH), written for the ATARI ST. It was one of the most
popular word processors available for that system. This document is as far as I know the only
description of that file format that is available online.

## The container

Every SDO file starts with the bytes `73 64 6f 63`, that is `sdoc` in most ASCII-compatible
encodings, including UTF-8 and the [ATARI ST Character Set](https://en.wikipedia.org/wiki/Atari_ST_character_set).

Following this is a sequence of sub-files, that is a 4 byte lowercase alphanumeric name,
a 32bit big-endian length-specifier and then a binary section of that length. In pseudo-code,
that is

```
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

This section is usually 128 bytes long

### Character Sets `cset`

A sequence of NULL terminated strings that represent the names of the charsets used in the document.
The names have variable length and are always separated by two NULL bytes (not including the previous NULL terminator).

This section is usually 80 bytes long

### Unknown `sysp`

This section is usually 110 bytes long

### Page Buffer `pbuf`

This section contains information on the pages in the document. It contains the number of pages,
two unknown values (or some other 8 bytes), five times the tag `unde` in ASCII, which may or may
not be related to my documents using the german language and 34 bytes of information for every
page.

```
page_count = be_u32();
be_u32();
be_u32();
for i in 0..5 {
  tag("unde");
}
for p in 0..page_count {
  take(34);
}
```

The length of this section depends on the content

### Text Buffer `tebu`

This section contains the bulk of the document content. It is made up of a sequence of
drawing commands that usually correspond to a single character. Note that there is no
space *character*, instead the offset between characters is longer, wherever a space
character would be used in other encodings.

Commands seems to be always a multiple of two bytes wide. The following is what I know
about these commands:

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

### Unknown `hcim`

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