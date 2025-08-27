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

        // Check for single character search (exact match or Unicode block)
        if text.chars().count() == 1 {
            if let Some(chr) = text.chars().next() {
                // First, try exact character match
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
        mut cache: BTreeMap<char, String>,
        params: &SearchParams,
    ) -> BTreeMap<char, String> {
        // Apply fuzzy search if enabled
        if params.search_name && !params.split_text.is_empty() {
            cache = Self::fuzzy_search(cache, params);
        }

        // Apply skeleton search
        cache = Self::skeleton_search(cache, params);

        cache
    }

    fn fuzzy_search(
        cache: BTreeMap<char, String>,
        params: &SearchParams,
    ) -> BTreeMap<char, String> {
        const MAX_EDIT_DISTANCE: usize = 2;

        cache
            .into_iter()
            .filter(|(_, name)| {
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
                    // First try exact substring match
                    if search_name.contains(term) {
                        return true;
                    }

                    // Then try fuzzy match on words
                    search_name.split_whitespace().any(|word| {
                        if word.len() < 3 || term.len() < 3 {
                            // For very short strings, require exact match
                            word == term
                        } else {
                            // Use edit distance for longer strings
                            let distance = word.sz_edit_distance(term);
                            distance <= MAX_EDIT_DISTANCE
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
