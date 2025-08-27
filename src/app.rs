use ahash::AHashSet as HashSet;
use egui_dnd::dnd;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;

use crate::categories::{
    Category, CharacterInspector, UnicodeCategory, UnicodeCollection, create_default_categories,
};
use crate::glyph::{GlyphScale, available_characters, char_name};
use crate::search::{SearchEngine, SearchParams};
use crate::ui::{
    CANCELLATION, COLLECTION, HAMBURGER, LOWER_UPPER_CASE, MAGNIFIER, NAME_BADGE, RECENTLY_USED,
    SEARCH, SUBSET, collection_id, recently_used_id, search_id,
};

// Inspector view mode - either related characters or font variations
#[derive(Debug, Clone, Copy, PartialEq)]
enum InspectorViewMode {
    RelatedCharacters,
    FontVariations,
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct GlyphanaApp {
    // The character the user selected for inspection.
    selected_char: char,
    // Whether to only search in the subsets selected on the left panel.
    search_only_categories: bool,
    // Also search the glyph's name.
    search_name: bool,
    // If search is case sensitive.
    case_sensitive: bool,
    recently_used: VecDeque<char>,
    recently_used_max_len: usize,
    collection: HashSet<char>,
    selected_category: egui::Id,
    ui_search_text: String,
    #[serde(skip)]
    search_text: String,
    #[serde(skip)]
    split_search_text: Vec<String>,
    #[serde(skip)]
    split_search_text_lower: Vec<String>,
    #[serde(skip)]
    default_font_id: egui::FontId,
    #[serde(skip)]
    font_size: f32,

    categories: Vec<Category>,
    #[serde(skip)]
    full_glyph_cache: BTreeMap<char, String>,
    #[serde(skip)]
    showed_glyph_cache: BTreeMap<char, String>,
    #[serde(skip)]
    search_active: bool, // Track if search is currently active
    pixels_per_point: f32,
    glyph_scale: GlyphScale,

    // Inspector view mode - either related characters or font variations
    #[serde(skip)]
    inspector_view_mode: InspectorViewMode,
}

impl Default for GlyphanaApp {
    fn default() -> Self {
        Self {
            selected_char: Default::default(),
            ui_search_text: Default::default(),
            search_text: Default::default(),
            split_search_text: Default::default(),
            split_search_text_lower: Default::default(),
            search_only_categories: false,
            case_sensitive: false,
            search_name: false,
            default_font_id: egui::FontId::new(24.0, egui::FontFamily::Name(NOTO_SANS.into())),
            font_size: 18.0,
            recently_used: Default::default(),
            recently_used_max_len: 1000,
            collection: Default::default(),
            selected_category: recently_used_id(),
            categories: create_default_categories(),
            full_glyph_cache: Default::default(),
            showed_glyph_cache: Default::default(),
            search_active: false,
            pixels_per_point: Default::default(),
            glyph_scale: GlyphScale::Normal,
            inspector_view_mode: InspectorViewMode::RelatedCharacters,
        }
    }
}

impl GlyphanaApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Add the Noto fonts -- what we use to cover as much unicode as possible for now.
        cc.egui_ctx.set_fonts(Self::fonts());

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Re-initialize categories after deserialization
            for category in &mut app.categories {
                category.unicode_category = Self::get_unicode_category_for_name(&category.name);
            }
            app
        } else {
            Default::default()
        }
    }

    fn get_unicode_category_for_name(name: &str) -> UnicodeCategory {
        // Map category names back to their Unicode categories
        // This is needed for deserialization since we skip the unicode_category field
        match name {
            "Emoji" => {
                use unicode_blocks as ub;
                UnicodeCategory::MultiBlock(crate::categories::UnicodeMultiBlock(vec![
                    ub::EMOTICONS,
                    ub::TRANSPORT_AND_MAP_SYMBOLS,
                    ub::ALCHEMICAL_SYMBOLS,
                    ub::SYMBOLS_AND_PICTOGRAPHS_EXTENDED_A,
                    ub::SYMBOLS_FOR_LEGACY_COMPUTING,
                ]))
            }
            "Parentheses" => {
                let chars = vec![
                    '\u{0028}', '\u{0029}', '\u{005B}', '\u{005D}', '\u{007B}', '\u{007D}',
                    '\u{0F3A}', '\u{0F3B}', '\u{0F3C}', '\u{0F3D}', '\u{169B}', '\u{169C}',
                    '\u{2045}', '\u{2046}', '\u{207D}', '\u{207E}', '\u{208D}', '\u{208E}',
                    '\u{2308}', '\u{2309}', '\u{230A}', '\u{230B}', '\u{2329}', '\u{232A}',
                    '\u{2768}', '\u{2769}', '\u{276A}', '\u{276B}', '\u{276C}', '\u{276D}',
                    '\u{276E}', '\u{276F}', '\u{2770}', '\u{2771}', '\u{2772}', '\u{2773}',
                    '\u{2774}', '\u{2775}', '\u{27C5}', '\u{27C6}', '\u{27E6}', '\u{27E7}',
                    '\u{27E8}', '\u{27E9}', '\u{27EA}', '\u{27EB}', '\u{27EC}', '\u{27ED}',
                    '\u{27EE}', '\u{27EF}', '\u{2983}', '\u{2984}', '\u{2985}', '\u{2986}',
                    '\u{2987}', '\u{2988}', '\u{2989}', '\u{298A}', '\u{298B}', '\u{298C}',
                    '\u{298D}', '\u{298E}', '\u{298F}', '\u{2990}', '\u{2991}', '\u{2992}',
                    '\u{2993}', '\u{2994}', '\u{2995}', '\u{2996}', '\u{2997}', '\u{2998}',
                    '\u{29D8}', '\u{29D9}', '\u{29DA}', '\u{29DB}', '\u{29FC}', '\u{29FD}',
                    '\u{2E22}', '\u{2E23}', '\u{2E24}', '\u{2E25}', '\u{2E26}', '\u{2E27}',
                    '\u{2E28}', '\u{2E29}', '\u{2E55}', '\u{2E56}', '\u{2E57}', '\u{2E58}',
                    '\u{2E59}', '\u{2E5A}', '\u{2E5B}', '\u{2E5C}', '\u{3008}', '\u{3009}',
                    '\u{300A}', '\u{300B}', '\u{300C}', '\u{300D}', '\u{300E}', '\u{300F}',
                    '\u{3010}', '\u{3011}', '\u{3014}', '\u{3015}', '\u{3016}', '\u{3017}',
                    '\u{3018}', '\u{3019}', '\u{301A}', '\u{301B}', '\u{FE59}', '\u{FE5A}',
                    '\u{FE5B}', '\u{FE5C}', '\u{FE5D}', '\u{FE5E}', '\u{FF08}', '\u{FF09}',
                    '\u{FF3B}', '\u{FF3D}', '\u{FF5B}', '\u{FF5D}', '\u{FF5F}', '\u{FF60}',
                    '\u{FF62}', '\u{FF63}',
                ];
                UnicodeCategory::Collection(UnicodeCollection(chars.into_iter().collect()))
            }
            _ => {
                // Try to match standard categories from create_default_categories
                for category in create_default_categories() {
                    if category.name == name {
                        return category.unicode_category;
                    }
                }
                // Default to empty collection if not found
                UnicodeCategory::Collection(UnicodeCollection(HashSet::new()))
            }
        }
    }

    fn fonts() -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();

        // Add Noto Sans
        fonts.font_data.insert(
            NOTO_SANS.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoSans-Regular.otf"
            ))),
        );

        // Add Noto Sans Mono
        fonts.font_data.insert(
            NOTO_SANS_MONO.to_owned(),
            // NotoSansMono not available, use NotoSans as fallback
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoSans-Regular.otf"
            ))),
        );

        // Add Noto Sans Symbols
        fonts.font_data.insert(
            NOTO_SANS_SYMBOLS.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoSansSymbols-Regular.ttf"
            ))),
        );

        // Add Noto Sans Symbols 2
        fonts.font_data.insert(
            NOTO_SANS_SYMBOLS2.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoSansSymbols2-Regular.ttf"
            ))),
        );

        // Add Noto Sans Math
        fonts.font_data.insert(
            NOTO_SANS_MATH.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoSansMath-Regular.ttf"
            ))),
        );

        // Add Noto Music
        fonts.font_data.insert(
            NOTO_MUSIC.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoMusic-Regular.ttf"
            ))),
        );

        // Add Noto Emoji (black and white)
        fonts.font_data.insert(
            NOTO_EMOJI.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/NotoEmoji-Regular.ttf"
            ))),
        );

        // Add Emoji Icon font from master
        fonts.font_data.insert(
            EMOJI_ICON.to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/emoji-icon-font.ttf"
            ))),
        );

        // Configure font families - create base font list to avoid duplication
        // For UI: Use black & white emojis
        let ui_base_fonts = vec![
            NOTO_EMOJI.to_owned(), // Black & white emoji for UI
            EMOJI_ICON.to_owned(),
            NOTO_SANS_SYMBOLS.to_owned(),
            NOTO_SANS_SYMBOLS2.to_owned(),
            NOTO_SANS_MATH.to_owned(),
            NOTO_MUSIC.to_owned(),
        ];

        // Proportional font family (for UI elements)
        let mut proportional_fonts = vec![NOTO_SANS.to_owned()];
        proportional_fonts.extend(ui_base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Proportional, proportional_fonts);

        // Monospace font family
        let mut monospace_fonts = vec![NOTO_SANS_MONO.to_owned()];
        monospace_fonts.extend(ui_base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Monospace, monospace_fonts);

        // Named NotoSans font family (for general text)
        let mut noto_sans_fonts = vec![NOTO_SANS.to_owned()];
        noto_sans_fonts.extend(ui_base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Name(NOTO_SANS.into()), noto_sans_fonts);

        // Named NotoEmoji font family (black & white emoji for UI)
        let mut emoji_fonts = vec![
            NOTO_EMOJI.to_owned(), // Black & white emoji
            EMOJI_ICON.to_owned(),
            NOTO_SANS.to_owned(),
        ];
        emoji_fonts.extend(vec![
            NOTO_SANS_SYMBOLS.to_owned(),
            NOTO_SANS_SYMBOLS2.to_owned(),
            NOTO_SANS_MATH.to_owned(),
            NOTO_MUSIC.to_owned(),
        ]);
        fonts
            .families
            .insert(egui::FontFamily::Name(NOTO_EMOJI.into()), emoji_fonts);

        fonts
    }
}

