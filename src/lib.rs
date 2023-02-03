#![warn(clippy::all)]
#![feature(option_result_contains)]

pub const CANCELLATION: char = '🗙';
pub const COG_WHEEL: char = '⚙';
pub const HAMBURGER: char = '☰';
pub const MAGNIFIER: char = '🔍';
pub const DOCUMENT_WITH_TEXT: char = '🖹';
pub const SUBSET: char = '⊂';

pub const NOTO_SANS: &'static str = "noto-sans";
pub const NOTO_SANS_MATH: &'static str = "noto-sans-math";

static NOTO_SANS_FONT: &'static [u8] = include_bytes!("../assets/NotoSans-Regular.otf");

mod app;
pub use app::GlyphanaApp;
