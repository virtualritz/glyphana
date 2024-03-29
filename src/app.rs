use ahash::AHashSet as HashSet;
use enum_dispatch::enum_dispatch;
use finl_unicode::categories::CharacterCategories;
//use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use unicode_blocks as ub;

use crate::*;

const CAT_START: usize = 3;

#[enum_dispatch]
trait CharacterInspector {
    fn characters(&self) -> Vec<char>;
    fn contains(&self, c: char) -> bool;
}

impl CharacterInspector for ub::UnicodeBlock {
    fn characters(&self) -> Vec<char> {
        (self.start()..self.end() + 1)
            .filter_map(char::from_u32)
            .collect()
    }

    fn contains(&self, c: char) -> bool {
        self.contains(c)
    }
}

#[derive(Clone, Debug, Default)]
struct UnicodeMultiBlock(Vec<ub::UnicodeBlock>);

impl CharacterInspector for UnicodeMultiBlock {
    fn characters(&self) -> Vec<char> {
        self.0
            .iter()
            .flat_map(|block| (block.start()..block.end() + 1).filter_map(char::from_u32))
            .collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0.iter().any(|block| block.contains(c))
    }
}

#[derive(Clone, Debug, Default)]
struct UnicodeCollection(HashSet<char>);

impl CharacterInspector for UnicodeCollection {
    fn characters(&self) -> Vec<char> {
        self.0.iter().copied().collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0.get(&c).is_some()
    }
}

#[derive(Clone, Debug, Default)]
struct NamedUnicodeCollections(Vec<(String, HashSet<char>)>);

impl CharacterInspector for NamedUnicodeCollections {
    fn characters(&self) -> Vec<char> {
        self.0
            .iter()
            .flat_map(|collection: &(String, HashSet<char>)| {
                collection.1.iter().copied().collect::<Vec<_>>()
            })
            .collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0
            .iter()
            .any(|collection: &(String, HashSet<char>)| collection.1.get(&c).is_some())
    }
}

#[derive(Clone, Debug)]
#[enum_dispatch(CharacterInspector)]
enum UnicodeCategory {
    Block(ub::UnicodeBlock),
    MultiBlock(UnicodeMultiBlock),
    Collection(UnicodeCollection),
    NamedCollections(NamedUnicodeCollections),
}

#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
enum GlyphScale {
    Small,
    Medium,
    Large,
}

