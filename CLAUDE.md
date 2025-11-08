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

- **`main.rs`**: Entry point that orchestrates the entire process in the following order:
  1. Creates output directory (`dist/`)
  2. Runs all parsers via `get_all()` to fetch and output individual municipality JSON files
  3. Generates `list.json` via `generate_list_json()`
  4. Generates `all.json` via `generate_all_json()`
  5. Generates `all_feed.xml` via `generate_rss_feed()`

- **`lib.rs`**: Contains shared utilities and the main coordination logic:
  - `get_all()`: Executes all city-specific parsers
  - `generate_list_json()`: Creates list of supported municipality codes
  - `generate_all_json()`: Creates unified JSON file with all active disasters
  - `generate_rss_feed()`: Generates RSS feed from all collected data
  - `to_half_width()`: Utility for converting full-width to half-width numbers
  - `load_previous_guid_mapping()`: Loads GUID mappings from previous RSS feed for deduplication

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

The system supports multiple Japanese municipalities across various prefectures, each identified by their 6-digit JIS X 0402 code. The complete list is maintained in the README and reflected in the parser modules.

**To count the current number of supported municipalities**:
- Check the "対応市区町村" section in README.md and count the entries with 6-digit codes
- Use `grep -c "^    \* .*[0-9]\{6\}$" README.md` to count programmatically from README
- Count parser files: `find src/parse -name "parse_*.rs" | wc -l` to count actual implementation files
- Check `src/lib.rs` for import statements or function calls in `get_all()`

**Note**: Minor discrepancies between these counts may exist due to manual updates or development in progress. This is normal and should not be a cause for concern.

## Municipal Code Verification

**CRITICAL**: Always verify the correct JIS X 0402 municipal code before implementing parsers.

### Authoritative Data Sources

1. **Primary Source** - JSON API (recommended):
   - URL: `https://jmc.osumiakari.jp/joint_all.json`
   - Contains comprehensive list of all Japanese municipalities with their official 6-digit codes
   - Includes both basic municipalities (市区町村) and special entities (一部事務組合)
   - Use this to verify codes before creating parsers

2. **Alternative Source** - Web Interface:
   - URL: `https://jmc.osumiakari.jp/all/`
   - Human-readable format of the same data
   - Useful for quick lookups and verification

### Verification Process

Before implementing a parser:
1. Look up the municipality name in `https://jmc.osumiakari.jp/joint_all.json`
2. Verify the 6-digit code matches what you plan to use
3. For fire departments serving multiple municipalities (一部事務組合):
   - Each constituent municipality gets its own parser file
   - All parsers use the same source URL but filter for their specific municipality
   - Each uses its own correct JIS X 0402 code

**Example**: Osaka South Fire Union (大阪南消防組合) serves 8 municipalities:
- Kashiwara (柏原市): 272213
- Habikino (羽曳野市): 272230 (not 272221)
- Fujiidera (藤井寺市): 272264 (not 272248)
- Tondabayashi (富田林市): 272141 (not 272256)
- Kawachinagano (河内長野市): 272167 (not 272264)
- Taishi (太子町): 273813 (not 273635)
- Kanan (河南町): 273821 (not 273643)
- Chihayaakasaka (千早赤阪村): 273830 (not 273660)

## Adding New Municipalities

To add support for a new municipality, follow these steps in order:

1. **Verify municipal code**:
   - Check `https://jmc.osumiakari.jp/joint_all.json` for the correct 6-digit JIS X 0402 code
   - Never assume or guess the code - always verify from the authoritative source

2. **Create parser file**: `src/parse/parse_XXXXXX.rs` (where XXXXXX is the verified 6-digit municipal code)
   - Implement the `return_XXXXXX()` function that returns `Result<(), Box<dyn std::error::Error>>`
   - Follow existing parser patterns for consistency
   - Use `HttpRequestConfig` for HTTP requests
   - Output to `dist/XXXXXX.json` in the standard format

3. **Add module declaration**: In `src/parse/mod.rs`, add:
   ```rust
   pub mod parse_XXXXXX;
   ```

4. **Add import statement**: At the top of `src/lib.rs`, add:
   ```rust
   use crate::parse::parse_XXXXXX::return_XXXXXX;
   ```

5. **Add function call**: In the `get_all()` function in `src/lib.rs`, add:
   ```rust
   return_XXXXXX()?;
   ```

6. **Update README.md**: Add the municipality to the "対応市区町村" section with its name, fire department name, and verified 6-digit code

7. **Test the implementation**:
   - Run `cargo check` to verify compilation
   - Run `cargo run` to execute all parsers including the new one
   - Verify `dist/XXXXXX.json` is created with correct format
   - Check that the new municipality appears in `dist/list.json` and `dist/all.json`

## Output Files

- `dist/XXXXXX.json`: Individual municipality data files
- `dist/list.json`: Array of all supported municipal codes
- `dist/all.json`: Unified JSON file containing all municipalities with active disasters, using jisx0402 codes as keys (municipalities with no active disasters are excluded)
- `dist/all_feed.xml`: RSS 2.0 feed combining all emergency dispatches

## Dependencies

Key dependencies as defined in `Cargo.toml`:

- **reqwest** (0.12.15 with blocking and json features): HTTP client for fetching web pages
- **scraper** (0.23.1): HTML parsing and CSS selector support
- **serde_json** (1.0.140): JSON serialization/deserialization
- **chrono** (0.4.40): Date and time handling for RSS feed generation
- **regex** (1.11.1): Pattern matching for file operations
- **encoding_rs** (0.8.35): Character encoding handling (primarily for Shift_JIS support)

**Note**: The project uses Rust edition 2024 as specified in `Cargo.toml`.

## HTTP Configuration System

The codebase uses a centralized `HttpRequestConfig` struct for handling different website requirements:

- **Default headers**: Defined as constants in `lib.rs`:
  - `Accept`: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
  - `Accept-Language`: "ja,en-US;q=0.7,en;q=0.3"
  - `Connection`: "keep-alive"
  - `Content-Type`: "application/x-www-form-urlencoded"
  - `User-Agent`: Defined by `ACCESS_UA` constant
- **Character encoding**: Use `.with_shift_jis(true)` only when the target website specifically uses Shift_JIS encoding (check the HTML meta charset or test for garbled text)
- **Custom headers**: Override defaults using methods like `.with_accept()`, `.with_accept_language()`, `.with_connection()`, `.with_content_type()`

### Example Usage:
```rust
// Basic configuration
let config = HttpRequestConfig::new(HOST, GET_SOURCE);

// With Shift_JIS encoding
let config = HttpRequestConfig::new(HOST, GET_SOURCE)
    .with_shift_jis(true);

// With custom headers
let config = HttpRequestConfig::new(HOST, GET_SOURCE)
    .with_accept("custom/accept")
    .with_accept_language("ja-JP");

// Fetch content using the configuration
let body = get_source_with_config(&config)?;
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

## RSS Feed GUID Deduplication System

The RSS feed generation includes a sophisticated GUID deduplication system to prevent duplicate notifications in RSS readers, particularly for municipalities that use substitute timestamps (like 柏市 and 松江市).

### Problem Background

Some municipalities (notably 柏市 122173 and 松江市 322016) do not provide individual disaster occurrence times. Instead, they use the fire department's information update time for all disasters. This causes issues where:

1. **Same disaster, different timestamps**: When the fire department updates their information page, the same ongoing disaster gets a new GUID based on the update time
2. **RSS reader duplicate notifications**: RSS readers see different GUIDs and notify users of the "same" disaster multiple times
3. **GUID format**: Standard format is `YYYYMMDDHHMM-jisx0402`, but substitute timestamps make this unreliable for duplicate detection

### Solution: Blacklist-Based GUID Inheritance

The system implements a **blacklist approach** rather than a whitelist approach:

- **Core principle**: Same disaster essence gets the same GUID, regardless of timestamp changes
- **Disaster essence**: Combination of `address` + `disaster_type` (excluding time information)
- **GUID inheritance**: If the same disaster essence existed in the previous RSS feed, reuse its GUID
- **New disasters**: Generate new GUIDs using the existing mechanism

### Implementation Details

The system is implemented in `lib.rs` within the `generate_rss_feed()` function:

#### 1. Previous GUID Mapping Loading
```rust
// Load GUID mapping from previous RSS feed
let previous_guid_mapping = load_previous_guid_mapping();
```

The `load_previous_guid_mapping()` function parses the existing `dist/all_feed.xml` file to extract:
- Disaster descriptions (addresses)
- Disaster types
- Associated GUIDs

#### 2. GUID Determination Logic
```rust
// Create disaster essence (excluding time)
let disaster_essence = format!("{}-{}", address, disaster_type);

// Inherit GUID if same essence exists, otherwise generate new
let guid = if let Some(existing_guid) = previous_guid_mapping.get(&disaster_essence) {
    existing_guid.clone() // Same essence = same GUID
} else {
    // New disaster = generate new GUID using existing mechanism
    format!("{}{:02}{:02}{:02}{:02}-{}",
        disaster_date.year(), disaster_date.month(), disaster_date.day(),
        parsed_time.hour(), parsed_time.minute(), jisx0402)
};
```

### Key Benefits

1. **No new files created**: Uses existing `dist/all_feed.xml` for GUID tracking
2. **Existing GUID generation preserved**: New disasters still use the original timestamp-based GUID format
3. **RSS reader compatibility**: Leverages RSS readers' built-in GUID-based duplicate detection
4. **Automatic cleanup**: When disasters are resolved and removed from source websites, their GUIDs naturally disappear from the system

### Behavior for Different Scenarios

- **New disaster**: Gets a new timestamp-based GUID → RSS reader notifies user
- **Same disaster with time update**: Gets the same GUID as before → RSS reader filters as duplicate
- **Content change (address/type)**: Gets a new GUID → RSS reader notifies user of the change
- **Resolved disaster**: Removed from source → GUID mapping is not carried forward

### Technical Notes

- **Memory-only processing**: All GUID mapping is done in memory during RSS generation
- **No persistence files**: Does not create additional files beyond the standard output
- **Backward compatibility**: Maintains full compatibility with existing RSS readers and GUID formats
- **Minimal code changes**: Adds functionality without modifying existing GUID generation mechanisms

This system effectively solves the duplicate notification problem while maintaining the integrity of the existing emergency dispatch information collection and distribution system.