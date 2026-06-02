use ahash::AHashSet as HashSet;
use egui_dnd::dnd;
use include_flate::flate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::sync::Arc;
use unicode_case_mapping;

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

fn to_uppercase_string(s: &str) -> String {
    s.chars()
        .map(|c| {
            let mapped = unicode_case_mapping::to_uppercase(c);
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

// Compressed font data
flate!(static NOTO_SANS_DATA: [u8] from "assets/NotoSans-Regular.otf");
flate!(static NOTO_SANS_SYMBOLS_DATA: [u8] from "assets/NotoSansSymbols-Regular.ttf");
flate!(static NOTO_SANS_SYMBOLS2_DATA: [u8] from "assets/NotoSansSymbols2-Regular.ttf");
flate!(static NOTO_SANS_MATH_DATA: [u8] from "assets/NotoSansMath-Regular.ttf");
flate!(static NOTO_MUSIC_DATA: [u8] from "assets/NotoMusic-Regular.ttf");
flate!(static NOTO_EMOJI_DATA: [u8] from "assets/NotoEmoji-Regular.ttf");

use crate::categories::{
    Category, CharacterInspector, UnicodeCategory, UnicodeCollection, create_default_categories,
};
use crate::font_manager::{FontManager, NotoFontMapping};
use crate::glyph::{GlyphScale, available_characters, char_name};
use crate::search::{SearchEngine, SearchParams};
use crate::ui::{
    CANCELLATION, COLLECTION, HAMBURGER, LOWER_UPPER_CASE, MAGNIFIER, NAME_BADGE, RECENTLY_USED,
    SEARCH, SUBSET, collection_id, recently_used_id, search_id,
};

// Inspector view mode - either related characters or font variations
// #[derive(Debug, Clone, Copy, PartialEq)]
// enum InspectorViewMode {
//     RelatedCharacters,
//     FontVariations,
// }

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
    // #[serde(skip)]
    // inspector_view_mode: InspectorViewMode,

    // Keep the app in tray when closed (persisted)
    keep_in_tray: bool,

    // Track if we should restore window
    #[serde(skip)]
    should_restore_window: bool,

    // Track if we're running on Wayland
    #[serde(skip)]
    is_wayland: bool,

    // File dialog for export
    #[serde(skip)]
    file_dialog: egui_file_dialog::FileDialog,

    // Settings dialog state
    #[serde(skip)]
    show_settings_dialog: bool,

    // Font manager
    #[serde(skip)]
    font_manager: Option<FontManager>,

    // Settings tab
    #[serde(skip)]
    settings_tab: SettingsTab,

    // Toast notifications
    #[serde(skip)]
    toasts: egui_notify::Toasts,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum SettingsTab {
    #[default]
    Categories,
    Fonts,
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
            // inspector_view_mode: InspectorViewMode::RelatedCharacters,
            keep_in_tray: false,
            should_restore_window: false,
            is_wayland: false,
            file_dialog: egui_file_dialog::FileDialog::new(),
            show_settings_dialog: false,
            font_manager: FontManager::new().ok(),
            settings_tab: SettingsTab::default(),
            toasts: egui_notify::Toasts::default().with_anchor(egui_notify::Anchor::BottomRight),
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

        // Detect if we're running on Wayland
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        eprintln!("Running on Wayland: {}", is_wayland);

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            eprintln!("Loaded app state, keep_in_tray: {}", app.keep_in_tray);
            // Re-initialize categories after deserialization
            for category in &mut app.categories {
                category.unicode_category = Self::unicode_category_for_name(&category.name);
            }
            // Re-initialize file dialog since it's skipped in serialization
            app.file_dialog = egui_file_dialog::FileDialog::new();
            app.should_restore_window = false;
            app.is_wayland = is_wayland;
            app.show_settings_dialog = false;
            app.font_manager = FontManager::new().ok();
            app.settings_tab = SettingsTab::default();

            // Re-initialize required fonts for categories after deserialization
            for category in &mut app.categories {
                category.required_font = Category::get_required_font(&category.name);
            }

            // Initialize fonts for visible categories
            app.initialize_required_fonts(&cc.egui_ctx);

            // Re-run search if there was an active search query
            if !app.ui_search_text.is_empty() {
                app.search_active = true;
                app.update_search_text_and_cache();
            }

            app
        } else {
            let app = Self {
                is_wayland,
                ..Self::default()
            };

            // Initialize fonts for visible categories
            app.initialize_required_fonts(&cc.egui_ctx);

            app
        }
    }

    fn unicode_category_for_name(name: &str) -> UnicodeCategory {
        use crate::categories::PropertyType;

        // Map category names back to their Unicode categories
        // This is needed for deserialization since we skip the unicode_category field

        // Check property-based categories first
        match name {
            "Uppercase Letters" => {
                return UnicodeCategory::Property(PropertyType::UppercaseLetters);
            }
            "Lowercase Letters" => {
                return UnicodeCategory::Property(PropertyType::LowercaseLetters);
            }
            "All Letters" => return UnicodeCategory::Property(PropertyType::AllLetters),
            "Math Symbols" => return UnicodeCategory::Property(PropertyType::MathSymbols),
            "Currency Symbols" => return UnicodeCategory::Property(PropertyType::CurrencySymbols),
            "Punctuation" => return UnicodeCategory::Property(PropertyType::Punctuation),
            "Decimal Numbers" => return UnicodeCategory::Property(PropertyType::DecimalNumbers),
            "All Numbers" => return UnicodeCategory::Property(PropertyType::AllNumbers),
            "All Symbols" => return UnicodeCategory::Property(PropertyType::AllSymbols),
            _ => {}
        }

        // Then check special cases
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

        // Add Noto Sans (compressed)
        fonts.font_data.insert(
            NOTO_SANS.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_SANS_DATA)),
        );

        // Add Noto Sans Mono
        fonts.font_data.insert(
            NOTO_SANS_MONO.to_owned(),
            // NotoSansMono not available, use NotoSans as fallback
            Arc::new(egui::FontData::from_static(&NOTO_SANS_DATA)),
        );

        // Add Noto Sans Symbols (compressed)
        fonts.font_data.insert(
            NOTO_SANS_SYMBOLS.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_SANS_SYMBOLS_DATA)),
        );

        // Add Noto Sans Symbols 2 (large file - 1.2M, compressed)
        fonts.font_data.insert(
            NOTO_SANS_SYMBOLS2.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_SANS_SYMBOLS2_DATA)),
        );

        // Add Noto Sans Math (compressed)
        fonts.font_data.insert(
            NOTO_SANS_MATH.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_SANS_MATH_DATA)),
        );

        // Add Noto Music (compressed)
        fonts.font_data.insert(
            NOTO_MUSIC.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_MUSIC_DATA)),
        );

        // Add Noto Emoji (black and white, compressed)
        fonts.font_data.insert(
            NOTO_EMOJI.to_owned(),
            Arc::new(egui::FontData::from_static(&NOTO_EMOJI_DATA)),
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

        // Register NotoSansMono as a named font family
        fonts.families.insert(
            egui::FontFamily::Name(NOTO_SANS_MONO.into()),
            vec![NOTO_SANS_MONO.to_owned(), NOTO_SANS.to_owned()],
        );

        // Register other named font families for font variations
        fonts.families.insert(
            egui::FontFamily::Name(NOTO_SANS_SYMBOLS.into()),
            vec![NOTO_SANS_SYMBOLS.to_owned(), NOTO_SANS.to_owned()],
        );

        fonts.families.insert(
            egui::FontFamily::Name(NOTO_SANS_SYMBOLS2.into()),
            vec![NOTO_SANS_SYMBOLS2.to_owned(), NOTO_SANS.to_owned()],
        );

        fonts.families.insert(
            egui::FontFamily::Name(NOTO_SANS_MATH.into()),
            vec![NOTO_SANS_MATH.to_owned(), NOTO_SANS.to_owned()],
        );

        fonts.families.insert(
            egui::FontFamily::Name(NOTO_MUSIC.into()),
            vec![NOTO_MUSIC.to_owned(), NOTO_SANS.to_owned()],
        );

        fonts
    }
}

