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

The following image illustrates the effect of the different bold modes
on a glyph:

<figure>
<img src="{% link /img/bold-mode.png %}">
<figcaption>Bold Font ("Leicht" / "Normal", "Stark")</figcaption>
</figure>

The black character is the original glyph, the gray characters define
pixels that are on (ink'ed) additionally in each of the three modes.

## `-----` (underlined)

`-----` underlines a glyph.

## Kursiv (italic)

*Kursiv* modifies the glyph to lean forward 1:4

## Gross (tall)

*Gross* increases the height of a glyph by *1.5* (150%).

## Klein (small)

*Klein* reduces the height of a glyph by *0.75* (75%).
