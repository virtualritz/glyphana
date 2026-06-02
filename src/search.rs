use crate::categories::{Category, CharacterInspector};
use glyph_names;
use std::collections::BTreeMap;
use stringzilla::szs::{DeviceScope, LevenshteinDistancesUtf8};
use unicode_case_mapping;
use unicode_normalization::UnicodeNormalization;
use unicode_skeleton::UnicodeSkeleton;

// Helper function to normalize text for accent-insensitive matching
fn normalize_for_matching(s: &str) -> String {
    // First decompose Unicode characters (NFD normalization)
    // This separates base characters from their combining marks (accents)
    let decomposed: String = s.nfd().collect();

    // Remove combining marks (accents, diacritics)
    let without_accents: String = decomposed
        .chars()
        .filter(|&c| {
            // Keep base characters, remove combining marks
            // Combining marks are in the ranges:
            // U+0300–U+036F (Combining Diacritical Marks)
            // U+1AB0–U+1AFF (Combining Diacritical Marks Extended)
            // U+1DC0–U+1DFF (Combining Diacritical Marks Supplement)
            // U+20D0–U+20FF (Combining Diacritical Marks for Symbols)
            // U+FE20–U+FE2F (Combining Half Marks)
            let code = c as u32;
            !((0x0300..=0x036F).contains(&code)
                || (0x1AB0..=0x1AFF).contains(&code)
                || (0x1DC0..=0x1DFF).contains(&code)
                || (0x20D0..=0x20FF).contains(&code)
                || (0xFE20..=0xFE2F).contains(&code))
        })
        .collect();

    // Also try unicode_skeleton for additional normalization
    // This handles more complex character variants
    let skeleton = without_accents.skeleton_chars().collect::<String>();

    // Return the most normalized form
    if !skeleton.is_empty() {
        skeleton
    } else {
        without_accents
    }
}

// Helper functions to convert unicode-case-mapping results to strings
fn to_lowercase_string(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mapped = unicode_case_mapping::to_lowercase(c);
            let mut result = String::new();
            for &code in &mapped {
                if code != 0
                    && let Some(ch) = char::from_u32(code)
                {
                    result.push(ch);
                }
            }
            if result.is_empty() {
                result.push(c); // Maps to itself
            }
            result
        })
        .collect::<Vec<_>>()
        .join("")
}

pub struct SearchParams {
    pub text: String,
    pub split_text: Vec<String>,
    pub split_text_lower: Vec<String>,
    pub search_only_categories: bool,
    pub search_name: bool,
    pub case_sensitive: bool,
}

impl SearchParams {
    pub fn new(
        text: String,
        search_only_categories: bool,
        search_name: bool,
        case_sensitive: bool,
    ) -> Self {
        let split_text: Vec<String> = text.split_whitespace().map(str::to_string).collect();
        let split_text_lower: Vec<String> = if !case_sensitive {
            split_text.iter().map(|s| to_lowercase_string(s)).collect()
        } else {
            vec![]
        };

        Self {
            text,
            split_text,
            split_text_lower,
            search_only_categories,
            search_name,
            case_sensitive,
        }
    }
}

pub struct SearchEngine;

impl SearchEngine {
    pub fn search(
        params: &SearchParams,
        full_cache: &BTreeMap<char, String>,
        categories: &[Category],
        selected_category_id: egui::Id,
    ) -> BTreeMap<char, String> {
        // Early return for empty search
        if params.text.is_empty() {
            return full_cache.clone();
        }

        // Filter by categories if needed
        let base_cache = if params.search_only_categories {
            Self::filter_by_categories(full_cache, categories, selected_category_id)
        } else {
            full_cache.clone()
        };

        // Apply search filters
        let mut results = Self::apply_search_filters(base_cache, params);

        // Merge in special pattern matches (hex codes, decimal codes, etc.)
        if let Some(special) = Self::search_special_patterns(&params.text, full_cache) {
            results.extend(special);
        }

        results
    }

