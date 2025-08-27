use ahash::AHashSet as HashSet;
use finl_unicode::categories::CharacterCategories;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use unicode_blocks as ub;

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Category {
    pub name: String,
    #[serde(skip)]
    pub unicode_category: UnicodeCategory,
}

impl Category {
    pub fn new(name: &str, unicode_category: UnicodeCategory) -> Self {
        Self {
            name: name.to_string(),
            unicode_category,
        }
    }

    pub fn id(&self) -> egui::Id {
        egui::Id::new(&self.name)
    }
}

impl Hash for Category {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

pub trait CharacterInspector {
    fn characters(&self) -> Vec<char>;
    fn contains(&self, c: char) -> bool;
}

impl CharacterInspector for ub::UnicodeBlock {
    fn characters(&self) -> Vec<char> {
        let mut chars = vec![];
        let start = self.start();
        let end = self.end();
        for code_point in start..=end {
            if let Some(c) = char::from_u32(code_point) {
                chars.push(c);
            }
        }
        chars
    }

    fn contains(&self, c: char) -> bool {
        let code = c as u32;
        code >= self.start() && code <= self.end()
    }
}

pub struct UnicodeMultiBlock(pub Vec<ub::UnicodeBlock>);

impl CharacterInspector for UnicodeMultiBlock {
    fn characters(&self) -> Vec<char> {
        let mut chars = vec![];
        for block in &self.0 {
            chars.extend(block.characters());
        }
        chars
    }

