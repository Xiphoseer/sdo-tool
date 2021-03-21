# Character sets

This page contains information on Signum! character sets.

See also: [E24 Charset Format]({{ 'formats/eset' | relative_url}})

## ASH font discs

Application Systems sold a collection of font discs. The named ones were
professional creations, while the *Signum-Font-eXchange* (SiFoX) was set
up to redistribute user-generated fonts to all Signum licensees.

Some of these fonts ended up in disk images on the web. The following
pages use those to display a preview of the keyboard mapping using the
editor font. I'm not (re)distributing the actual font files on purpose.

<ul>
{% assign fdiscs = site.fdiscs | sort: "title" %}
{% for disc in fdiscs %}
<li><a href="{{ disc.url | relative_url }}">{{disc.title}}</a></li>
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