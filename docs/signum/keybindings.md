# Key Bindings

Almost all features of signum were available using key bindings or keyboard
shortcuts. This allowed complex interactions to be recorded, saved and re-used
as keyboard macros.

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
