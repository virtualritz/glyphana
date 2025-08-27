use glyph_names;
use std::collections::BTreeMap;
use unicode_case_mapping;

// Helper functions to convert unicode-case-mapping results to strings
fn to_lowercase_string(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mapped = unicode_case_mapping::to_lowercase(c);
            let mut result = String::new();
            for &code in &mapped {
                if code != 0 {
                    if let Some(ch) = char::from_u32(code) {
                        result.push(ch);
                    }
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

fn to_uppercase_string(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mapped = unicode_case_mapping::to_uppercase(c);
            let mut result = String::new();
            for &code in &mapped {
                if code != 0 {
                    if let Some(ch) = char::from_u32(code) {
                        result.push(ch);
                    }
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

pub fn char_name(chr: char) -> String {
    // Try special/hardcoded names first
    if let Some(name) = special_char_name(chr) {
        return name.to_string();
    }

    // Try Unicode names
    if let Some(name) = unicode_names2::name(chr) {
        return title_case(&name.to_string());
    }

    // Try Adobe glyph names as fallback
    if let Some(adobe_name) = glyph_names::glyph_name(chr as u32) {
        // Adobe names are typically camelCase, convert to title case with spaces
        let spaced = adobe_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && c.is_uppercase() {
                    vec![' ', c]
                } else {
                    vec![c]
                }
            })
            .collect::<String>();
        return title_case(&spaced);
    }

    // Default to Unicode code point
    format!("U+{:04X}", chr as u32)
}

pub fn special_char_name(chr: char) -> Option<&'static str> {
    match chr {
        '\0' => Some("Null"),
        '\x01' => Some("Start of Heading"),
        '\x02' => Some("Start of Text"),
        '\x03' => Some("End of Text"),
        '\x04' => Some("End of Transmission"),
        '\x05' => Some("Enquiry"),
        '\x06' => Some("Acknowledge"),
        '\x07' => Some("Bell"),
        '\x08' => Some("Backspace"),
        '\t' => Some("Tab"),
        '\n' => Some("Line Feed"),
        '\x0b' => Some("Vertical Tab"),
        '\x0c' => Some("Form Feed"),
        '\r' => Some("Carriage Return"),
        '\x0e' => Some("Shift Out"),
        '\x0f' => Some("Shift In"),
        '\x10' => Some("Data Link Escape"),
        '\x11' => Some("Device Control 1"),
        '\x12' => Some("Device Control 2"),
        '\x13' => Some("Device Control 3"),
        '\x14' => Some("Device Control 4"),
        '\x15' => Some("Negative Acknowledge"),
        '\x16' => Some("Synchronous Idle"),
        '\x17' => Some("End of Transmission Block"),
        '\x18' => Some("Cancel"),
        '\x19' => Some("End of Medium"),
        '\x1a' => Some("Substitute"),
        '\x1b' => Some("Escape"),
        '\x1c' => Some("File Separator"),
        '\x1d' => Some("Group Separator"),
        '\x1e' => Some("Record Separator"),
        '\x1f' => Some("Unit Separator"),
        ' ' => Some("Space"),
        '\x7f' => Some("Delete"),
        '\u{00a0}' => Some("Non-breaking Space"),
        '\u{00ad}' => Some("Soft Hyphen"),
        '\u{2000}' => Some("En Quad"),
        '\u{2001}' => Some("Em Quad"),
        '\u{2002}' => Some("En Space"),
        '\u{2003}' => Some("Em Space"),
        '\u{2004}' => Some("Three-per-em Space"),
        '\u{2005}' => Some("Four-per-em Space"),
        '\u{2006}' => Some("Six-per-em Space"),
        '\u{2007}' => Some("Figure Space"),
        '\u{2008}' => Some("Punctuation Space"),
        '\u{2009}' => Some("Thin Space"),
        '\u{200a}' => Some("Hair Space"),
        '\u{200b}' => Some("Zero Width Space"),
        '\u{200c}' => Some("Zero Width Non-joiner"),
        '\u{200d}' => Some("Zero Width Joiner"),
        '\u{200e}' => Some("Left-to-right Mark"),
        '\u{200f}' => Some("Right-to-left Mark"),
        '\u{2028}' => Some("Line Separator"),
        '\u{2029}' => Some("Paragraph Separator"),
        '\u{202a}' => Some("Left-to-right Embedding"),
        '\u{202b}' => Some("Right-to-left Embedding"),
        '\u{202c}' => Some("Pop Directional Formatting"),
        '\u{202d}' => Some("Left-to-right Override"),
        '\u{202e}' => Some("Right-to-left Override"),
        '\u{202f}' => Some("Narrow No-break Space"),
        '\u{205f}' => Some("Medium Mathematical Space"),
        '\u{2060}' => Some("Word Joiner"),
        '\u{2061}' => Some("Function Application"),
        '\u{2062}' => Some("Invisible Times"),
        '\u{2063}' => Some("Invisible Separator"),
        '\u{2064}' => Some("Invisible Plus"),
        '\u{2066}' => Some("Left-to-right Isolate"),
        '\u{2067}' => Some("Right-to-left Isolate"),
        '\u{2068}' => Some("First Strong Isolate"),
        '\u{2069}' => Some("Pop Directional Isolate"),
        '\u{206a}' => Some("Inhibit Symmetric Swapping"),
        '\u{206b}' => Some("Activate Symmetric Swapping"),
        '\u{206c}' => Some("Inhibit Arabic Form Shaping"),
        '\u{206d}' => Some("Activate Arabic Form Shaping"),
        '\u{206e}' => Some("National Digit Shapes"),
        '\u{206f}' => Some("Nominal Digit Shapes"),
        '\u{feff}' => Some("Zero Width No-break Space"),
        '\u{fff9}' => Some("Interlinear Annotation Anchor"),
        '\u{fffa}' => Some("Interlinear Annotation Separator"),
        '\u{fffb}' => Some("Interlinear Annotation Terminator"),
        '\u{fffc}' => Some("Object Replacement Character"),
        '\u{fffd}' => Some("Replacement Character"),
        _ => None,
    }
}

pub fn available_characters(
    ctx: &egui::Context,
    family: egui::FontFamily,
) -> BTreeMap<char, String> {
    ctx.fonts(|f| {
        f.lock()
            .fonts
            .font(&egui::FontId::new(10.0, family)) // size is arbitrary for getting the characters
            .characters()
            .iter()
            .filter(|(chr, _)| !chr.is_whitespace() && !chr.is_ascii_control())
            .map(|(&chr, _)| (chr, char_name(chr)))
            .collect()
    })
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            // Use unicode_case_mapping for proper case conversion
            let upper = to_uppercase_string(&c.to_string());
            let rest = to_lowercase_string(chars.as_str());
            upper + &rest
        }
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            if word.len() <= 3 && word != "And" {
                to_lowercase_string(word)
            } else {
                capitalize(word)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(serde::Deserialize, serde::Serialize, Copy, Clone, PartialEq)]
pub enum GlyphScale {
    Tiny,
    Small,
    Normal,
    Large,
    Huge,
}

impl From<GlyphScale> for f32 {
    fn from(scale: GlyphScale) -> Self {
        match scale {
            GlyphScale::Tiny => 0.5,
            GlyphScale::Small => 0.75,
            GlyphScale::Normal => 1.0,
            GlyphScale::Large => 1.5,
            GlyphScale::Huge => 2.0,
        }
    }
}
