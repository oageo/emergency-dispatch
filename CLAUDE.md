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

Parsers fall into three shapes. Identify which one the target site needs before writing code:

1. **Single page** (most parsers): fetch one URL, parse the current-dispatch section.
2. **Two-step / recursive** (`parse_092011` 宇都宮市, `parse_092029` 足利市, `parse_112038` 川口市, `parse_352047` 萩市): the list page only has links, so each linked detail page must be fetched to get address/time/type. Filter on the list page first (skip 終了/鎮火 links, or require an "unread" marker like `img.new`) so you only fetch what you need.
3. **Multi-source** (`parse_092029` 足利市): one municipality's information is split across several list URLs, all collected into one output. Note `generate_rss_feed()` uses only `source[0]` for the RSS title and link, so every entry in `source` should carry the plain department name — putting a category suffix in `name` leaks into RSS titles.

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

The complete list is maintained in the "対応市区町村" section of README.md, one entry per 6-digit JIS X 0402 code. To count implementations, use `find src/parse -name "parse_*.rs" | wc -l`. Small discrepancies against README are normal and not a cause for concern.

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
   call_parser!(return_XXXXXX());
   ```
   Note: `call_parser!` is a macro defined inside `get_all()` that catches errors without stopping the entire process. Do not use `return_XXXXXX()?;` directly.

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

See `Cargo.toml` for versions (Rust edition 2024). What each is for:

- **reqwest** (blocking, json): HTTP client for fetching web pages
- **scraper**: HTML parsing and CSS selector support
- **serde_json**: JSON serialization
- **chrono**: date/time handling for RSS generation and recency filtering
- **regex**: pattern matching for file operations
- **encoding_rs**: character encoding (Shift_JIS and EUC-JP decoding)
- **lazy_static**: used for `SOURCE_CACHE` (in-process HTTP response cache)

## HTTP Configuration System

The codebase uses a centralized `HttpRequestConfig` struct for handling different website requirements:

- **Default headers**: Defined as constants in `lib.rs`:
  - `Accept`: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
  - `Accept-Language`: "ja,en-US;q=0.7,en;q=0.3"
  - `Connection`: "keep-alive"
  - `Content-Type`: "application/x-www-form-urlencoded"
  - `User-Agent`: Defined by `ACCESS_UA` constant
- **Character encoding**: Default is UTF-8. Use `.with_shift_jis(true)` or `.with_euc_jp(true)` only when the target site uses that encoding (check the HTML meta charset, or test for garbled text). Example: `parse_172014` 金沢市消防局 is EUC-JP.
- **Custom headers**: Override defaults using methods like `.with_accept()`, `.with_accept_language()`, `.with_connection()`, `.with_content_type()`
- **In-process HTTP caching**: `SOURCE_CACHE` (a `lazy_static` `Mutex<HashMap>`) automatically caches responses by URL within a single process run. Multiple parsers sharing the same source URL (e.g., 松本広域消防局's 8 municipality parsers) will only trigger one actual HTTP request.

### Example Usage:
```rust
// Basic configuration
let config = HttpRequestConfig::new(HOST, GET_SOURCE);

// With Shift_JIS (or .with_euc_jp(true) for EUC-JP) encoding
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
2. **Confirm the data is actually in the HTML**: If the target container is empty in the fetched HTML, the page is populating it with JavaScript. Read the page's `<script src>` files to find the URL it fetches and target that directly. Example: 福島市 (`parse_072010`) renders an empty `div.dispatch_info ul.list`; `theme/base/js/list_e_1001-2.js` Ajax-fetches `/section/syoubou-info/history.html`, which is what the parser requests.
3. **Identify HTML structure**: Look for common Japanese patterns like `◆現在の出動`, `出動情報`, `災害情報`
4. **Test encoding**: Start with default (UTF-8), only add `.with_shift_jis(true)` / `.with_euc_jp(true)` if text appears garbled
5. **Decide how past information is excluded** (see below)