impl From<GlyphScale> for f32 {
    fn from(g: GlyphScale) -> f32 {
        match g {
            GlyphScale::Small => 18.0,
            GlyphScale::Medium => 24.0,
            GlyphScale::Large => 36.0,
        }
    }
}

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
// if we add new fields, give them default values when deserializing old state
#[serde(default)]
pub struct GlyphanaApp {
    // The category the user selected for inspection.
    //selected_category: usize;
    // The character the user selected for inspection.
    selected_char: char,
    // The string the user entered into the search field.
    // Whether to onlky search in the subsets selected on the left panel.
    search_only_categories: bool,
    // Also search the glyph's name.
    search_name: bool,
    // If search is case sensitive.
    case_sensitive: bool,
    recently_used: VecDeque<char>,
    recently_used_max_len: usize,
    collection: HashSet<char>,
    selected_category: usize,
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
    #[serde(skip)]
    categories: Vec<(String, UnicodeCategory)>,
    #[serde(skip)]
    full_glyph_cache: BTreeMap<char, String>,
    #[serde(skip)]
    shown_glyph_cache: BTreeMap<char, String>,
    pixels_per_point: f32,
    glyph_scale: GlyphScale,
    stay_on_top: bool,
    #[serde(skip)]
    show_prefs: bool,
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
            selected_category: Default::default(),
            categories: {
                let custom = vec![
                    (
                        "Emoji".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::EMOTICONS,
                            ub::TRANSPORT_AND_MAP_SYMBOLS,
                            ub::ALCHEMICAL_SYMBOLS,
                            ub::SYMBOLS_AND_PICTOGRAPHS_EXTENDED_A,
                            ub::SYMBOLS_FOR_LEGACY_COMPUTING,
                        ])),
                    ),
                    (
                        ub::ARROWS.name().to_string(),
                        UnicodeCategory::NamedCollections(NamedUnicodeCollections({
                            let arrows = UnicodeMultiBlock(vec![
                                ub::ARROWS,
                                ub::SUPPLEMENTAL_ARROWS_A,
                                ub::SUPPLEMENTAL_ARROWS_B,
                                ub::SUPPLEMENTAL_ARROWS_C,
                            ])
                            .characters();

                            vec![
                                (
                                    "Left Arrows".to_string(),
                                    HashSet::<char>::from_iter(arrows.iter().filter_map(|&c| {
                                        if char_name(c).contains("left") {
                                            Some(c)
                                        } else {
                                            None
                                        }
                                    })),
                                ),
                                (
                                    "Right Arrows".to_string(),
                                    HashSet::<char>::from_iter(arrows.iter().filter_map(|&c| {
                                        if char_name(c).contains("right") {
                                            Some(c)
                                        } else {
                                            None
                                        }
                                    })),
                                ),
                                (
                                    "Up Arrows".to_string(),
                                    HashSet::<char>::from_iter(arrows.iter().filter_map(|&c| {
                                        if char_name(c).contains("up") {
                                            Some(c)
                                        } else {
                                            None
                                        }
                                    })),
                                ),
                                (
                                    "Down Arrows".to_string(),
                                    HashSet::<char>::from_iter(arrows.iter().filter_map(|&c| {
                                        if char_name(c).contains("down") {
                                            Some(c)
                                        } else {
                                            None
                                        }
                                    })),
                                ),
                                (
                                    "Other Arrows".to_string(),
                                    HashSet::<char>::from_iter(arrows.iter().filter_map(|&c| {
                                        let n = char_name(c);
                                        if n.contains("circle")
                                            || n.contains("west")
                                            || n.contains("east")
                                            || n.contains("up")
                                            || n.contains("down")
                                            || n.contains("shaft")
                                        {
                                            Some(c)
                                        } else {
                                            None
                                        }
                                    })),
                                ),
                            ]
                        })),
                    ),
                    (
                        ub::CURRENCY_SYMBOLS.name().to_string(),
                        UnicodeCategory::Block(ub::CURRENCY_SYMBOLS),
                    ),
                    (
                        "Latin".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::BASIC_LATIN,
                            ub::LATIN_1_SUPPLEMENT,
                            ub::LATIN_EXTENDED_ADDITIONAL,
                            ub::LATIN_EXTENDED_A,
                            ub::LATIN_EXTENDED_B,
                            ub::LATIN_EXTENDED_C,
                            ub::LATIN_EXTENDED_D,
                            ub::LATIN_EXTENDED_E,
                            ub::LATIN_EXTENDED_F,
                            ub::LATIN_EXTENDED_G,
                        ])),
                    ),
                    (
                        ub::LETTERLIKE_SYMBOLS.name().to_string(),
                        UnicodeCategory::Block(ub::LETTERLIKE_SYMBOLS),
                    ),
                    (
                        "Math Symbols".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::MATHEMATICAL_OPERATORS,
                            ub::SUPPLEMENTAL_MATHEMATICAL_OPERATORS,
                            ub::MATHEMATICAL_ALPHANUMERIC_SYMBOLS,
                            ub::MISCELLANEOUS_MATHEMATICAL_SYMBOLS_A,
                            ub::MISCELLANEOUS_MATHEMATICAL_SYMBOLS_B,
                        ])),
                    ),
                    (
                        "Parentheses".to_string(),
                        UnicodeCategory::Collection(UnicodeCollection(
                            vec![
                                '\u{0028}', '\u{0029}', '\u{005B}', '\u{005D}', '\u{007B}',
                                '\u{007D}', '\u{0F3A}', '\u{0F3B}', '\u{0F3C}', '\u{0F3D}',
                                '\u{169B}', '\u{169C}', '\u{2045}', '\u{2046}', '\u{207D}',
                                '\u{207E}', '\u{208D}', '\u{208E}', '\u{2308}', '\u{2309}',
                                '\u{230A}', '\u{230B}', '\u{2329}', '\u{232A}', '\u{2768}',
                                '\u{2769}', '\u{276A}', '\u{276B}', '\u{276C}', '\u{276D}',
                                '\u{276E}', '\u{276F}', '\u{2770}', '\u{2771}', '\u{2772}',
                                '\u{2773}', '\u{2774}', '\u{2775}', '\u{27C5}', '\u{27C6}',
                                '\u{27E6}', '\u{27E7}', '\u{27E8}', '\u{27E9}', '\u{27EA}',
                                '\u{27EB}', '\u{27EC}', '\u{27ED}', '\u{27EE}', '\u{27EF}',
                                '\u{2983}', '\u{2984}', '\u{2985}', '\u{2986}', '\u{2987}',
                                '\u{2988}', '\u{2989}', '\u{298A}', '\u{298B}', '\u{298C}',
                                '\u{298D}', '\u{298E}', '\u{298F}', '\u{2990}', '\u{2991}',
                                '\u{2992}', '\u{2993}', '\u{2994}', '\u{2995}', '\u{2996}',
                                '\u{2997}', '\u{2998}', '\u{29D8}', '\u{29D9}', '\u{29DA}',
                                '\u{29DB}', '\u{29FC}', '\u{29FD}', '\u{2E22}', '\u{2E23}',
                                '\u{2E24}', '\u{2E25}', '\u{2E26}', '\u{2E27}', '\u{2E28}',
                                '\u{2E29}', '\u{2E55}', '\u{2E56}', '\u{2E57}', '\u{2E58}',
                                '\u{2E59}', '\u{2E5A}', '\u{2E5B}', '\u{2E5C}', '\u{3008}',
                                '\u{3009}', '\u{300A}', '\u{300B}', '\u{300C}', '\u{300D}',
                                '\u{300E}', '\u{300F}', '\u{3010}', '\u{3011}', '\u{3014}',
                                '\u{3015}', '\u{3016}', '\u{3017}', '\u{3018}', '\u{3019}',
                                '\u{301A}', '\u{301B}', '\u{FE59}', '\u{FE5A}', '\u{FE5B}',
                                '\u{FE5C}', '\u{FE5D}', '\u{FE5E}', '\u{FF08}', '\u{FF09}',
                                '\u{FF3B}', '\u{FF3D}', '\u{FF5B}', '\u{FF5D}', '\u{FF5F}',
                                '\u{FF60}', '\u{FF62}', '\u{FF63}',
                            ]
                            .iter()
                            .copied()
                            .collect::<HashSet<_>>(),
                        )),
                    ),
                    (
                        "Pictographs".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::ALCHEMICAL_SYMBOLS,
                            ub::ANCIENT_SYMBOLS,
                            ub::MISCELLANEOUS_SYMBOLS,
                            ub::MISCELLANEOUS_SYMBOLS_AND_ARROWS,
                            ub::MISCELLANEOUS_SYMBOLS_AND_PICTOGRAPHS,
                            ub::SUPPLEMENTAL_SYMBOLS_AND_PICTOGRAPHS,
                            ub::SYMBOLS_AND_PICTOGRAPHS_EXTENDED_A,
                            ub::SYMBOLS_FOR_LEGACY_COMPUTING,
                            ub::TRANSPORT_AND_MAP_SYMBOLS,
                        ])),
                    ),
                    (
                        "Punctuation".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::GENERAL_PUNCTUATION,
                            ub::SUPPLEMENTAL_PUNCTUATION,
                        ])),
                    ),
                    (
                        ub::DINGBATS.name().to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::DINGBATS,
                            ub::ORNAMENTAL_DINGBATS,
                        ])),
                    ),
                    (
                        ub::GEOMETRIC_SHAPES.name().to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::GEOMETRIC_SHAPES,
                            ub::GEOMETRIC_SHAPES_EXTENDED,
                        ])),
                    ),
                    (
                        ub::MUSICAL_SYMBOLS.name().to_string(),
                        UnicodeCategory::Block(ub::MUSICAL_SYMBOLS),
                    ),
                    (
                        "Tabletop Symbols".to_string(),
                        UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                            ub::MAHJONG_TILES,
                            ub::DOMINO_TILES,
                            ub::PLAYING_CARDS,
                            ub::CHESS_SYMBOLS,
                        ])),
                    ),
                ];
                /*
                custom.append(
                    &mut ub::all::ALL
                        .iter()
                        .map(|&block| (block.name().to_string(), UnicodeCategory::Block(*block)))
                        .collect::<Vec<_>>(),
                );*/
                custom
            },
            full_glyph_cache: Default::default(),
            shown_glyph_cache: Default::default(),

            pixels_per_point: Default::default(),
            glyph_scale: GlyphScale::Medium,
            stay_on_top: false,
            show_prefs: false,
        }
    }
}

