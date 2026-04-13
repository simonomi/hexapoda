# <img height=40 align=top src="https://github.com/simonomi/hexapoda/blob/main/icon/bug%20colored%20large.png?raw=true"> hexapoda

a colorful modal hex editor

(the name comes from [the subphylum](https://en.wikipedia.org/wiki/Hexapoda))

[![asciicast](https://asciinema.org/a/fsVwqdn846Ar5CQZ.svg)](https://asciinema.org/a/fsVwqdn846Ar5CQZ)

## status

currently, hexapoda is very unpolished, and missing some major features. if you'd be interested in using it, please let me know! if enough people want, i'd be willing to make it more accessible and write some docs

## features

- [color-codes bytes](https://simonomi.dev/blog/color-code-your-bytes) by value
- modal editing
	- selection-first, like [Kakoune](https://kakoune.org) and [Helix](https://helix-editor.com)
- multiple selections
	- split selection(s) into #-byte chunks
- undo/redo
- inspect the current selection(s)
	- signed, unsigned, fixed-point, UTF-8, color
- mark notable offsets
- jump to selected offset

### notable features that are missing (for now)

- search
- diffing
- inserting bytes
	- only replacing and deleting right now
