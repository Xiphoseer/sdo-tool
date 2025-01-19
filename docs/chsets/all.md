# All character sets

This page lists all character sets that appear somewhere on this site.

<div class="table-responsive">
<table style="width: 100%;">
<thead>
    <tr>
        <th>Key</th>
        <th>Name</th>
        <th>Disc</th>
        <th>Page</th>
    </tr>
</thead>
<tbody>
{% for font in site.chsets %}
{% assign key = font.disc %}
{% assign disk = site.fdiscs | where_exp:"disc","disc.short == key" | first %}
<tr>
    <td><code>{{font.name}}</code></td>
    <td>{{font.full_name}}</td>
    <td><a href="{{disk.url}}#{{font.name}}">{{ disk.short }}</a></td>
    <td>{{font.page}}</td>
</tr>
{% endfor %}
</tbody>
</table>
</div>
