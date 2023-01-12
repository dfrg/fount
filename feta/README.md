# feta

This crate provides a high level interface for reading *f*ont m*eta*data. It 
is built on top of the [read-fonts] low level parsing library.

This is still in very early stages and not yet ready for use.

## Features

The first cut of the library intends to expose the following metadata:

* Variation axes and named instances
    * Conversion to normalized coordinates
* Global font metrics with variation support (units per em, ascender, descender, etc)
* Glyph metrics with variation support (advance width, left side-bearing, etc)
* Codepoint to nominal glyph identifier mapping
    * Unicode variation selectors

Future goals include:

* Localized strings
* Layout feature enumeration
    * Coverage (is a particular glyph processed by a feature?)
* Color palettes
* Bitmap strikes

## Non-goals

This library is not intended to support glyph scaling (loading and hinting of outlines) or
shaping (processing of subsitution and positioning features).

[read-fonts]: https://github.com/googlefonts/fontations/tree/main/read-fonts
