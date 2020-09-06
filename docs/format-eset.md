# The Editor Charset file format (E24)

Signum! has their very own bitmapped font format which is not documented
anywhere that could be found. The basic layout is this:

```
+-----+-----+-----+-----+
| "e" | "s" | "e" | "t" |
+-----+-----+-----+-----+
| "0" | "0" | "0" | "1" |
+-----+-----+-----+-----+
|       font type       |
+-----+-----+-----+-----+
```

For all fonts I have tried, the font type is `0x00000010`,
which is big-endian for 128. That corresponds to the
default 7-bit character fonts used in Signum! 1/2.

## 127-character fonts

The header is followed by 128 bytes of *something*, which may or may
not be related to kerning or guide-rules at font creation.

After that, there is a big-endian 32-bit byte-length specifier
for the character data buffer, which is the largest and last
part of the file.

After that length, there are 127 BE, 32bit offsets into the
character buffer, that correspond to the 127 characters. This
may be used for random access to any of the characters.

Finally, there is the character buffer itself, which is a
sequence of 4 byte character headers and a variable amount
of two-byte character bitmap rows.

```
+--------+--------+
|  top   | height |
+--------+--------+
| width  |  ???   |
+--------+--------+
|        .        |
+        .        +
|        .        |
+-----------------+
```

That last character header byte is probably just padding. The number
of bitmap rows is `height`; `top` indicates the distance of that bitmap
from the upper edge of a 24 pixel line and `width` is the space
that should be reserved/skipped to move past this character and draw
the next one.

## Character Encoding

See also: [Font Mappings](font-mappings.html)

The standard notation to talk about Signum! font character identity
was introduced by the program itself and its "ASCII" compatibility mode.
A single letter corresponds to the character you get when pressing
that key on an ATARI keyboard, a `z` after a number indicates that
it's the variant from the numpad and a `Z` indicates that it's the
shift of that variant.

It is basically going backwards from a font file through the
keyboard mapping in the image above and checking which key would
appear on that spot in a normal text input on a standard german
keyboard, like [this one](https://commons.wikimedia.org/wiki/File:Atari_1040_STE.jpg).

To turn this into something useful, you may need to

* figure out what scancode these characters represent to simulate
  the Signum! input method.
* Find the ATARI-ASCII version of these base characters to decode
  an actual `ASCIO.TAB` file, that is saved on some floppy disk.

```
   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
   |_0 |_1 |_2 |_3 |_4 |_5 |_6 |_7 |_8 |_9 |_A |_B |_C |_D |_E |_F |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|0_|   | (Z| )Z| /Z| *Z| 0Z| 1Z| 2Z| 3Z| 4Z| 5Z| 6Z| 7Z| 8Z| 9Z| (z|
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|1_| )z| /z| *z| 0z| 1z| 2z| 3z| 4z| 5z| 6z| 7z| 8z| 9z|   |   |   |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|2_| § | ! | " | # | $ | % | & | ´ | ( | ) | * | + | , | - | . | / |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|3_| 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | : | ; | < | = | > | ? |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|4_| ü | A | B | C | D | E | F | G | H | I | J | K | L | M | N | O |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|5_| P | Q | R | S | T | U | V | W | X | Y | Z | ö | Ü | ä | ^ | _ |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|6_| ` | a | b | c | d | e | f | g | h | i | j | k | l | m | n | o |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
|7_| p | q | r | s | t | u | v | w | x | y | z | Ö | | | Ä | ~ | ß |
+--+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
```