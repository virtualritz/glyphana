#![warn(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use anyhow::Result;
use eframe::NativeOptions;
use egui::ViewportBuilder;
#[cfg(not(target_os = "linux"))]
use std::sync::{Arc, Mutex};
// Import the library modules
use glyphana::GlyphanaApp;
#[cfg(not(target_os = "linux"))]
use tray_icon::TrayIcon;
use tray_icon::{
    Icon as TrayIconData, TrayIconBuilder,
    menu::{AboutMetadata, Menu, MenuItem, PredefinedMenuItem},
};

// Menu item IDs
const SHOW_ID: &str = "show";
const QUIT_ID: &str = "quit";

fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // Load icon bytes for both eframe and tray icon
    let icon_bytes = include_bytes!("../assets/icon-1024.png");

    // Create eframe icon using the built-in function
    let eframe_icon = eframe::icon_data::from_png_bytes(icon_bytes).expect("Failed to load icon");

    // Create tray icon from the eframe icon data
    let tray_icon = TrayIconData::from_rgba(
        eframe_icon.rgba.clone(),
        eframe_icon.width as u32,
        eframe_icon.height as u32,
    )?;

    // Create tray menu
    let tray_menu = Menu::new();
    let show_item = MenuItem::with_id(SHOW_ID, "Show Glyphana", true, None);
    let quit_item = MenuItem::with_id(QUIT_ID, "Quit", true, None);

    tray_menu.append_items(&[
        &show_item,
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::about(
            None,
            Some(AboutMetadata {
                name: Some("Glyphana".to_string()),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
                copyright: Some("© 2024 Moritz Moeller".to_string()),
                authors: Some(vec!["Moritz Moeller".to_string()]),
                comments: Some("Unicode Glyph Explorer".to_string()),
                website: Some("https://github.com/virtualritz/glyphana".to_string()),
                website_label: Some("GitHub Repository".to_string()),
                icon: None,
                ..Default::default()
            }),
        ),
        &PredefinedMenuItem::separator(),
        &quit_item,
    ])?;

    // Initialize tray icon for Linux
    #[cfg(target_os = "linux")]
    {
        let tray_icon_clone = tray_icon.clone();
        std::thread::spawn(move || {
            gtk::init().unwrap();

            // Create menu inside the thread for Linux
            let menu = Menu::new();
            let show_item = MenuItem::with_id(SHOW_ID, "Show Glyphana", true, None);
            let quit_item = MenuItem::with_id(QUIT_ID, "Quit", true, None);

            menu.append_items(&[
                &show_item,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::about(
                    None,
                    Some(AboutMetadata {
                        name: Some("Glyphana".to_string()),
                        version: Some(env!("CARGO_PKG_VERSION").to_string()),
                        copyright: Some("© 2024 Moritz Moeller".to_string()),
                        authors: Some(vec!["Moritz Moeller".to_string()]),
                        comments: Some("Unicode Glyph Explorer".to_string()),
                        website: Some("https://github.com/virtualritz/glyphana".to_string()),
                        website_label: Some("GitHub Repository".to_string()),
                        icon: None,
                        ..Default::default()
                    }),
                ),
                &PredefinedMenuItem::separator(),
                &quit_item,
            ])
            .unwrap();

            let _tray = TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip("Glyphana - Unicode Glyph Explorer")
                .with_icon(tray_icon_clone)
                .build()
                .unwrap();

            gtk::main();
        });
    }

    // Store tray icon for non-Linux platforms
    #[cfg(not(target_os = "linux"))]
    let tray_holder = Arc::new(Mutex::new(None::<TrayIcon>));

    // Clone for non-Linux platforms
    #[cfg(not(target_os = "linux"))]
    let tray_holder_clone = tray_holder.clone();

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(eframe_icon)
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Glyphana",
        native_options,
        Box::new(move |creation_context| {
            // Create tray icon after window creation on non-Linux platforms
            #[cfg(not(target_os = "linux"))]
            {
                let tray = TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_tooltip("Glyphana - Unicode Glyph Explorer")
                    .with_icon(tray_icon)
                    .build()
                    .unwrap();
                *tray_holder_clone.lock().unwrap() = Some(tray);
            }

            Ok(Box::new(GlyphanaApp::new(creation_context)))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run native app: {:?}", e))?;

    Ok(())
}
