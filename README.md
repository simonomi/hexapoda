# hexapoda

a colorful modal hex editor

(the name comes from [the subphylum](https://wikipedia.org/wiki/Hexapoda))

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

- inserting bytes
	- only replacing and deleting right now
- custom keybinds
- search
- diffing
