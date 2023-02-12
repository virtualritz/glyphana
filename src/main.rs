#![warn(clippy::all)]
#![allow(clippy::blocks_in_if_conditions)]
#![feature(option_result_contains)]
#![feature(let_chains)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use image::ImageDecoder;
use include_flate::flate;
use std::error::Error;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    //gtk::init().unwrap();

    let icon = load_icon();

    /*
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
        icon_data: Some(icon),
        //renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Glyphana",
        native_options,
        Box::new(|creation_context| Box::new(glyphana::GlyphanaApp::new(creation_context))),
    )?;

    Ok(())
}

fn load_icon() -> eframe::IconData {
    flate!(static ICON: [u8] from "assets/icon-1024.png");
    let icon: &[u8] = &ICON;

    let (icon_rgba, icon_width, icon_height) = {
        let image = image::codecs::png::PngDecoder::new(icon).expect("Failed to decode icon");
        let mut rgba = vec![0; image.total_bytes() as _];
        let (width, height) = image.dimensions();
        image.read_image(&mut rgba).unwrap();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}