    fn search_special_patterns(
        text: &str,
        full_cache: &BTreeMap<char, String>,
    ) -> Option<BTreeMap<char, String>> {
        // Helper to create single character result
        let single_char_result = |chr: char, name: &String| -> BTreeMap<char, String> {
            let mut result = BTreeMap::new();
            result.insert(chr, name.clone());
            result
        };

        // Don't treat single character as special pattern if it's a regular letter
        // This allows normal case-sensitive/insensitive search to work
        if text.chars().count() == 1
            && let Some(chr) = text.chars().next()
        {
            // Skip special handling for alphabetic characters to allow case sensitivity
            if chr.is_alphabetic() {
                return None;
            }

            // For non-alphabetic single characters, try exact match
            if let Some(name) = full_cache.get(&chr) {
                return Some(single_char_result(chr, name));
            }

            // If no exact match, return all characters in the same Unicode block
            if let Some(block) = unicode_blocks::find_unicode_block(chr) {
                let results: BTreeMap<char, String> = full_cache
                    .iter()
                    .filter(|(chr, _)| {
                        let code = **chr as u32;
                        code >= block.start() && code <= block.end()
                    })
                    .map(|(chr, name)| (*chr, name.clone()))
                    .collect();

                if !results.is_empty() {
                    return Some(results);
                }
            }
        }

        // Check for hex code search (U+XXXX or 0xXXXX format)
        if let Some(chr) = Self::parse_hex_code(text)
            && let Some(name) = full_cache.get(&chr)
        {
            return Some(single_char_result(chr, name));
        }

        // Check for decimal code search
        if let Ok(code) = text.parse::<u32>()
            && let Some(chr) = char::from_u32(code)
            && let Some(name) = full_cache.get(&chr)
        {
            return Some(single_char_result(chr, name));
        }

        None
    }

    fn parse_hex_code(text: &str) -> Option<char> {
        let cleaned = to_lowercase_string(text.trim());

        // Try U+XXXX format
        if let Some(hex) = cleaned.strip_prefix("u+")
            && let Ok(code) = u32::from_str_radix(hex, 16)
        {
            return char::from_u32(code);
        }

        // Try 0xXXXX format
        if let Some(hex) = cleaned.strip_prefix("0x")
            && let Ok(code) = u32::from_str_radix(hex, 16)
        {
            return char::from_u32(code);
        }

        // Try plain hex (require at least one digit so purely alphabetic
        // strings like "ae", "face", "bad" fall through to name search)
        if cleaned.chars().all(|c| c.is_ascii_hexdigit())
            && cleaned.chars().any(|c| c.is_ascii_digit())
            && cleaned.len() <= 6
            && let Ok(code) = u32::from_str_radix(&cleaned, 16)
        {
            return char::from_u32(code);
        }

        None
    }

    fn filter_by_categories(
        cache: &BTreeMap<char, String>,
        categories: &[Category],
        selected_id: egui::Id,
    ) -> BTreeMap<char, String> {
        // Find selected category
        let selected_category = categories.iter().find(|cat| cat.id() == selected_id);

        if let Some(category) = selected_category {
            cache
                .iter()
                .filter(|(chr, _)| category.unicode_category.contains(**chr))
                .map(|(chr, name)| (*chr, name.clone()))
                .collect()
        } else {
            // If no valid category found, return full cache as fallback
            cache.clone()
        }
    }

    fn apply_search_filters(
        cache: BTreeMap<char, String>,
        params: &SearchParams,
    ) -> BTreeMap<char, String> {
        // If search_name is enabled, do fuzzy name search
        if params.search_name && !params.split_text.is_empty() {
            Self::fuzzy_search(cache, params)
        } else {
            // Otherwise do character-based skeleton search
            Self::skeleton_search(cache, params)
        }
    }