    fn contains(&self, c: char) -> bool {
        self.0.iter().any(|block| block.contains(c))
    }
}

pub struct UnicodeCollection(pub HashSet<char>);

impl CharacterInspector for UnicodeCollection {
    fn characters(&self) -> Vec<char> {
        self.0.iter().copied().collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0.contains(&c)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum PropertyType {
    UppercaseLetters,
    LowercaseLetters,
    MathSymbols,
    CurrencySymbols,
    Punctuation,
    DecimalNumbers,
    AllLetters,
    AllNumbers,
    AllSymbols,
}

impl PropertyType {
    fn matches(&self, c: char) -> bool {
        match self {
            PropertyType::UppercaseLetters => c.is_letter_uppercase(),
            PropertyType::LowercaseLetters => c.is_letter_lowercase(),
            PropertyType::MathSymbols => c.is_symbol_math(),
            PropertyType::CurrencySymbols => c.is_symbol_currency(),
            PropertyType::Punctuation => c.is_punctuation(),
            PropertyType::DecimalNumbers => c.is_number_decimal(),
            PropertyType::AllLetters => c.is_letter(),
            PropertyType::AllNumbers => c.is_number(),
            PropertyType::AllSymbols => c.is_symbol(),
        }
    }
}

pub enum UnicodeCategory {
    Block(ub::UnicodeBlock),
    MultiBlock(UnicodeMultiBlock),
    Collection(UnicodeCollection),
    Property(PropertyType),
}

impl Default for UnicodeCategory {
    fn default() -> Self {
        UnicodeCategory::Collection(UnicodeCollection(HashSet::new()))
    }
}

impl CharacterInspector for UnicodeCategory {
    fn characters(&self) -> Vec<char> {
        match self {
            UnicodeCategory::Block(block) => block.characters(),
            UnicodeCategory::MultiBlock(multi_block) => multi_block.characters(),
            UnicodeCategory::Collection(collection) => collection.characters(),
            UnicodeCategory::Property(prop_type) => {
                // For property-based categories, scan common Unicode ranges
                let mut chars = Vec::new();
                // Scan common ranges where these properties are found
                let ranges = vec![
                    (0x0020, 0x007E),   // Basic ASCII printable
                    (0x00A0, 0x024F),   // Latin Extended
                    (0x0370, 0x052F),   // Greek, Cyrillic
                    (0x2000, 0x206F),   // General Punctuation
                    (0x2070, 0x218F),   // Superscripts and Subscripts
                    (0x2190, 0x21FF),   // Arrows
                    (0x2200, 0x22FF),   // Mathematical Operators
                    (0x20A0, 0x20CF),   // Currency Symbols
                    (0x2500, 0x257F),   // Box Drawing
                    (0x2600, 0x26FF),   // Miscellaneous Symbols
                    (0x1F300, 0x1F5FF), // Emoji
                ];

                for (start, end) in ranges {
                    for code in start..=end {
                        if let Some(c) = char::from_u32(code) {
                            if prop_type.matches(c) {
                                chars.push(c);
                            }
                        }
                    }
                }
                chars
            }
        }
    }

    fn contains(&self, c: char) -> bool {
        match self {
            UnicodeCategory::Block(block) => block.contains(c),
            UnicodeCategory::MultiBlock(multi_block) => multi_block.contains(c),
            UnicodeCategory::Collection(collection) => collection.contains(c),
            UnicodeCategory::Property(prop_type) => prop_type.matches(c),
        }
    }
}

// Data-driven category initialization to fix DRY violations
pub fn create_default_categories() -> Vec<Category> {
    let category_data = vec![
        (
            "Latin",
            vec![
                ub::BASIC_LATIN,
                ub::LATIN_1_SUPPLEMENT,
                ub::LATIN_EXTENDED_A,
                ub::LATIN_EXTENDED_B,
            ],
        ),
        (
            "Emoji",
            vec![
                ub::EMOTICONS,
                ub::MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS,
                ub::SUPPLEMENTAL_SYMBOLS_AND_PICTOGRAPHS,
                ub::TRANSPORT_AND_MAP_SYMBOLS,
            ],
        ),
        (
            "Arrows",
            vec![
                ub::ARROWS,
                ub::SUPPLEMENTAL_ARROWS_A,
                ub::SUPPLEMENTAL_ARROWS_B,
                ub::SUPPLEMENTAL_ARROWS_C,
                ub::MISCELLANEOUS_SYMBOLS_AND_ARROWS,
            ],
        ),
        (
            "Math",
            vec![
                ub::MATHEMATICAL_OPERATORS,
                ub::SUPPLEMENTAL_MATHEMATICAL_OPERATORS,
                ub::MATHEMATICAL_ALPHANUMERIC_SYMBOLS,
                ub::MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A,
                ub::MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B,
            ],
        ),
        (
            "Technical",
            vec![
                ub::MISCELLANEOUS_TECHNICAL,
                ub::CONTROL_PICTURES,
                ub::OPTICAL_CHARACTER_RECOGNITION,
            ],
        ),
        (
            "Symbols",
            vec![
                ub::MISCELLANEOUS_SYMBOLS,
                ub::MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS,
            ],
        ),
        ("Currency", vec![ub::CURRENCY_SYMBOLS]),
        (
            "Music",
            vec![
                ub::MUSICAL_SYMBOLS,
                ub::BYZANTINE_MUSICAL_SYMBOLS,
                ub::ANCIENT_GREEK_MUSICAL_NOTATION,
            ],
        ),
        (
            "Box Drawing",
            vec![ub::BOX_DRAWING, ub::BLOCK_ELEMENTS, ub::GEOMETRIC_SHAPES],
        ),
    ];

    // Add property-based categories first (using finl_unicode)
    let mut categories = vec![
        Category::new(
            "Uppercase Letters",
            UnicodeCategory::Property(PropertyType::UppercaseLetters),
        ),
        Category::new(
            "Lowercase Letters",
            UnicodeCategory::Property(PropertyType::LowercaseLetters),
        ),
        Category::new(
            "All Letters",
            UnicodeCategory::Property(PropertyType::AllLetters),
        ),
        Category::new(
            "Math Symbols",
            UnicodeCategory::Property(PropertyType::MathSymbols),
        ),
        Category::new(
            "Punctuation",
            UnicodeCategory::Property(PropertyType::Punctuation),
        ),
        Category::new(
            "Decimal Numbers",
            UnicodeCategory::Property(PropertyType::DecimalNumbers),
        ),
        Category::new(
            "All Numbers",
            UnicodeCategory::Property(PropertyType::AllNumbers),
        ),
        Category::new(
            "All Symbols",
            UnicodeCategory::Property(PropertyType::AllSymbols),
        ),
    ];

    // Note: Removed duplicate Currency Symbols as we now have a property-based one

    // Add multi-block categories from data
    for (name, blocks) in category_data {
        // Skip Currency since we have property-based version
        if name == "Currency" {
            continue;
        }
        categories.push(Category::new(
            name,
            UnicodeCategory::MultiBlock(UnicodeMultiBlock(blocks)),
        ));
    }

    // Add single block categories
    let single_blocks = vec![
        ("Greek and Coptic", ub::GREEK_AND_COPTIC),
        ("Cyrillic", ub::CYRILLIC),
        ("Hebrew", ub::HEBREW),
        ("Arabic", ub::ARABIC),
    ];

    for (name, block) in single_blocks {
        categories.push(Category::new(name, UnicodeCategory::Block(block)));
    }

    categories
}
