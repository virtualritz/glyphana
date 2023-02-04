#![warn(clippy::all)]
#![feature(option_result_contains)]
use include_flate::flate;

pub const CANCELLATION: char = '🗙';
pub const COG_WHEEL: char = '⚙';
pub const HAMBURGER: char = '☰';
pub const MAGNIFIER: char = '🔍';
pub const DOCUMENT_WITH_TEXT: char = '🖹';
pub const SUBSET: char = '⊂';

pub const NOTO_SANS: &'static str = "noto-sans";
flate!(pub static NOTO_SANS_FONT: [u8] from "assets/NotoSans-Regular.otf");

pub const NOTO_SANS_MATH: &'static str = "noto-sans-math";
flate!(pub static NOTO_SANS_MATH_FONT: [u8] from "assets/NotoSansMath-Regular.ttf");

/*
pub const NOTO_COLOR_EMOJI: &'static str = "noto-color-emoji";
flate!(pub static NOTO_COLOR_EMOJI_FONT: [u8] from "assets/NotoColorEmoji-Regular.ttf");
*/

pub const NOTO_EMOJI: &'static str = "noto-emoji";
flate!(pub static NOTO_EMOJI_FONT: [u8] from "assets/NotoEmoji-Regular.ttf");

mod app;
pub use app::GlyphanaApp;
