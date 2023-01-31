use ahash::AHashSet as HashSet;
use egui::{pos2, vec2, Color32, Frame, Painter, Pos2, Rect, Shape, Stroke, Ui, Vec2};
use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;
use unicode_blocks as ub;

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
// if we add new fields, give them default values when deserializing old state
#[serde(default)]
pub struct GlyphanaApp {
    // The character the user selected for inspection.
    selected_char: char,
    // The string the user entered into the search field.
    search_text: String,
    #[serde(skip)]
    font_id: egui::FontId,
    #[serde(skip)]
    categories: Vec<(String, UnicodeCategory)>,
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

struct UnicodeCollection(HashSet<char>);

impl CharacterInspector for UnicodeCollection {
    fn characters(&self) -> Vec<char> {
        self.0.iter().map(|&c| c).collect()
    }
    fn contains(&self, c: char) -> bool {
        self.0.contains(&c)
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
            font_id: egui::FontId::proportional(18.0),
            categories: vec![
                (
                    ub::ARROWS.name().to_string(),
                    UnicodeCategory::Block(ub::ARROWS),
                ),
                //(ub::PARENTHESES.name().to_string(), vec![ub::PARENTHESES])
                (
                    "Punctuation".to_string(),
                    UnicodeCategory::Block(ub::GENERAL_PUNCTUATION),
                ),
                (
                    ub::CURRENCY_SYMBOLS.name().to_string(),
                    UnicodeCategory::Block(ub::CURRENCY_SYMBOLS),
                ),
                //(ub::PICTOGRAPHS.name().to_string(), vec![ub::PICTOGRAPHS]),
                (
                    ub::LETTERLIKE_SYMBOLS.name().to_string(),
                    UnicodeCategory::Block(ub::LETTERLIKE_SYMBOLS),
                ),
                (
                    ub::EMOTICONS.name().to_string(),
                    UnicodeCategory::Block(ub::EMOTICONS),
                ),
                (
                    ub::DINGBATS.name().to_string(),
                    UnicodeCategory::Block(ub::DINGBATS),
                ),
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

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for GlyphanaApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        let mut named_chars = &mut BTreeMap::<char, std::string::String>::new();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
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
                    self.search_text = self.search_text.to_lowercase();
                    ui.button(format!("{}", super::MAGNIFIER));

                });

                named_chars = self
                    .named_chars
                    .entry(self.font_id.family.clone())
                    .or_insert_with(|| available_characters(ui, self.font_id.family.clone()));
            });
        });


        egui::SidePanel::left("category_panel").show(ctx, |ui| {
            ui.selectable_label(false, "Favorites");

            ui.separator();

            for category in &self.categories {
                ui.selectable_label(false, &category.0);
            }
        });

        egui::SidePanel::right("character_panel").show(ctx, |ui| {
            let scale = 3.0;

            /*
            let painter = Painter::new(
                ui.ctx().clone(),
                ui.layer_id(),
                ui.available_rect_before_wrap(),
            );

            self.paint(&painter);

            // Make sure we allocate what we used (everything)
            ui.expand_to_include_rect(painter.clip_rect());*/

            let color = if ui.visuals().dark_mode {
                Color32::from_additive_luminance(196)
            } else {
                Color32::from_black_alpha(240)
            };

            /*Frame::canvas(ui.style()).show(ui, |ui| {
                ui.ctx().request_repaint();
                let time = ui.input().time;

                let desired_size = ui.available_width() * vec2(1.0, 0.35);
                let (_id, rect) = ui.allocate_space(desired_size);

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                    rect,
                );

                let mut shapes = vec![epaint::Shape];

                ui.painter().extend(shapes);
            });*/

            //RichText::new("&").size(20.0);

            //let (response, painter) =
            //    ui.allocate_painter(Vec2::new(ui.available_width(), 300.0), Sense::hover());

            //Shape::text("g").scale(scale).build(ui);
            /*
            // here you can get the font_metrics

            let path = "assets/NotoSans-Regular.ttf";
            let File { mut fonts } = File::open(path).unwrap();
            let font = fonts[0];
            let glyph = font.draw('&').unwrap().unwrap();

            if let Some(metrics) = state.font_metrics {
                let width = metrics.left_side_bearing + metrics.right_side_bearing;
                let height = metrics.ascender - metrics.descender;

                egui::Rect::new()
                    .w_h(width * scale, height * scale)
                    .build(ui, || {
                        egui::Text::new("g")
                            .scale(scale)
                            .build(ui);

                        egui::Line::new()
                            .start(0.0, metrics.ascender * scale)
                            .end(width * scale, metrics.ascender * scale)
                            .build(ui);

                        egui::Line::new()
                            .start(0.0, metrics.cap_height * scale)
                            .end(width * scale, metrics.cap_height * scale)
                            .build(ui);

                        egui::Line::new()
                            .start(0.0, metrics.baseline * scale)
                            .end(width * scale, metrics.baseline * scale)
                            .build(ui);
                    });
                */
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::splat(2.0);

                    for (&chr, name) in named_chars {
                        if self.search_text.is_empty()
                            || name.contains(&self.search_text)
                            || *self.search_text == chr.to_string()
                        {
                            let button = egui::Button::new(
                                egui::RichText::new(chr.to_string()).font(self.font_id.clone()),
                            )
                            .frame(true)
                            .min_size(egui::Vec2 {
                                x: self.font_id.size * 2.,
                                y: self.font_id.size * 2.,
                            });

                            let tooltip_ui = |ui: &mut egui::Ui| {
                                ui.label(
                                    egui::RichText::new(chr.to_string()).font(self.font_id.clone()),
                                );
                                ui.label(format!("{}\nU+{:X}\n\nDouble-click to copy 📋", name, chr as u32));
                            };

                            let hover_button = ui.add(button).on_hover_ui(tooltip_ui);

                            if hover_button.double_clicked() {
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
        // Web services / operating systems / browsers
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
