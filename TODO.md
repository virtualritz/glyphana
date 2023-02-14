# Search Field

* [x] Search all characters in the search string in the font.
* [x] Use search string to find characters in the Adobe glyph database
      via the `glyph-names` crate.
* [x] Split search string by spaces and use each substring separately
      for search.
* [ ] Allow selecting unicode categories via `finl_unicode` crate.
* [ ] Allow fuzzy matching on character names via `ngrammatic` crate.
* [ ] Compare/find matches via case folding/`focaccia` crate.
* [ ] Use `unicode_normalization::char::decompose_canonical` to e.g.
      decompose `'Å'` into `['A','\u{30a}']` and use the first element
      for search.

# Left Panel

* [x] Display selected char.
* [ ] Convert case for selected char via `unicode-case-mapping` crate.
* [ ] Display character variants in middle panel (including ligatures).

# Middle Panel

* [ ] Color background of each glyph by match %. Green = max match.
      Orange = min match.

# Right Panel

* [ ] Display horizontal glyph metrics
* [ ] Fix width of Unicode/UTF-8 area (egui issue)
* [ ] Block resizing area if character name, Unicode/UTF-8 and
      Collect button would be pushed outside of window bounds
