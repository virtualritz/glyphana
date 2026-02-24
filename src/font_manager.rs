use anyhow::Result;
use google_fonts::Font;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages font loading using the google-fonts crate
#[derive(Clone)]
pub struct FontManager {
    loaded_fonts: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl FontManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            loaded_fonts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Check if a font is already loaded
    pub fn is_cached(&self, font_name: &str) -> bool {
        if let Ok(loaded) = self.loaded_fonts.lock() {
            loaded.contains_key(font_name)
        } else {
            false
        }
    }

    /// Load font using google-fonts crate
    pub fn load_font(&self, font_name: &str, _font_url: &str) -> Result<Vec<u8>> {
        // Check if already loaded in memory
        if let Ok(loaded) = self.loaded_fonts.lock()
            && let Some(data) = loaded.get(font_name)
        {
            return Ok(data.clone());
        }

        // Map font names to google-fonts Font enum variants
        // Based on what's available in google-fonts crate
        let font_data = match font_name {
            "NotoSansArabic-Regular" => Font::NotoSansArabicRegular.get_with_cache()?,
            "NotoSansHebrew-Regular" => Font::NotoSansHebrewRegular.get_with_cache()?,
            "NotoSansDevanagari-Regular" => Font::NotoSansDevanagariRegular.get_with_cache()?,
            "NotoSansBengali-Regular" => Font::NotoSansBengaliRegular.get_with_cache()?,
            "NotoSansTamil-Regular" => Font::NotoSansTamilRegular.get_with_cache()?,
            "NotoSansThai-Regular" => Font::NotoSansThaiRegular.get_with_cache()?,
            "NotoSansGeorgian-Regular" => Font::NotoSansGeorgianRegular.get_with_cache()?,
            "NotoSansArmenian-Regular" => Font::NotoSansArmenianRegular.get_with_cache()?,
            "NotoSansEthiopic-Regular" => Font::NotoSansEthiopicRegular.get_with_cache()?,
            "NotoSansCherokee-Regular" => Font::NotoSansCherokeeRegular.get_with_cache()?,
            "NotoSansCanadianAboriginal-Regular" => {
                Font::NotoSansCanadianAboriginalRegular.get_with_cache()?
            }
            "NotoSansKhmer-Regular" => Font::NotoSansKhmerRegular.get_with_cache()?,
            "NotoSansMyanmar-Regular" => Font::NotoSansMyanmarRegular.get_with_cache()?,
            "NotoSansSinhala-Regular" => Font::NotoSansSinhalaRegular.get_with_cache()?,
            "NotoSansTelugu-Regular" => Font::NotoSansTeluguRegular.get_with_cache()?,
            "NotoSansKannada-Regular" => Font::NotoSansKannadaRegular.get_with_cache()?,
            "NotoSansMalayalam-Regular" => Font::NotoSansMalayalamRegular.get_with_cache()?,
            "NotoSansGujarati-Regular" => Font::NotoSansGujaratiRegular.get_with_cache()?,
            "NotoSansGurmukhi-Regular" => Font::NotoSansGurmukhiRegular.get_with_cache()?,
            "NotoSansOriya-Regular" => Font::NotoSansOriyaRegular.get_with_cache()?,
            "NotoSansTibetan-Regular" => Font::NotoSerifTibetanRegular.get_with_cache()?,
            "NotoSansMongolian-Regular" => Font::NotoSansMongolianRegular.get_with_cache()?,
            "NotoSansLao-Regular" => Font::NotoSansLaoRegular.get_with_cache()?,
            // CJK fonts - using SC (Simplified Chinese), TC (Traditional Chinese), JP (Japanese), KR (Korean)
            "NotoSansCJKsc-Regular" => Font::NotoSansSCRegular.get_with_cache()?,
            "NotoSansCJKjp-Regular" => Font::NotoSansJPRegular.get_with_cache()?,
            "NotoSansCJKkr-Regular" => Font::NotoSansKRRegular.get_with_cache()?,
            "NotoSansCJKtc-Regular" => Font::NotoSansTCRegular.get_with_cache()?,
            _ => {
                return Err(anyhow::anyhow!("Unknown font: {}", font_name));
            }
        };

        // Store in memory cache
        if let Ok(mut loaded) = self.loaded_fonts.lock() {
            loaded.insert(font_name.to_string(), font_data.clone());
        }

        Ok(font_data)
    }

    /// Get download progress (always returns None since google-fonts handles this internally)
    pub fn download_progress(&self, _font_name: &str) -> Option<f32> {
        None
    }

    /// Clear font from cache
    pub fn clear_font_cache(&self, font_name: &str) -> Result<()> {
        if let Ok(mut loaded) = self.loaded_fonts.lock() {
            loaded.remove(font_name);
        }
        Ok(())
    }

