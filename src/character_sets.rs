// ----------------------------------------------------------------------------

/// A menu bar in which you can select different demo windows to show.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct MainWindow {
    about_is_open: bool,
    about: About,
    //demos: Demos,
    //tests: Tests,
    tables: Tables
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            about_is_open: true,
            about: Default::default(),
            //demos: Default::default(),
            //tests: Default::default(),
        }
    }
}

impl MainWindow {
    /// Show the app ui (menu bar and windows).
    pub fn ui(&mut self, ctx: &Context) {
        /*if is_mobile(ctx) {
            self.mobile_ui(ctx);
        } else {
            self.desktop_ui(ctx);
        }*/

        self.desktop_ui(ctx);
    }

    /*
    fn mobile_ui(&mut self, ctx: &Context) {
        if self.about_is_open {
            let screen_size = ctx.input().screen_rect.size();
            let default_width = (screen_size.x - 20.0).min(400.0);

            let mut close = false;
            egui::Window::new(self.about.name())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .default_width(default_width)
                .default_height(ctx.available_rect().height() - 46.0)
                .vscroll(true)
                .open(&mut self.about_is_open)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    self.about.ui(ui);
                    ui.add_space(12.0);
                    ui.vertical_centered_justified(|ui| {
                        if ui
                            .button(egui::RichText::new("Continue to the demo!").size(20.0))
                            .clicked()
                        {
                            close = true;
                        }
                    });
                });
            self.about_is_open &= !close;
        } else {
            self.mobile_top_bar(ctx);
            self.show_windows(ctx);
        }
    }

    fn mobile_top_bar(&mut self, ctx: &Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let font_size = 16.5;

                ui.menu_button(egui::RichText::new("⏷ demos").size(font_size), |ui| {
                    ui.set_style(ui.ctx().style()); // ignore the "menu" style set by `menu_button`.
                    self.demo_list_ui(ui);
                    if ui.ui_contains_pointer() && ui.input().pointer.any_click() {
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    use egui::special_emojis::{GITHUB, TWITTER};
                    ui.hyperlink_to(
                        egui::RichText::new(TWITTER).size(font_size),
                        "https://twitter.com/ernerfeldt",
                    );
                    ui.hyperlink_to(
                        egui::RichText::new(GITHUB).size(font_size),
                        "https://github.com/emilk/egui",
                    );
                });
            });
        });
    }
    */

    fn desktop_ui(&mut self, ctx: &Context) {

        egui::TopBottomPanel::top("bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                file_menu_button(ui);
            });
        });

        egui::ScrollArea::horizontal().show(ui, |ui| {
                        self.table_ui(ui);
                    });


        egui::SidePanel::right("egui_demo_panel")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                egui::trace!(ui);
                ui.vertical_centered(|ui| {
                    ui.heading("✒ egui demos");
                });

                ui.separator();

                self.demo_list_ui(ui);
            });



        self.show_windows(ctx);
    }

    /// Show the open windows.
    fn show_windows(&mut self, ctx: &Context) {
        self.about.show(ctx, &mut self.about_is_open);
        self.demos.windows(ctx);
        self.tests.windows(ctx);
    }

    fn demo_list_ui(&mut self, ui: &mut egui::Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                ui.toggle_value(&mut self.about_is_open, self.about.name());

                self.favorites.
                ui.separator();
                for
                ui.separator();

                if ui.button("Organize windows").clicked() {
                    ui.ctx().memory().reset_areas();
                }
            });
        });
    }
}