impl eframe::App for GlyphanaApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eprintln!("Saving app state, keep_in_tray: {}", self.keep_in_tray);
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let ctx = &ctx;
        // If we should restore the window, keep trying
        if self.should_restore_window {
            // Unminimize and focus
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            // Keep requesting repaints until the window is actually restored
            ctx.request_repaint();
            // Reset flag after a few frames to avoid infinite loop
            self.should_restore_window = false;
        }

        // Handle window close request (X button)
        if ctx.input(|i| i.viewport().close_requested()) {
            eprintln!("Close requested, keep_in_tray: {}", self.keep_in_tray);
            if self.keep_in_tray {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);

                if self.is_wayland {
                    // On Wayland, we can't properly hide/minimize to tray
                    // The window stays visible but the close is cancelled
                    eprintln!("Wayland: Window stays visible (minimize to tray not supported)");
                } else {
                    // On X11/Windows/macOS, minimize the window to tray
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    // Keep requesting repaints so we can respond to tray events
                    ctx.request_repaint_after(std::time::Duration::from_millis(100));
                }
            }
            // If keep_in_tray is false, let the close proceed normally
        }

        // Handle tray icon events
        use tray_icon::{TrayIconEvent, menu::MenuEvent};

        // Check for tray icon clicks
        if let Ok(TrayIconEvent::Click { .. }) = TrayIconEvent::receiver().try_recv() {
            eprintln!("Tray icon clicked");
            if self.is_wayland {
                // On Wayland, just focus the window
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else {
                // On X11/Windows/macOS, restore from minimized state
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.should_restore_window = true;
            }
            ctx.request_repaint();
        }

        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id.as_ref() {
                "show" => {
                    eprintln!("Show Glyphana clicked");
                    if self.is_wayland {
                        // On Wayland, just focus the window (can't restore from minimized)
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    } else {
                        // On X11/Windows/macOS, restore from minimized state
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    ctx.request_repaint();
                }
                "quit" => {
                    // Quit application - force exit
                    std::process::exit(0);
                }
                _ => {}
            }
        }

        // Keep requesting repaints when minimized to handle tray events (not on Wayland)
        if !self.is_wayland
            && ctx.input(|i| i.viewport().minimized.unwrap_or(false))
            && self.keep_in_tray
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // Update file dialog
        self.file_dialog.update(ctx);

        // Check if the user picked a file for export
        if let Some(path) = self.file_dialog.take_picked() {
            // Add .txt extension if not present
            let mut path = path.to_path_buf();
            if path.extension().is_none() {
                path.set_extension("txt");
            }
            self.export_collection(path);
        }

        // Update settings dialog
        self.show_settings_dialog(ctx);

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
        self.render_top_panel(ui);

        // Left side panel with categories
        self.render_side_panel(ui);

        // Right side panel with character preview (always visible)
        self.render_right_panel(ui);

        // Central panel with glyphs
        self.render_central_panel(ui);

        // Show toast notifications
        self.toasts.show(ctx);
    }
}

