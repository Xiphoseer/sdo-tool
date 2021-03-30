# The Editor Charset file format (E24)

Signum! has their very own bitmapped font format which is not documented
anywhere that I could find. The basic layout is this:

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

For the mapping between character codes and keys on the keyboard,
have a look at [the character sets page]({{ '/chsets' | relative_url }}).