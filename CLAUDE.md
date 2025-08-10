# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 重要: ユーザーからの指示
このプロジェクトは日本語が母語の日本人によって開発されています。可能な限り日本語で回答してください。 ただし、技術的な用語は無理に翻訳を行わずとも問題ありません。

## Project Overview

This is a Rust application that scrapes emergency dispatch information from various Japanese fire departments and provides it in a unified JSON format. The system fetches data from fire department websites across Japan and outputs standardized emergency dispatch information.

## Development Commands

- **Build and run**: `cargo run`
- **Build only**: `cargo build`
- **Run tests**: `cargo test`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`

## Architecture

### Core Components

- **`main.rs`**: Entry point that orchestrates the entire process - creates output directory, runs all parsers, generates metadata
- **`lib.rs`**: Contains shared utilities and the main coordination logic:
  - `get_all()`: Executes all city-specific parsers
  - `generate_list_json()`: Creates list of supported municipality codes
  - `generate_rss_feed()`: Generates RSS feed from all collected data
  - `to_half_width()`: Utility for converting full-width to half-width numbers

### Parser Architecture

Each supported municipality has its own parser module in `src/parse/`:
- **File naming**: `parse_XXXXXX.rs` where XXXXXX is the 6-digit municipal code (JIS X 0402)
- **Function naming**: Each parser exports a `return_XXXXXX()` function
- **Parser responsibility**: Fetch HTML from fire department website, parse disaster information, output to `dist/XXXXXX.json`

### Data Format

All parsers output JSON files to the `dist/` directory with this structure:
```json
{
  "disasters": [
    {
      "address": "Full address starting with prefecture",
      "time": "HH:MM format time", 
      "type": "Type of emergency dispatch"
    }
  ],
  "jisx0402": "6-digit municipal code",
  "source": [
    {
      "name": "Fire department name",
      "url": "Source website URL"
    }
  ]
}
```

### HTTP Client Configuration

All parsers use a common User-Agent string defined in `lib.rs` as `ACCESS_UA` that identifies the bot and project. Parsers typically set up custom headers for their specific target websites.

### Supported Municipalities

The system supports 15 Japanese municipalities across various prefectures, each identified by their 6-digit JIS X 0402 code. The complete list is maintained in the README and reflected in the parser modules.

## Adding New Municipalities

To add support for a new municipality:

1. Create `src/parse/parse_XXXXXX.rs` (where XXXXXX is the municipal code)
2. Implement `return_XXXXXX()` function following existing patterns
3. Add module declaration to `src/parse/mod.rs`
4. Add function call to `get_all()` in `lib.rs`
5. Add import statement at the top of `lib.rs`

## Output Files

- `dist/XXXXXX.json`: Individual municipality data files
- `dist/list.json`: Array of all supported municipal codes
- `dist/all_feed.xml`: RSS 2.0 feed combining all emergency dispatches

## Dependencies

- **reqwest**: HTTP client for fetching web pages
- **scraper**: HTML parsing and CSS selector support
- **serde_json**: JSON serialization/deserialization
- **chrono**: Date and time handling for RSS feed generation
- **regex**: Pattern matching for file operations
- **encoding_rs**: Character encoding handling