impl eframe::App for GlyphanaApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for screen DPI changes
        let current_ppp = ctx.pixels_per_point();
        if self.pixels_per_point != current_ppp && current_ppp > 0.0 {
            self.pixels_per_point = current_ppp;
        }

        // Update the glyph cache if needed
        if self.full_glyph_cache.is_empty() {
            self.update_full_glyph_cache(ctx);
        }

        // Top panel with search and controls
        self.render_top_panel(ctx);

        // Left side panel with categories
        self.render_side_panel(ctx);

        // Right side panel with character preview (always visible)
        self.render_right_panel(ctx);

        // Central panel with glyphs
        self.render_central_panel(ctx);
    }
}

// UI rendering methods
impl GlyphanaApp {
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            #[allow(deprecated)]
            egui::menu::bar(ui, |ui| {
                // Hamburger menu
                ui.menu_button(HAMBURGER.to_string(), |ui| {
                    #[cfg(debug_assertions)]
                    if ui.button("Reset App State").clicked() {
                        *self = Self::default();
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    ui.separator();

                    ui.add_enabled_ui(false, |ui| ui.button("Glyph Size"));
                    ui.vertical(|ui| {
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Tiny, "Tiny");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Small, "Small");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Normal, "Normal");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Large, "Large");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Huge, "Huge");
                    });

