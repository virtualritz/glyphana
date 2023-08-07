#![warn(clippy::all)]
#![allow(clippy::blocks_in_if_conditions)]
#![feature(let_chains)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use image::ImageDecoder;
use include_flate::flate;
use std::error::Error;
/*use tray_icon::{
    icon::Icon,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};*/

mod app;
pub use app::GlyphanaApp;

pub const CANCELLATION: char = '🗙';
pub const COG_WHEEL: char = '⚙';
pub const HAMBURGER: char = '☰';
pub const MAGNIFIER: char = '🔍';
pub const NAME_BADGE: char = '📛';
pub const LOWER_UPPER_CASE: char = '🗛';
pub const PUSH_PIN: char = '📌';
pub const SUBSET: char = '⊂';

pub const NOTO_SANS: &str = "noto-sans";
flate!(pub static NOTO_SANS_FONT: [u8] from "assets/NotoSans-Regular.otf");

pub const NOTO_SANS_MATH: &str = "noto-sans-math";
flate!(pub static NOTO_SANS_MATH_FONT: [u8] from "assets/NotoSansMath-Regular.ttf");

/*
pub const NOTO_COLOR_EMOJI: &'static str = "noto-color-emoji";
flate!(pub static NOTO_COLOR_EMOJI_FONT: [u8] from "assets/NotoColorEmoji-Regular.ttf");
*/

pub const NOTO_EMOJI: &str = "noto-emoji";
flate!(pub static NOTO_EMOJI_FONT: [u8] from "assets/NotoEmoji-Regular.ttf");

pub const EMOJI_ICON: &str = "emoji-icon";
flate!(pub static EMOJI_ICON_FONT: [u8] from "assets/emoji-icon-font.ttf");

pub const NOTO_SYMBOLS: &str = "noto-symbols";
flate!(pub static NOTO_SYMBOLS_FONT: [u8] from "assets/NotoSansSymbols-Regular.ttf");

pub const NOTO_SYMBOLS2: &str = "noto-symbols2";
flate!(pub static NOTO_SYMBOLS2_FONT: [u8] from "assets/NotoSansSymbols2-Regular.ttf");

/*
pub const NOTO_SIGN_WRITING: &str = "noto-sign_writing";
flate!(pub static NOTO_SIGN_WRITING_FONT: [u8] from "assets/NotoSansSignWriting-Regular.ttf");
*/

pub const NOTO_MUSIC: &str = "noto-music";
flate!(pub static NOTO_MUSIC_FONT: [u8] from "assets/NotoMusic-Regular.ttf");

fn main() -> Result<(), Box<dyn Error>> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let icon = load_icon();

    /* Tray icon stuff: works but no menu messages reach the GlyphanaApp::update()
     * method.

    let tray_icon = {
        let icon = icon.clone();
        Icon::from_rgba(icon.rgba, icon.width, icon.height).unwrap()
    };

    #[cfg(target_os = "linux")]
    std::thread::spawn(move || {
        gtk::init().unwrap();

        let tray_menu = Box::new(Menu::new());

        tray_menu.append_items(&[
            &MenuItem::new("Open Glyphana", true, None),
            &PredefinedMenuItem::separator(),
            &MenuItem::new("Quit Glyphana", true, None),
        ]);

        let _tray_icon = TrayIconBuilder::new()
            .with_menu(tray_menu)
            .with_icon(tray_icon)
            .build()
            .unwrap();

        gtk::main();
    });

    #[cfg(not(target_os = "linux"))]
    let tray_menu = Box::new(Menu::new());

    #[cfg(not(target_os = "linux"))]
    tray_menu.append_items(&[
        &PredefinedMenuItem::show_all(Some("Open Glyphana")),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(Some("Quit Glyphana")),
    ]);

    #[cfg(not(target_os = "linux"))]
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(tray_menu)
        .with_icon(tray_icon)
        .build()
        .unwrap();
    */

    let native_options = eframe::NativeOptions {
        icon_data: Some(icon),
        //renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        "Glyphana",
        native_options,
        Box::new(|creation_context| Box::new(crate::GlyphanaApp::new(creation_context))),
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
