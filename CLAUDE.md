# Glyphana - Unicode Glyph Explorer

## The Golden Rule
When unsure about implementation details, ALWAYS ask the developer.

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
- Fuzzy string matching with edit distance

## Development Commands

### Build
```bash
cargo build
```
**Note:** Never run `cargo build --release` unless explicitly instructed by the user. Always use debug builds for development and testing.

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
cargo clippy --fix --allow-dirty
```

## Code Style and Patterns

### Guidelines
- **CRITICAL: ALWAYS run `cargo test` and ensure the code compiles and tests pass WITHOUT ANY WARNINGS BEFORE committing!**
  - First run: `cargo test` to ensure everything compiles and passes
  - Then run: `cargo fmt` to format the code
  - Then run: `cargo clippy --fix --allow-dirty` and fix any issues
  - Finally run: `cargo test` one more time to verify everything is clean
  - Only commit when there are ZERO warnings and all tests pass

- **CRITICAL: Address ALL warnings before EVERY commit!** This includes:
  - Unused imports, variables, and functions
  - Dead code warnings
  - Deprecated API usage
  - Type inference ambiguities
  - Any clippy warnings or suggestions
  - Never use `#[allow(warnings)]` or similar suppressions without explicit user approval

- Write idiomatic and canonical Rust code
- PREFER functional style over imperative style (use `map`/`collect` over `for` loops)
- AVOID unnecessary allocations, conversions, copies
- AVOID using `unsafe` code unless absolutely necessary
- AVOID return statements; structure functions with if...else blocks instead
- DO NOT change any public-facing API without presenting a change proposal to the user first

### Anchor Comments
- Use `AIDEV-NOTE:`, `AIDEV-TODO:`, or `AIDEV-QUESTION:` for AI/developer comments
- **Do not remove `AIDEV-NOTE`s** without explicit human instruction
- Add anchor comments for complex, important, or confusing code

## What AI Must NEVER Do
1. **Never modify test files** - Tests encode human intent
2. **Never change API contracts** - Breaks real applications
3. **Never commit secrets** - Use environment variables
4. **Never assume business logic** - Always ask
5. **Never remove AIDEV- comments** - They're there for a reason

## Writing Instructions For User Interaction
- Be concise
- AVOID weasel words and flattery
- Use simple sentences with technical jargon where appropriate
- Do NOT overexplain basic concepts
- Maintain a neutral viewpoint

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