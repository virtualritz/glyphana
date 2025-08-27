#![warn(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::error::Error;

// Import the library modules
use glyphana::GlyphanaApp;

fn main() -> Result<(), Box<dyn Error>> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let icon = load_icon()?;

    /* Tray icon stuff: works but no menu messages reach the GlyphanaApp::update()
     * method.

    let tray_icon = {
        let icon = icon.clone();
        tray_icon::icon::Icon::from_rgba(icon.rgba, icon.width, icon.height).unwrap()
    };

    #[cfg(target_os = "linux")]
    std::thread::spawn(move || {
        gtk::init().unwrap();

        use tray_icon::{
            menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
            TrayIconBuilder,
        };

        let tray_menu = Box::new(Menu::new());
        let quit = MenuItem::new("Quit Glyphana", true, None);
        tray_menu.append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("Glyphana".to_string()),
                    copyright: Some("Copyright Moritz Moeller 2023".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit,
        ]);

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(tray_menu)
            .with_icon(tray_icon)
            .build()
            .unwrap();

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let _tray_icon = TrayIconBuilder::new().with_icon(tray_icon).build().unwrap();
    */

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(icon),
        //renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Glyphana",
        native_options,
        Box::new(|creation_context| Ok(Box::new(GlyphanaApp::new(creation_context)))),
    )?;

    Ok(())
}

fn load_icon() -> Result<egui::viewport::IconData, Box<dyn Error>> {
    let icon_bytes = include_bytes!("../assets/icon-1024.png");

    let (icon_rgba, icon_width, icon_height) = {
        use image::ImageDecoder;
        use std::io::Cursor;

        let decoder = image::codecs::png::PngDecoder::new(Cursor::new(icon_bytes))?;
        let total_bytes = decoder.total_bytes();
        let mut rgba = vec![0; total_bytes as usize];
        let (width, height) = decoder.dimensions();
        decoder.read_image(&mut rgba)?;
        (rgba, width, height)
    };

    Ok(egui::viewport::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    })
}