    /// Clear all cached fonts
    pub fn clear_all_cache(&self) -> Result<()> {
        if let Ok(mut loaded) = self.loaded_fonts.lock() {
            loaded.clear();
        }
        Ok(())
    }

    /// Get list of cached fonts
    pub fn list_cached_fonts(&self) -> Result<Vec<String>> {
        if let Ok(loaded) = self.loaded_fonts.lock() {
            Ok(loaded.keys().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get cache size in bytes
    pub fn cache_size(&self) -> Result<u64> {
        let mut size = 0;
        if let Ok(loaded) = self.loaded_fonts.lock() {
            for data in loaded.values() {
                size += data.len() as u64;
            }
        }
        Ok(size)
    }
}

/// Mapping of Unicode scripts to Noto font names
pub struct NotoFontMapping;

impl NotoFontMapping {
    /// Get font info for a Unicode script/block
    pub fn font_for_script(script: &str) -> Option<(&'static str, &'static str)> {
        match script.to_lowercase().as_str() {
            "arabic" => Some(("NotoSansArabic-Regular", "google-fonts")),
            "hebrew" => Some(("NotoSansHebrew-Regular", "google-fonts")),
            "devanagari" => Some(("NotoSansDevanagari-Regular", "google-fonts")),
            "bengali" => Some(("NotoSansBengali-Regular", "google-fonts")),
            "tamil" => Some(("NotoSansTamil-Regular", "google-fonts")),
            "thai" => Some(("NotoSansThai-Regular", "google-fonts")),
            "georgian" => Some(("NotoSansGeorgian-Regular", "google-fonts")),
            "armenian" => Some(("NotoSansArmenian-Regular", "google-fonts")),
            "ethiopic" => Some(("NotoSansEthiopic-Regular", "google-fonts")),
            "cherokee" => Some(("NotoSansCherokee-Regular", "google-fonts")),
            "canadian aboriginal" | "canadian_aboriginal" => {
                Some(("NotoSansCanadianAboriginal-Regular", "google-fonts"))
            }
            "khmer" => Some(("NotoSansKhmer-Regular", "google-fonts")),
            "myanmar" => Some(("NotoSansMyanmar-Regular", "google-fonts")),
            "sinhala" => Some(("NotoSansSinhala-Regular", "google-fonts")),
            "telugu" => Some(("NotoSansTelugu-Regular", "google-fonts")),
            "kannada" => Some(("NotoSansKannada-Regular", "google-fonts")),
            "malayalam" => Some(("NotoSansMalayalam-Regular", "google-fonts")),
            "gujarati" => Some(("NotoSansGujarati-Regular", "google-fonts")),
            "gurmukhi" => Some(("NotoSansGurmukhi-Regular", "google-fonts")),
            "oriya" => Some(("NotoSansOriya-Regular", "google-fonts")),
            "tibetan" => Some(("NotoSansTibetan-Regular", "google-fonts")),
            "mongolian" => Some(("NotoSansMongolian-Regular", "google-fonts")),
            "lao" => Some(("NotoSansLao-Regular", "google-fonts")),
            "cjk" | "cjk_unified" | "cjk unified ideographs" => {
                Some(("NotoSansCJKsc-Regular", "google-fonts"))
            }
            "hiragana" | "katakana" => Some(("NotoSansCJKjp-Regular", "google-fonts")),
            "hangul" => Some(("NotoSansCJKkr-Regular", "google-fonts")),
            _ => None,
        }
    }

    /// Get all available Noto font mappings
    pub fn all_mappings() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("Arabic", "NotoSansArabic-Regular", "google-fonts"),
            ("Hebrew", "NotoSansHebrew-Regular", "google-fonts"),
            ("Devanagari", "NotoSansDevanagari-Regular", "google-fonts"),
            ("Bengali", "NotoSansBengali-Regular", "google-fonts"),
            ("Tamil", "NotoSansTamil-Regular", "google-fonts"),
            ("Thai", "NotoSansThai-Regular", "google-fonts"),
            (
                "CJK Simplified Chinese",
                "NotoSansCJKsc-Regular",
                "google-fonts",
            ),
            ("CJK Japanese", "NotoSansCJKjp-Regular", "google-fonts"),
            ("CJK Korean", "NotoSansCJKkr-Regular", "google-fonts"),
            (
                "CJK Traditional Chinese",
                "NotoSansCJKtc-Regular",
                "google-fonts",
            ),
            ("Georgian", "NotoSansGeorgian-Regular", "google-fonts"),
            ("Armenian", "NotoSansArmenian-Regular", "google-fonts"),
            ("Ethiopic", "NotoSansEthiopic-Regular", "google-fonts"),
        ]
    }
}
