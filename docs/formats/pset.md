# The Printer Charset file format (P24, P9, L30)

The printer file format is a variant of the [Editor Charset file format (E24)](eset)
and, apart from the magic bytes at the start of the file is the same across all
printer types:

- `ps24` for the 24-needle printers (`*.P24` files)
- `ps09` for the 9-needle printers (`*.P9` files)
- `ls30` for the laser printers (`*.L30` files)

```
+-----+-----+-----+-----+
|  four character code  |
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

The header is followed by 128 bytes of *something*.

After that, there is a big-endian 32-bit byte-length specifier
for the character data buffer, which is the largest and last
part of the file.

After that length, there are 127 BE, 32bit offsets into the
character buffer, that correspond to the 127 characters. This
may be used for random access to any of the characters.

Finally, there is the character buffer itself, which is a
sequence of 4 byte character headers and a variable amount
of bitmap data.

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
of bitmap rows is `height`, `width` is the number of bytes in a row.
`top` indicates the distance of that bitmap from the upper edge of the line.

For the mapping between character codes and keys on the keyboard,
have a look at [the character sets page]({{ '/chsets' | relative_url }}).
