# Glyphana - Unicode Glyph Explorer

## Project Overview
Glyphana is a Rust-based Unicode glyph exploration and collection tool built with egui. It allows users to search, browse, inspect, and collect Unicode glyphs across different categories and fonts.

## Architecture

### Main Components
- **app.rs**: Core application logic including UI, search, and glyph management
- **main.rs**: Entry point and font loading

### Key Features
- Unicode block browsing and search
- Drag-and-drop category organization  
- Recently used glyph tracking
- Glyph collection management
- Multi-font support (Noto fonts)
- Glyph name and skeleton search

## Development Commands

### Build
```bash
cargo build
```

### Run
```bash
cargo run
```

### Test
```bash
cargo test
```

### Lint & Format
```bash
cargo fmt
cargo clippy
```

## Dependencies
- **egui/eframe** (0.32.1): GUI framework
- **egui_dnd** (0.13.0): Drag and drop support
- **rusttype** (0.9.3): Font rendering
- **unicode-blocks** (0.1.9): Unicode block definitions
- **unicode_names2**: Unicode character names
- **ahash**: Fast hashing
- **stringzilla** (3.12): Fuzzy string matching with edit distance

## Known Issues & TODOs
- Color emoji support needs implementation (requires NotoColorEmoji font file)

## Recent Changes
- Updated to egui 0.32.1 and latest dependencies
- Fixed platform support for Linux (added wayland/x11 features)
- Improved drag and drop implementation for egui_dnd 0.13.0
- Enhanced glyph search with fuzzy matching using stringzilla
- Fixed ascender/descender lines in single glyph view with proper labels
- Improved glyph preview sizing consistency across fonts
- Added search support for Unicode block names, hex codes, and decimal codes
- Fixed deprecated API usage (menu::bar, copied_text)