# Font Modifiers

Signum supports 6 algorithmic font modifiers for every glphy:

- [*Breit* (wide)](#breit-wide)
- [*Fett* (bold)](#fett-bold)
- [`-----` (underlined)](#------underlined)
- [*Kursiv* (italic)](#kursiv-italic)
- [*Gross* (tall)](#gross-tall)
- [*Klein* (small)](#klein-small)

<figure>
<img src="{% link /img/fontmodf.png %}" width="300px">
<figcaption>Font Modifier Output</figcaption>
</figure>

Modifiers are selected from a toolbar at the bottom and can be
freely combined, except for the contradictory tall & small, which
increase and decrease glyph height respectively.

<figure>
<img src="{% link /img/fontmod-toolbar.png %}">
<figcaption>Font Modifier Toolbar (<code>SIGNUM2.PRG</code>)</figcaption>
</figure>

## Breit (wide)

*Breit* doubles the width of a glyph.

## Fett (bold)

The 24-needle printer driver has three modes for computing bold glyphs:

- *leicht* (Light)
- *normal*
- *stark* (strong)

<figure>
<img src="{% link /img/bold-prn24.png %}">
<figcaption>Bold Font Printer Mode (<code>PRN24.PRG</code>)</figcaption>
</figure>

The bold text in normal mode looks like the following:

<figure>
<img src="{% link /img/bold-normal.png %}">
<figcaption>Bold Font ("Normal")</figcaption>
</figure>

## `-----` (underlined)

`-----` underlines a glyph.

## Kursiv (italic)

*Kursiv* modifies the glyph to lean forward 1:4

## Gross (tall)

*Klein* increases the height of a glyph.

## Klein (small)

*Klein* reduces the height of a glyph.