    fn fuzzy_search(
        cache: BTreeMap<char, String>,
        params: &SearchParams,
    ) -> BTreeMap<char, String> {
        const MAX_EDIT_DISTANCE: usize = 2;

        let device = DeviceScope::default().expect("failed to create stringzilla device scope");
        let levenshtein = LevenshteinDistancesUtf8::new(&device, 0, 1, 1, 1)
            .expect("failed to create levenshtein engine");
        let edit_distance = |a: &str, b: &str| -> usize {
            levenshtein
                .compute(&device, &[a], &[b])
                .map(|d| d[0])
                .unwrap_or(usize::MAX)
        };

        cache
            .into_iter()
            .filter(|(chr, name)| {
                // Also check if the character itself matches
                let chr_str = chr.to_string();
                if params.case_sensitive {
                    if chr_str.contains(&params.text) {
                        return true;
                    }
                } else {
                    // Try case-insensitive match
                    if to_lowercase_string(&chr_str).contains(&to_lowercase_string(&params.text)) {
                        return true;
                    }

                    // Try accent-insensitive match (e.g., 'a' matches 'à', 'á', 'â', etc.)
                    let normalized_char = normalize_for_matching(&chr_str);
                    let normalized_search = normalize_for_matching(&params.text);
                    if normalized_char.contains(&normalized_search) {
                        return true;
                    }
                }

                // Check name (Unicode and Adobe)
                let search_name = if params.case_sensitive {
                    name.clone()
                } else {
                    to_lowercase_string(name)
                };

                // Also get Adobe glyph name if available
                let adobe_name = glyph_names::glyph_name(*chr as u32).map(|n| {
                    if params.case_sensitive {
                        n.to_string()
                    } else {
                        to_lowercase_string(&n)
                    }
                });

                let search_terms = if params.case_sensitive {
                    &params.split_text
                } else {
                    &params.split_text_lower
                };

                // Check if all search terms match with fuzzy logic
                search_terms.iter().all(|term| {
                    // Handle common terminology mappings with fuzzy matching
                    // "umlaut" (and typos like "unlaut") -> "diaeresis" (for German umlauts like Ä, ö, ü)
                    let search_variations = if term.len() >= 5 && edit_distance(term, "umlaut") <= 1
                    {
                        // If it's close to "umlaut" (edit distance <= 1), search for both "diaeresis" and the original term
                        vec!["diaeresis".to_string(), term.clone()]
                    } else {
                        vec![term.clone()]
                    };

                    search_variations.iter().any(|search_term| {
                        // First try exact substring match anywhere in the Unicode name
                        if search_name.contains(search_term) {
                            return true;
                        }

                        // Try accent-insensitive matching by normalizing both strings
                        let normalized_name = normalize_for_matching(&search_name);
                        let normalized_term = normalize_for_matching(search_term);
                        if normalized_name.contains(&normalized_term) {
                            return true;
                        }

                        // Also check Adobe glyph name
                        if let Some(ref an) = adobe_name {
                            if an.contains(search_term) {
                                return true;
                            }

                            // Try accent-insensitive matching on Adobe name too
                            let normalized_adobe = normalize_for_matching(an);
                            if normalized_adobe.contains(&normalized_term) {
                                return true;
                            }
                        }

                        // Then try fuzzy match on individual words
                        search_name.split_whitespace().any(|word| {
                            if word.len() < 3 || search_term.len() < 3 {
                                // For very short strings, also check if word starts with term
                                word == search_term || word.starts_with(search_term)
                            } else {
                                // Use edit distance for longer strings
                                let distance = edit_distance(word, search_term);
                                // For case sensitive, require exact match or very close match
                                // but not just case differences
                                if params.case_sensitive && distance > 0 {
                                    // If they're the same when lowercased, it's just a case difference
                                    // which shouldn't match in case-sensitive mode
                                    if to_lowercase_string(word) == to_lowercase_string(search_term)
                                    {
                                        false
                                    } else {
                                        distance <= MAX_EDIT_DISTANCE
                                    }
                                } else {
                                    distance <= MAX_EDIT_DISTANCE
                                }
                            }
                        })
                    })
                })
            })
            .collect()
    }