impl GlyphanaApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Add the Noto fonts -- what we use to cover as much unicode as possible for now.
        cc.egui_ctx.set_fonts(Self::fonts());

        // Load previous app state (if any).
        let mut glyphana = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        glyphana.pixels_per_point = cc.egui_ctx.pixels_per_point();

        glyphana.default_font_id = egui::FontId::new(
            glyphana.glyph_scale.into(),
            egui::FontFamily::Name(NOTO_SANS.into()),
        );

        glyphana
    }

    fn _available_characters(
        &self,
        ui: &egui::Ui,
        family: egui::FontFamily,
    ) -> BTreeMap<char, String> {
        ui.fonts(|f| {
            f.lock()
                .fonts
                .font(&egui::FontId::new(10.0, family)) // size is arbitrary for getting the characters
                .characters()
                .iter()
                .filter(|chr| !chr.is_whitespace() && !chr.is_ascii_control())
                .map(|&chr| {
                    //println!("{}", chr);
                    (chr, char_name(chr))
                })
                .collect()
        })
    }

    fn fonts() -> egui::FontDefinitions {
        let mut fonts = egui::FontDefinitions::default();

        //let mut font_data: BTreeMap<String, egui::FontData> = BTreeMap::new();
        //let mut families = BTreeMap::new();

        fonts.font_data.insert(
            NOTO_SANS.to_owned(),
            egui::FontData::from_static(&NOTO_SANS_FONT),
        );
        fonts.font_data.insert(
            NOTO_SANS_MATH.to_owned(),
            egui::FontData::from_static(&NOTO_SANS_MATH_FONT).tweak(egui::FontTweak {
                //y_offset_factor: 0.14, // move it up
                ..Default::default()
            }),
        );
        fonts.font_data.insert(
            NOTO_EMOJI.to_owned(),
            egui::FontData::from_static(&NOTO_EMOJI_FONT).tweak(egui::FontTweak {
                scale: 0.73, // make it smaller
                //y_offset_factor: 0.15, // move it down
                ..Default::default()
            }),
        );
        fonts.font_data.insert(
            EMOJI_ICON.to_owned(),
            egui::FontData::from_static(&EMOJI_ICON_FONT).tweak(egui::FontTweak {
                scale: 0.75, // make it smaller
                //y_offset_factor: 0.45, // move it down
                ..Default::default()
            }),
        );
        fonts.font_data.insert(
            NOTO_SYMBOLS.to_owned(),
            egui::FontData::from_static(&NOTO_SYMBOLS_FONT).tweak(egui::FontTweak {
                scale: 0.75, // make it smaller
                //y_offset_factor: 0.2, // move it up
                ..Default::default()
            }),
        );
        fonts.font_data.insert(
            NOTO_SYMBOLS2.to_owned(),
            egui::FontData::from_static(&NOTO_SYMBOLS2_FONT).tweak(egui::FontTweak {
                scale: 0.8, // make it smaller
                //y_offset_factor: -0.243, // move it up
                ..Default::default()
            }),
        );
        /*fonts.font_data.insert(
            NOTO_SIGN_WRITING.to_owned(),
            egui::FontData::from_static(&NOTO_SIGN_WRITING_FONT).tweak(egui::FontTweak {
                scale: 1.0,             // make it smaller
                y_offset_factor: 0.5, // move it up
                ..Default::default()
            }),
        );*/
        fonts.font_data.insert(
            NOTO_MUSIC.to_owned(),
            egui::FontData::from_static(&NOTO_MUSIC_FONT).tweak(egui::FontTweak {
                //scale: 0.7, // make it smaller
                ..Default::default()
            }),
        );

        fonts.families.insert(
            egui::FontFamily::Name(NOTO_SANS.into()),
            vec![
                NOTO_SANS.to_owned(),
                NOTO_SANS_MATH.to_owned(),
                NOTO_EMOJI.to_owned(),
                EMOJI_ICON.to_owned(),
                NOTO_SYMBOLS.to_owned(),
                NOTO_SYMBOLS2.to_owned(),
                //NOTO_SIGN_WRITING.to_owned(),
                NOTO_MUSIC.to_owned(),
            ],
        );

        fonts
    }

    fn settings_ui(&mut self, _ui: &mut egui::Ui) {}
}