### Excluding Past Information

**CRITICAL**: Only currently-active dispatches belong in the output. Sites expose this differently, so pick the mechanism that matches the source:

- **Separate current/past sections**: Take only the current one. 生駒市 (`parse_292095`) breaks at 「現在、火災等の災害は発生していません」; 高知市/土佐市 (`parse_392014`, `parse_392057`) select only the first `div.panel-body table`, since the second is 「過去の災害経過情報」.
- **Resolution keywords in a mixed list**: Skip entries containing 鎮火 / 誤報 / 終了 (`parse_092029`, `parse_112038`, `parse_231002`).
- **Undifferentiated history feed**: When the page is just a rolling log with no current/past distinction, filter by timestamp — keep only the last 24 hours (`parse_072010` 福島市, `parse_082031` 土浦市, `parse_231002` 名古屋市). Where the source omits the year, infer it from the current date and treat a resulting future date as the previous year.

Explicit `continue` on a known "no disasters" sentence is preferred over relying on later parsing steps to fail, matching how other parsers read.

### Common Japanese Text Patterns

Some patterns found across multiple municipalities (analyze each case individually):
- **Time format conversion**: `時` → `:`, `分` → `` (empty)
- **Section markers**: `◆`, `●`, `・` often indicate different content sections
- **Address formatting**: Prefecture prefix is usually added by the parser; `地内` is commonly stripped
- **Empty state indicators**: Various phrases indicate no current dispatches

## Testing Individual Parsers

To test a specific municipality parser during development:
1. Temporarily modify `get_all()` in `lib.rs` to call only the desired parser function
2. Run `cargo run` to execute only that parser
3. Check the output in `dist/XXXXXX.json`

## UTF-8 String Processing Guidelines

**CRITICAL**: When working with Japanese text, always use UTF-8-safe string operations to avoid `is_char_boundary` panics.

`find()` and `rfind()` return **byte indices**, but Japanese characters are 2-4 bytes each. Slicing with those indices can split a character in half and panic at runtime.

```rust
// ❌ UNSAFE: byte indices from find() used for slicing
if let Some(start) = text.find(start_marker) {
    let after_start = &text[start..];              // can panic
    let extracted = &after_start[..after_start.find(end_marker)?];  // can panic
}

// ✅ SAFE: split() never lands mid-character
let extracted = text
    .split(start_marker).nth(1)
    .and_then(|s| s.split(end_marker).next())
    .unwrap_or("");
```

Guidelines:

1. **Use `split()` / `split_once()` for segmentation** — never combine `find()` with slicing
2. **Use `replace()` for simple removal**, not `replace_range()` with found indices
3. **Handle missing patterns gracefully** with `unwrap_or()`, `?`, or `let ... else { continue; }` — a source site can change its wording at any time
4. **Prefer character-based operations** (`chars()`) over byte-based ones
5. **Test with real fire department data**

Before committing parser code, verify:
- [ ] No `&text[index..]`, `&text[..index]`, or `&text[start..end]` derived from `find()`
- [ ] No `replace_range()` with `find()` indices
- [ ] Every extraction step handles the pattern being absent
- [ ] Verified against actual fire department output

When an `is_char_boundary` panic does occur, find the slice operation in the stack trace, identify which `find()`/`rfind()` produced the index, and convert that step to `split()`.

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

### Behavior for Different Scenarios

- **New disaster**: Gets a new timestamp-based GUID → RSS reader notifies user
- **Same disaster with time update**: Gets the same GUID as before → RSS reader filters as duplicate
- **Content change (address/type)**: Gets a new GUID → RSS reader notifies user of the change
- **Resolved disaster**: Removed from source → GUID mapping is not carried forward

The mapping is built in memory from the previous `dist/all_feed.xml` on each run, so no extra files are created and resolved disasters drop out of the system automatically.

This system effectively solves the duplicate notification problem while maintaining the integrity of the existing emergency dispatch information collection and distribution system.