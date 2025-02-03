# Printer Drivers

The printer drivers for Signum!2 were essential in adapting its documents
to a wide variety of printer hardware. Specific font files were used for
different printer types:

- `*.P24` for 24-needle dot matrix printers,
- `*.P9` for 9-needle printers, and
- `*.L30` for laser printers.

Users were instructed to rename the printer driver that matched their
setup to `SPRINT.PRG`, so that it could be invoked directly from the editor.

## ESC/P

Printing at the time involved sending ASCII text to a character device. Additional
commands could be sent using `ESC` (ASCII 27) sequences, as specified by
[ESC/P](https://en.wikipedia.org/wiki/ESC/P). Signum used the `ESC *` command
in particular to print in graphics mode.

## 24-Needle

The default 24-needle printer driver was `PR24N.PRG`. This driver intentionally
skipped the second-to-last pixel of every row of pixels in a glyph, because in
360 dpi horizontal graphics mode, it was not possible to print adjacent dots
and Signum wanted to ensure the right edge of each glyph got printed accurately.

There was a `PR24_KAD.PRG` variant that did not skip this pixel, for use with
inkjet printers that did not have this limitation.

<figure>
<img src="{% link /img/pr24n-quality.png %}">
<figcaption>Print Quality / Druckqualität (<code>PR24N.PRG</code>)</figcaption>
</figure>

The print dialog provided several customization options, allowing users to select
between different ESC/P command variants based on their printer's capabilities.
This included options for absolute positioning or 1/360th inch line movements,
required for the 360dpi vertical resolution mode.

<figure>
<img src="{% link /img/pr24n-printer.png %}">
<figcaption>Printer Type / Druckertyp (<code>PR24N.PRG</code>)</figcaption>
</figure>

<figure>
<img src="{% link /img/pr24n-paper.png %}">
<figcaption>Paper Settings / Papierart (<code>PR24N.PRG</code>)</figcaption>
</figure>

## Laserprinter

The printer driver for ATARI laster printers was `PRATL.PRG`. The printer
driver for HP-LaserJet and compatible printers was `PR30L.PRG`.

## 9-Needle

The printer driver for 9-needle printers was `PR9N.PRG`.

## Spooler, Premul

To further enhance usability, Signum!2 offered a spooler and programs that
could redirect print jobs to the ATARI’s serial (RS232) or parallel
(Centronics) ports, or save output as a .PRD file for later printing,
allowing for seamless integration with various printing setups.

- `PEMV24.PRG` to redirect to the RS232 port
- `PEM_CENT.PRG` to redirect to the parallel port
- `PREMUL.PRG` to redirect to a `*.PRD` file