// UI rendering methods
impl GlyphanaApp {
    fn render_top_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
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

                    if ui.button("Export Collection…").clicked() {
                        // Open the file dialog to save a file
                        self.file_dialog.save_file();
                    }

                    ui.separator();

                    if ui.button("Settings…").clicked() {
                        self.show_settings_dialog = true;
                        ui.close_menu();
                    }

                    ui.separator();

                    ui.checkbox(&mut self.keep_in_tray, "Keep in Tray");

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        // Always close the app when Quit is clicked - force exit
                        std::process::exit(0);
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
                    if ui
                        .toggle_value(&mut self.case_sensitive, LOWER_UPPER_CASE.to_string())
                        .on_hover_text("Case Sensitive")
                        .changed()
                    {
                        self.update_search_text_and_cache();
                    }

                    // Search names toggle
                    ui.add_enabled_ui(!self.case_sensitive, |ui| {
                        if ui
                            .toggle_value(&mut self.search_name, NAME_BADGE.to_string())
                            .on_hover_text("Search Glyph Names")
                            .changed()
                        {
                            self.update_search_text_and_cache();
                        }
                    });

                    // Search only in categories toggle
                    if ui
                        .toggle_value(&mut self.search_only_categories, SUBSET.to_string())
                        .on_hover_text("Search Only Selected Category")
                        .changed()
                    {
                        self.update_search_text_and_cache();
                    }
                });
            });
        });
    }

    fn render_side_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("side_panel").show_inside(ui, |ui| {
            ui.heading("Categories");

            // Handle drag and drop
            let selected_category = self.selected_category;
            let mut category_clicked = None;

            // Filter to only show visible categories
            let mut visible_categories: Vec<&mut Category> = self
                .categories
                .iter_mut()
                .filter(|cat| cat.visible)
                .collect();

            let response = dnd(ui, "category_dnd").show(
                visible_categories.iter_mut(),
                |ui, category, handle, _| {
                    ui.horizontal(|ui| {
                        handle.ui(ui, |ui| {
                            ui.label("≡");
                        });

                        let is_selected = selected_category == category.id();

                        // Check if this category needs a font that's being downloaded
                        let is_downloading = category
                            .required_font
                            .as_ref()
                            .and_then(|font_name| {
                                self.font_manager
                                    .as_ref()
                                    .and_then(|fm| fm.download_progress(font_name))
                            })
                            .is_some();

                        // Create label with download indicator
                        let label_text = if is_downloading {
                            format!("{} ⏳", category.name)
                        } else {
                            category.name.clone()
                        };

                        let response = ui.selectable_label(is_selected, label_text);

                        if response.clicked() {
                            category_clicked = Some(category.id());
                        }

                        // Show download indicator if font is being downloaded
                        if is_downloading {
                            response.on_hover_text("Downloading font...");
                            // Request continuous repaints while downloading
                            ui.ctx().request_repaint();
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

                    // Check if this category needs a font to be downloaded
                    if let Some(category) = self.categories.iter().find(|c| c.id() == cat_id)
                        && let Some(ref font_name) = category.required_font
                        && let Some(ref font_manager) = self.font_manager
                        && !font_manager.is_cached(font_name)
                        && let Some((_, font_url)) =
                            NotoFontMapping::font_for_script(&category.name)
                    {
                        // Clone what we need for the background task
                        let font_name_clone = font_name.clone();
                        let font_url_clone = font_url.to_string();
                        let font_manager_clone = font_manager.clone();
                        let ctx_clone = ui.ctx().clone();

                        // Download font in background thread
                        std::thread::spawn(move || {
                            match font_manager_clone.load_font(&font_name_clone, &font_url_clone) {
                                Ok(font_data) => {
                                    // Read existing font definitions and add the new font
                                    let mut font_definitions =
                                        ctx_clone.fonts(|f| f.definitions().clone());

                                    font_definitions.font_data.insert(
                                        font_name_clone.clone(),
                                        egui::FontData::from_owned(font_data).into(),
                                    );

                                    // Add to proportional family
                                    font_definitions
                                        .families
                                        .entry(egui::FontFamily::Proportional)
                                        .or_default()
                                        .push(font_name_clone.clone());

                                    ctx_clone.set_fonts(font_definitions);
                                    ctx_clone.request_repaint();

                                    eprintln!("Successfully loaded font: {}", font_name_clone);
                                }
                                Err(e) => {
                                    eprintln!("Failed to download font {}: {}", font_name_clone, e);
                                }
                            }
                        });
                    }
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
    fn related_characters(&self, ch: char) -> Vec<char> {
        let mut related = Vec::new();
        let code_point = ch as u32;

        // First, get all characters that normalize to the same skeleton
        // This will find accent variants, case variants, etc.
        let base_skeleton = self.normalize_char_for_matching(ch);

        // Search through available characters to find matches
        for &other_char in self.full_glyph_cache.keys() {
            if other_char != ch {
                let other_skeleton = self.normalize_char_for_matching(other_char);
                if !base_skeleton.is_empty() && base_skeleton == other_skeleton {
                    related.push(other_char);
                }
            }
        }

        // Add case variations using proper Unicode case mapping
        if ch.is_lowercase() {
            let upper_str = to_uppercase_string(&ch.to_string());
            for upper in upper_str.chars() {
                if upper != ch {
                    related.push(upper);
                }
            }
        } else if ch.is_uppercase() {
            let lower_str = to_lowercase_string(&ch.to_string());
            for lower in lower_str.chars() {
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
                if cp != code_point
                    && let Some(nearby_char) = char::from_u32(cp)
                    && !related.contains(&nearby_char)
                {
                    related.push(nearby_char);
                }
            }
        }

        // Add diacritic variations and ligatures for Latin characters
        // Get the base character by removing accents
        let base_char_str = self.normalize_char_for_matching(ch);
        let base_char = base_char_str
            .chars()
            .next()
            .unwrap_or(ch)
            .to_ascii_lowercase();

        // AIDEV-NOTE: Expanded mapping to include:
        // - Both uppercase and lowercase variations
        // - Ligatures (Æ/æ for A, Œ/œ for O, etc.)
        // - Nordic letters (Ø/ø for O)
        // - More complete diacritic coverage
        // This ensures users see all semantically related characters
        let diacritic_variations: Vec<(char, Vec<char>)> = vec![
            (
                'a',
                vec![
                    'à', 'á', 'â', 'ã', 'ä', 'å', 'ā', 'ă', 'ą', 'ǎ', 'ȁ', 'ȃ', 'À', 'Á', 'Â', 'Ã',
                    'Ä', 'Å', 'Ā', 'Ă', 'Ą', 'Ǎ', 'Ȁ', 'Ȃ', 'Æ', 'æ', 'Ǽ', 'ǽ', 'Ǣ',
                    'ǣ', // A-E ligatures
                ],
            ),
            (
                'e',
                vec![
                    'è', 'é', 'ê', 'ë', 'ē', 'ė', 'ę', 'ě', 'ȅ', 'ȇ', 'ẽ', 'È', 'É', 'Ê', 'Ë', 'Ē',
                    'Ė', 'Ę', 'Ě', 'Ȅ', 'Ȇ', 'Ẽ',
                ],
            ),
            (
                'i',
                vec![
                    'ì', 'í', 'î', 'ï', 'ī', 'į', 'ı', 'ǐ', 'ĩ', 'Ì', 'Í', 'Î', 'Ï', 'Ī', 'Į', 'İ',
                    'Ǐ', 'Ĩ', 'Ĳ', 'ĳ', // I-J ligature
                ],
            ),
            (
                'o',
                vec![
                    'ò', 'ó', 'ô', 'õ', 'ö', 'ø', 'ō', 'ő', 'ǒ', 'ȍ', 'ȏ', 'ơ', 'Ò', 'Ó', 'Ô', 'Õ',
                    'Ö', 'Ø', 'Ō', 'Ő', 'Ǒ', 'Ȍ', 'Ȏ', 'Ơ', 'Œ', 'œ', 'Ǿ',
                    'ǿ', // O-E ligatures and O with stroke and acute
                ],
            ),
            (
                'u',
                vec![
                    'ù', 'ú', 'û', 'ü', 'ū', 'ů', 'ű', 'ų', 'ǔ', 'ũ', 'ȕ', 'ȗ', 'Ù', 'Ú', 'Û', 'Ü',
                    'Ū', 'Ů', 'Ű', 'Ų', 'Ǔ', 'Ũ', 'Ȕ', 'Ȗ',
                ],
            ),
            ('c', vec!['ç', 'ć', 'č', 'ĉ', 'Ç', 'Ć', 'Č', 'Ĉ']),
            ('n', vec!['ñ', 'ń', 'ň', 'ņ', 'Ñ', 'Ń', 'Ň', 'Ņ']),
            (
                's',
                vec!['ś', 'š', 'ş', 'ŝ', 'ș', 'Ś', 'Š', 'Ş', 'Ŝ', 'Ș', 'ß'],
            ),
            ('z', vec!['ź', 'ž', 'ż', 'Ź', 'Ž', 'Ż']),
            ('d', vec!['ď', 'đ', 'Ď', 'Đ', 'Ð', 'ð']),
            ('g', vec!['ğ', 'ģ', 'ĝ', 'ġ', 'Ğ', 'Ģ', 'Ĝ', 'Ġ']),
            ('h', vec!['ĥ', 'ħ', 'Ĥ', 'Ħ']),
            ('j', vec!['ĵ', 'Ĵ']),
            ('k', vec!['ķ', 'ĸ', 'Ķ']),
            ('l', vec!['ł', 'ľ', 'ļ', 'ŀ', 'Ł', 'Ľ', 'Ļ', 'Ŀ']),
            ('r', vec!['ř', 'ŕ', 'ŗ', 'Ř', 'Ŕ', 'Ŗ']),
            ('t', vec!['ť', 'ţ', 'ŧ', 'ț', 'Ť', 'Ţ', 'Ŧ', 'Ț', 'Þ', 'þ']),
            ('w', vec!['ŵ', 'ẅ', 'ẃ', 'Ŵ', 'Ẅ', 'Ẃ']),
            ('y', vec!['ý', 'ÿ', 'ŷ', 'ȳ', 'Ý', 'Ÿ', 'Ŷ', 'Ȳ']),
        ];

        for (base, variations) in diacritic_variations {
            if base_char == base {
                for var in variations {
                    if var != ch && !related.contains(&var) {
                        related.push(var);
                    }
                }
                break;
            }
        }

        // Sort by Unicode proximity to the original character
        let char_code = ch as i64;
        related.sort_by_key(|&c| (c as i64 - char_code).abs());

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        related.retain(|&c| seen.insert(c));

        // Limit to first 48 related characters to show more variations
        related.truncate(48);
        related
    }

    // Helper function to normalize a character for matching
    fn normalize_char_for_matching(&self, ch: char) -> String {
        use unicode_normalization::UnicodeNormalization;
        use unicode_skeleton::UnicodeSkeleton;

        let s = ch.to_string();

        // First decompose Unicode characters (NFD normalization)
        let decomposed: String = s.nfd().collect();

        // Remove combining marks (accents, diacritics)
        let without_accents: String = decomposed
            .chars()
            .filter(|&c| {
                let code = c as u32;
                !((0x0300..=0x036F).contains(&code)
                    || (0x1AB0..=0x1AFF).contains(&code)
                    || (0x1DC0..=0x1DFF).contains(&code)
                    || (0x20D0..=0x20FF).contains(&code)
                    || (0xFE20..=0xFE2F).contains(&code))
            })
            .collect();

        // Use lowercase for case-insensitive matching
        let lowercase = to_lowercase_string(&without_accents);

        // Try unicode_skeleton for additional normalization
        let skeleton = lowercase.skeleton_chars().collect::<String>();

        if !skeleton.is_empty() {
            skeleton
        } else {
            lowercase
        }
    }

    // Get available fonts that have the character
    fn export_collection(&self, path: std::path::PathBuf) {
        use std::fs::File;
        use std::io::Write;

        // Prepare the content
        let mut content = String::new();
        content.push_str("Glyphana Character Collection\n");
        content.push_str("=============================\n\n");

        // Sort characters for consistent output
        let mut sorted_chars: Vec<char> = self.collection.iter().copied().collect();
        sorted_chars.sort();

        for ch in sorted_chars {
            let name = char_name(ch);
            let code = format!("U+{:04X}", ch as u32);
            let decimal = ch as u32;
            let html = format!("&#{};", decimal);

            content.push_str(&format!(
                "{} - {} - {} ({}) - HTML: {}\n",
                ch, name, code, decimal, html
            ));
        }

        content.push_str(&format!("\nTotal: {} characters\n", self.collection.len()));

        // Write to file
        if let Ok(mut file) = File::create(&path)
            && let Err(e) = file.write_all(content.as_bytes())
        {
            eprintln!("Failed to write file: {}", e);
        }
    }

    // Font variations feature commented out for now
    /*
    fn font_variations(
        &self,
        ch: char,
        ctx: &egui::Context,
    ) -> Vec<(&'static str, egui::FontFamily)> {
        // List all available fonts
        let all_fonts = [
            (NOTO_SANS, egui::FontFamily::Name(NOTO_SANS.into())),
            (
                NOTO_SANS_MONO,
                egui::FontFamily::Name(NOTO_SANS_MONO.into()),
            ),
            (
                NOTO_SANS_SYMBOLS,
                egui::FontFamily::Name(NOTO_SANS_SYMBOLS.into()),
            ),
            (
                NOTO_SANS_SYMBOLS2,
                egui::FontFamily::Name(NOTO_SANS_SYMBOLS2.into()),
            ),
            (
                NOTO_SANS_MATH,
                egui::FontFamily::Name(NOTO_SANS_MATH.into()),
            ),
            (NOTO_MUSIC, egui::FontFamily::Name(NOTO_MUSIC.into())),
            (NOTO_EMOJI, egui::FontFamily::Name(NOTO_EMOJI.into())),
        ];

        // Filter fonts that actually contain this character
        all_fonts
            .into_iter()
            .filter(|(_, font_family)| {
                // Check if the font contains this character by querying the font system
                ctx.fonts(|f| {
                    let mut fonts_lock = f.lock();
                    let font = fonts_lock
                        .fonts
                        .font(&egui::FontId::new(20.0, font_family.clone()));
                    font.characters().contains_key(&ch)
                })
            })
            .collect()
    }
    */

    fn render_right_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("character_preview").show_inside(ui, |ui| {
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

                        // Wrap long names to fit the panel width
                        let available_width = ui.available_width();
                        let char_width = 8.0; // Approximate character width in pixels
                        let max_chars = (available_width / char_width).max(20.0) as usize;

                        let wrapped_name = textwrap::wrap(&name, max_chars);
                        for line in wrapped_name {
                            ui.label(line.to_string());
                        }

                        ui.separator();

                        // Unicode codepoint - use egui's striped grid
                        egui::Grid::new("glyph_codepoints")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Unicode");
                                let unicode_string = format!("U+{:04X}", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&unicode_string).monospace())
                                    .on_hover_text("Click to copy Unicode")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(unicode_string.clone());
                                    self.toasts
                                        .info(format!("Copied {unicode_string}"))
                                        .duration(Some(std::time::Duration::from_secs(2)));
                                }
                                ui.end_row();

                                ui.label("Decimal");
                                let decimal_string = format!("{}", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&decimal_string).monospace())
                                    .on_hover_text("Click to copy decimal")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(decimal_string.clone());
                                    self.toasts
                                        .info(format!("Copied {decimal_string}"))
                                        .duration(Some(std::time::Duration::from_secs(2)));
                                }
                                ui.end_row();

                                ui.label("HTML");
                                let html_string = format!("&#x{:04X};", self.selected_char as u32);
                                if ui
                                    .button(egui::RichText::new(&html_string).monospace())
                                    .on_hover_text("Click to copy HTML entity")
                                    .clicked()
                                {
                                    ui.ctx().copy_text(html_string.clone());
                                    self.toasts
                                        .info(format!("Copied {html_string}"))
                                        .duration(Some(std::time::Duration::from_secs(2)));
                                }
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

                        // Just show related characters without the header
                        self.render_related_characters(ui);
                    } else {
                        ui.label("Select a character to see details");
                    }
                },
            );
        });
    }

    fn render_related_characters(&mut self, ui: &mut egui::Ui) {
        let related_chars = self.related_characters(self.selected_char);

        if related_chars.is_empty() {
            ui.label("No related characters found");
        } else {
            // Add scrolling support for the related characters
            egui::ScrollArea::vertical()
                .max_height(ui.available_height()) // Use remaining height
                .show(ui, |ui| {
                    // Create a grid for related characters
                    let columns = 3;
                    let button_size =
                        ui.available_width() / columns as f32 - ui.spacing().item_spacing.x;

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
                                let is_in_collection = self.collection.contains(&ch);

                                // Draw background matching main grid style
                                painter.rect_filled(
                                    rect,
                                    2.0,
                                    if is_in_collection {
                                        egui::Color32::from_rgb(40, 60, 40)
                                    } else {
                                        egui::Color32::from_rgb(30, 30, 30)
                                    },
                                );

                                // Draw character
                                painter.text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    ch,
                                    egui::FontId::new(
                                        24.0,
                                        egui::FontFamily::Name(NOTO_SANS.into()),
                                    ),
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
                });
        }
    }

    // Font variations feature commented out for now
    /*
    fn render_font_variations(&mut self, ui: &mut egui::Ui) {
        let fonts = self.font_variations(self.selected_char, ui.ctx());

        if fonts.is_empty() {
            ui.label("No font variations available");
        } else {
            egui::ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    // Create a grid for font variations
                    let columns = 2;
                    let button_size =
                        ui.available_width() / columns as f32 - ui.spacing().item_spacing.x;

                    egui::Grid::new("font_variations_grid")
                        .num_columns(columns)
                        .spacing([ui.spacing().item_spacing.x, ui.spacing().item_spacing.y])
                        .show(ui, |ui| {
                            let font_count = fonts.len();
                            for (i, (font_name, font_family)) in fonts.into_iter().enumerate() {
                                let response = ui.allocate_response(
                                    egui::Vec2::splat(button_size),
                                    egui::Sense::click(),
                                );

                                let rect = response.rect;
                                let painter = ui.painter();

                                // Draw background matching main grid style
                                painter.rect_filled(rect, 2.0, egui::Color32::from_rgb(30, 30, 30));

                                // Draw character
                                painter.text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    self.selected_char,
                                    egui::FontId::new(24.0, font_family.clone()),
                                    ui.visuals().text_color(),
                                );

                                // Draw font name below
                                painter.text(
                                    rect.center() + egui::Vec2::new(0.0, button_size * 0.3),
                                    egui::Align2::CENTER_CENTER,
                                    font_name,
                                    egui::FontId::new(9.0, egui::FontFamily::Proportional),
                                    ui.visuals().weak_text_color(),
                                );

                                // Handle click
                                if response.clicked() {
                                    ui.ctx().copy_text(self.selected_char.to_string());
                                    self.toasts
                                        .info(format!("Copied {}", self.selected_char))
                                        .duration(Some(std::time::Duration::from_secs(2)));
                                }

                                // Show tooltip
                                response.on_hover_text(format!(
                                    "{}\n{}\nClick to copy",
                                    font_name,
                                    char_name(self.selected_char)
                                ));

                                // End row every 2 fonts
                                if (i + 1) % columns == 0 && i < font_count - 1 {
                                    ui.end_row();
                                }
                            }
                        });
                });
        }
    }
    */

    fn render_central_panel(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Always show the glyph grid
            self.render_glyph_grid(ui);
        });
    }

    fn render_glyph_grid(&mut self, ui: &mut egui::Ui) {
        let glyphs_to_show = self.glyphs_to_show();

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

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(spacing, spacing);

                for (chr, name) in glyphs_to_show {
                    let response = ui
                        .allocate_response(egui::vec2(base_size, base_size), egui::Sense::click());

                    // Handle double-click to copy
                    if response.double_clicked() {
                        ui.ctx().copy_text(chr.to_string());
                        self.toasts
                            .info(format!("Copied {chr}"))
                            .duration(Some(std::time::Duration::from_secs(2)));
                    } else if response.clicked() {
                        self.selected_char = chr;
                        self.add_to_recently_used(chr);
                    }

                    // Draw glyph
                    let rect = response.rect;
                    let is_in_collection = self.collection.contains(&chr);

                    // Simple background without hover (original fast rendering)
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

    fn glyphs_to_show(&self) -> Vec<(char, String)> {
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

    fn show_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings_dialog {
            return;
        }

        let mut open = self.show_settings_dialog;

        egui::Window::new("Settings")
            .open(&mut open)
            .resizable(true)
            .default_size([600.0, 400.0])
            .show(ctx, |ui| {
                // Tab selector
                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.settings_tab,
                        SettingsTab::Categories,
                        "Categories",
                    );
                    ui.selectable_value(&mut self.settings_tab, SettingsTab::Fonts, "Fonts");
                });

                ui.separator();

                match self.settings_tab {
                    SettingsTab::Categories => self.show_categories_settings(ui),
                    SettingsTab::Fonts => self.show_fonts_settings(ui),
                }
            });

        self.show_settings_dialog = open;
    }

    fn show_categories_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Select which Unicode categories to display:");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                for category in &mut self.categories {
                    let mut visible = category.visible;

                    ui.horizontal(|ui| {
                        // Show checkbox for visibility
                        if ui.checkbox(&mut visible, &category.name).changed() {
                            category.visible = visible;
                        }

                        // Show font requirement if any
                        if let Some(ref font) = category.required_font {
                            ui.label(format!("(requires {})", font));
                        }
                    });
                }
            });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("Show All").clicked() {
                for category in &mut self.categories {
                    category.visible = true;
                }
            }
            if ui.button("Hide All").clicked() {
                for category in &mut self.categories {
                    category.visible = false;
                }
            }
        });
    }

    fn show_fonts_settings(&mut self, ui: &mut egui::Ui) {
        if let Some(ref font_manager) = self.font_manager {
            ui.label("Fonts are downloaded automatically when needed.");
            ui.separator();

            // Show cache info
            if let Ok(cache_size) = font_manager.cache_size() {
                let size_mb = cache_size as f64 / 1_048_576.0;
                ui.label(format!("Cache size: {:.2} MB", size_mb));
            }

            if let Ok(cached_fonts) = font_manager.list_cached_fonts()
                && !cached_fonts.is_empty()
            {
                ui.separator();
                ui.label(format!("Cached fonts: {}", cached_fonts.len()));

                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .show(ui, |ui| {
                        for font_name in cached_fonts {
                            ui.horizontal(|ui| {
                                ui.label(&font_name);
                                if ui
                                    .small_button("×")
                                    .on_hover_text("Delete cached font")
                                    .clicked()
                                {
                                    font_manager.clear_font_cache(&font_name).ok();
                                }
                            });
                        }
                    });
            }

            ui.separator();

            if ui.button("Clear All Cache").clicked() {
                font_manager.clear_all_cache().ok();
            }

            ui.label("Fonts are cached in your system's cache directory and will be reused across sessions.");
        } else {
            ui.label("Font manager not available");
        }
    }

    /// Download fonts for initially visible categories
    fn initialize_required_fonts(&self, ctx: &egui::Context) {
        if let Some(ref font_manager) = self.font_manager {
            // Check all visible categories for required fonts
            for category in &self.categories {
                if category.visible
                    && let Some(ref font_name) = category.required_font
                    && !font_manager.is_cached(font_name)
                {
                    // Download the font in background
                    if let Some((_, font_url)) = NotoFontMapping::font_for_script(&category.name) {
                        let font_name_clone = font_name.clone();
                        let font_url_clone = font_url.to_string();
                        let font_manager_clone = font_manager.clone();
                        let ctx_clone = ctx.clone();

                        std::thread::spawn(move || {
                            eprintln!(
                                "Downloading font for initially visible category: {}",
                                font_name_clone
                            );
                            match font_manager_clone.load_font(&font_name_clone, &font_url_clone) {
                                Ok(font_data) => {
                                    // Read existing font definitions and add the new font
                                    let mut font_definitions =
                                        ctx_clone.fonts(|f| f.definitions().clone());

                                    font_definitions.font_data.insert(
                                        font_name_clone.clone(),
                                        egui::FontData::from_owned(font_data).into(),
                                    );

                                    // Add to proportional family
                                    font_definitions
                                        .families
                                        .entry(egui::FontFamily::Proportional)
                                        .or_default()
                                        .push(font_name_clone.clone());

                                    ctx_clone.set_fonts(font_definitions);
                                    ctx_clone.request_repaint();

                                    eprintln!("Successfully loaded font: {}", font_name_clone);
                                }
                                Err(e) => {
                                    eprintln!("Failed to download font {}: {}", font_name_clone, e);
                                }
                            }
                        });
                    }
                }
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
                .map(|s| to_lowercase_string(s))
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

        let visuals = &ui.ctx().global_style().visuals;
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

        // Calculate baseline position
        let baseline_y = top + glyph_scale;

        // Draw the character - position it so most characters sit on baseline
        painter.text(
            egui::Pos2::new(center.x, baseline_y - glyph_scale * 0.39),
            egui::Align2::CENTER_CENTER,
            self.selected_char,
            egui::FontId::new(glyph_scale, font_family),
            glyph_color,
        );

        // Get character width (approximate for now)
        let char_width = glyph_scale * 0.6; // Approximate width
        let char_left = center.x - char_width / 2.0;
        let char_right = center.x + char_width / 2.0;

        // Draw ascender line
        painter.line_segment(
            [
                egui::Pos2::new(left, baseline_y - v_metrics.ascent),
                egui::Pos2::new(right, baseline_y - v_metrics.ascent),
            ],
            stroke,
        );

        // Draw baseline
        painter.line_segment(
            [
                egui::Pos2::new(left, baseline_y),
                egui::Pos2::new(right, baseline_y),
            ],
            stroke,
        );

        // Draw descender line
        painter.line_segment(
            [
                egui::Pos2::new(left, baseline_y - v_metrics.descent),
                egui::Pos2::new(right, baseline_y - v_metrics.descent),
            ],
            stroke,
        );

        // Draw vertical lines for character width
        painter.line_segment(
            [
                egui::Pos2::new(char_left, baseline_y - v_metrics.ascent),
                egui::Pos2::new(char_left, baseline_y - v_metrics.descent),
            ],
            stroke,
        );

        painter.line_segment(
            [
                egui::Pos2::new(char_right, baseline_y - v_metrics.ascent),
                egui::Pos2::new(char_right, baseline_y - v_metrics.descent),
            ],
            stroke,
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
