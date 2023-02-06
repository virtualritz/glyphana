#![warn(clippy::all)]
#![allow(clippy::blocks_in_if_conditions)]
#![feature(option_result_contains)]
#![feature(let_chains)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use image::ImageDecoder;
use include_flate::flate;
use std::error::Error;

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

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn Error>> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        icon_data: Some(load_icon()), // an example
        ..Default::default()
    };

    eframe::run_native(
        "Glyphana",
        native_options,
        Box::new(|creation_context| Box::new(glyphana::GlyphanaApp::new(creation_context))),
    )?;

    Ok(())
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(eframe_template::TemplateApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
