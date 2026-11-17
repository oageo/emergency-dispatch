# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 重要: ユーザーからの指示
このプロジェクトは日本語が母語の日本人によって開発されています。可能な限り日本語で回答してください。 ただし、技術的な用語は無理に翻訳を行わずとも問題ありません。

## Project Overview

A Rust application that scrapes emergency dispatch information from Japanese fire department websites and outputs it in a unified JSON format (plus an RSS feed).

## Development Commands

- **Build and run**: `cargo run`
- **Build only**: `cargo build`
- **Run tests**: `cargo test`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Run clippy**: `cargo clippy`

## Architecture

### Core Components

- **`main.rs`**: creates `dist/`, then runs in order: `get_all()` (all parsers) → `generate_list_json()` → `generate_all_json()` → `generate_rss_feed()`.
- **`lib.rs`**: shared utilities and coordination:
  - `get_all()`: executes every parser through the `call_parser!` macro, which logs an error and continues instead of stopping the run
  - `generate_list_json()` / `generate_all_json()` / `generate_rss_feed()`: build the combined outputs from the `dist/XXXXXX.json` files
  - `HttpRequestConfig` / `get_source_with_config()`: HTTP fetching (see [HTTP Configuration](#http-configuration))
  - `to_half_width()`: converts full-width digits to half-width
  - `load_previous_guid_mapping()`: reads GUIDs from the previous RSS feed (see [RSS GUID Deduplication](#rss-guid-deduplication))
- **`build.rs`**: downloads Let's Encrypt intermediate certificates at build time (see [HTTP Configuration](#http-configuration)).

### Output Files

- `dist/XXXXXX.json`: one file per municipality
- `dist/list.json`: array of supported municipal codes
- `dist/all.json`: all municipalities with active disasters, keyed by jisx0402 (municipalities with none are excluded)
- `dist/all_feed.xml`: RSS 2.0 feed of all dispatches

`list.json`, `all.json` and the RSS feed are built from **whatever `dist/XXXXXX.json` files exist**, not from the parser list. A file left behind by a removed or renamed parser stays in `list.json` (and, if it had disasters, in `all.json` and the RSS feed) until it is deleted from `dist/` on the production server.

### Data Format

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

### Supported Municipalities

The complete list is the "対応市区町村" section of README.md, one entry per 6-digit JIS X 0402 code. To count implementations, use `find src/parse -name "parse_*.rs" | wc -l`. Small discrepancies against README are normal.

## Municipal Codes

**CRITICAL**: Always verify the 6-digit JIS X 0402 code before implementing a parser. Never assume or guess it.

- **Source**: `https://jmc.osumiakari.jp/joint_all.json` (human-readable: `https://jmc.osumiakari.jp/all/`). Includes basic municipalities (市区町村) and special entities (一部事務組合).
- **Same name in several prefectures**: e.g. 南部町, 日野町, 清水町, 池田町, 美里町 each exist in more than one prefecture. Pick the code whose first two digits match the prefecture (北海道 = 01, 鳥取県 = 31, …).
- **Check digit**: the 6th digit is `(11 - (d1×6 + d2×5 + d3×4 + d4×3 + d5×2) mod 11) mod 10`. A wrong check digit means the code is wrong; a correct one does not prove the code belongs to the intended municipality, so always confirm the name in the source.

Example — 大阪南消防組合 serves 8 municipalities: 柏原市 272213, 羽曳野市 272221, 藤井寺市 272264, 富田林市 272141, 河内長野市 272167, 太子町 273813, 河南町 273821, 千早赤阪村 273830.

## Parser Architecture

Each supported municipality has its own module in `src/parse/`:
- **File**: `parse_XXXXXX.rs` (XXXXXX = municipal code)
- **Function**: `return_XXXXXX() -> Result<(), Box<dyn std::error::Error>>`
- **Responsibility**: fetch the source, extract current dispatches, write `dist/XXXXXX.json`

### Parser shapes

Identify which shape the target site needs before writing code:

1. **Single page** (most parsers): fetch one URL, parse the current-dispatch section.
2. **Two-step**: the list page lacks address/time/type, so detail pages must be fetched. Narrow the list first so only needed details are fetched — by title (`parse_332135` 赤磐市 fetches only 「…発生連絡」 dated today/yesterday), by an "unread" marker (`img.new` on mailio.jp: `parse_092029` 足利市, `parse_112038` 川口市, `parse_362018` 徳島市, `parse_272043` 池田市), or by resolution keywords (`parse_092011` 宇都宮市, `parse_352047` 萩市). If the list already carries the needed text, read it there and fetch a detail page only when it is truncated (`parse_312037` 倉吉市).
3. **Multi-source** (`parse_092029` 足利市): one municipality's information is split across several URLs, all collected into one output. `generate_rss_feed()` uses only `source[0]` for the RSS title and link, so every `source` entry should carry the plain department name.

### Departments serving several municipalities

- **一部事務組合 / 広域連合**: one parser file per constituent municipality, all fetching the same URL (fetched once thanks to `SOURCE_CACHE`) and filtering by the start of the address. Match the full prefix the source uses — town addresses may include the 郡 (`西伯郡大山町…` in `parse_313866`). Watch for names that are prefixes of each other (士幌町 / 上士幌町). `source.name` is the union's department name. Examples: `parse_202029` 松本市 (松本広域消防局), `parse_042153` 大崎市, `parse_312029` 米子市, `parse_012076` 帯広市 (とかち広域消防局).
- **Joint dispatch center (共同指令センター)** publishing several independent departments: `source.name` (and the README entry) is each **original department's** name, not the center's. Examples: `parse_012173` 江別市消防本部 (札幌市消防局's site), `parse_242012` 津市消防本部 (三重中央消防指令センター's site).

## Adding New Municipalities

1. **Verify the municipal code** (see [Municipal Codes](#municipal-codes)).
2. **Create** `src/parse/parse_XXXXXX.rs`, following existing parsers and using `HttpRequestConfig`.
3. **Register** in three places, each kept **sorted by code**:
   - `src/parse/mod.rs`: `pub mod parse_XXXXXX;`
   - top of `src/lib.rs`: `use crate::parse::parse_XXXXXX::return_XXXXXX;`
   - `get_all()` in `src/lib.rs`: `call_parser!(return_XXXXXX());` (never `return_XXXXXX()?;`)
4. **Update README.md** "対応市区町村": `    * 市町村名（消防本部名） - XXXXXX` under its prefecture. Prefectures and entries are both in code order; add a new `* 都道府県` heading in its code position if needed.
5. **Test** (see [Testing Individual Parsers](#testing-individual-parsers)), then `cargo check` and `cargo clippy`.

When parallel branches add municipalities in the same area, re-check the code order after merging.

To **change a municipality's code** (e.g. it was wrong), rename the file and every occurrence (function name, `jisx0402`, output file name, `println!`, `mod.rs`, `lib.rs`, README), and delete the old `dist/XXXXXX.json` on the production server (see [Output Files](#output-files)).

## Parser Development Approach

1. **Compare with existing parsers**: many departments use the same vendor systems (e.g. the `table.SGINFO` pages read by `parse_012025` 函館市 and `parse_122173` 柏市; mailio.jp backnumbers; ikkr.jp).
2. **Confirm the data is actually in the HTML**: if the target container is empty, the page fills it with JavaScript. Read its scripts or embeds to find the real data URL and request that directly. Examples: 福島市 (`parse_072010`) requests the `/section/syoubou-info/history.html` that `list_e_1001-2.js` loads; 大崎市 (`parse_042153`) reads the published Google Sheets TSV that its Google Sites embed loads.
3. **Check the encoding**: start with UTF-8; add `.with_shift_jis(true)` / `.with_euc_jp(true)` only if the meta charset or garbled text says so.
4. **Collect real samples**, including closure messages and notices, not just the current page. Past list pages, monthly archives and the Internet Archive (`web.archive.org/cdx/search/cdx?url=…`) are useful; the Internet Archive throttles parallel requests, so fetch slowly.
5. **Decide how past information is excluded** (below).

### Excluding Past Information

**CRITICAL**: only currently-active dispatches belong in the output. Pick the mechanism that matches the source:

- **Separate current/past sections**: take only the current one. 生駒市 (`parse_292095`) breaks at 「現在、火災等の災害は発生していません」; 高知市/土佐市 (`parse_392014`, `parse_392057`) take only the first `div.panel-body table`.
- **Resolution keywords in a mixed list**: skip entries containing 鎮火 / 誤報 / 終了 / 完了 (`parse_092029`, `parse_112038`, `parse_231002`, `parse_242012`).
- **Closure messages as separate posts** (dispatch and closure are different articles): skip the closure posts — by title when the body is ambiguous (`parse_312037` 倉吉市, `parse_332135` 赤磐市) — and let the time window drop the dispatch. Do not try to pair closures with dispatches; the project accepts that a finished dispatch stays listed until the window expires.
- **Undifferentiated history feed**: keep only the last 24 hours (`parse_072010` 福島市, `parse_082031` 土浦市, `parse_231002` 名古屋市). mailio.jp's `img.new` marker disappears exactly 24 hours after posting, so it serves the same purpose. Where the source omits the year, infer it and treat a resulting future date as the previous year.

Prefer an explicit `continue` on a known "no disasters" sentence over relying on later parsing steps to fail.

### Time

Use each dispatch's own time when available. Some sources only show a page update time ("…現在"); then use it as a substitute, as 柏市 (`parse_122173`), 松江市 (`parse_322016`) and とかち (`parse_012076`) do. [RSS GUID Deduplication](#rss-guid-deduplication) keeps this from producing duplicate notifications.

### Common Japanese Text Patterns

- **Time**: `時` → `:`, `分` → `` (empty); convert full-width digits with `to_half_width()`
- **Address**: add the prefecture prefix; strip `地内`; remove spaces inside addresses
- **Section markers**: `◆`, `●`, `・`
- **Empty state**: phrases such as 「災害は発生しておりません」

## HTTP Configuration

All requests go through `HttpRequestConfig` and `get_source_with_config()`:

- **Default headers** (constants in `lib.rs`): `Accept`, `Accept-Language`, `Connection: keep-alive`, `Content-Type`, and `User-Agent` = `ACCESS_UA`, which identifies the bot and project. Override with `.with_accept()`, `.with_accept_language()`, `.with_connection()`, `.with_content_type()`.
- **Encoding**: `.with_shift_jis(true)` / `.with_euc_jp(true)` (e.g. `parse_172014` 金沢市 is EUC-JP).
- **Host header**: sent by default from the `host` argument. Call `.without_host_header()` when the URL redirects to another host (e.g. published Google Sheets: `docs.google.com` → `*.googleusercontent.com`); otherwise the fixed Host is sent to the redirect target and the resulting 404 page comes back as `Ok`. Example: `parse_042153`.
- **Missing intermediate certificates**: a server that omits its intermediate certificate may fail with `error sending request` depending on the environment (`openssl s_client -connect HOST:443 -showcerts` shows one certificate / verify code 21). For Let's Encrypt Gen Y certificates, call `.with_extra_root_certificates(LETS_ENCRYPT_GEN_Y_CERTS)`; `build.rs` downloads them at build time (no PEM is committed). Example: `parse_242012`.
- **In-process cache**: `SOURCE_CACHE` caches responses by URL within one run, so parsers sharing a URL trigger one request.

```rust
let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
let body = get_source_with_config(&config)?;
```

## Testing Individual Parsers

1. Temporarily make `get_all()` in `lib.rs` call only the parser(s) under test, then `cargo run`.
2. Check `dist/XXXXXX.json`. When nothing is currently active, verify extraction against saved samples (e.g. serve a test page locally and point the parser at it temporarily) or by temporarily widening the time window.
3. Restore `lib.rs` and the parser afterwards.

## UTF-8 String Processing Guidelines

**CRITICAL**: `find()` / `rfind()` return **byte** indices, and Japanese characters are 2–4 bytes. Slicing with them can split a character and panic (`is_char_boundary`).

```rust
// ❌ UNSAFE: byte indices from find() used for slicing
if let Some(start) = text.find(start_marker) {
    let after_start = &text[start..];              // can panic
}

// ✅ SAFE: split() never lands mid-character
let extracted = text
    .split(start_marker).nth(1)
    .and_then(|s| s.split(end_marker).next())
    .unwrap_or("");
```

1. Use `split()` / `split_once()` for segmentation — never `find()` + slicing, and no `replace_range()` with found indices
2. Use `replace()` for simple removal and `chars()` for character-based work
3. Handle every missing pattern with `unwrap_or()`, `?`, or `let ... else { continue; }` — sites change their wording
4. Verify against real fire department data

When an `is_char_boundary` panic occurs, find the slice in the stack trace and convert that step to `split()`.

## RSS GUID Deduplication

Sources that only give a page update time (柏市, 松江市, とかち) would otherwise give the same ongoing disaster a new GUID (`YYYYMMDDHHMM-jisx0402`) on every update, and RSS readers would notify again.

`generate_rss_feed()` therefore reads the previous `dist/all_feed.xml` via `load_previous_guid_mapping()` and keys each item by its **essence**, `address` + `disaster_type` (time excluded):

- Essence seen in the previous feed → reuse its GUID (no new notification)
- New essence, or a changed address/type → new timestamp-based GUID (notified)
- Disaster gone from the source → its mapping is not carried forward

The mapping is rebuilt in memory on each run; no extra files are created.
