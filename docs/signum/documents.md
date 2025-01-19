# Signum! Document

This page contains general information on the Signum! doucments as they are
presented to the users. To learn more about the file format, have a look
a [this page](/formats/sdoc).

## Document

A document is made up of a sequence of pages. All of these have an internal
index that never changes once a page is added, so deleted pages appear as
gaps in a lookup-by-index table.

Each page also has a physical page number (a one-based ordinal for its
position in the current file) and a logical page number (one-based ordinal
for its position in a multi-file document).

## Page

The page content is placed on a relatively granular grid, with different
vertical and horizontal resolution. Every unit of vertical resolution (line)
represents 1/54 of an inch, every unit of horizontal resolution (unnamed)
represents 1/90 of an inch.

In the editor, the height of a single line is two pixels and one horizontal
unit is a single pixel. That's a resolution of 108 dots-per-inch (DPI)
vertically and 90 dpi horizontally. It also means that a box that would appear
square in print will be less wide than tall in the editor. The characters
come from the `*.E24` font files and are up to 24px high and 16px wide (not
including modifiers).

The printing resolution depends on the kind of printer available, each of
which have their respective associated font formats. The 24-needle printers
use the `*.P24` font files, which have a vertical resolution of 72px. Thus,
they have a vertical resolution of 6 pixels per line or 324 DPI. At this
resolution, the mismatch between vertical and horizontal units is easily
compensated for.

<figure>
    <img src="../img/layout-settings.png">
    <figcaption>Layout Settings</figcaption>
</figure>