                    ui.separator();

                    if ui.button("Clear Recently Used").clicked() {
                        self.recently_used.clear();
                        ui.close_kind(egui::UiKind::Menu);
                    }

                    ui.separator();

                    ui.add_enabled_ui(false, |ui| ui.button("Export Collection…"));

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                // Search bar and controls on the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Clear button with icon
                    if ui
                        .button(CANCELLATION.to_string())
                        .on_hover_text("Clear Search")
                        .clicked()
                    {
                        self.ui_search_text.clear();
                        self.search_active = false;
                        self.update_search_text_and_cache();
                    }

                    // Search field
                    let search_response = ui.add(
                        egui::TextEdit::singleline(&mut self.ui_search_text)
                            .hint_text(format!("{} Search", MAGNIFIER)),
                    );

                    // When search text changes or Enter is pressed, activate search
                    if search_response.changed()
                        || (search_response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if !self.ui_search_text.is_empty() {
                            // Activate search and select Search category
                            self.search_active = true;
                            self.selected_category = search_id();
                        }
                        self.update_search_text_and_cache();
                    }

                    // Case sensitive toggle
                    ui.toggle_value(&mut self.case_sensitive, LOWER_UPPER_CASE.to_string())
                        .on_hover_text("Case Sensitive");

                    // Search names toggle
                    ui.add_enabled_ui(!self.case_sensitive, |ui| {
                        ui.toggle_value(&mut self.search_name, NAME_BADGE.to_string())
                            .on_hover_text("Search Glyph Names");
                    });