    fn skeleton_search(
        cache: BTreeMap<char, String>,
        params: &SearchParams,
    ) -> BTreeMap<char, String> {
        if params.split_text.is_empty() {
            return cache;
        }

        cache
            .into_iter()
            .filter(|(chr, name)| {
                // Convert character to string for comparison
                let chr_str = chr.to_string();

                // Check character match
                let char_matches = if params.case_sensitive {
                    chr_str.contains(&params.text)
                } else {
                    // Case-insensitive match
                    let case_match =
                        to_lowercase_string(&chr_str).contains(&to_lowercase_string(&params.text));

                    // Accent-insensitive match
                    let accent_match = normalize_for_matching(&chr_str)
                        .contains(&normalize_for_matching(&params.text));

                    case_match || accent_match
                };

                // Check name match if enabled
                let name_matches = if params.search_name {
                    if params.case_sensitive {
                        name.contains(&params.text)
                    } else {
                        // Case-insensitive match
                        let case_match =
                            to_lowercase_string(name).contains(&to_lowercase_string(&params.text));

                        // Accent-insensitive match
                        let accent_match = normalize_for_matching(name)
                            .contains(&normalize_for_matching(&params.text));

                        case_match || accent_match
                    }
                } else {
                    false
                };

                char_matches || name_matches
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glyph::char_name;
    use std::collections::BTreeMap;

    fn create_test_cache() -> BTreeMap<char, String> {
        let mut cache = BTreeMap::new();

        // Add some test characters with names
        cache.insert('A', "Latin Capital Letter A".to_string());
        cache.insert('a', "Latin Small Letter a".to_string());
        cache.insert('-', "Hyphen Minus".to_string()); // Change to match word boundary better
        cache.insert('‐', "Hyphen".to_string());
        cache.insert('\u{ad}', "Soft Hyphen".to_string());
        cache.insert('—', "Em Dash".to_string());
        cache.insert('–', "En Dash".to_string());
        cache.insert('α', "Greek Small Letter Alpha".to_string());
        cache.insert('Α', "Greek Capital Letter Alpha".to_string());
        cache.insert('β', "Greek Small Letter Beta".to_string());
        cache.insert('😀', "Grinning Face".to_string());
        cache.insert('🔍', "Magnifying Glass Tilted Left".to_string());
        cache.insert(' ', "Space".to_string());
        cache.insert('\u{00A0}', "No-break Space".to_string());
        cache.insert('1', "Digit One".to_string());
        cache.insert('2', "Digit Two".to_string());
        cache.insert('+', "Plus Sign".to_string());
        cache.insert('=', "Equals Sign".to_string());

        cache
    }

    #[test]
    fn empty_search_returns_all() {
        let cache = create_test_cache();
        let params = SearchParams::new("".to_string(), false, false, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), cache.len());
        assert_eq!(results, cache);
    }

    #[test]
    fn single_character_exact_match() {
        let cache = create_test_cache();
        let params = SearchParams::new("A".to_string(), false, false, true);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));
    }

