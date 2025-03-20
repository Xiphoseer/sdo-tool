# Monospace character sets

This page lists all character sets marked with `monospace: true` (unproportional)

{% assign fonts = site.chsets | where_exp:"f","f.monospace" %}
{% include chset-list.html chsets=fonts %}