                    // Search only in categories toggle
                    ui.toggle_value(&mut self.search_only_categories, SUBSET.to_string())
                        .on_hover_text("Search Only Selected Category");
                });
            });
        });
    }

    fn render_side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Categories");

            // Handle drag and drop
            let selected_category = self.selected_category;
            let mut category_clicked = None;

            let response = dnd(ui, "category_dnd").show_vec(
                &mut self.categories,
                |ui, category, handle, _state| {
                    ui.horizontal(|ui| {
                        handle.ui(ui, |ui| {
                            ui.label("≡");
                        });

                        let is_selected = selected_category == category.id();
                        if ui.selectable_label(is_selected, &category.name).clicked() {
                            category_clicked = Some(category.id());
                        }
                    });
                },
            );

            if let Some(cat_id) = category_clicked {
                // Toggle selection - if already selected, deselect (set to invalid ID)
                if self.selected_category == cat_id {
                    self.selected_category = egui::Id::new("__none__");
                } else {
                    self.selected_category = cat_id;
                    // Deactivate search when selecting a category
                    self.search_active = false;
                }
                self.update_search_text_and_cache();
            }

            if response.final_update().is_some() {
                self.update_search_text_and_cache();
            }

            ui.separator();

            // Special categories
            if ui
                .selectable_label(self.selected_category == recently_used_id(), RECENTLY_USED)
                .clicked()
            {
                // Toggle selection
                if self.selected_category == recently_used_id() {
                    self.selected_category = egui::Id::new("__none__");
                } else {
                    self.selected_category = recently_used_id();
                    self.search_active = false; // Deactivate search
                }
                self.update_search_text_and_cache();
            }

            if ui
                .selectable_label(self.selected_category == collection_id(), COLLECTION)
                .clicked()
            {
                // Toggle selection
                if self.selected_category == collection_id() {
                    self.selected_category = egui::Id::new("__none__");
                } else {
                    self.selected_category = collection_id();
                    self.search_active = false; // Deactivate search
                }
                self.update_search_text_and_cache();
            }

            // Only enable Search category when there's search text
            ui.add_enabled_ui(!self.ui_search_text.is_empty(), |ui| {
                if ui
                    .selectable_label(self.selected_category == search_id(), SEARCH)
                    .clicked()
                {
                    // Toggle selection
                    if self.selected_category == search_id() {
                        self.selected_category = egui::Id::new("__none__");
                    } else {
                        self.selected_category = search_id();
                        self.search_active = true; // Activate search when Search category is selected
                    }
                    self.update_search_text_and_cache();
                }
            });
        });
    }

    // Get related characters for a given character
    fn get_related_characters(&self, ch: char) -> Vec<char> {
        let mut related = Vec::new();
        let code_point = ch as u32;

        // Add case variations
        if ch.is_lowercase() {
            for upper in ch.to_uppercase() {
                if upper != ch {
                    related.push(upper);
                }
            }
        } else if ch.is_uppercase() {
            for lower in ch.to_lowercase() {
                if lower != ch {
                    related.push(lower);
                }
            }
        }

        // Add nearby characters in the same block
        if let Some(block) = unicode_blocks::find_unicode_block(ch) {
            let start = block.start().max(code_point.saturating_sub(3));
            let end = block.end().min(code_point + 4);

            for cp in start..=end {
                if cp != code_point {
                    if let Some(nearby_char) = char::from_u32(cp) {
                        if !related.contains(&nearby_char) {
                            related.push(nearby_char);
                        }
                    }
                }
            }
        }

        // Add diacritic variations for Latin characters
        if ch.is_ascii_alphabetic() {
            let base_char = ch.to_ascii_lowercase();
            let diacritic_variations: Vec<(char, Vec<char>)> = vec![
                ('a', vec!['à', 'á', 'â', 'ã', 'ä', 'å', 'ā', 'ă', 'ą']),
                ('e', vec!['è', 'é', 'ê', 'ë', 'ē', 'ė', 'ę', 'ě']),
                ('i', vec!['ì', 'í', 'î', 'ï', 'ī', 'į', 'ı']),
                ('o', vec!['ò', 'ó', 'ô', 'õ', 'ö', 'ø', 'ō', 'ő']),
                ('u', vec!['ù', 'ú', 'û', 'ü', 'ū', 'ů', 'ű', 'ų']),
                ('c', vec!['ç', 'ć', 'č']),
                ('n', vec!['ñ', 'ń', 'ň']),
                ('s', vec!['ś', 'š', 'ş']),
                ('z', vec!['ź', 'ž', 'ż']),
            ];

            for (base, variations) in diacritic_variations {
                if base_char == base {
                    for var in variations {
                        if !related.contains(&var) {
                            related.push(var);
                        }
                    }
                    break;
                }
            }
        }

        // Limit to first 12 related characters for UI space
        related.truncate(12);
        related
    }

    // Get available fonts that have the character
    fn get_font_variations(&self, ch: char) -> Vec<(&'static str, egui::FontFamily)> {
        let mut fonts = Vec::new();

        // Check which fonts can display this character
        fonts.push((NOTO_SANS, egui::FontFamily::Name(NOTO_SANS.into())));

        // Add symbol fonts for symbol characters
        if ch as u32 >= 0x2000 {
            fonts.push((
                NOTO_SANS_SYMBOLS,
                egui::FontFamily::Name(NOTO_SANS_SYMBOLS.into()),
            ));
            fonts.push((
                NOTO_SANS_SYMBOLS2,
                egui::FontFamily::Name(NOTO_SANS_SYMBOLS2.into()),
            ));
        }

        // Add math font for mathematical symbols
        if (ch as u32 >= 0x2200 && ch as u32 <= 0x22FF)
            || (ch as u32 >= 0x2100 && ch as u32 <= 0x214F)
        {
            fonts.push((
                NOTO_SANS_MATH,
                egui::FontFamily::Name(NOTO_SANS_MATH.into()),
            ));
        }

        // Add music font for musical symbols
        if ch as u32 >= 0x1D100 && ch as u32 <= 0x1D1FF {
            fonts.push((NOTO_MUSIC, egui::FontFamily::Name(NOTO_MUSIC.into())));
        }

        // Add emoji font for emoji characters
        if ch as u32 >= 0x1F300 || (ch as u32 >= 0x2600 && ch as u32 <= 0x27BF) {
            fonts.push((NOTO_EMOJI, egui::FontFamily::Name(NOTO_EMOJI.into())));
        }

        fonts
    }

    fn render_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("character_preview").show(ctx, |ui| {
            // Large character preview with paint_glyph
            let rect = ui.available_rect_before_wrap();
            let scale = rect.width().min(rect.height() * 0.4);

            let (response, painter) =
                ui.allocate_painter(egui::Vec2::new(scale, scale * 1.2), egui::Sense::click());

            self.paint_glyph(scale * 0.8, ui, response, painter);

            ui.separator();

            // Display character info
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    // Character name
                    if self.selected_char != '\0' {
                        ui.heading(self.selected_char.to_string());

                        let name = char_name(self.selected_char);
                        ui.label(&name);

                        ui.separator();

                        // Unicode codepoint
                        egui::Grid::new("glyph_codepoints")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Unicode:");
                                let unicode_string = format!("U+{:04X}", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&unicode_string).monospace())
                                    .on_hover_text("Click to copy Unicode")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(unicode_string);
                                }
                                ui.end_row();

                                ui.label("Decimal:");
                                let decimal_string = format!("{}", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&decimal_string).monospace())
                                    .on_hover_text("Click to copy decimal")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(decimal_string);
                                }
                                ui.end_row();

                                ui.label("HTML:");
                                let html_string = format!("&#x{:04X};", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&html_string).monospace())
                                    .on_hover_text("Click to copy HTML entity")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(html_string);
                                }
                                ui.end_row();
                            });

                        ui.separator();

                        // Collection button
                        if !self.collection.contains(&self.selected_char) {
                            if ui.button("Add to Collection").clicked() {
                                self.collection.insert(self.selected_char);
                            }
                        } else if ui.button("Remove from Collection").clicked() {
                            self.collection.remove(&self.selected_char);
                        }

                        ui.separator();

                        // Toggle between Related Characters and Font Variations
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    self.inspector_view_mode
                                        == InspectorViewMode::RelatedCharacters,
                                    "Related",
                                )
                                .clicked()
                            {
                                self.inspector_view_mode = InspectorViewMode::RelatedCharacters;
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.inspector_view_mode == InspectorViewMode::FontVariations,
                                    "Font Variations",
                                )
                                .clicked()
                            {
                                self.inspector_view_mode = InspectorViewMode::FontVariations;
                            }
                        });

                        ui.separator();

                        // Show the selected view
                        match self.inspector_view_mode {
                            InspectorViewMode::RelatedCharacters => {
                                self.render_related_characters(ui);
                            }
                            InspectorViewMode::FontVariations => {
                                self.render_font_variations(ui);
                            }
                        }
                    } else {
                        ui.label("Select a character to see details");
                    }
                },
            );
        });
    }

    fn render_related_characters(&mut self, ui: &mut egui::Ui) {
        let related_chars = self.get_related_characters(self.selected_char);

        if related_chars.is_empty() {
            ui.label("No related characters found");
        } else {
            // Create a grid for related characters
            let columns = 3;
            let button_size = ui.available_width() / columns as f32 - ui.spacing().item_spacing.x;

            egui::Grid::new("related_chars_grid")
                .num_columns(columns)
                .spacing([ui.spacing().item_spacing.x, ui.spacing().item_spacing.y])
                .show(ui, |ui| {
                    for (i, &ch) in related_chars.iter().enumerate() {
                        let response = ui.allocate_response(
                            egui::Vec2::splat(button_size),
                            egui::Sense::click(),
                        );

                        let rect = response.rect;
                        let painter = ui.painter();

                        // Draw background
                        let bg_color = if response.hovered() {
                            ui.visuals().widgets.hovered.bg_fill
                        } else {
                            ui.visuals().extreme_bg_color
                        };
                        painter.rect_filled(rect, 4.0, bg_color);

                        // Draw character
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            ch,
                            egui::FontId::new(24.0, egui::FontFamily::Name(NOTO_SANS.into())),
                            ui.visuals().text_color(),
                        );

                        // Draw character code below
                        let code_text = format!("U+{:04X}", ch as u32);
                        painter.text(
                            rect.center() + egui::Vec2::new(0.0, button_size * 0.3),
                            egui::Align2::CENTER_CENTER,
                            code_text,
                            egui::FontId::new(9.0, egui::FontFamily::Monospace),
                            ui.visuals().weak_text_color(),
                        );

                        // Handle click
                        if response.clicked() {
                            self.selected_char = ch;
                            self.add_to_recently_used(ch);
                        }

                        // Show tooltip
                        if response.hovered() {
                            response.on_hover_text(format!(
                                "{}\nU+{:04X}\nClick to select",
                                char_name(ch),
                                ch as u32
                            ));
                        }

                        // End row every 3 characters
                        if (i + 1) % columns == 0 && i < related_chars.len() - 1 {
                            ui.end_row();
                        }
                    }
                });
        }
    }

    fn render_font_variations(&mut self, ui: &mut egui::Ui) {
        let fonts = self.get_font_variations(self.selected_char);

        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                for (font_name, font_family) in fonts {
                    ui.horizontal(|ui| {
                        // Show font name label
                        ui.label(
                            egui::RichText::new(font_name)
                                .size(10.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                    });

                    // Draw the character in this font
                    let response = ui.allocate_response(
                        egui::Vec2::new(ui.available_width(), 40.0),
                        egui::Sense::click(),
                    );

                    let rect = response.rect;
                    let painter = ui.painter();

                    // Draw background
                    let bg_color = if response.hovered() {
                        ui.visuals().widgets.hovered.bg_fill
                    } else {
                        ui.visuals().extreme_bg_color
                    };
                    painter.rect_filled(rect, 4.0, bg_color);

                    // Draw the character
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.selected_char,
                        egui::FontId::new(28.0, font_family.clone()),
                        ui.visuals().text_color(),
                    );

                    // Copy on click
                    if response.clicked() {
                        ui.ctx().copy_text(self.selected_char.to_string());
                    }

                    response.on_hover_text("Click to copy character");

                    ui.separator();
                }
            });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Always show the glyph grid
            self.render_glyph_grid(ui);
        });
    }

    #[allow(dead_code)]
    fn _render_single_glyph_view(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                self.selected_char = '\0';
            }

            ui.separator();

            if ui.button("Copy Character").clicked() {
                ui.ctx().copy_text(self.selected_char.to_string());
            }

            if ui.button("Copy Unicode").clicked() {
                ui.ctx()
                    .copy_text(format!("U+{:04X}", self.selected_char as u32));
            }

            if !self.collection.contains(&self.selected_char) {
                if ui.button("Add to Collection").clicked() {
                    self.collection.insert(self.selected_char);
                }
            } else if ui.button("Remove from Collection").clicked() {
                self.collection.remove(&self.selected_char);
            }
        });

        ui.separator();

        // Large character preview with ascender/descender lines
        let rect = ui.available_rect_before_wrap();
        let scale = rect.width().min(rect.height() * 0.5);
        let (response, painter) =
            ui.allocate_painter(egui::Vec2::new(scale, scale * 1.2), egui::Sense::click());

        self.paint_glyph(scale * 0.8, ui, response, painter);

        ui.separator();

        // Display character info
        ui.heading(format!("Character: {}", self.selected_char));
        ui.label(format!("Name: {}", char_name(self.selected_char)));
        ui.label(format!(
            "Code: U+{:04X} ({})",
            self.selected_char as u32, self.selected_char as u32
        ));

        ui.separator();

        // Font preview with proper ascender/descender lines
        self._render_font_preview(ui);
    }

    #[allow(dead_code)]
    fn _render_font_preview(&self, ui: &mut egui::Ui) {
        use rusttype::{Font, Scale};

        // Try to load a font for metrics
        let font_data = include_bytes!("../assets/NotoSans-Regular.otf");
        if let Some(font) = Font::try_from_bytes(font_data) {
            let scale = Scale::uniform(100.0);
            let v_metrics = font.v_metrics(scale);

            let baseline = 100.0;
            let ascent_line = baseline - v_metrics.ascent;
            let descent_line = baseline - v_metrics.descent;

            ui.group(|ui| {
                let (response, painter) =
                    ui.allocate_painter(egui::Vec2::new(200.0, 150.0), egui::Sense::hover());

                let rect = response.rect;

                // Draw guidelines with labels
                let line_color = egui::Color32::from_rgb(100, 100, 100);
                let label_color = egui::Color32::from_rgb(150, 150, 150);

                // Ascender line
                painter.line_segment(
                    [
                        rect.left_top() + egui::vec2(0.0, ascent_line),
                        rect.right_top() + egui::vec2(0.0, ascent_line),
                    ],
                    egui::Stroke::new(1.0, line_color),
                );
                painter.text(
                    rect.left_top() + egui::vec2(5.0, ascent_line - 15.0),
                    egui::Align2::LEFT_BOTTOM,
                    "ascender",
                    egui::FontId::default(),
                    label_color,
                );

                // Baseline
                painter.line_segment(
                    [
                        rect.left_top() + egui::vec2(0.0, baseline),
                        rect.right_top() + egui::vec2(0.0, baseline),
                    ],
                    egui::Stroke::new(2.0, line_color),
                );
                painter.text(
                    rect.left_top() + egui::vec2(5.0, baseline - 5.0),
                    egui::Align2::LEFT_BOTTOM,
                    "baseline",
                    egui::FontId::default(),
                    label_color,
                );

                // Descender line
                painter.line_segment(
                    [
                        rect.left_top() + egui::vec2(0.0, descent_line),
                        rect.right_top() + egui::vec2(0.0, descent_line),
                    ],
                    egui::Stroke::new(1.0, line_color),
                );
                painter.text(
                    rect.left_top() + egui::vec2(5.0, descent_line + 15.0),
                    egui::Align2::LEFT_TOP,
                    "descender",
                    egui::FontId::default(),
                    label_color,
                );

                // Draw the character - use appropriate font for emoji
                let font_family = if self.selected_char as u32 >= 0x1F300
                    || (self.selected_char as u32 >= 0x2600 && self.selected_char as u32 <= 0x27BF)
                {
                    // Emoji ranges
                    egui::FontFamily::Name(NOTO_EMOJI.into())
                } else {
                    egui::FontFamily::Name(NOTO_SANS.into())
                };

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    self.selected_char.to_string(),
                    egui::FontId::new(72.0, font_family),
                    egui::Color32::WHITE,
                );
            });
        }
    }

    fn render_glyph_grid(&mut self, ui: &mut egui::Ui) {
        let glyphs_to_show = self.get_glyphs_to_show();

        if glyphs_to_show.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No glyphs to display");
            });
            return;
        }

        // Calculate grid dimensions
        let scale_factor: f32 = self.glyph_scale.into();
        let base_size = 48.0 * scale_factor;
        let spacing = 4.0;

        let available_width = ui.available_width();
        let columns = ((available_width - spacing) / (base_size + spacing)).floor() as usize;
        let _columns = columns.max(1);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                for (chr, name) in glyphs_to_show {
                    let response = ui
                        .allocate_response(egui::vec2(base_size, base_size), egui::Sense::click());

                    // Handle double-click to copy
                    if response.double_clicked() {
                        ui.ctx().copy_text(chr.to_string());
                    } else if response.clicked() {
                        self.selected_char = chr;
                        self.add_to_recently_used(chr);
                    }

                    // Draw glyph
                    let rect = response.rect;
                    let is_in_collection = self.collection.contains(&chr);

                    ui.painter().rect_filled(
                        rect,
                        2.0,
                        if is_in_collection {
                            egui::Color32::from_rgb(40, 60, 40)
                        } else {
                            egui::Color32::from_rgb(30, 30, 30)
                        },
                    );

                    // Use appropriate font for emoji
                    let font_family = if chr as u32 >= 0x1F300
                        || (chr as u32 >= 0x2600 && chr as u32 <= 0x27BF)
                    {
                        // Emoji ranges
                        egui::FontFamily::Name(NOTO_EMOJI.into())
                    } else {
                        self.default_font_id.family.clone()
                    };

                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        chr.to_string(),
                        egui::FontId::new(base_size * 0.6, font_family),
                        egui::Color32::WHITE,
                    );

                    // Show enhanced tooltip with more info
                    response.on_hover_ui(|ui| {
                        ui.label(egui::RichText::new(chr.to_string()).size(24.0));
                        ui.label(&name);
                        ui.label(format!("U+{:04X}", chr as u32));
                        ui.separator();
                        ui.label("Double-click to copy");
                    });
                }
            });
        });
    }

    fn get_glyphs_to_show(&self) -> Vec<(char, String)> {
        if self.selected_category == recently_used_id() {
            self.recently_used
                .iter()
                .map(|&c| (c, char_name(c)))
                .collect()
        } else if self.selected_category == collection_id() {
            let mut glyphs: Vec<_> = self.collection.iter().map(|&c| (c, char_name(c))).collect();
            glyphs.sort_by_key(|&(c, _)| c);
            glyphs
        } else if self.search_active
            && (self.selected_category == search_id() || !self.search_text.is_empty())
        {
            // Only show search results if search is active
            self.showed_glyph_cache
                .iter()
                .map(|(&c, n)| (c, n.clone()))
                .collect()
        } else {
            // Show glyphs from selected category
            let category = self
                .categories
                .iter()
                .find(|c| c.id() == self.selected_category);

            if let Some(cat) = category {
                cat.unicode_category
                    .characters()
                    .into_iter()
                    .filter_map(|c| self.full_glyph_cache.get(&c).map(|n| (c, n.clone())))
                    .collect()
            } else {
                // No category selected - show all available glyphs
                self.full_glyph_cache
                    .iter()
                    .map(|(&c, n)| (c, n.clone()))
                    .collect()
            }
        }
    }
}