    #[test]
    fn case_insensitive_character_search() {
        let cache = create_test_cache();
        let params = SearchParams::new("a".to_string(), false, false, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find both 'A' and 'a' when case insensitive
        assert!(results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
    }

    #[test]
    fn case_sensitive_character_search() {
        let cache = create_test_cache();
        let params = SearchParams::new("a".to_string(), false, false, true);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should only find 'a' when case sensitive
        assert!(!results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
    }

    #[test]
    fn search_by_name_substring() {
        let cache = create_test_cache();
        let params = SearchParams::new("hyphen".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find all hyphen-related characters
        assert!(results.contains_key(&'-')); // Hyphen Minus
        assert!(results.contains_key(&'‐')); // Hyphen
        assert!(results.contains_key(&'\u{ad}')); // Soft Hyphen
        assert!(!results.contains_key(&'—')); // Em Dash (doesn't contain "hyphen")
    }

    #[test]
    fn search_by_name_case_sensitive() {
        let cache = create_test_cache();

        // Test with correct case "Greek"
        let params = SearchParams::new("Greek".to_string(), false, true, true);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find Greek letters (name contains "Greek")
        assert!(results.contains_key(&'α'));
        assert!(results.contains_key(&'Α'));
        assert!(results.contains_key(&'β'));

        // Test that lowercase "greek" doesn't match when case sensitive
        let params_lower = SearchParams::new("greek".to_string(), false, true, true);
        let results_lower = SearchEngine::search(&params_lower, &cache, &[], egui::Id::new("test"));
        assert_eq!(results_lower.len(), 0);
    }

    #[test]
    fn search_by_name_case_insensitive() {
        let cache = create_test_cache();
        let params = SearchParams::new("greek".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find Greek letters regardless of case
        assert!(results.contains_key(&'α'));
        assert!(results.contains_key(&'Α'));
        assert!(results.contains_key(&'β'));
    }

    #[test]
    fn hex_code_search() {
        let cache = create_test_cache();

        // Test U+ format
        let params = SearchParams::new("U+0041".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));

        // Test 0x format
        let params = SearchParams::new("0x41".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));

        // Test plain hex
        let params = SearchParams::new("41".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));
    }

    #[test]
    fn decimal_code_search() {
        let cache = create_test_cache();

        // 65 is the decimal code for 'A'
        let params = SearchParams::new("65".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));
    }

    #[test]
    fn multiple_word_search() {
        let cache = create_test_cache();
        let params = SearchParams::new("latin letter".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find Latin letters
        assert!(results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
        // Should not find Greek letters or other characters
        assert!(!results.contains_key(&'α'));
        assert!(!results.contains_key(&'-'));
    }

    #[test]
    fn fuzzy_search_with_typo() {
        let cache = create_test_cache();
        // "hypen" is 1 edit away from "hyphen" (missing 'h')
        let params = SearchParams::new("hypen".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should still find hyphen-related characters due to fuzzy matching
        assert!(results.contains_key(&'-')); // Hyphen Minus
        assert!(results.contains_key(&'‐')); // Hyphen
        assert!(results.contains_key(&'\u{ad}')); // Soft Hyphen
    }

    #[test]
    fn search_emoji() {
        let cache = create_test_cache();

        // Search by emoji character
        let params = SearchParams::new("😀".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'😀'));

        // Search by emoji name
        let params = SearchParams::new("grinning".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert!(results.contains_key(&'😀'));
    }

    #[test]
    fn search_special_characters() {
        let cache = create_test_cache();

        // Search for space
        let params = SearchParams::new("space".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert!(results.contains_key(&' '));
        assert!(results.contains_key(&'\u{00A0}')); // No-break Space
    }

    #[test]
    fn search_with_name_disabled() {
        let cache = create_test_cache();

        // With search_name disabled, "hyphen" should not find anything
        // (since no character is literally the string "hyphen")
        let params = SearchParams::new("hyphen".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn search_partial_word_match() {
        let cache = create_test_cache();

        // Search for "mag" should find "Magnifying Glass"
        let params = SearchParams::new("mag".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert!(results.contains_key(&'🔍'));
    }

    #[test]
    fn combined_flags() {
        let cache = create_test_cache();

        // Case sensitive + search names for "latin" (lowercase)
        let params = SearchParams::new("latin".to_string(), false, true, true);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 0); // "latin" lowercase won't match "Latin" in names

        // Case sensitive + search names for "Latin" (correct case)
        let params = SearchParams::new("Latin".to_string(), false, true, true);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert_eq!(results.len(), 2); // Should find 'A' and 'a' (both have "Latin" in name)
        assert!(results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
    }

    #[test]
    fn search_mathematical_symbols() {
        let cache = create_test_cache();

        // Search for plus sign
        let params = SearchParams::new("+".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert!(results.contains_key(&'+'));

        // Search by name
        let params = SearchParams::new("plus".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        assert!(results.contains_key(&'+'));
    }

    #[test]
    fn umlaut_search() {
        let mut cache = BTreeMap::new();

        // Add German umlauts with their proper Unicode names
        cache.insert('Ä', "LATIN CAPITAL LETTER A WITH DIAERESIS".to_string());
        cache.insert('ä', "LATIN SMALL LETTER A WITH DIAERESIS".to_string());
        cache.insert('Ö', "LATIN CAPITAL LETTER O WITH DIAERESIS".to_string());
        cache.insert('ö', "LATIN SMALL LETTER O WITH DIAERESIS".to_string());
        cache.insert('Ü', "LATIN CAPITAL LETTER U WITH DIAERESIS".to_string());
        cache.insert('ü', "LATIN SMALL LETTER U WITH DIAERESIS".to_string());

        // Also add regular diaeresis character
        cache.insert('¨', "DIAERESIS".to_string());

        // Search for "umlaut" should find all German umlauts
        let params = SearchParams::new("umlaut".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find all German umlauts
        assert!(results.contains_key(&'Ä'), "Should find Ä");
        assert!(results.contains_key(&'ä'), "Should find ä");
        assert!(results.contains_key(&'Ö'), "Should find Ö");
        assert!(results.contains_key(&'ö'), "Should find ö");
        assert!(results.contains_key(&'Ü'), "Should find Ü");
        assert!(results.contains_key(&'ü'), "Should find ü");

        // Should also find the diaeresis character itself
        assert!(results.contains_key(&'¨'), "Should find diaeresis");
    }

    #[test]
    fn umlaut_search_with_typos() {
        let mut cache = BTreeMap::new();

        // Add German umlauts with their proper Unicode names
        cache.insert('Ä', "LATIN CAPITAL LETTER A WITH DIAERESIS".to_string());
        cache.insert('ä', "LATIN SMALL LETTER A WITH DIAERESIS".to_string());
        cache.insert('Ö', "LATIN CAPITAL LETTER O WITH DIAERESIS".to_string());
        cache.insert('ö', "LATIN SMALL LETTER O WITH DIAERESIS".to_string());
        cache.insert('Ü', "LATIN CAPITAL LETTER U WITH DIAERESIS".to_string());
        cache.insert('ü', "LATIN SMALL LETTER U WITH DIAERESIS".to_string());

        // Test various typos of "umlaut"
        let typos = vec!["unlaut", "umaut", "umlat", "umlauts"];

        for typo in typos {
            let params = SearchParams::new(typo.to_string(), false, true, false);
            let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

            // Should find German umlauts even with typos
            assert!(results.contains_key(&'Ä'), "Should find Ä for '{}'", typo);
            assert!(results.contains_key(&'ä'), "Should find ä for '{}'", typo);
            assert!(results.contains_key(&'Ö'), "Should find Ö for '{}'", typo);
            assert!(results.contains_key(&'ö'), "Should find ö for '{}'", typo);
            assert!(results.contains_key(&'Ü'), "Should find Ü for '{}'", typo);
            assert!(results.contains_key(&'ü'), "Should find ü for '{}'", typo);
        }
    }

    #[test]
    fn search_small_finds_small_letters() {
        let cache = create_test_cache();
        // search_name=true, case_sensitive=false
        let params = SearchParams::new("small".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // "small" should match names like "Latin Small Letter a", "Greek Small Letter Alpha", etc.
        assert!(
            results.contains_key(&'a'),
            "Should find 'a' (Latin Small Letter a)"
        );
        assert!(
            results.contains_key(&'α'),
            "Should find 'α' (Greek Small Letter Alpha)"
        );
        assert!(
            results.contains_key(&'β'),
            "Should find 'β' (Greek Small Letter Beta)"
        );
        assert!(!results.contains_key(&'A'), "Should NOT find 'A' (Capital)");
    }

    #[test]
    fn search_ae_finds_ae_ligature() {
        let mut cache = create_test_cache();
        cache.insert('æ', "Latin Small Letter Ae".to_string());
        cache.insert('Æ', "Latin Capital Letter Ae".to_string());

        let params = SearchParams::new("ae".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert!(
            results.contains_key(&'æ'),
            "Should find 'æ' (Latin Small Letter Ae)"
        );
        assert!(
            results.contains_key(&'Æ'),
            "Should find 'Æ' (Latin Capital Letter Ae)"
        );
    }

    /// Test with real char_name() output to catch name format mismatches
    #[test]
    fn search_with_real_char_names() {
        let mut cache = BTreeMap::new();
        // Use actual char_name() to get the names the app would use
        for ch in ['a', 'A', 'æ', 'Æ', 'α', 'β', '+', '1'] {
            cache.insert(ch, char_name(ch));
        }

        // Debug: print actual names
        for (ch, name) in &cache {
            eprintln!("  {:?} => {:?}", ch, name);
        }

        // "small" with name search should find lowercase letters
        let params = SearchParams::new("small".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        eprintln!(
            "Results for 'small': {:?}",
            results.keys().collect::<Vec<_>>()
        );
        assert!(
            !results.is_empty(),
            "search for 'small' should not be empty"
        );
        assert!(
            results.contains_key(&'a'),
            "Should find 'a' — name is {:?}",
            cache.get(&'a')
        );

        // "ae" with name search should find æ
        let params = SearchParams::new("ae".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));
        eprintln!("Results for 'ae': {:?}", results.keys().collect::<Vec<_>>());
        assert!(
            results.contains_key(&'æ'),
            "Should find 'æ' — name is {:?}",
            cache.get(&'æ')
        );
    }
}
