# Key Bindings

Almost all features of signum were available using key bindings or keyboard
shortcuts. This allowed complex interactions to be recorded, saved and re-used
as keyboard macros.

## Writing Mode

The behavior of writing "normal" text depends on the *writing mode* settings in the
*functions* menu:

<figure>
<img src="{% link /img/writing-mode.png %}">
<figcaption>"Funktionen" / "Schreibmodi:" (<code>SIGNUM2.PRG</code>)</figcaption>
</figure>

### Automatic Insertion

When *automatic insertion* (Autom. Einfügen) was active, typing a character would
move the rest of the line before inserting the character. If it was off, the
characters would be written on top of the existing line. Signum! allows for 0
offset between character positions, so characters can be stacked *on top* of each other.

### Automatic Line Feed

If the *automatic line feed* (Autom. Zeilenvorschub) was active and the space
bar was pressed at a position past the configured right margin of the page, Signum
would insert a new main line one *main line distance* below the current one, adjust
the horizontal position to the start (carriage return) of the line or the *tabulator*
/ *indent* and move the overhanging word to that line, if necessary.

### RETURN creates line

<kbd>RETURN</kbd> moves the cursor down by one *main line distance* in the same
way that the automatic line feed does. If the *RETURN creates line* mode is active,
this will insert that amount of new lines, otherwise, it would just move the cursor.

### RETURN creates paragraph

When *RETURN creates line* was active, the *RETURN creates paragraph* mode would
additionally mark the new line with the **paragraph** attribute.

### Indent to Cursor

When *indent to cursor* was activated, SIGNUM would remember the horizontal
offset of the current cursor position and use that for all subsequent line feeds.

## Cursor Position

The arrow keys (<kbd>&rarr;</kbd>, <kbd>&larr;</kbd>, <kbd>&uarr;</kbd>, <kbd>&darr;</kbd>)
along with the modifiers <kbd>CTRL</kbd>, <kbd>SHIFT</kbd>, <kbd>TAB</kbd> and <kbd>HOME</kbd>
were used to move the cursor around on the current page.

### Horizontal Movement

| Sequence | Effect |
|---|---|
| <kbd>&rarr;</kbd> | Move right by one *space width* |
| <kbd>CTRL</kbd><kbd>&rarr;</kbd> | Move right by 3/90 inches (3 microsteps) |
| <kbd>SHIFT</kbd><kbd>&rarr;</kbd> | Move right by 1/90 inches (1 microsteps) |
| <kbd>CTRL</kbd><kbd>SHIFT</kbd><kbd>&rarr;</kbd> | Move right to next char (if any), ignore index lines |
| <kbd>&larr;</kbd> | Move left by one *space width*, stop at left page margin |
| <kbd>CTRL</kbd><kbd>&larr;</kbd> | Move left by 3/90 inches (3 microsteps) |
| <kbd>SHIFT</kbd><kbd>&larr;</kbd> | Move left by 1/90 inches (1 microsteps) |
| <kbd>CTRL</kbd><kbd>SHIFT</kbd><kbd>&larr;</kbd> | Move left to next char (if any), ignore index lines |
| <kbd>SHIFT</kbd><kbd>HOME</kbd> | Move to the start of the current line or indent |
| <kbd>HOME</kbd> | Move after the last character in the current line, incl. index lines |
| <kbd>CTRL</kbd><kbd>HOME</kbd> | Move after the last character of the current word |
| <kbd>CTRL</kbd><kbd>SHIFT</kbd><kbd>HOME</kbd> | Move after to the end of the current character |
| <kbd>TAB</kbd> | Move the cursor to the next tabulator position, without moving text |
| <kbd>CTRL</kbd><kbd>TAB</kbd> | Move the cursor and text after it to the next tabulator position |
| <kbd>SHIFT</kbd><kbd>TAB</kbd> | Move the cursor to the previous tabulator position, without moving text |

### Vertical Movement

