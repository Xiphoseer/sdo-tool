# Prior Art

I'm not nearly the first person to have extracted data from Signum! files. In
fact, Application Systems regularily pointed people that had trouble with their
files to third-party developers to look into those problems. In this section,
I want to present all of the strategies and tools that I came across while
building this tool.

## 1. Buy and run Signum on an ATARI

One way that works is to find someone with a working Atari ST machine,
get the installation files from ASH, use or [buy a product key][Signum!] and load
up the document. You can then print the document to paper or save it as ASCII text.
You can even define which ASCII character to use for any given Signum Scancode.

Make sure to export any hardcopy-images embedded in the file and to convert them
to a more common format using something like zView.

## 2. Run Signum, use an emulator

You can also set-up an emulator and install Signum! there with a similar effect.
The major advantage is that you don't need actual Atari Hardware and that you
can redirect the *ESC/P* (Epson) or *PCL* (HP) printer commands to a file,
which can then be interpreted by a virtual printer to create a PDF or image.

You may need an original TOS ROM (or MagiC, EmuTOS didn't seem to work for me)
and that Signum!2 only prints in bitmap mode, so any character information
is lost this way too.

## 3. Print with Signum!4, in Draft mode

**Signum!4** features a "Draft"-Mode (under "Parameter" &rArr; "Druckqualität"
/ ^Q), which embeds the actual text in the printer output as opposed to printing
in graphics mode. I'm not sure whether it considers the ASCII mapping file(s).

While the positioning information is relatively accurate, not all font attributes
(e.g. tall, small) are supported.

If you set the output channel (*Ausgabekanal* / ^K) to file (*Druck-Datei*) and set
the driver to a PCL one like `HPDESKJ.TRB`, then you can convert the output
`PRDAT.OUT` to PDF via ghostpcl using a command like

```sh
gpcl6 -dNOPAUSE -sDEVICE=pdfwrite -sOutputFile=PRDAT.PDF PRDAT.OUT
```

## 4. Use a converter

I actually found a working copy of papyrus 7 (demo) after a lot of digging around
and it turns out that this actually just provides `TEXTCONV.PRG` by Andreas Pirner.
Version 1.23 is the one that comes with papyrus.

I was initially not convinced by the visual output of the produced RTF files,
but RTF is a semi-structured plain text format so it could be very useful to
get data out of Signum!2 files. It does seem to use some custom control sequences
that would only be supported by payprus e.g. `\signumpixels`, `\m`, or `\fsh`.

There's also the *SignumRead* (`S/MYUTIL/SIGNUMRE.M`) program from the
[Megamax Modula-2][MM2] development environment, which scrapes some text
from the SDO files.

## 5. Use an alternative

Finally, there's also [*FaaastPrint*][FPRINT], which is really similar to my tool
or the Signum!4 draft mode.

This program loads an SDO file and can generate "fast" printer output, i.e.
ESC/P or PCL commands that print characters with the default printer font. I've not
tried it, but it seems very customizable by way of the "Konvert" tool, but also
really hard to use, because every character in a charset needs a custom command.

[Signum!]: https://www.ashshop.biz/diverses/atari/textverarbeitung/874/signum-2-download
[MM2]: http://www.tempel.org/files-d.html#MM2
[FPRINT]: https://www.planetemu.net/rom/atari-st-applications-st/faaast-print-for-signum-files-19xx-ingo-sprick-de-2
