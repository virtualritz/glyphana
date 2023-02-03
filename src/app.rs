use ahash::AHashMap as HashMap;
use egui::{Color32, Painter};
use enum_dispatch::enum_dispatch;
use log::{info, warn};
use std::collections::BTreeMap;
use unicode_blocks as ub;

use crate::*;

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
    search_text: String,
    // Whether to onlky search in the subsets selected on the left panel.
    search_only_categories: bool,
    // Search only the glyph's name.
    search_only_name: bool,
    // if search is case sensitive
    case_sensitive: bool,
    #[serde(skip)]
    default_font_id: egui::FontId,
    #[serde(skip)]
    categories: Vec<(String, UnicodeCategory, bool)>,
    #[serde(skip)]
    named_chars: BTreeMap<egui::FontFamily, BTreeMap<char, String>>,
}

#[enum_dispatch]
trait CharacterInspector {
    fn characters(&self) -> Vec<char>;
    fn contains(&self, c: char) -> bool;
}

impl CharacterInspector for ub::UnicodeBlock {
    fn characters(&self) -> Vec<char> {
        (self.start()..self.end() + 1)
            .into_iter()
            .filter_map(|c| char::from_u32(c))
            .collect()
    }

    fn contains(&self, c: char) -> bool {
        self.contains(c)
    }
}

struct UnicodeMultiBlock(Vec<ub::UnicodeBlock>);

impl CharacterInspector for UnicodeMultiBlock {
    fn characters(&self) -> Vec<char> {
        self.0
            .iter()
            .map(|block| {
                (block.start()..block.end() + 1)
                    .into_iter()
                    .filter_map(|c| char::from_u32(c))
            })
            .flatten()
            .collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0.iter().find(|block| block.contains(c)).is_some()
    }
}

struct UnicodeCollection(HashMap<char, String>);

impl CharacterInspector for UnicodeCollection {
    fn characters(&self) -> Vec<char> {
        self.0.iter().map(|(&c, _)| c).collect()
    }

    fn contains(&self, c: char) -> bool {
        self.0.get(&c).is_some()
    }
}

#[enum_dispatch(CharacterInspector)]
enum UnicodeCategory {
    Block(ub::UnicodeBlock),
    MultiBlock(UnicodeMultiBlock),
    Collection(UnicodeCollection),
}

impl Default for GlyphanaApp {
    fn default() -> Self {
        Self {
            selected_char: Default::default(),
            search_text: Default::default(),
            search_only_categories: false,
            case_sensitive: false,
            search_only_name: false,
            default_font_id: egui::FontId::new(18.0, egui::FontFamily::Name(NOTO_SANS.into())),
            categories: vec![
                (
                    "Favorites".to_string(),
                    UnicodeCategory::Collection(UnicodeCollection(HashMap::new())),
                    false,
                ),
                /*(
                    ub::ARROWS.name().to_string(),
                    UnicodeCategory::Block(ub::ARROWS),
                    false,
                ),*/
                //(ub::PARENTHESES.name().to_string(), vec![ub::PARENTHESES])
                (
                    "Punctuation".to_string(),
                    UnicodeCategory::MultiBlock(UnicodeMultiBlock(vec![
                        ub::GENERAL_PUNCTUATION,
                        ub::SUPPLEMENTAL_PUNCTUATION,
                    ])),
                    false,
                ),
                (
                    ub::CURRENCY_SYMBOLS.name().to_string(),
                    UnicodeCategory::Block(ub::CURRENCY_SYMBOLS),
                    false,
                ),
                //(ub::PICTOGRAPHS.name().to_string(), vec![ub::PICTOGRAPHS]),
                (
                    ub::LETTERLIKE_SYMBOLS.name().to_string(),
                    UnicodeCategory::Block(ub::LETTERLIKE_SYMBOLS),
                    false,
                ),
                /*(
                    ub::EMOTICONS.name().to_string(),
                    UnicodeCategory::Block(ub::EMOTICONS),
                    false,
                ),
                (
                    ub::DINGBATS.name().to_string(),
                    UnicodeCategory::Block(ub::DINGBATS),
                    false,
                ),
                (
                    ub::MATHEMATICAL_OPERATORS.name().to_string(),
                    UnicodeCategory::Block(ub::MATHEMATICAL_OPERATORS),
                    false,
                ),*/
            ],
            named_chars: Default::default(),
        }
    }
}