Vertical movement is always restricted to the current *text area* i.e. header, main content, or footer
of a page. If trying to navigate further, the curser will stop at the start/end of the area.

| Sequence | Effect |
|---|---|
| <kbd>&uarr;</kbd> | Move up by one *main line distance* but no further than the nearest main line |
| <kbd>CTRL</kbd><kbd>&uarr;</kbd> | Move up by one *index line distance* |
| <kbd>SHIFT</kbd><kbd>&uarr;</kbd> | Move up by 1/54 inches (1 line) |
| <kbd>CTRL</kbd><kbd>SHIFT</kbd><kbd>&uarr;</kbd> | Move up to the nearest non-empty line |
| <kbd>&darr;</kbd> | Move down by one *main line distance* but no further than the nearest main line |
| <kbd>CTRL</kbd><kbd>&darr;</kbd> | Move down by one *index line distance*  |
| <kbd>SHIFT</kbd><kbd>&darr;</kbd> | Move down by 1/54 inches (1 line) |
| <kbd>CTRL</kbd><kbd>SHIFT</kbd><kbd>&darr;</kbd> | Move down to the nearest non-empty line |

[working area]: ./documents.md#working-area

## Escape Sequences

There was a quick reference of the available escape sequences in the *Infos*
menu of Signum itself.

<figure>
<img src="{% link /img/escape-sequences.png %}">
<figcaption>"Escape-Sequenzen" (<code>SIGNUM2.PRG</code>)</figcaption>
</figure>

