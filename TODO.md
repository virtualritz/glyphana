# Search

[x] Search all characters in the search string in the font.
[x] Use search string to find characters in the Adobe glyph database via `glyph-names` crate.
[x] Split search string by spaces and search each separately.
[ ] Allow selecting unicode categories via `finl_unicode` crate.
[ ] Allow fuzzy matching on character names via `ngrammatic` crate.
[ ] Compare/find matches via case folding/`focaccia` crate.
[ ] Use `unicode_normalization::char::decompose_canonical` to e.g. decompose `'Å'` into `['A','\u{30a}']` and use the first element for search.

# Result
[ ] Color resuly background by match %. Green = max match. Orange = min match.

# Left Panel

[x] Display selected char.
[ ] Convert case for selected char via `unicode-case-mapping` crate.
[ ] Display character variants in middle panel (including ligatures).