// Helper methods
impl GlyphanaApp {
    fn update_full_glyph_cache(&mut self, ctx: &egui::Context) {
        // Get characters from multiple font families to ensure we capture all glyphs including emoji
        let mut all_chars = available_characters(ctx, egui::FontFamily::Name(NOTO_SANS.into()));

        // Also get characters from the emoji font family
        let emoji_chars = available_characters(ctx, egui::FontFamily::Name(NOTO_EMOJI.into()));
        all_chars.extend(emoji_chars);

        // Also check Proportional family which includes all fonts
        let prop_chars = available_characters(ctx, egui::FontFamily::Proportional);
        all_chars.extend(prop_chars);

        self.full_glyph_cache = all_chars;
        self.update_search_text_and_cache();
    }

    fn update_search_text_and_cache(&mut self) {
        self.search_text = self.ui_search_text.clone();
        self.split_search_text = self
            .search_text
            .split_whitespace()
            .map(str::to_string)
            .collect();
        self.split_search_text_lower = if !self.case_sensitive {
            self.split_search_text
                .iter()
                .map(|s| s.to_lowercase())
                .collect()
        } else {
            vec![]
        };

        // Use the new search engine
        let params = SearchParams::new(
            self.search_text.clone(),
            self.search_only_categories,
            self.search_name,
            self.case_sensitive,
        );

        self.showed_glyph_cache = SearchEngine::search(
            &params,
            &self.full_glyph_cache,
            &self.categories,
            self.selected_category,
        );
    }

