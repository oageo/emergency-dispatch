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

## HTTP Configuration System

The codebase uses a centralized `HttpRequestConfig` struct for handling different website requirements:

- **Default headers**: Defined as constants in `lib.rs` (Accept, Accept-Language, Connection, Content-Type)
- **Character encoding**: Use `.with_shift_jis(true)` only when the target website specifically uses Shift_JIS encoding (check the HTML meta charset or test for garbled text)
- **Custom headers**: Override defaults using methods like `.with_accept()`, `.with_accept_language()`

### Example Usage:
```rust
let config = HttpRequestConfig::new(HOST, GET_SOURCE)
    .with_shift_jis(true)  // Only when target site uses Shift_JIS
    .with_accept("custom/accept");
```

## Parser Development Approach

When creating a new parser, follow this analysis process:

1. **Compare with existing parsers**: Check if the target website shares patterns with existing municipalities
2. **Identify HTML structure**: Look for common Japanese patterns like `◆現在の出動`, `出動情報`, `災害情報`
3. **Test encoding**: Start with default (UTF-8), only add `.with_shift_jis(true)` if text appears garbled
4. **Pattern matching**: Look for existing text processing patterns that might apply

### Common Japanese Text Patterns

Some patterns found across multiple municipalities (analyze each case individually):
- **Time format conversion**: `時` → `:`, `分` → `` (empty)
- **Section markers**: `◆`, `●`, `・` often indicate different content sections
- **Address formatting**: Some regions require prefecture prefixes
- **Empty state indicators**: Various phrases indicate no current dispatches

## Testing Individual Parsers

To test a specific municipality parser during development:
1. Temporarily modify `get_all()` in `lib.rs` to call only the desired parser function
2. Run `cargo run` to execute only that parser
3. Check the output in `dist/XXXXXX.json`

## UTF-8 String Processing Guidelines

**CRITICAL**: When working with Japanese text, always use UTF-8-safe string operations to avoid `is_char_boundary` panics.

### Core Principle: Never Mix Byte Indices with Character Data

The fundamental issue is that `find()` returns **byte indices**, but Japanese characters can be 2-4 bytes each. Using byte indices for string slicing can split characters in the middle, causing panics.

### Safe Text Processing Strategies

#### 1. Text Extraction Between Markers
```rust
// ❌ UNSAFE: Using byte indices from find()
if let Some(start) = text.find(start_marker) {
    let after_start = &text[start..];  // Can panic on multibyte chars
    if let Some(end) = after_start.find(end_marker) {
        let extracted = &after_start[..end];  // Can panic
    }
}

// ✅ SAFE: Using split() for extraction
let extracted = text
    .split(start_marker).nth(1)  // Get text after start marker
    .and_then(|s| s.split(end_marker).next())  // Get text before end marker
    .unwrap_or("");

// ✅ SAFE: Chain multiple splits for complex extraction
let result = text
    .split(first_marker).nth(1).unwrap_or("")
    .split(second_marker).next().unwrap_or("")
    .split(third_marker).next().unwrap_or("");
```

#### 2. Removing Unwanted Text Patterns
```rust
// ❌ UNSAFE: Using indices for removal
if let (Some(start), Some(end)) = (text.find(open_char), text.find(close_char)) {
    text.replace_range(start..=end, "");  // Can panic
}

// ✅ SAFE: Replacement-based removal
let cleaned = text.replace(unwanted_pattern, "");

// ✅ SAFE: Split-based removal for complex patterns
let cleaned = if text.contains(open_char) && text.contains(close_char) {
    let parts: Vec<&str> = text.split(open_char).collect();
    if parts.len() >= 2 {
        let before = parts[0];
        let after_parts: Vec<&str> = parts[1].split(close_char).collect();
        if after_parts.len() >= 2 {
            format!("{}{}", before, after_parts[1])
        } else {
            before.to_string()
        }
    } else {
        text.to_string()
    }
} else {
    text.to_string()
};
```

#### 3. Address and Location Processing
```rust
// ✅ SAFE: Progressive text refinement
let processed_address = original_text
    .split(city_marker).nth(1).unwrap_or("")  // Extract after city name
    .split(suffix_marker).next().unwrap_or("")  // Extract before suffix
    .replace(unwanted_chars, "")  // Remove unwanted characters
    .trim()  // Clean whitespace
    .to_string();

// ✅ SAFE: Conditional formatting
let final_address = if processed_address.contains(suffix) {
    let base = processed_address.split(suffix).next().unwrap_or("");
    format!("{}{}{}", prefix, base, suffix)
} else {
    format!("{}{}", prefix, processed_address)
};
```

#### 4. Time and Date Processing
```rust
// ✅ SAFE: Character-based replacements
let formatted_time = raw_time
    .chars()
    .rev()
    .take(char_count)
    .collect::<String>()
    .chars()
    .rev()
    .collect::<String>()
    .replace(hour_char, ":")
    .replace(minute_char, "");
```

### Universal Safe Patterns

#### Pattern 1: Sequential Text Extraction
```rust
fn extract_between_markers(text: &str, start: &str, end: &str) -> Option<&str> {
    text.split(start).nth(1)?.split(end).next()
}
```

#### Pattern 2: Multi-Stage Text Processing
```rust
fn process_text_safely(text: &str, markers: &[&str], replacements: &[(&str, &str)]) -> String {
    let mut result = text;

    // Stage 1: Extract using markers
    for (i, marker) in markers.iter().enumerate() {
        if let Some(extracted) = result.split(marker).nth(1) {
            result = extracted;
        }
    }

    // Stage 2: Apply replacements
    let mut final_result = result.to_string();
    for (from, to) in replacements {
        final_result = final_result.replace(from, to);
    }

    final_result.trim().to_string()
}
```

#### Pattern 3: Conditional Text Transformation
```rust
fn transform_conditionally(text: &str, conditions: &[&str], transformations: &[fn(&str) -> String]) -> String {
    for (condition, transform) in conditions.iter().zip(transformations.iter()) {
        if text.contains(condition) {
            return transform(text);
        }
    }
    text.to_string()
}
```

### Development Guidelines

1. **Always use `split()` for text segmentation** - never combine `find()` with slicing
2. **Use `replace()` for simple character/pattern removal**
3. **Chain operations safely** - each step should handle the absence of expected patterns
4. **Test with real Japanese data** containing various character combinations
5. **Prefer character-based operations** (`chars()`) over byte-based operations
6. **Use `unwrap_or()` and `unwrap_or_else()`** to handle missing patterns gracefully

### Error Prevention Checklist

Before committing parser code, verify:
- [ ] No `&text[index..]` or `&text[..index]` or `&text[start..end]` patterns
- [ ] No `replace_range()` with `find()` indices
- [ ] All text extraction uses `split()` or `replace()`
- [ ] Error handling for missing patterns (using `unwrap_or()`)
- [ ] Testing with actual fire department data

### Debugging UTF-8 Issues

When encountering `is_char_boundary` panics:
1. **Identify the operation**: Look for slice operations in the stack trace
2. **Find the byte index source**: Usually from `find()`, `rfind()`, or similar methods
3. **Replace with character-safe alternatives**: Use the patterns above
4. **Verify with diverse test data**: Include various Japanese character combinations