#![warn(clippy::all)]
#![allow(clippy::blocks_in_if_conditions)]
#![feature(option_result_contains)]
#![feature(let_chains)]
use include_flate::flate;

pub const CANCELLATION: char = '🗙';
pub const COG_WHEEL: char = '⚙';
pub const HAMBURGER: char = '☰';
pub const MAGNIFIER: char = '🔍';
pub const NAME_BADGE: char = '📛';
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

pub const NOTO_SYMBOLS: &str = "noto-symbols";
flate!(pub static NOTO_SYMBOLS_FONT: [u8] from "assets/NotoSansSymbols-Regular.ttf");

pub const NOTO_SYMBOLS2: &str = "noto-symbols2";
flate!(pub static NOTO_SYMBOLS2_FONT: [u8] from "assets/NotoSansSymbols2-Regular.ttf");

pub const NOTO_SIGN_WRITING: &str = "noto-sign_writing";
flate!(pub static NOTO_SIGN_WRITING_FONT: [u8] from "assets/NotoSansSignWriting-Regular.ttf");

pub const NOTO_MUSIC: &str = "noto-music";
flate!(pub static NOTO_MUSIC_FONT: [u8] from "assets/NotoMusic-Regular.ttf");

mod app;
pub use app::GlyphanaApp;
