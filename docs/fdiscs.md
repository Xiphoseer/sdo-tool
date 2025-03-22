---
title: ASH Font Discs
layout: page
---

Application Systems sold a collection of font discs. The named ones were
professional creations, while the *Signum-Font-eXchange* (SiFoX) was set
up to redistribute user-generated fonts to Signum licensees.

## Disc index

{% assign groups = site.fdiscs | group_by:"series" %}
{% for group in groups %}
{% unless group.name == empty %}
<h3>{{group.name}}</h3>
{% endunless %}
{% for disc in group.items %}<a href="{{ disc.url | relative_url }}">{{ disc.link_name | default:disc.short }}</a>{% unless forloop.last %}, {% endunless %}{% endfor %}
{% endfor %}

### Signum-Font-eXchange (SiFoX)

By sending in a complete font of your own design for distribution through
SiFoX or paying 30,- DM, you would get one SiFoX-Disc of fonts for your
printer type (24 needle, 9 needle or laser) that you didn't have already.
There was a list of fonts available in the manual if you wanted a specific
one.

Some of these fonts ended up in disk images on the web. The following
pages use those to display a preview of the keyboard mapping using the
editor font. I'm not (re)distributing the actual font files on purpose.

### Copyright

Application Systems explicitly addresses the copyright of SiFoX fonts in
their second Design-Guide (*547 neue Signum! Zeichensätze - noch ein
Buch zur Gestaltung*):

> Eine oft gestellte Frage betrifft die Vielzahl an Fontdisketten, die
> von uns vertrieben werden. Es scheint nämlich nicht ganz klar zu sein,
> daß die Fonts ebenfalls einem Urheberrecht unterliegen. Weder die
> SiFoX-Fonts noch die professionellen Fontdisketten sind daher als
> public domain, also jedermann zugänglich zu betrachten.
> 
> Beim SiFoX müsste man im Zweifelsfalle jedem Pixelkünstler einzeln
> fragen, ob der Font kopiert werden darf. Klar steht nur fest, daß
> derjenige, der einen Font zum SiFoX einschickt, damit einverstanden
> ist, daß wir ihn in der festgelegten Form der SiFoX-Disketten
> vertreiben und die entsprechende Austausch-Gegenleistung liefern.

In other words, the font copyright remains with their authors, there's
a limited license granted to ASH to re-distribute the fonts under the
terms of the SiFoX itself, for use with Signum!.