impl eframe::App for GlyphanaApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        //println!("{:?}", self.categories[1].1);

        // Fill character caches on first run.
        if self.full_glyph_cache.is_empty() {
            self.full_glyph_cache = available_characters(ctx, self.default_font_id.family.clone());
            self.update_search_text_and_shown_glyph_cache();
        }

        let mut show_prefs = self.show_prefs;

        egui::Window::new("⚙ Preferences")
            .open(&mut show_prefs)
            .vscroll(true)
            .show(ctx, |ui| {
                self.settings_ui(ui);
            });

        self.show_prefs = show_prefs;

        //

        /*if let Ok(event) = tray_icon::TrayEvent::receiver().try_recv() {
            info!("tray event: {event:?}");
        }

        use tray_icon::menu::MenuEvent;
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id {
                2 => { info!("Baaaaar"); frame.set_always_on_top(true) },
                4 => frame.close(),
                _ => unreachable!(),
            };
        }*/

        // Update global app state.
        frame.set_always_on_top(self.stay_on_top);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Hamburger menu.
            egui::menu::bar(ui, |ui| {
                ui.menu_button(format!("{}", super::HAMBURGER), |ui| {
                    #[cfg(debug_assertions)]
                    if ui.button("⟲ Factory Reset App State").clicked() {
                        *self = Self::default();
                    }

                    #[cfg(debug_assertions)]
                    ui.separator();

                    /*if ui.button("Customize List…").clicked() {
                        todo!()
                    }

                    ui.separator();*/

                    ui.add_enabled_ui(false, |ui| ui.button("🗛 Glyph Size"));

                    ui.vertical(|ui| {
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Small, "Small");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Medium, "Medium");
                        ui.radio_value(&mut self.glyph_scale, GlyphScale::Large, "Large");
                    });

                    self.default_font_id.size = self.glyph_scale.into();

                    ui.separator();

                    if ui.button("🗑 Clear Recently Used").clicked() {
                        self.recently_used.clear();
                    }

                    ui.separator();

                    ui.add_enabled_ui(false, |ui| ui.button("Export Collection…"));

                    if ui.button("⚙ Preferences…").clicked() {
                        self.show_prefs = true;
                    }

                    ui.separator();

                    if ui.button("🗙 Quit").clicked() {
                        frame.close();
                    }
                });

                ui.toggle_value(&mut self.stay_on_top, format!("{}", super::PUSH_PIN))
                    .on_hover_ui(|ui| {
                        ui.label("Pin Glyphana Window to Top");
                    });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{}", super::CANCELLATION)).clicked() {
                        self.ui_search_text.clear();
                        self.update_search_text_and_shown_glyph_cache();
                    }

                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.ui_search_text)
                                //.desired_width(120.0)
                                .hint_text("🔍 Search"),
                        )
                        .changed()
                    {
                        self.selected_category = 2;
                        self.update_search_text_and_shown_glyph_cache();
                    }
                    //self.search_text = decancer::cure(&self.ui_search_text).into_str();

                    if !self.case_sensitive {
                        self.search_text = self.search_text.to_lowercase();
                    }

                    ui.toggle_value(
                        &mut self.case_sensitive,
                        format!("{}", super::LOWER_UPPER_CASE),
                    )
                    .on_hover_ui(|ui| {
                        self.selected_category = 2;
                        ui.label("Match Case");
                    });

                    if ui
                        .add_enabled_ui(!self.case_sensitive, |ui| {
                            ui.toggle_value(&mut self.search_name, format!("{}", super::NAME_BADGE))
                                .on_hover_ui(|ui| {
                                    ui.label("Include Glyph Name in Search");
                                })
                        })
                        .response
                        .changed()
                    {
                        self.selected_category = 2;
                        self.update_search_text_and_shown_glyph_cache();
                    }
                });
            });
        });

        egui::SidePanel::left("categories").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    /*egui::Grid::new("custom_categories")
                    .num_columns(1)
                    //.spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {*/
                    if ui
                        .selectable_value(&mut self.selected_category, 0, "Recently Used")
                        .changed()
                    {
                        self.update_search_text_and_shown_glyph_cache();
                    }
                    // ui.end_row();

                    if ui
                        .selectable_value(&mut self.selected_category, 1, "Collection")
                        .changed()
                    {
                        self.update_search_text_and_shown_glyph_cache();
                    }
                    // ui.end_row();

                    if ui
                        .add_enabled(!self.ui_search_text.is_empty(), |ui: &mut egui::Ui| {
                            ui.selectable_value(&mut self.selected_category, 2, "Search")
                        })
                        .changed()
                    {
                        self.update_search_text_and_shown_glyph_cache();
                    }
                    //   ui.end_row();
                    // });

                    ui.separator();

                    /*egui::Grid::new("categories")
                    .num_columns(1)
                    .striped(true)
                    .show(ui, |ui| {*/
                    let categories = self.categories.clone();
                    for (i, category) in &mut categories.iter().enumerate() {
                        if ui
                            .selectable_value(
                                &mut self.selected_category,
                                i + CAT_START,
                                &category.0,
                            )
                            .changed()
                        {
                            self.update_search_text_and_shown_glyph_cache();
                        }

                        //ui.end_row();
                    }
                });
            });
        });

        /*
        match self.selected_category {
            2 => self.update_search_text_and_shown_glyph_cache(),
            _ => {
                self.search_text.clear();
                self.shown_glyph_cache = self.full_glyph_cache.clone();
            } //self.update_search_text_and_shown_glyph_cache();
        }*/

        /*
        if 2 != self.selected_category {
            self.search_text.clear();
            self.shown_glyph_cache = self.full_glyph_cache.clone();
        }*/

        egui::SidePanel::right("character_preview").show(ctx, |ui| {
            //egui::ScrollArea::vertical().show(ui, |ui| {
            //let (id, rect) = ui.allocate_space(egui::vec2(50.0, 50.0));

            //let painter = ui.painter();

            /*let painter = Painter::new(
                ctx.clone(),
                egui::LayerId::new(egui::Order::Foreground, id),
                rect,
            );*/

            // TODO: make painter fill the entire panel width (scale the glyph accordingly).
            let rect = ui.available_rect_before_wrap();

            let scale = rect.max.x - rect.min.x;

            let (response, painter) =
                ui.allocate_painter(egui::Vec2::new(scale, 1.2 * scale), egui::Sense::click());

            //painter.on_hover_ui(ui.label("Click to Copy 📋"));

            //let painter =
            //Painter::new(ctx.clone(), ui.layer_id(), ui.available_rect_before_wrap());

            self.paint_glyph(scale, ui, response, painter);

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    egui::Grid::new("glyph_name")
                        .num_columns(1)
                        //.min_col_width(scale)
                        .min_row_height(60.0)
                        //.spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.end_row();
                            ui.centered_and_justified(|ui| {
                                let name = textwrap::wrap(
                                    &title_case(
                                        &unicode_names2::name(self.selected_char)
                                            .map(|name| name.to_string().to_lowercase())
                                            .unwrap_or_else(String::new),
                                    ),
                                    18,
                                )
                                .join("\n");

                                ui.label(name);
                            });

                            ui.end_row();
                        });

                    egui::Grid::new("flip_case")
                        .num_columns(1)
                        //.min_col_width(scale)
                        //.spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.centered_and_justified(|ui| {
                                let is_cased = self.selected_char.is_letter_cased();

                                let (can_be_flipped, flipped_case) =
                                    if self.selected_char.is_letter_uppercase() {
                                        let flipped_case =
                                            unicode_case_mapping::to_lowercase(self.selected_char);
                                        (
                                            0 != flipped_case[0] && 0 == flipped_case[1],
                                            flipped_case[0],
                                        )
                                    } else {
                                        let flipped_case =
                                            unicode_case_mapping::to_uppercase(self.selected_char);
                                        (
                                            0 != flipped_case[0] && 0 == flipped_case[1],
                                            flipped_case[0],
                                        )
                                    };

                                if ui
                                    .add_enabled(is_cased && can_be_flipped, |ui: &mut egui::Ui| {
                                        ui.button("Flip Case")
                                    })
                                    .clicked()
                                {
                                    self.selected_char = char::from_u32(flipped_case).unwrap()
                                };
                            });
                            ui.end_row();
                        });

                    egui::Grid::new("glyph_codepoints")
                        .num_columns(2)
                        .min_col_width(scale / 2.1)
                        //.spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.end_row();
                            // Unicode
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.label("Unicode");
                            });

                            let unicode_hex: [u8; 4] = bytemuck::cast(self.selected_char);

                            let mut unicode_hex_string_ui = String::from("U+");
                            let mut unicode_html_string = String::from("&#");

                            if 0 != unicode_hex[3] {
                                unicode_hex_string_ui += &format!("{:02X}\u{2009}", unicode_hex[3]);
                                unicode_html_string += &format!("{:02X}", unicode_hex[3]);
                            }

                            unicode_hex[..2].iter().rev().for_each(|&uc| {
                                if 0 != uc || !unicode_hex_string_ui.is_empty() {
                                    unicode_hex_string_ui += &format!("{uc:02X}\u{2009}");
                                    unicode_html_string += &format!("{uc:02X}");
                                }
                            });

                            if ui
                                .button(egui::RichText::new(unicode_hex_string_ui).monospace())
                                .on_hover_ui(|ui| {
                                    ui.label("Click to Copy Unicode as HTML");
                                })
                                .clicked()
                            {
                                ui.output_mut(|o| o.copied_text = unicode_html_string);
                            }

                            ui.end_row();

                            // Utf8
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                ui.label("UTF-8");
                            });

                            let mut utf_eight = [0u8; 4];
                            encode_unicode::Utf8Char::new(self.selected_char)
                                .to_slice(&mut utf_eight);

                            let mut utf_eight_string_ui = String::new();
                            let mut utf_eight_string = String::new();

                            if 0 != utf_eight[3] {
                                utf_eight_string_ui += &format!("{:02X}\u{2009}", utf_eight[3]);
                                utf_eight_string += &format!("{:02X}", utf_eight[3]);
                            }

                            utf_eight[..2].iter().rev().for_each(|&uc| {
                                if 0 != uc || !utf_eight_string.is_empty() {
                                    utf_eight_string_ui += &format!("{uc:02X}\u{2009}");
                                    utf_eight_string += &format!("{uc:02X}");
                                }
                            });

                            if ui
                                .button(egui::RichText::new(utf_eight_string_ui).monospace())
                                .on_hover_ui(|ui| {
                                    ui.label("Click to Copy UTF8 Code");
                                })
                                .clicked()
                            {
                                ui.output_mut(|o| o.copied_text = utf_eight_string);
                            }

                            ui.end_row();
                        });

                    egui::Grid::new("collect")
                        .num_columns(1)
                        //.min_col_width(scale)
                        //.spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.end_row();

                            ui.centered_and_justified(|ui| {
                                let is_in_collection =
                                    self.collection.contains(&self.selected_char);

                                if ui
                                    .add(egui::SelectableLabel::new(is_in_collection, "Collect"))
                                    .on_hover_ui(|ui: &mut egui::Ui| {
                                        ui.label("Add Glyp to Collection");
                                    })
                                    .clicked()
                                {
                                    if is_in_collection {
                                        self.collection.remove(&self.selected_char);
                                    } else {
                                        self.collection.insert(self.selected_char);
                                    }
                                }
                            });
                            ui.end_row();
                        });
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

                    //info!("ã == a is {}", focaccia::unicode_full_case_eq("a", "ã"));

                    let recently_used = self.recently_used.clone();
                    self.shown_glyph_cache.iter().for_each(|(&chr, name)| {
                        let button = egui::Button::new(
                            egui::RichText::new(chr.to_string()).font(self.default_font_id.clone()),
                        )
                        .frame(true)
                        .min_size(egui::Vec2::splat(self.default_font_id.size * 2.));

                        let tooltip_ui = |ui: &mut egui::Ui| {
                            ui.label(
                                egui::RichText::new(chr.to_string())
                                    .font(self.default_font_id.clone()),
                            );
                            ui.label(format!(
                                "{}\nU+{:X}\n\nDouble-click to copy 📋",
                                capitalize(name),
                                chr as u32
                            ));
                        };

                        let hover_button = ui
                            .add_sized(egui::Vec2::splat(self.default_font_id.size * 2.), button)
                            .on_hover_ui(tooltip_ui);

                        if hover_button.double_clicked() {
                            // Send to clipboard.
                            ui.output_mut(|o| o.copied_text = chr.to_string());

                            /*use enigo::KeyboardControllable;
                            let mut enigo = enigo::Enigo::new();
                            let alt_tab = "{+ALT}{TAB}{-ALT}".to_string();
                            enigo.key_sequence_parse(&(alt_tab.clone() + &chr.to_string() + &alt_tab))
                            */
                        } else if hover_button.clicked() {
                            self.selected_char = chr;
                            self.recently_used.push_back(chr);
                            if self.recently_used_max_len <= recently_used.len() {
                                self.recently_used.pop_front();
                            }
                        }
                    });
                });
            });
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}