    fn add_to_recently_used(&mut self, chr: char) {
        // Remove if already exists
        if let Some(pos) = self.recently_used.iter().position(|&c| c == chr) {
            self.recently_used.remove(pos);
        }

        // Add to front
        self.recently_used.push_front(chr);

        // Trim to max length
        while self.recently_used.len() > self.recently_used_max_len {
            self.recently_used.pop_back();
        }
    }

    fn paint_glyph(
        &mut self,
        scale: f32,
        ui: &mut egui::Ui,
        response: egui::Response,
        painter: egui::Painter,
    ) {
        let rect = response.rect;
        let center = rect.center();
        let glyph_scale = scale * 0.8;
        let offset = scale * 0.12;

        let left = rect.min.x + offset;
        let top = rect.min.y + offset;
        let right = rect.max.x - offset;

        // Try to get font metrics
        let font_data = include_bytes!("../assets/NotoSans-Regular.otf");
        let v_metrics = if let Some(font) = rusttype::Font::try_from_bytes(font_data) {
            font.v_metrics(rusttype::Scale::uniform(glyph_scale))
        } else {
            // Fallback metrics if font loading fails
            rusttype::VMetrics {
                ascent: glyph_scale * 0.8,
                descent: -glyph_scale * 0.2,
                line_gap: glyph_scale * 0.1,
            }
        };

        let visuals = &ui.ctx().style().visuals;
        let dark_mode = visuals.dark_mode;

        let glyph_color = if dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };

        let mut stroke = visuals.widgets.noninteractive.fg_stroke;
        let info_text_color = stroke.color;
        stroke.color = stroke
            .color
            .linear_multiply(info_text_color.r() as f32 / 255.0 * 0.3);

        // Draw the glyph - use appropriate font family for emoji
        // Check if the character is likely an emoji based on Unicode ranges
        let font_family = if self.selected_char as u32 >= 0x1F300
            || (self.selected_char as u32 >= 0x2600 && self.selected_char as u32 <= 0x27BF)
        {
            // Emoji ranges
            egui::FontFamily::Name(NOTO_EMOJI.into())
        } else {
            egui::FontFamily::Name(NOTO_SANS.into())
        };

        painter.text(
            egui::Pos2::new(center.x, top + scale + glyph_scale * 0.023),
            egui::Align2::CENTER_BOTTOM,
            self.selected_char,
            egui::FontId::new(glyph_scale, font_family),
            glyph_color,
        );

        // Draw ascender line
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale - v_metrics.ascent),
                egui::Pos2::new(right, top + glyph_scale - v_metrics.ascent),
            ],
            stroke,
        );

        // Label for ascender
        painter.text(
            egui::Pos2::new(left - 5.0, top + glyph_scale - v_metrics.ascent),
            egui::Align2::RIGHT_CENTER,
            "ascender",
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            stroke.color,
        );

        // Draw baseline
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale),
                egui::Pos2::new(right, top + glyph_scale),
            ],
            stroke,
        );

        // Label for baseline
        painter.text(
            egui::Pos2::new(left - 5.0, top + glyph_scale),
            egui::Align2::RIGHT_CENTER,
            "baseline",
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            stroke.color,
        );

        // Draw descender line
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale - v_metrics.descent),
                egui::Pos2::new(right, top + glyph_scale - v_metrics.descent),
            ],
            stroke,
        );

        // Label for descender
        painter.text(
            egui::Pos2::new(left - 5.0, top + glyph_scale - v_metrics.descent),
            egui::Align2::RIGHT_CENTER,
            "descender",
            egui::FontId::new(10.0, egui::FontFamily::Proportional),
            stroke.color,
        );

        ui.expand_to_include_rect(painter.clip_rect());
    }
}

// Font name constants
pub const NOTO_SANS: &str = "NotoSans";
pub const NOTO_SANS_MONO: &str = "NotoSansMono";
pub const NOTO_SANS_SYMBOLS: &str = "NotoSansSymbols";
pub const NOTO_SANS_SYMBOLS2: &str = "NotoSansSymbols2";
pub const NOTO_SANS_MATH: &str = "NotoSansMath";
pub const NOTO_MUSIC: &str = "NotoMusic";
pub const NOTO_EMOJI: &str = "NotoEmoji";
pub const EMOJI_ICON: &str = "EmojiIcon";
