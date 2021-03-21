# All character sets

This page lists all character sets that appear somewhere on this site.

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
<tr>
    <td><code>{{font.name}}</code></td>
    <td>{{font.full_name}}</td>
    <td>{{font.disc}}</td>
    <td>{{font.page}}</td>
</tr>
{% endfor %}
</tbody>
</table>