# Character sets

This page contains information on Signum! character sets.

One Signum! document could have up to 7 so-called *character sets*
or *chsets* for short. Each character set is a collection of up to
127 characters which are indentified by a 7-bit number.

These character sets were identified by a filename without the
extension (e.g. `ANTIKRO`), which was used to find the correspoding
editor (`E24`) and printer (`P24`, `P09`, `L30`) font files.

See also: [E24 Font Format]({{ 'formats/eset' | relative_url}})

## Key codes

Other than their graphical representation, these charsets did not
carry any information about the meaning of each glyph. The users
relied on the fact that the glyph in the E24 file that was
printed on screen matched the glyph in the printer font file at
the same position.

The following images show an Atari ST keyboard with every key
that can produce a Signum! character marked with *H/L* corresponding
to the hexadecimal value for that key used in the font files.

For example, the uppercase letter *A* is on the Button next to *Control*
if *Shift* is held. It has the code `$41` in ASCII and Signum and is
marked as 4 (high) over 1 (low) in this diagram.

![]({{ 'img/kbNUMS.png' | relative_url }})
![]({{ 'img/npNUMS.png' | relative_url }})

<!--See also: [Font Mappings](font-mappings.html)-->

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

{::options parse_block_html="true" /}
<div class="table-responsive keytable">

|HL|_0 |_1 |_2 |_3 |_4 |_5 |_6 |_7 |_8 |_9 |_A |_B |_C |_D |_E |_F |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|0_|   | (Z| )Z| /Z| *Z| 0Z| 1Z| 2Z| 3Z| 4Z| 5Z| 6Z| 7Z| 8Z| 9Z| (z|
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|1_| )z| /z| *z| 0z| 1z| 2z| 3z| 4z| 5z| 6z| 7z| 8z| 9z| +z| -z| .z|
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|2_| § | ! | " | # | $ | % | & | ' | ( | ) | * | + | , | - | . | / |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|3_| 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | : | ; | < | = | > | ? |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|4_| ü | A | B | C | D | E | F | G | H | I | J | K | L | M | N | O |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|5_| P | Q | R | S | T | U | V | W | X | Y | Z | ö | Ü | ä | ^ | _ |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|6_| ` | a | b | c | d | e | f | g | h | i | j | k | l | m | n | o |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
|7_| p | q | r | s | t | u | v | w | x | y | z | Ö |\| | Ä | ~ | ß |
|--|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|

</div>

## ASH font discs

Application Systems sold a collection of font discs. The named ones were
professional creations, while the *Signum-Font-eXchange* (SiFoX) was set
up to redistribute user-generated fonts to Signum licensees.

By sending in a complete font of your own design for distribution through
SiFoX or paying 30,- DM, you would get one SiFoX-Disc of fonts for your
printer type (24 needle, 9 needle or laser) that you didn't have already.
There was a list of fonts available in the manual if you wanted a specific
one.

Some of these fonts ended up in disk images on the web. The following
pages use those to display a preview of the keyboard mapping using the
editor font. I'm not (re)distributing the actual font files on purpose.

<ul>
{% assign fdiscs = site.fdiscs | sort: "sort-key" %}
{% for disc in fdiscs %}
<li><a href="{{ disc.url | relative_url }}">{{ disc.name | default:disc.short }}</a></li>
{% endfor %}
</ul>

---

Some non-standard fonts that I came across are listed [here](other). There's
also a [list of all charsets](all) and a [list of characters not in Unicode](missing).

## Links

### Delta Labs

- Disk 1 of 074: <http://downloads.atari-home.de/Public_Domain/Serie_Delta-Labs/>
- <https://www.deltalabs.biz/atari-whiteline-cd-gamma.htm>

### Misc

- <https://www.atariuptodate.de/en/10975/signum-zeichensaetze-fuer-24-nadeldrucker>
- <https://www.atariuptodate.de/en/10910/signum-fonts>
- <https://www.atariuptodate.de/en/11907/signum-fonts-krani>

### Tools

- <https://www.atariuptodate.de/de/4550/sig-pic>
- <https://www.atariuptodate.de/de/10974/sigtogem>

### ST Scancodes

- <https://freemint.github.io/tos.hyp/en/scancode.html>
- <https://www.stcarchiv.de/stc1987/12/neue-tastaturbelegung-in-modula-2>
- <https://temlib.org/AtariForumWiki/index.php/Atari_ST_Scancode_diagram_by_Unseen_Menace>
- <https://www.jchr.be/atari/omikron.html#inkey>