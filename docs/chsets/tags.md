# Tagged Charsets

This page collects the *tags* of the fonts from the "Anwendungsverzeichnis" of the Signum! books "zur Gestaltung".

<ul>
{% for tag in site.data.tags %}
<li><a href="#{{tag.key}}">{{ tag.title | default:tag.key }}</a>
{% endfor %}
</ul>

{% for tag in site.data.tags %}
{% include tag-list.html tag=tag %}
{% endfor %}
