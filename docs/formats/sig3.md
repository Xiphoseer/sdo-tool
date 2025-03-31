# Signum 3/4 Formate

From `EXPERTE/FORMATS.SDK`

## Kurzanleitung zum Verändern der Helpdateien FONTED.HLP und DOCED.HLP:

`@` in der ersten Spalte ist Fluchtsymbol gemäß:  
`@identifierHelp` für Identifier.  
`@+identifierHelp` auch für diesen Identifier (d.h dieselbe Helpinformation auch hierfür).  
`@@identifierHelp` für Unteridentifier. (Die Information für den übergeordneten Identifier endet hier nicht!).  
`@@+identifierHelp` auch für diesen Unteridentifier  
`@/` Beginn einer neuen Seite (neue Box).  
`@!` Kommentarzeile.  
`@stop` Ende der Helpinformation für Identifier.  
`@@stop` dito für Unteridentifier.

*identifier* ist dabei die vom System festgelegte Kennung für eine bestimmte Hilfestellung.
Hinter *identifier* dürfen höchstens blanks folgen! Die Namen der Identifier erhalten Sie,
wenn Sie den gewünschten Menüpunkt bei gedrückter SHIFT- und ALTERNATE-Taste an-
klicken.

Die Helpinformation eines Identifiers endet bei einem `@stop` oder einem neuen `@identifier`.
Ebenso endet die Helpinformation eines Unteridentifiers bei einem `@@stop` oder einem
neuen `@@identifier`.

Unteridentifier werden für den Fonteditor momentan nicht verwendet.

Beispiel:

```
@notfound
Hierzu gibt es keine weitere Hilfestellung.
Weitere Informationen zum Fonteditor finden
Sie im Kapitel IV des Handbuchs.
```

## Aufbau der Makro-Dateien.

Wenn Sie Makros programmiert haben, und deren Inhalt geändert werden soll, dann kön-
nen Sie sich ebenfalls eines ASCII-Editors bedienen. Der Aufbau der Makros ist dabei
wie folgt:

NAME: Sequenzen

Der Name ist die definierte zweibuchstabige Sequenz zum Auslösen des Makros, wie sie
bei der Programmierung gewählt wurde. **Sequenzen** steht für den Inhalt des Makros.
Diese können folgendes enthalten:

`\` Leitet eine Kommandosequenz ein.  
`\al` ALTERNATE-Taste. Beispiel: `\al\ta` = ALTERNATE-Tab.  
`\ba` BACKSPACE  
`\cd` CURSOR DOWN.  
`\ck` Controlkode folgt. Das nächste Zeichen ist als Controlkode zu interpretieren.  
`\cl` CURSOR RIGHT (!).  
`\cr` CURSOR LEFT (!).  
`\co` CONTROL-Taste. Beispiel: \co\ta = CONTROL-Tab.  
`\cu` CURSOR UP.  
`\de` Betätigung der DELETE-Taste.\esStart einer Escapesequenz.  
`\ho` HOME-Taste.  
`\in` INSERT-Taste.  
`\ls` SHIFT-Taste. Beispiel: `\ls\ho` = SHIFT-HOME oder `\ls\es` = SHIFT-ESCAPE oder `\ls\cu` = SHIFT-CURSOR UP. Wird auch immer vor Großbuchstaben eingefügt, die ganz normal auszugeben sind.  
`\re` RETURN-Taste.  
`\ta` TAB-Taste.  
`abcdef` Text **abcdef**, der ganz normal ausgegeben wird.

Beispiel:

```
ii: \esi\in\ck\ls*\cki\ckb
```

`ii:` bedeutet Makro `ii`.  
`\esi` steht für ESC i, also Indexeintrag auslösen.  
`\in` bedeutet Insert, in diesem Fall Einfügen des Indexwortes.  
`\ck\ls*` bedeutet Controlkode SHIFT-*, also Wort selektieren.  
`\cki` steht für Controlkode i, das markierte Wort wird mit dem Attribut *italic* versehen.  
`\ckb` heißt Controlkode b, in diesem Fall: Das selektierte Wort jetzt auch noch fett darstellen.

## Ändern der Kombitastenfunktion:

Wie so ziemlich alles in Signum läßt sich auch die Belegung der Kombitasten umdefinie-
ren. Mit einem ASCII-Editor können Sie beispielsweise die Datei `Oldtotli.ktl` im Ordner
Doced.sys öffnen und den Inhalt entsprechend verändern.
Das Format dieser Datei ist ziemlich einfach:

    Schalter, Char1, Char2, Ergebnis; Darstellung1, Darstellung2

Mit *Schalter* legen Sie fest, ob die Kombination an- oder ausgeschaltet ist. 1 steht da-
bei für an und 0 für aus.
Char1 ist das Kombizeichen 1, nach dessen Eingabe auf das zweiten Zeichen Char2 ge-
wartet wird. Das Ergebnis ist das aus der Kombination resultierende Zeichen.
Nach dem Strichpunkt folgt die Darstellung in der Dialogbox. Darstellung 1 ist das Son-
derzeichen, das normalerweise *Ergebnis* entspricht. Danach folgt ein Text, der vor die-
sem Sonderzeichen ausgegeben wird.
Char1, Char2, Ergebnis und Darstellung1 dürfen als Zeichen (mit vorangestelltem `c`), als
Dezimalzahl oder als Hexadezimalzahl (vorangestelltes `$`) eingegeben werden.

Beispiel:
```
1, c., ca, 134; $86, .a ->
```
`1` bedeutet, daß die Sequenz eingeschaltet ist.  
`c.` Bedeutet Character `.`, also das Zeichen Punkt soll kombiniert werden mit `ca`, also Character `a`.  
Das Ergebnis soll Zeichen Nummer 134 (dezimal) sein.  
In Hexadezimaler Angabe folgt hier `$86`, was ebenfalls 134 entspricht, um ein entsprechendes Zeichen in der Dialogbox darzustellen.  
Danach folgt noch `.a -> `, was einfach genauso in der Dialogbox erscheint.
