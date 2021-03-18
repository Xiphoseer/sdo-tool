# The Signum! image format (IMC)

Signum! comes with its very own compressed image format. As with the
other formats, this isn't documented anywhere but fortunately, I could
find some assembly that was originally disassembled from some other
program which works for standard 640x400 ST High monochrome images.

## Header

```
+-----+-----+-----+-----+-----+-----+-----+-----+
| "b" | "i" | "m" | "c" | "0" | "0" | "0" | "2" |
+-----+-----+-----+-----+-----+-----+-----+-----+
|    compressed size    | uc. width | u. height |
+-----+-----+-----+-----+-----+-----+-----+-----+
| h-#chunks | v-#chunks | bit-stream  byte-size |
+-----+-----+-----+-----+-----+-----+-----+-----+
| byte-stream byte-size | final XOR |    ???    |
+-----+-----+-----+-----+-----+-----+-----+-----+
|          ???          |          ???          |
+-----+-----+-----+-----+-----+-----+-----+-----+
```

## Chunks

The image is split into a number of 16x16 pixel chunks. In a standard ST High
resolution of 640x400, there are 40x25 chunks. The order of the chunks is
left-to-right, then top-to-bottom. While reading, each chunk is prepared in
a temporary buffer of 32 bytes. Here too, the bytes are stored left-to-right,
top-to-bottom, that is 2x16 bytes.

It's important to know that the output format of this is a raw monochrome
bitmap file, that has a fixed resolution (as specified in the header) where
1 means black (ink) and 0 means white (no-ink), as is the case for the ATARI
video RAM and the E24 font files.

## Bit-Stream vs. Byte-Stream

The actual image data consists of one bit-stream and one byte stream.
Both are used like iterators and random access (adresses into the buffer)
is meaningless from one image to the next. That is because the bits and
bytes are used as needed and one line of chunks may only require a single bit.

The basic algorithm (pseudocode) is this:

```rust
fn load_image() {
    for y in 0..vnum_chunks {
        if next_bit() {
            for x in 0..hnum_chunks {
                if next_bit() {
                    load_chunk(x, y)
                }
            }
        }
    }
}
```

If a chunk gets loaded, the next two bits represent a number from
0 to 3, which indicates how the chunk data is stored in the byte-stream.

Strategy 3 is just to take 32 bytes from the byte-stream and stick them
in the chunk in that order (every pair of two bytes is a row of 16 pixels,
for a total of 16 rows).

Strategies 0-2 introduce the idea of 8x8 pixel subchunks. The next four bits
indicate which sub-chunk is serialized:

```
+---+---+
| 1 | 2 |
+---+---+
| 3 | 4 |
+---+---+
```

For every chunk that is serialized, the next byte in the byte-stream is
yet another bitmap that indicates which of the 8 rows of the sub-chunk
are non-zero. For every bit in that byte (high to low significance)
that is set, load one more byte from the byte stream and load it into
the appropriate place of the chunk buffer.

When you implement this on a linear 32 byte chunk buffer, this looks
like sub-chunk 1 and 2 are interleaved and sub-chunk 3 and 4 are as well.

```
00..16: 10 20 11 21 12 22 13 23 14 24 15 25 16 26 17 27
00..32: 30 40 31 41 32 42 33 43 34 44 35 45 36 46 37 47
```

When that is loaded, strategy 0 doesn't do anything else. Strategy 1 takes
a two byte accumulator, initialized with the first two bytes, and for
every remaining byte pair, replaces the value with the XOR of that pair
and the accumulator and uses the result as the new accumulator.

Strategy 2 does the same as strategy 1, but with a 4 byte rolling XOR. The
effect of these transformations is that if the entire chunk is covered in
some sort of pattern, you can get away with storing less bytes per chunk,
because the XOR just recreates the pattern for you.

## Final XOR

When all chunks are loaded, every byte in every row of pixels with an even
index (assuming you count from 0) is XORed with the upper byte of the final
XOR header field and every byte in every row of pixels with an off index
is XORed with the lower byte of that header.

This has the same effect as the XOR strategies in the sub chunks, except
that it works on the entire screen and is thus suited to turn entire
backgrounds with some patterns into something that doesn't need to be stored.

Note: When the XOR header bytes are 0, applying an XOR does nothing,
so you can skip this step if that is the case.