Some longer form descriptions were in the reference section (*Nachschlagteil*)
of *Das Signum! Buch* (see [ASH-Books](/signum/references#ash-books)).

| Sequence | Effect |
|---|---|
| <kbd>ESC</kbd><kbd>+</kbd> | Go to the next page (keyboard only, not numpad +) |
| <kbd>ESC</kbd><kbd>-</kbd> | Go to the previous page (keyboard only, not numpad -) |
| <kbd>ESC</kbd><kbd>a</kbd> | Sets the *paragraph marker* ("Absatz") |
| <kbd>ESC</kbd><kbd>A</kbd> | Unsets the *paragraph marker* ("Absatz") |
| <kbd>ESC</kbd><kbd>b</kbd> | Sets the *wide* ("Breit") [font-modifier] |
| <kbd>ESC</kbd><kbd>B</kbd> | Unsets the *wide* ("Breit") [font-modifier] |
| <kbd>ESC</kbd><kbd>c</kbd> | Sets the *cursive* ("Kursiv") [font-modifier] |
| <kbd>ESC</kbd><kbd>C</kbd> | Unsets the *cursive* ("Kursiv") [font-modifier] |
| <kbd>ESC</kbd><kbd>CTRL</kbd><kbd>&darr;</kbd> | Go to the next *paragraph marker* |
| <kbd>ESC</kbd><kbd>CTRL</kbd><kbd>&uarr;</kbd> | Go to the previous *paragraph marker* |
| <kbd>ESC</kbd><kbd>&darr;</kbd> | Go to the end of the page |
| <kbd>ESC</kbd><kbd>&uarr;</kbd> | Go to the start of the page |
| <kbd>ESC</kbd><kbd>d</kbd><kbd>i</kbd><kbd>5</kbd> | Set the *index line distance* (to any number, e.g. `5`) |
| <kbd>ESC</kbd><kbd>d</kbd><kbd>l</kbd><kbd>1</kbd><kbd>2</kbd> | Set the *main line distance* (to any 2-digit number, e.g. `12`) |
| <kbd>ESC</kbd><kbd>d</kbd><kbd>s</kbd><kbd>2</kbd> | Set the *blocking* ("Sperrung", to any number, e.g. `2`) |
| <kbd>ESC</kbd><kbd>d</kbd><kbd>w</kbd><kbd>0</kbd><kbd>8</kbd> | Set the *space width* (to any 2-digit number, e.g. `08`) |
| <kbd>ESC</kbd><kbd>e</kbd> | Set an indent |
| <kbd>ESC</kbd><kbd>E</kbd> | Unset an indent |
| <kbd>ESC</kbd><kbd>f</kbd> | Sets the *bold* ("Fett") [font-modifier] |
| <kbd>ESC</kbd><kbd>F</kbd> | Unsets the *bold* ("Fett") [font-modifier] |
| <kbd>ESC</kbd><kbd>g</kbd> | Sets the *tall* ("Gross") [font-modifier] |
| <kbd>ESC</kbd><kbd>G</kbd> | Unsets the *tall* ("Gross") [font-modifier] |
| <kbd>ESC</kbd><kbd>h</kbd> | Mark a *main line* ("Hauptzeile") |
| <kbd>ESC</kbd><kbd>H</kbd> | Unmark a *main line* |
| <kbd>ESC</kbd><kbd>k</kbd> | Sets the *small* ("Klein") [font-modifier] |
| <kbd>ESC</kbd><kbd>K</kbd> | Unsets the *small* ("Klein") [font-modifier] |
| <kbd>ESC</kbd><kbd>n</kbd> | Mark a *footnote* (with cursor on a number) |
| <kbd>ESC</kbd><kbd>4</kbd><kbd>A</kbd> | Insert a single character (e.g. `A`) from another font (e.g. the 4th one) |
| <kbd>ESC</kbd><kbd>s</kbd><kbd>0</kbd><kbd>1</kbd><kbd>5</kbd> | Go to page *nnn* (any 3 digit number, e.g. `015`) |
| <kbd>ESC</kbd><kbd>t</kbd> | Set the *text attribute* |
| <kbd>ESC</kbd><kbd>T</kbd> | Unset the *text attribute* |
| <kbd>ESC</kbd><kbd>u</kbd> | Sets the *underlined* ("-----") [font-modifier] |
| <kbd>ESC</kbd><kbd>U</kbd> | Unsets the *underlined* ("-----") [font-modifier] |
| <kbd>ESC</kbd><kbd>x</kbd> | Go to *header area* |
| <kbd>ESC</kbd><kbd>y</kbd> | Go to *footer area* |
| <kbd>ESC</kbd><kbd>z</kbd><kbd>3</kbd> | Change the active font (e.g. to the 3rd one) |
| <kbd>ESC</kbd><kbd>z</kbd><kbd>ALT</kbd><kbd>3</kbd> | Change the active <kbd>ALT</kbd> font (e.g. to the 3rd one) |
| <kbd>ESC</kbd><kbd>z</kbd><kbd>CTRL</kbd><kbd>3</kbd> | Change the active <kbd>CTRL</kbd> font (e.g. to the 3rd one) |

[font-modifier]: /signum/font-modifiers

## Function Keys

<figure>
<img src="{% link /img/function-keys.png %}">
<figcaption>"Funktionstasten" (<code>SIGNUM2.PRG</code>)</figcaption>
</figure>

| Sequence | Effect |
|---|---|
| <kbd>F1</kbd><kbd>a</kbd> | Call keyboard program / macro (e.g. `a`) |
| <kbd>F2</kbd> | Insert blank line at cursor (above current content) |
| <kbd>F3</kbd> | Delete line at cursor |
| <kbd>F4</kbd> | Insert blank line, move cursor |
| <kbd>F5</kbd> | Delete line above cursor |
| <kbd>F6</kbd> | Reserved (e.g. for accessories) |
| <kbd>F7</kbd> | Pull up a word from the next line |
| <kbd>CTRL</kbd><kbd>F7</kbd> | Pull up the next line |
| <kbd>F8</kbd> | Add current line to accumulator, move cursor to next line |
| <kbd>F9</kbd> | Insert the content of the accumulator at (above) the cursor |
| <kbd>SHIFT</kbd><kbd>F9</kbd> | Clear the accumulator |
| <kbd>F10</kbd> | Redraw the whole screen |