// .auto_shrink([false;2])
impl GlyphanaApp {
    fn update_search_text_and_shown_glyph_cache(&mut self) {
        // Update character cache.
        if 2 == self.selected_category {
            //info!("Updating cache");
            self.search_text = self.ui_search_text.clone();

            self.split_search_text = self
                .search_text
                .split(' ')
                .filter_map(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                })
                .collect();

            self.split_search_text_lower = self
                .split_search_text
                .iter()
                .map(|s| s.to_lowercase())
                .collect();

            self.selected_category = 2;

            self.shown_glyph_cache = self
                .full_glyph_cache
                .clone()
                .into_iter()
                // Filter by search string.
                .filter(|(chr, name)| {
                    //let mut tmp = [0u8; 4];

                    //let cmp_chr unicode_case_mapping::case_folded(chr).unwrap_or_else(|| chr as _)
                    //let chr_str = chr.encode_utf8(&mut tmp);

                    //info!("{}", chr);
                    //let cured_chr = decancer::cure(chr_str).into_str();
                    let chr = match self.case_sensitive {
                        true => *chr,
                        false => {
                            let lower_case = unicode_case_mapping::to_lowercase(*chr);
                            match lower_case[0] {
                                0 => *chr,
                                _ => char::from_u32(lower_case[0]).unwrap(),
                            }
                        }
                    };

                    (self.search_name
                        && self
                            .split_search_text_lower
                            .iter()
                            .any(|text| name.contains(text)))
                        || (!self.search_name && self.search_text.contains(&chr.to_string()))
                        || self.split_search_text_lower.iter().any(|text| {
                            glyph_names::glyph_name(chr as _).is_some_and(|t| t.contains(text))
                        })
                        || self.search_text.chars().any(|c| {
                            //cured_chr.chars().next().unwrap() == c ||
                            unicode_skeleton::confusable([chr].into_iter(), [c].into_iter())
                        })
                })
                .collect()
        } else {
            //info!("Updating full cache for category {}!", self.selected_category);
            self.shown_glyph_cache = self
                .full_glyph_cache
                .clone()
                .into_iter()
                // Filter by category.
                .filter(|(chr, _)| {
                    match self.selected_category {
                        0 => self.recently_used.contains(chr),
                        1 => self.collection.contains(chr),
                        _ => self.categories[self.selected_category - CAT_START]
                            .1
                            .contains(*chr),
                    }

                    // If no category is selected display all glyphs in the font
                    /*||
                    self
                        .categories
                        .iter()
                        .fold(true, |none_selected, cat| none_selected && !cat.2)*/
                })
                .collect();
        }
    }

    fn paint_glyph(
        &mut self,
        scale: f32,
        ui: &mut egui::Ui,
        response: egui::Response,
        painter: egui::Painter,
    ) {
        /*let color = if ui.visuals().dark_mode {
            egui::Color32::from_additive_luminance(196)
        } else {
            egui::Color32::from_black_alpha(240)
        };*/

        //egui::RichText::new("&").size(20.0);

        //let (response, painter) =
        //    ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

        let rect = response.rect;

        let center = rect.center();

        let glyph_scale = scale * 0.8;

        // We add a 12% frame around the paint area.
        let offset = glyph_scale * 0.12;

        let left = rect.min.x + offset;
        let top = rect.min.y + offset;

        let right = rect.max.x - offset;
        let bottom = rect.max.y - offset;

        let font = rusttype::Font::try_from_bytes(&NOTO_SANS_FONT).unwrap();

        //read

        let v_metrics = font.v_metrics(rusttype::Scale::uniform(glyph_scale));

        let visuals = &ui.ctx().style().visuals;
        let dark_mode = visuals.dark_mode;

        let glyph_color = if dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };

        let mut stroke = visuals.widgets.noninteractive.fg_stroke;

        stroke.color = stroke
            .color
            .linear_multiply(stroke.color.r() as f32 / 255.0);

        painter.text(
            egui::Pos2::new(center.x, top),
            egui::Align2::CENTER_TOP,
            self.selected_char,
            egui::FontId::new(glyph_scale, egui::FontFamily::Name(NOTO_SANS.into())),
            glyph_color,
        );

        // Ascent
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale - v_metrics.ascent),
                egui::Pos2::new(right, top + glyph_scale - v_metrics.ascent),
            ],
            stroke,
        );

        // Baseline
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale),
                egui::Pos2::new(right, top + glyph_scale),
            ],
            stroke,
        );

        // Descent
        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale - v_metrics.descent),
                egui::Pos2::new(right, top + glyph_scale - v_metrics.descent),
            ],
            stroke,
        );

        ui.expand_to_include_rect(painter.clip_rect());
    }
}

