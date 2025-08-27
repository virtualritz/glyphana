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
    COLLECTION, MAGNIFIER, NAME_BADGE, RECENTLY_USED, SEARCH, collection_id, recently_used_id,
    search_id,
};

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
    pixels_per_point: f32,
    glyph_scale: GlyphScale,
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
            pixels_per_point: Default::default(),
            glyph_scale: GlyphScale::Normal,
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

        // AIDEV-NOTE: Color emoji support needs NotoColorEmoji.ttf file
        // The file exists in assets/fonts/ but needs to be enabled properly
        // This would require additional font rendering support for color glyphs

        // Configure font families - create base font list to avoid duplication
        let base_fonts = vec![
            NOTO_EMOJI.to_owned(),
            EMOJI_ICON.to_owned(),
            NOTO_SANS_SYMBOLS.to_owned(),
            NOTO_SANS_SYMBOLS2.to_owned(),
            NOTO_SANS_MATH.to_owned(),
            NOTO_MUSIC.to_owned(),
        ];

        // Proportional font family
        let mut proportional_fonts = vec![NOTO_SANS.to_owned()];
        proportional_fonts.extend(base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Proportional, proportional_fonts);

        // Monospace font family
        let mut monospace_fonts = vec![NOTO_SANS_MONO.to_owned()];
        monospace_fonts.extend(base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Monospace, monospace_fonts);

        // Named NotoSans font family
        let mut noto_sans_fonts = vec![NOTO_SANS.to_owned()];
        noto_sans_fonts.extend(base_fonts.clone());
        fonts
            .families
            .insert(egui::FontFamily::Name(NOTO_SANS.into()), noto_sans_fonts);

        // Named NotoEmoji font family (emoji prioritized)
        let mut emoji_fonts = vec![
            NOTO_EMOJI.to_owned(),
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

        // Side panel with categories
        self.render_side_panel(ctx);

        // Central panel with glyphs
        self.render_central_panel(ctx);
    }
}

// UI rendering methods
impl GlyphanaApp {
    fn render_top_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(MAGNIFIER.to_string());

                let search_response = ui.text_edit_singleline(&mut self.ui_search_text);
                if search_response.changed() {
                    self.update_search_text_and_cache();
                }

                if ui.button("Clear").clicked() {
                    self.ui_search_text.clear();
                    self.update_search_text_and_cache();
                }

                ui.separator();

                ui.checkbox(
                    &mut self.search_only_categories,
                    "Search only selected category",
                );
                ui.checkbox(
                    &mut self.search_name,
                    format!("{} Search names", NAME_BADGE),
                );
                ui.checkbox(&mut self.case_sensitive, "Case sensitive");

                ui.separator();

                ui.label("Size:");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.glyph_scale == GlyphScale::Tiny, "Tiny")
                        .clicked()
                    {
                        self.glyph_scale = GlyphScale::Tiny;
                    }
                    if ui
                        .selectable_label(self.glyph_scale == GlyphScale::Small, "Small")
                        .clicked()
                    {
                        self.glyph_scale = GlyphScale::Small;
                    }
                    if ui
                        .selectable_label(self.glyph_scale == GlyphScale::Normal, "Normal")
                        .clicked()
                    {
                        self.glyph_scale = GlyphScale::Normal;
                    }
                    if ui
                        .selectable_label(self.glyph_scale == GlyphScale::Large, "Large")
                        .clicked()
                    {
                        self.glyph_scale = GlyphScale::Large;
                    }
                    if ui
                        .selectable_label(self.glyph_scale == GlyphScale::Huge, "Huge")
                        .clicked()
                    {
                        self.glyph_scale = GlyphScale::Huge;
                    }
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
                self.selected_category = cat_id;
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
                self.selected_category = recently_used_id();
                self.update_search_text_and_cache();
            }

            if ui
                .selectable_label(self.selected_category == collection_id(), COLLECTION)
                .clicked()
            {
                self.selected_category = collection_id();
                self.update_search_text_and_cache();
            }

            if ui
                .selectable_label(self.selected_category == search_id(), SEARCH)
                .clicked()
            {
                self.selected_category = search_id();
                self.update_search_text_and_cache();
            }
        });
    }

    fn render_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Show single glyph view if a character is selected
            if self.selected_char != '\0' {
                self.render_single_glyph_view(ui);
            } else {
                self.render_glyph_grid(ui);
            }
        });
    }

    fn render_single_glyph_view(&mut self, ui: &mut egui::Ui) {
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

        // Display character info
        ui.heading(format!("{}", self.selected_char));
        ui.label(format!("Name: {}", char_name(self.selected_char)));
        ui.label(format!(
            "Code: U+{:04X} ({})",
            self.selected_char as u32, self.selected_char as u32
        ));

        ui.separator();

        // Font preview with proper ascender/descender lines
        self.render_font_preview(ui);
    }

    fn render_font_preview(&self, ui: &mut egui::Ui) {
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

                // Draw the character
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    self.selected_char.to_string(),
                    egui::FontId::new(72.0, egui::FontFamily::Name(NOTO_SANS.into())),
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

                    if response.clicked() {
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

                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        chr.to_string(),
                        egui::FontId::new(base_size * 0.6, self.default_font_id.family.clone()),
                        egui::Color32::WHITE,
                    );

                    // Show tooltip
                    response.on_hover_text(&name);
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
        } else if self.selected_category == search_id() || !self.search_text.is_empty() {
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
                vec![]
            }
        }
    }
}

// Helper methods
impl GlyphanaApp {
    fn update_full_glyph_cache(&mut self, _ctx: &egui::Context) {
        // AIDEV-TODO: Update this to use proper font range detection from egui
        // For now, use a simplified version
        self.full_glyph_cache = available_characters();
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
