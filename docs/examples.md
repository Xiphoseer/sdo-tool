# Examples

There is a `PHYSIK.SDO` example file that comes with Signum!2, which uses different
fonts, multiple formulas and some character formatting. We can use this example to
demonstrate how a signum document is layed out and printed.

## PDF

This is the document converted to a PDF (Portable Document Format) file. The
`--xoffset` / `--yoffset` flags were used to center the text on an A4 Page.

[PHYSIK.pdf](img/PHYSIK.pdf)

## Editor

This is a render of the document with the editor font, *without* skew compensation.
It has 108 DPI of vertical resolution and 90 DPI horizontal resolution, relative
to the theoretical paper model.

<figure>
    <img src="{{ 'img/physik-editor.png' | relative_url }}">
    <figcaption>Physik/Editor</figcaption>
</figure>

## Printer

This is a render of the document with a printer font, *with* skew-compensation,
so that the entire document is 324 DPI relative to the theoretical paper model.

<figure>
    <img src="{{ 'img/physik-printer.png' | relative_url }}">
    <figcaption>Physik/Printer</figcaption>
</figure>