fn available_characters(ctx: &egui::Context, family: egui::FontFamily) -> BTreeMap<char, String> {
    ctx.fonts(|f| {
        f.lock()
            .fonts
            .font(&egui::FontId::new(10.0, family)) // size is arbitrary for getting the characters
            .characters()
            .iter()
            .filter(|chr| !chr.is_whitespace() && !chr.is_ascii_control())
            .map(|&chr| (chr, char_name(chr)))
            .collect()
    })
}

fn char_name(chr: char) -> String {
    special_char_name(chr)
        .map(|s| s.to_owned())
        .or_else(|| unicode_names2::name(chr).map(|name| name.to_string().to_lowercase()))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn special_char_name(chr: char) -> Option<&'static str> {
    #[allow(clippy::match_same_arms)] // many "flag"
    match chr {
        // Special private-use-area extensions found in `emoji-icon-font.ttf`:
        // Private use area extensions:
        '\u{FE4E5}' => Some("flag japan"),
        '\u{FE4E6}' => Some("flag usa"),
        '\u{FE4E7}' => Some("flag"),
        '\u{FE4E8}' => Some("flag"),
        '\u{FE4E9}' => Some("flag"),
        '\u{FE4EA}' => Some("flag great britain"),
        '\u{FE4EB}' => Some("flag"),
        '\u{FE4EC}' => Some("flag"),
        '\u{FE4ED}' => Some("flag"),
        '\u{FE4EE}' => Some("flag south korea"),
        '\u{FE82C}' => Some("number sign in square"),
        '\u{FE82E}' => Some("digit one in square"),
        '\u{FE82F}' => Some("digit two in square"),
        '\u{FE830}' => Some("digit three in square"),
        '\u{FE831}' => Some("digit four in square"),
        '\u{FE832}' => Some("digit five in square"),
        '\u{FE833}' => Some("digit six in square"),
        '\u{FE834}' => Some("digit seven in square"),
        '\u{FE835}' => Some("digit eight in square"),
        '\u{FE836}' => Some("digit nine in square"),
        '\u{FE837}' => Some("digit zero in square"),

        // Special private-use-area extensions found in `emoji-icon-font.ttf`:
        // Web services/operating systems/browsers
        '\u{E600}' => Some("web-dribbble"),
        '\u{E601}' => Some("web-stackoverflow"),
        '\u{E602}' => Some("web-vimeo"),
        '\u{E603}' => Some("web-twitter"),
        '\u{E604}' => Some("web-facebook"),
        '\u{E605}' => Some("web-googleplus"),
        '\u{E606}' => Some("web-pinterest"),
        '\u{E607}' => Some("web-tumblr"),
        '\u{E608}' => Some("web-linkedin"),
        '\u{E60A}' => Some("web-stumbleupon"),
        '\u{E60B}' => Some("web-lastfm"),
        '\u{E60C}' => Some("web-rdio"),
        '\u{E60D}' => Some("web-spotify"),
        '\u{E60E}' => Some("web-qq"),
        '\u{E60F}' => Some("web-instagram"),
        '\u{E610}' => Some("web-dropbox"),
        '\u{E611}' => Some("web-evernote"),
        '\u{E612}' => Some("web-flattr"),
        '\u{E613}' => Some("web-skype"),
        '\u{E614}' => Some("web-renren"),
        '\u{E615}' => Some("web-sina-weibo"),
        '\u{E616}' => Some("web-paypal"),
        '\u{E617}' => Some("web-picasa"),
        '\u{E618}' => Some("os-android"),
        '\u{E619}' => Some("web-mixi"),
        '\u{E61A}' => Some("web-behance"),
        '\u{E61B}' => Some("web-circles"),
        '\u{E61C}' => Some("web-vk"),
        '\u{E61D}' => Some("web-smashing"),
        '\u{E61E}' => Some("web-forrst"),
        '\u{E61F}' => Some("os-windows"),
        '\u{E620}' => Some("web-flickr"),
        '\u{E621}' => Some("web-picassa"),
        '\u{E622}' => Some("web-deviantart"),
        '\u{E623}' => Some("web-steam"),
        '\u{E624}' => Some("web-github"),
        '\u{E625}' => Some("web-git"),
        '\u{E626}' => Some("web-blogger"),
        '\u{E627}' => Some("web-soundcloud"),
        '\u{E628}' => Some("web-reddit"),
        '\u{E629}' => Some("web-delicious"),
        '\u{E62A}' => Some("browser-chrome"),
        '\u{E62B}' => Some("browser-firefox"),
        '\u{E62C}' => Some("browser-ie"),
        '\u{E62D}' => Some("browser-opera"),
        '\u{E62E}' => Some("browser-safari"),
        '\u{E62F}' => Some("web-google-drive"),
        '\u{E630}' => Some("web-wordpress"),
        '\u{E631}' => Some("web-joomla"),
        '\u{E632}' => Some("lastfm"),
        '\u{E633}' => Some("web-foursquare"),
        '\u{E634}' => Some("web-yelp"),
        '\u{E635}' => Some("web-drupal"),
        '\u{E636}' => Some("youtube"),
        '\u{F189}' => Some("vk"),
        '\u{F1A6}' => Some("digg"),
        '\u{F1CA}' => Some("web-vine"),
        '\u{F8FF}' => Some("os-apple"),

        // Special private-use-area extensions found in `Ubuntu-Light.ttf`
        '\u{F000}' => Some("uniF000"),
        '\u{F001}' => Some("fi"),
        '\u{F002}' => Some("fl"),
        '\u{F506}' => Some("one seventh"),
        '\u{F507}' => Some("two sevenths"),
        '\u{F508}' => Some("three sevenths"),
        '\u{F509}' => Some("four sevenths"),
        '\u{F50A}' => Some("five sevenths"),
        '\u{F50B}' => Some("six sevenths"),
        '\u{F50C}' => Some("one ninth"),
        '\u{F50D}' => Some("two ninths"),
        '\u{F50E}' => Some("four ninths"),
        '\u{F50F}' => Some("five ninths"),
        '\u{F510}' => Some("seven ninths"),
        '\u{F511}' => Some("eight ninths"),
        '\u{F800}' => Some("zero.alt"),
        '\u{F801}' => Some("one.alt"),
        '\u{F802}' => Some("two.alt"),
        '\u{F803}' => Some("three.alt"),
        '\u{F804}' => Some("four.alt"),
        '\u{F805}' => Some("five.alt"),
        '\u{F806}' => Some("six.alt"),
        '\u{F807}' => Some("seven.alt"),
        '\u{F808}' => Some("eight.alt"),
        '\u{F809}' => Some("nine.alt"),
        '\u{F80A}' => Some("zero.sups"),
        '\u{F80B}' => Some("one.sups"),
        '\u{F80C}' => Some("two.sups"),
        '\u{F80D}' => Some("three.sups"),
        '\u{F80E}' => Some("four.sups"),
        '\u{F80F}' => Some("five.sups"),
        '\u{F810}' => Some("six.sups"),
        '\u{F811}' => Some("seven.sups"),
        '\u{F812}' => Some("eight.sups"),
        '\u{F813}' => Some("nine.sups"),
        '\u{F814}' => Some("zero.sinf"),
        '\u{F815}' => Some("one.sinf"),
        '\u{F816}' => Some("two.sinf"),
        '\u{F817}' => Some("three.sinf"),
        '\u{F818}' => Some("four.sinf"),
        '\u{F819}' => Some("five.sinf"),
        '\u{F81A}' => Some("six.sinf"),
        '\u{F81B}' => Some("seven.sinf"),
        '\u{F81C}' => Some("eight.sinf"),
        '\u{F81D}' => Some("nine.sinf"),
        _ => None,
    }
}

fn _other_char_name(chr: char) -> Option<&'static str> {
    #[allow(clippy::match_same_arms)] // many "flag"
    match chr {
        // Manually added
        '\u{00DF}' => Some("sz"),
        '\u{1E9E}' => Some("SZ"),
        '\u{00C4}' => Some("A umlaut"),
        '\u{00E4}' => Some("a umlaut"),
        '\u{00CB}' => Some("E umlaut"),
        '\u{00EB}' => Some("e umlaut"),
        '\u{00CF}' => Some("I umlaut"),
        '\u{00EF}' => Some("i umlaut"),
        '\u{00D6}' => Some("O umlaut"),
        '\u{00F6}' => Some("o umlaut"),
        '\u{00DC}' => Some("U umlaut"),
        '\u{00FC}' => Some("u umlaut"),
        '\u{0178}' => Some("Y umlaut"),
        '\u{00FF}' => Some("y umlaut"),
        _ => None,
    }
}

/// Capitalizes the first character.
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Capitalizes the first character of every word.
fn title_case(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
