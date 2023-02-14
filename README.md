# Glyphana

[![Build Status](https://github.com/virtualritz/glyphana/workflows/CI/badge.svg)](https://github.com/virtualritz/glyphana/actions?workflow=CI)

Glyphana is a tool to search for and discover unicode glyphs.

It is inspired by the [macOS Character Viewer](https://support.apple.com/guide/mac-help/use-emoji-and-symbols-on-mac-mchlp1560/mac)
which I sorely miss on Linux and Windows.

![Searching for the characters contained in 'grinning cat' as well as matching
the term against each glyp's description](screenshot.png)

## Caveat

This was hacked over the course of a few free hours here and there.

It currently is limited to most of the functionality I personally miss from
Character Viewer. I.e. it is far from feature parity with this tool.

It also lacks the abilit of Windows Character Map to compose strings. A feature
I miss from Character Viewer and plan to add.

## Installation & Updates

For now you need to have [Rust installed](https://www.rust-lang.org/tools/install).

From a terminal (command line) run:

```
cargo install glyphana
```

Make sure you have `cargo-update` installed.
You only need to do that once:

```
cargo install cargo-update
```

After that, updating is as simple as:

```
cargo install-update glyphana
```

## Features

* Copy an individual character to the clipboard.

### Inspection

* Inspect individual characters (show name, Unicode, UTF-8).
* Copy Unicode as hex in HTML format to the clipboard,
* Copy UTF-8 as hex to the clipboard.
* Store character in a persistent collection.

### Browsing

* View recently inspected characters.
* View collected characters.
* Browse characters by categories.

### Search

* Search for individual characters.
  * Consider case.
* Search against Unicode character name.
* Search against the Adobe glyph database.


## Contributing

Features and bug fixes are welcome.

If you want to add a feature, check the [to-do list](TODO.md) first.

If there is something you like working
on, create an issue to later put your PR against. If it is something else not
in the list also create an issue to collect feedback before you start working
on it.

Before you do  your final commit that precedes your PR make sure that

```
cargo +nightly check --all-features
cargo +nightly fmt --all -- --check
cargo +nightly clippy -- -D warnings
```

all come out clean.

## License

**Apache 2.0** or **MIT** or **BSD 3 Clause**; at your discretion.

Glyphana currently uses (and embeds) solely [Noto family](https://fonts.google.com/noto)
fonts which are under the **Open Font License**.

For licenses used by dependencies see [licenses.html](licenses.html).