impl GlyphanaApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Start with the default fonts (we will be adding to them rather than replacing them).
        let mut fonts = egui::FontDefinitions::default();

        // Install my own font (maybe supporting non-latin characters).
        // .ttf and .otf files supported.
        fonts.font_data.insert(
            NOTO_SANS.to_owned(),
            egui::FontData::from_static(NOTO_SANS_FONT),
        );

        fonts.font_data.insert(
            NOTO_SANS_MATH.to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/NotoSansMath-Regular.ttf")),
        );

        // Put the font first (highest priority) for proportional text.
        fonts
            .families
            .entry(egui::FontFamily::Name(NOTO_SANS.into()))
            .or_default()
            .insert(0, NOTO_SANS.to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Name(NOTO_SANS_MATH.into()))
            .or_default()
            .insert(0, NOTO_SANS_MATH.to_owned());

        // Put my font as last fallback for monospace:
        /*fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("noto-sans".to_owned());*/

        // Tell egui to use these fonts
        cc.egui_ctx.set_fonts(fonts);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn available_characters(
        &self,
        ui: &egui::Ui,
        family: egui::FontFamily,
    ) -> BTreeMap<char, String> {
        info!("Running available_characters");
        ui.fonts(|f| {
            f.lock()
                .fonts
                .font(&egui::FontId::new(10.0, family)) // size is arbitrary for getting the characters
                .characters()
                .iter()
                .filter(|chr| !chr.is_whitespace() && !chr.is_ascii_control())
                .map(|&chr| {
                    println!("{}", chr);
                    (chr, char_name(chr))
                })
                .collect()
        })
    }
}

