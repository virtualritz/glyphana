use crate::categories::{Category, CharacterInspector};
use std::collections::BTreeMap;
use stringzilla::StringZilla;

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
            split_text.iter().map(|s| s.to_lowercase()).collect()
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

        // Check for special search patterns
        if let Some(results) = Self::search_special_patterns(&params.text, full_cache) {
            return results;
        }

        // Filter by categories if needed
        let base_cache = if params.search_only_categories {
            Self::filter_by_categories(full_cache, categories, selected_category_id)
        } else {
            full_cache.clone()
        };

        // Apply search filters
        Self::apply_search_filters(base_cache, params)
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
        if text.chars().count() == 1 {
            if let Some(chr) = text.chars().next() {
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
        }

        // Check for hex code search (U+XXXX or 0xXXXX format)
        if let Some(chr) = Self::parse_hex_code(text) {
            if let Some(name) = full_cache.get(&chr) {
                return Some(single_char_result(chr, name));
            }
        }

        // Check for decimal code search
        if let Ok(code) = text.parse::<u32>() {
            if let Some(chr) = char::from_u32(code) {
                if let Some(name) = full_cache.get(&chr) {
                    return Some(single_char_result(chr, name));
                }
            }
        }

        None
    }

    fn parse_hex_code(text: &str) -> Option<char> {
        let cleaned = text.trim().to_lowercase();

        // Try U+XXXX format
        if let Some(hex) = cleaned.strip_prefix("u+") {
            if let Ok(code) = u32::from_str_radix(hex, 16) {
                return char::from_u32(code);
            }
        }

        // Try 0xXXXX format
        if let Some(hex) = cleaned.strip_prefix("0x") {
            if let Ok(code) = u32::from_str_radix(hex, 16) {
                return char::from_u32(code);
            }
        }

        // Try plain hex
        if cleaned.chars().all(|c| c.is_ascii_hexdigit()) && cleaned.len() <= 6 {
            if let Ok(code) = u32::from_str_radix(&cleaned, 16) {
                return char::from_u32(code);
            }
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

        cache
            .into_iter()
            .filter(|(chr, name)| {
                // Also check if the character itself matches
                let chr_str = chr.to_string();
                if params.case_sensitive {
                    if chr_str.contains(&params.text) {
                        return true;
                    }
                } else if chr_str.to_lowercase().contains(&params.text.to_lowercase()) {
                    return true;
                }

                // Check name
                let search_name = if params.case_sensitive {
                    name.clone()
                } else {
                    name.to_lowercase()
                };

                let search_terms = if params.case_sensitive {
                    &params.split_text
                } else {
                    &params.split_text_lower
                };

                // Check if all search terms match with fuzzy logic
                search_terms.iter().all(|term| {
                    // First try exact substring match anywhere in the name
                    if search_name.contains(term) {
                        return true;
                    }

                    // Then try fuzzy match on individual words
                    search_name.split_whitespace().any(|word| {
                        if word.len() < 3 || term.len() < 3 {
                            // For very short strings, also check if word starts with term
                            word == term || word.starts_with(term)
                        } else {
                            // Use edit distance for longer strings
                            let distance = word.sz_edit_distance(term);
                            // For case sensitive, require exact match or very close match
                            // but not just case differences
                            if params.case_sensitive && distance > 0 {
                                // If they're the same when lowercased, it's just a case difference
                                // which shouldn't match in case-sensitive mode
                                if word.to_lowercase() == term.to_lowercase() {
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
                    chr_str.to_lowercase().contains(&params.text.to_lowercase())
                };

                // Check name match if enabled
                let name_matches = if params.search_name {
                    if params.case_sensitive {
                        name.contains(&params.text)
                    } else {
                        name.to_lowercase().contains(&params.text.to_lowercase())
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
    use std::collections::BTreeMap;

    fn create_test_cache() -> BTreeMap<char, String> {
        let mut cache = BTreeMap::new();

        // Add some test characters with names
        cache.insert('A', "Latin Capital Letter A".to_string());
        cache.insert('a', "Latin Small Letter a".to_string());
        cache.insert('-', "Hyphen Minus".to_string()); // Change to match word boundary better
        cache.insert('‐', "Hyphen".to_string());
        cache.insert('­', "Soft Hyphen".to_string());
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
    fn test_empty_search_returns_all() {
        let cache = create_test_cache();
        let params = SearchParams::new("".to_string(), false, false, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), cache.len());
        assert_eq!(results, cache);
    }

    #[test]
    fn test_single_character_exact_match() {
        let cache = create_test_cache();
        let params = SearchParams::new("A".to_string(), false, false, true);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));
    }

    #[test]
    fn test_case_insensitive_character_search() {
        let cache = create_test_cache();
        let params = SearchParams::new("a".to_string(), false, false, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find both 'A' and 'a' when case insensitive
        assert!(results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
    }

    #[test]
    fn test_case_sensitive_character_search() {
        let cache = create_test_cache();
        let params = SearchParams::new("a".to_string(), false, false, true);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should only find 'a' when case sensitive
        assert!(!results.contains_key(&'A'));
        assert!(results.contains_key(&'a'));
    }

    #[test]
    fn test_search_by_name_substring() {
        let cache = create_test_cache();
        let params = SearchParams::new("hyphen".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find all hyphen-related characters
        assert!(results.contains_key(&'-')); // Hyphen Minus
        assert!(results.contains_key(&'‐')); // Hyphen
        assert!(results.contains_key(&'­')); // Soft Hyphen
        assert!(!results.contains_key(&'—')); // Em Dash (doesn't contain "hyphen")
    }

    #[test]
    fn test_search_by_name_case_sensitive() {
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
    fn test_search_by_name_case_insensitive() {
        let cache = create_test_cache();
        let params = SearchParams::new("greek".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should find Greek letters regardless of case
        assert!(results.contains_key(&'α'));
        assert!(results.contains_key(&'Α'));
        assert!(results.contains_key(&'β'));
    }

    #[test]
    fn test_hex_code_search() {
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
    fn test_decimal_code_search() {
        let cache = create_test_cache();

        // 65 is the decimal code for 'A'
        let params = SearchParams::new("65".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 1);
        assert!(results.contains_key(&'A'));
    }

    #[test]
    fn test_multiple_word_search() {
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
    fn test_fuzzy_search_with_typo() {
        let cache = create_test_cache();
        // "hypen" is 1 edit away from "hyphen" (missing 'h')
        let params = SearchParams::new("hypen".to_string(), false, true, false);

        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        // Should still find hyphen-related characters due to fuzzy matching
        assert!(results.contains_key(&'-')); // Hyphen Minus
        assert!(results.contains_key(&'‐')); // Hyphen
        assert!(results.contains_key(&'­')); // Soft Hyphen
    }

    #[test]
    fn test_search_emoji() {
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
    fn test_search_special_characters() {
        let cache = create_test_cache();

        // Search for space
        let params = SearchParams::new("space".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert!(results.contains_key(&' '));
        assert!(results.contains_key(&'\u{00A0}')); // No-break Space
    }

    #[test]
    fn test_search_with_name_disabled() {
        let cache = create_test_cache();

        // With search_name disabled, "hyphen" should not find anything
        // (since no character is literally the string "hyphen")
        let params = SearchParams::new("hyphen".to_string(), false, false, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_partial_word_match() {
        let cache = create_test_cache();

        // Search for "mag" should find "Magnifying Glass"
        let params = SearchParams::new("mag".to_string(), false, true, false);
        let results = SearchEngine::search(&params, &cache, &[], egui::Id::new("test"));

        assert!(results.contains_key(&'🔍'));
    }

    #[test]
    fn test_combined_flags() {
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
    fn test_search_mathematical_symbols() {
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
}
