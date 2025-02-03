# Examples

There is a `PHYSIK.SDO` example file that comes with Signum!2, which uses different
fonts, multiple formulas and some character formatting. We can use this example to
demonstrate how a signum document is layed out and printed.

## PDF

This is the document converted to a PDF (Portable Document Format) file. The
`--xoffset` / `--yoffset` flags were used to center the text on an A4 Page.

[PHYSIK.pdf](img/PHYSIK.pdf)  
[MUSTER.pdf](img/MUSTER.pdf)

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

## ESC/P

This is the same document but "printed" as ESC/P commands using Signums very
own `PR24_KAD.PRG` [printer-driver](/signum/printer-drivers) and rendered to
an image using the `vescp` virtual printer in SDO-Toolbox.

This uses the 360dpi mode to get the full resolution, `.2` inches top edge
and `1.1` inches left edge, to center the text on an A4 media box
(2988 &times; 4212 px at 360 dpi).

<figure>
    <img src="{{ 'img/physik-pr24_kad.png' | relative_url }}">
    <figcaption>Physik/PRN24_KAD</figcaption>
</figure>

## From the books

The Signum! books "zur Gestaltung" have a set of example pages within
them. This page collects some of them as a reference, even if the original
SDO is not available:

<ul>
{% for example in site.examples %}
<li><a href="{{example.url | relative_url}}">{{example.title}}</a></li>
{% endfor %}
</ul>