impl eframe::App for GlyphanaApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // let mut named_chars = &mut BTreeMap::<char, std::string::String>::new();

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button(format!("{}", super::HAMBURGER), |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(format!("{}", super::CANCELLATION)).clicked() {
                        self.search_text.clear();
                    }
                    ui.text_edit_singleline(&mut self.search_text)
                            //.desired_width(120.0))
                            ;
                    if !self.case_sensitive {
                        self.search_text = self.search_text.to_lowercase();
                    }
                    ui.toggle_value(
                        &mut self.search_only_name,
                        format!("{}", super::DOCUMENT_WITH_TEXT),
                    );
                    ui.toggle_value(&mut self.case_sensitive, format!("{}", "Aa"));
                });
            });
        });

        egui::SidePanel::left("categories").show(ctx, |ui| {
            ui.toggle_value(&mut self.categories[0].2, "Favorites");

            ui.separator();

            for category in &mut self.categories[1..] {
                ui.toggle_value(&mut category.2, &category.0);
            }
        });

        egui::SidePanel::right("character_preview").show(ctx, |ui| {
            //egui::ScrollArea::vertical().show(ui, |ui| {
            //let (id, rect) = ui.allocate_space(egui::vec2(50.0, 50.0));

            //let painter = ui.painter();

            /*let painter = Painter::new(
                ctx.clone(),
                egui::LayerId::new(egui::Order::Foreground, id),
                rect,
            );*/

            let scale = 256.0;

            let (response, painter) =
                ui.allocate_painter(egui::Vec2::new(scale, 1.5 * scale), egui::Sense::click());
            //let painter =
            //Painter::new(ctx.clone(), ui.layer_id(), ui.available_rect_before_wrap());

            self.paint_glyph(scale * 0.8, ui, response, painter);

            //})
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let named_chars = self
                .named_chars
                .entry(self.default_font_id.family.clone())
                .or_insert_with(|| available_characters(ui, self.default_font_id.family.clone()));

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

                    for (&chr, name) in named_chars {
                        if (self.search_text.is_empty()
                            || name.contains(&self.search_text)
                            || self.search_text.contains(chr)
                            || glyph_names::glyph_name(chr as _).contains(&self.search_text))
                            && (self
                                .categories
                                .iter()
                                .filter(|cat| cat.2)
                                .fold(false, |contained, cat| contained || cat.1.contains(chr))
                                // If no category is selected display all glyphs in the font
                                || self
                                    .categories
                                    .iter()
                                    .fold(true, |none_selected, cat| none_selected && !cat.2))
                        {
                            let button = egui::Button::new(
                                egui::RichText::new(chr.to_string())
                                    .font(self.default_font_id.clone()),
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
                                .add_sized(
                                    egui::Vec2::splat(self.default_font_id.size * 2.),
                                    button,
                                )
                                .on_hover_ui(tooltip_ui);

                            if hover_button.clicked() {
                                self.selected_char = chr;
                            }

                            if hover_button.double_clicked() {
                                ui.output_mut(|o| o.copied_text = chr.to_string());
                            }
                        }
                    }
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

impl GlyphanaApp {
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

        painter.text(
            egui::Pos2::new(center.x, scale + rect.min.y),
            egui::Align2::CENTER_BOTTOM,
            self.selected_char,
            egui::FontId::new(glyph_scale, egui::FontFamily::Name(NOTO_SANS.into())),
            egui::Color32::WHITE,
        );

        ui.expand_to_include_rect(painter.clip_rect());

        //painter.extend(vec![egui::Shape::text("g").scale(40.0)]);

        // here you can get the font_metrics

        let font = rusttype::Font::try_from_bytes(NOTO_SANS_FONT).unwrap();

        let v_metrics = font.v_metrics(rusttype::Scale::uniform(glyph_scale));

        //let width = metrics.left_side_bearing + metrics.right_side_bearing;
        //let height = metrics.ascender - metrics.descender;

        let left = rect.min.x;
        let top = rect.min.y;

        let right = rect.max.x;
        let bottom = rect.max.y;

        //info!("{}", v_metrics.descent);

        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale - v_metrics.ascent),
                egui::Pos2::new(right, top + glyph_scale - v_metrics.ascent),
            ],
            egui::Stroke::new(1., egui::Color32::WHITE),
        );

        painter.line_segment(
            [
                egui::Pos2::new(left, top + glyph_scale),
                egui::Pos2::new(right, top + glyph_scale),
            ],
            egui::Stroke::new(1., egui::Color32::BLUE),
        );

        painter.line_segment(
            [
                egui::Pos2::new(left, top + scale - v_metrics.descent),
                egui::Pos2::new(right, top + scale - v_metrics.descent),
            ],
            egui::Stroke::new(1., egui::Color32::RED),
        );

        /*
        painter.line_segment(
            [
                egui::Pos2::new(
                    left,
                    top + glyph_scale - (v_metrics.descent + v_metrics.line_gap),
                ),
                egui::Pos2::new(
                    right,
                    top + glyph_scale - (v_metrics.descent + v_metrics.line_gap),
                ),
            ],
            egui::Stroke::new(1., egui::Color32::GREEN),
        );*/

        /*
            egui::Line::new()
                .start(0.0, metrics.cap_height * scale)
                .end(width * glyph_scale, metrics.cap_height * scale)
                .build(ui);

            egui::Line::new()
                .start(0.0, metrics.baseline * scale)
                .end(width * glyph_scale, metrics.baseline * scale)
                .build(ui);
        });
        */
    }
}

fn available_characters(ui: &egui::Ui, family: egui::FontFamily) -> BTreeMap<char, String> {
    ui.fonts(|f| {
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

fn other_char_name(chr: char) -> Option<&'static str> {
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
