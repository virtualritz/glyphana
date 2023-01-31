#![warn(clippy::all)]

pub const COG_WHEEL: char = '⚙';
pub const MAGNIFIER: char = '🔍';
pub const HAMBURGER: char = '☰';
pub const CANCELLATION: char = '🗙';

mod app;
pub use app::GlyphanaApp;
