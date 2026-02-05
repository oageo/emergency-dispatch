use std::fs;
use std::io::Write;
use std::collections::HashMap;
use std::sync::Mutex;
use serde_json::Value;
use chrono::{Local, NaiveTime, DateTime, Utc, Datelike, Timelike};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use encoding_rs::SHIFT_JIS;

pub mod parse;

lazy_static::lazy_static! {
    /// プロセス内HTTPレスポンスキャッシュ
    ///
    /// 同一プロセス内で同じURLへの複数回のリクエストを1回に削減します。
    /// プロセス終了時に自動的にクリアされます。
    static ref SOURCE_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub const ACCESS_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:141.0) Gecko/20100101 Firefox/141.0 edbot v0.1.1(https://github.com/oageo/emergency-dispatch)";

// HTTPリクエスト用のデフォルト値
const DEFAULT_ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const DEFAULT_ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const DEFAULT_CONNECTION: &str = "keep-alive";
const DEFAULT_CONTENT_TYPE: &str = "application/x-www-form-urlencoded";

#[derive(Debug, Clone)]
pub struct HttpRequestConfig {
    pub host: String,
    pub url: String,
    pub accept: Option<String>,
    pub accept_language: Option<String>,
    pub connection: Option<String>,
    pub content_type: Option<String>,
    pub use_shift_jis: bool,
}

impl HttpRequestConfig {
    pub fn new(host: &str, url: &str) -> Self {
        Self {
            host: host.to_string(),
            url: url.to_string(),
            accept: None,
            accept_language: None,
            connection: None,
            content_type: None,
            use_shift_jis: false,
        }
    }

    pub fn with_shift_jis(mut self, use_shift_jis: bool) -> Self {
        self.use_shift_jis = use_shift_jis;
        self
    }

    pub fn with_accept(mut self, accept: &str) -> Self {
        self.accept = Some(accept.to_string());
        self
    }

    pub fn with_accept_language(mut self, accept_language: &str) -> Self {
        self.accept_language = Some(accept_language.to_string());
        self
    }

    pub fn with_connection(mut self, connection: &str) -> Self {
        self.connection = Some(connection.to_string());
        self
    }

    pub fn with_content_type(mut self, content_type: &str) -> Self {
        self.content_type = Some(content_type.to_string());
        self
    }
}

pub fn get_source_with_config(config: &HttpRequestConfig) -> Result<String, Box<dyn std::error::Error>> {
    // キャッシュチェック
    {
        let cache = SOURCE_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&config.url) {
            println!("  [キャッシュ] {}", config.url);
            return Ok(cached.clone());
        }
    }

    // HTTPリクエスト処理
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::HOST, config.host.parse()?);
    headers.insert(
        reqwest::header::ACCEPT,
        config.accept.as_deref().unwrap_or(DEFAULT_ACCEPT).parse()?
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        config.accept_language.as_deref().unwrap_or(DEFAULT_ACCEPT_LANGUAGE).parse()?
    );
    headers.insert(
        reqwest::header::CONNECTION,
        config.connection.as_deref().unwrap_or(DEFAULT_CONNECTION).parse()?
    );
    headers.insert(
        reqwest::header::CONTENT_TYPE,
        config.content_type.as_deref().unwrap_or(DEFAULT_CONTENT_TYPE).parse()?
    );
    headers.insert(reqwest::header::USER_AGENT, ACCESS_UA.parse()?);

    let client = Client::builder()
        .default_headers(headers.clone())
        .build()?;

    println!("  [新規取得] {}", config.url);
    let res = match client.get(&config.url)
        .headers(headers)
        .send() {
            Ok(response) => response,
            Err(e) => {
                eprintln!("  [取得失敗] {}: {}", config.url, e);
                return Err(Box::new(e));
            }
        };

    let body = if config.use_shift_jis {
        let body_bytes = res.bytes()?;
        let (body, _, _) = SHIFT_JIS.decode(&body_bytes);
        body.into_owned()
    } else {
        res.text()?
    };

    // キャッシュに保存
    {
        let mut cache = SOURCE_CACHE.lock().unwrap();
        cache.insert(config.url.clone(), body.clone());
    }

    Ok(body)
}

pub fn to_half_width(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '０' => '0',
            '１' => '1',
            '２' => '2',
            '３' => '3',
            '４' => '4',
            '５' => '5',
            '６' => '6',
            '７' => '7',
            '８' => '8',
            '９' => '9',
            _ => c,
        })
        .collect()
}

use crate::parse::parse_011002::return_011002;
use crate::parse::parse_012025::return_012025;
use crate::parse::parse_012131::return_012131;
use crate::parse::parse_012173::return_012173;
use crate::parse::parse_012246::return_012246;
use crate::parse::parse_012319::return_012319;
use crate::parse::parse_012343::return_012343;
use crate::parse::parse_022098::return_022098;
use crate::parse::parse_062049::return_062049;
use crate::parse::parse_062103::return_062103;
use crate::parse::parse_064289::return_064289;
use crate::parse::parse_064611::return_064611;
use crate::parse::parse_072036::return_072036;
use crate::parse::parse_072117::return_072117;
use crate::parse::parse_075213::return_075213;
use crate::parse::parse_075221::return_075221;
use crate::parse::parse_082031::return_082031;
use crate::parse::parse_083020::return_083020;
use crate::parse::parse_092011::return_092011;
use crate::parse::parse_112089::return_112089;
use crate::parse::parse_112097::return_112097;
use crate::parse::parse_112127::return_112127;
use crate::parse::parse_112151::return_112151;
use crate::parse::parse_112259::return_112259;
use crate::parse::parse_112399::return_112399;
use crate::parse::parse_112411::return_112411;
use crate::parse::parse_112429::return_112429;
use crate::parse::parse_113263::return_113263;
use crate::parse::parse_113271::return_113271;
use crate::parse::parse_113417::return_113417;
use crate::parse::parse_113425::return_113425;
use crate::parse::parse_113433::return_113433;
use crate::parse::parse_113476::return_113476;
use crate::parse::parse_113484::return_113484;
use crate::parse::parse_113492::return_113492;
use crate::parse::parse_113697::return_113697;
use crate::parse::parse_121002::return_121002;
use crate::parse::parse_122025::return_122025;
use crate::parse::parse_122033::return_122033;
use crate::parse::parse_122050::return_122050;
use crate::parse::parse_122068::return_122068;
use crate::parse::parse_122106::return_122106;
use crate::parse::parse_122114::return_122114;
use crate::parse::parse_122122::return_122122;
use crate::parse::parse_122131::return_122131;
use crate::parse::parse_122157::return_122157;
use crate::parse::parse_122173::return_122173;
use crate::parse::parse_122181::return_122181;
use crate::parse::parse_122190::return_122190;
use crate::parse::parse_122238::return_122238;
use crate::parse::parse_122254::return_122254;
use crate::parse::parse_122262::return_122262;
use crate::parse::parse_122289::return_122289;
use crate::parse::parse_122297::return_122297;
use crate::parse::parse_122301::return_122301;
use crate::parse::parse_122319::return_122319;
use crate::parse::parse_122327::return_122327;
use crate::parse::parse_122335::return_122335;
use crate::parse::parse_122343::return_122343;
use crate::parse::parse_122351::return_122351;
use crate::parse::parse_122360::return_122360;
use crate::parse::parse_122378::return_122378;
use crate::parse::parse_122386::return_122386;
use crate::parse::parse_122394::return_122394;
use crate::parse::parse_123013::return_123013;
use crate::parse::parse_123293::return_123293;
use crate::parse::parse_123421::return_123421;
use crate::parse::parse_123471::return_123471;
use crate::parse::parse_123498::return_123498;
use crate::parse::parse_124036::return_124036;
use crate::parse::parse_124095::return_124095;
use crate::parse::parse_124109::return_124109;
use crate::parse::parse_124214::return_124214;
use crate::parse::parse_124222::return_124222;
use crate::parse::parse_124231::return_124231;
use crate::parse::parse_124249::return_124249;
use crate::parse::parse_124265::return_124265;
use crate::parse::parse_124273::return_124273;
use crate::parse::parse_124419::return_124419;
use crate::parse::parse_124435::return_124435;
use crate::parse::parse_124630::return_124630;
use crate::parse::parse_141003::return_141003;
use crate::parse::parse_141305::return_141305;
use crate::parse::parse_142018::return_142018;
use crate::parse::parse_142107::return_142107;
use crate::parse::parse_151009::return_151009;
use crate::parse::parse_152021::return_152021;
use crate::parse::parse_152137::return_152137;
use crate::parse::parse_153427::return_153427;
use crate::parse::parse_172031::return_172031;
use crate::parse::parse_231002::return_231002;
use crate::parse::parse_232068::return_232068;
use crate::parse::parse_261009::return_261009;
use crate::parse::parse_272141::return_272141;
use crate::parse::parse_272167::return_272167;
use crate::parse::parse_272213::return_272213;
use crate::parse::parse_272230::return_272230;
use crate::parse::parse_272264::return_272264;
use crate::parse::parse_272272::return_272272;
use crate::parse::parse_273813::return_273813;
use crate::parse::parse_273821::return_273821;
use crate::parse::parse_273830::return_273830;
use crate::parse::parse_282189::return_282189;
use crate::parse::parse_292010::return_292010;
use crate::parse::parse_292095::return_292095;
use crate::parse::parse_322016::return_322016;
use crate::parse::parse_342033::return_342033;
use crate::parse::parse_342122::return_342122;
use crate::parse::parse_344311::return_344311;
use crate::parse::parse_352047::return_352047;
use crate::parse::parse_355020::return_355020;
use crate::parse::parse_401005::return_401005;
use crate::parse::parse_401307::return_401307;
use crate::parse::parse_402231::return_402231;
use crate::parse::parse_403458::return_403458;
use crate::parse::parse_412015::return_412015;
use crate::parse::parse_412040::return_412040;
use crate::parse::parse_412082::return_412082;
use crate::parse::parse_412104::return_412104;
use crate::parse::parse_413275::return_413275;

pub fn get_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut error_count = 0;
    
    // マクロで各返却関数を呼び出し、エラーをハンドル
    macro_rules! call_parser {
        ($func:expr) => {
            if let Err(e) = $func {
                eprintln!("取得失敗: {}", e);
                error_count += 1;
            }
        };
    }

    call_parser!(return_011002());
    call_parser!(return_012025());
    call_parser!(return_012131());
    call_parser!(return_012173());
    call_parser!(return_012246());
    call_parser!(return_012319());
    call_parser!(return_012343());
    call_parser!(return_022098());
    call_parser!(return_062049());
    call_parser!(return_062103());
    call_parser!(return_064289());
    call_parser!(return_064611());
    call_parser!(return_072036());
    call_parser!(return_072117());
    call_parser!(return_075213());
    call_parser!(return_075221());
    call_parser!(return_082031());
    call_parser!(return_083020());
    call_parser!(return_092011());
    call_parser!(return_112089());
    call_parser!(return_112097());
    call_parser!(return_112127());
    call_parser!(return_112151());
    call_parser!(return_112259());
    call_parser!(return_112399());
    call_parser!(return_112411());
    call_parser!(return_112429());
    call_parser!(return_113263());
    call_parser!(return_113271());
    call_parser!(return_113417());
    call_parser!(return_113425());
    call_parser!(return_113433());
    call_parser!(return_113476());
    call_parser!(return_113484());
    call_parser!(return_113492());
    call_parser!(return_113697());
    call_parser!(return_121002());
    call_parser!(return_122025());
    call_parser!(return_122033());
    call_parser!(return_122050());
    call_parser!(return_122068());
    call_parser!(return_122106());
    call_parser!(return_122114());
    call_parser!(return_122122());
    call_parser!(return_122131());
    call_parser!(return_122157());
    call_parser!(return_122173());
    call_parser!(return_122181());
    call_parser!(return_122190());
    call_parser!(return_122238());
    call_parser!(return_122254());
    call_parser!(return_122262());
    call_parser!(return_122289());
    call_parser!(return_122297());
    call_parser!(return_122301());
    call_parser!(return_122319());
    call_parser!(return_122327());
    call_parser!(return_122335());
    call_parser!(return_122343());
    call_parser!(return_122351());
    call_parser!(return_122360());
    call_parser!(return_122378());
    call_parser!(return_122386());
    call_parser!(return_122394());
    call_parser!(return_123013());
    call_parser!(return_123293());
    call_parser!(return_123421());
    call_parser!(return_123471());
    call_parser!(return_123498());
    call_parser!(return_124036());
    call_parser!(return_124095());
    call_parser!(return_124109());
    call_parser!(return_124214());
    call_parser!(return_124222());
    call_parser!(return_124231());
    call_parser!(return_124249());
    call_parser!(return_124265());
    call_parser!(return_124273());
    call_parser!(return_124419());
    call_parser!(return_124435());
    call_parser!(return_124630());
    call_parser!(return_141003());
    call_parser!(return_141305());
    call_parser!(return_142018());
    call_parser!(return_142107());
    call_parser!(return_151009());
    call_parser!(return_152021());
    call_parser!(return_152137());
    call_parser!(return_153427());
    call_parser!(return_172031());
    call_parser!(return_231002());
    call_parser!(return_232068());
    call_parser!(return_261009());
    call_parser!(return_272141());
    call_parser!(return_272167());
    call_parser!(return_272213());
    call_parser!(return_272230());
    call_parser!(return_272264());
    call_parser!(return_272272());
    call_parser!(return_273813());
    call_parser!(return_273821());
    call_parser!(return_273830());
    call_parser!(return_282189());
    call_parser!(return_292010());
    call_parser!(return_292095());
    call_parser!(return_322016());
    call_parser!(return_342033());
    call_parser!(return_342122());
    call_parser!(return_344311());
    call_parser!(return_352047());
    call_parser!(return_355020());
    call_parser!(return_401005());
    call_parser!(return_401307());
    call_parser!(return_402231());
    call_parser!(return_403458());
    call_parser!(return_412015());
    call_parser!(return_412040());
    call_parser!(return_412082());
    call_parser!(return_412104());
    call_parser!(return_413275());

    // すべてのパーサーが失敗した場合はエラーを返す
    if error_count > 0 {
        eprintln!("\n合計 {} 件のパーサーが失敗しました", error_count);
        if error_count == 87 {
            // 87はすべてのパーサー数（error_count がこの値の場合、全て失敗）
            return Err("すべてのパーサーが失敗しました".into());
        }
    }
    
    Ok(())
}

// distディレクトリ内の「6桁の数字.json」ファイル名を取得し、Vecへ格納する関数
pub fn get_all_json() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let re = Regex::new(r"^\d{6}\.json$")?;
    let files = fs::read_dir("dist")?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name().into_string().ok()?;
            if re.is_match(&file_name) {
                Some(format!("dist/{}", file_name))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    Ok(files)
}

// 対応している地方公共団体コードの一覧を`list.json`に保存する関数
pub fn generate_list_json() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_all_json()?;
    let mut list = vec![];
    for file in files {
        let data = fs::read_to_string(&file)?;
        let json: Value = serde_json::from_str(&data)?;
        if let Some(jisx0402) = json["jisx0402"].as_str() {
            list.push(jisx0402.to_string());
        }
    }
    list.sort(); 
    let list_json_array = serde_json::to_string(&list)?;
    let mut file = fs::File::create("dist/list.json")?;
    file.write_all(list_json_array.as_bytes())?;
    println!("対応している地方公共団体コードの一覧が生成されました: dist/list.json");
    Ok(())
}

/// RSSフィードを生成する関数
pub fn generate_rss_feed() -> Result<(), Box<dyn std::error::Error>> {
    let mut all_disasters = vec![];
    let files = get_all_json().expect("RSSフィードの生成中に、JSONファイルの取得に失敗しました");

    // 前回のguidマッピングを読み込み
    let previous_guid_mapping = load_previous_guid_mapping();

    // 現在の日時を取得
    let now = Local::now();

    // 各JSONファイルを読み込む
    for file in files {
        let data = fs::read_to_string(&file)?;
        let json: Value = serde_json::from_str(&data)?;

        if let (Some(source), Some(disasters), Some(jisx0402)) = (
            json["source"].as_array(),
            json["disasters"].as_array(),
            json["jisx0402"].as_str()
        ) {
            if let (Some(source_name), Some(source_url)) = (
                source.get(0).and_then(|s| s["name"].as_str()),
                source.get(0).and_then(|s| s["url"].as_str()),
            ) {
                for disaster in disasters {
                    if let (Some(time_str), Some(disaster_type), Some(address)) = (
                        disaster["time"].as_str(),
                        disaster["type"].as_str(),
                        disaster["address"].as_str(),
                    ) {
                        // 時刻をNaiveTimeに変換
                        if let Ok(parsed_time) = NaiveTime::parse_from_str(time_str, "%H:%M") {
                            // 現在の日付を基準に日時を生成
                            let mut disaster_date = now.date_naive();
                            let disaster_datetime = disaster_date.and_time(parsed_time);

                            // 実行時刻をNaiveDateTimeに変換
                            let now_naive = now.naive_local();

                            // 実行時刻と比較して10分以上未来の場合、1日前の日付を設定
                            if disaster_datetime > now_naive + chrono::Duration::minutes(10) {
                                disaster_date = disaster_date.pred_opt().unwrap_or(disaster_date); // 1日前の日付に変更
                            }

                            // 災害の本質（時間以外の情報）をキーとする
                            let disaster_essence = format!("{}-{}", address, disaster_type);

                            // 同一本質の災害には同じguidを使用、新規なら新しいguidを生成
                            let guid = if let Some(existing_guid) = previous_guid_mapping.get(&disaster_essence) {
                                existing_guid.clone() // 同一本質なら前回guidを再利用
                            } else {
                                // 新規災害なら新しいguidを生成（既存機構）
                                format!("{}{:02}{:02}{:02}{:02}-{}",
                                    disaster_date.year(),
                                    disaster_date.month(),
                                    disaster_date.day(),
                                    parsed_time.hour(),
                                    parsed_time.minute(),
                                    jisx0402
                                )
                            };

                            let iso8601_time = format!("{}T{}", disaster_date, parsed_time);
                            all_disasters.push((
                                iso8601_time,
                                format!("{}（{}）", disaster_type, source_name), // タイトルに「disaster_type（source.name）」を表示
                                address.to_string(),
                                source_url.to_string(), // ソースURLを含める
                                jisx0402.to_string(),
                                guid,
                            ));
                        }
                    }
                }
            }
        }
    }

    // 時間順にソート
    all_disasters.sort_by_key(|(time, _, _, _, _, _)| {
        DateTime::parse_from_rfc3339(time).unwrap_or_else(|_| Utc::now().into())
    });

    // 重複するguidを解決（シーケンス番号を付与）
    use std::collections::HashMap;
    let mut guid_counts: HashMap<String, i32> = HashMap::new();

    for disaster in &mut all_disasters {
        let base_guid = disaster.5.clone(); // guidは6番目の要素
        let count = guid_counts.entry(base_guid.clone()).or_insert(0);
        *count += 1;

        if *count > 1 {
            disaster.5 = format!("{}-{:02}", base_guid, *count - 1);
        }
    }

    // RSSフィードを生成
    let mut rss_feed = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    rss_feed.push_str(r#"<rss version="2.0"><channel>"#);
    rss_feed.push_str("<title>日本の緊急車両出動フィード（非公式） by oageo</title>");
    rss_feed.push_str("<link>https://github.com/oageo/emergency-dispatch</link>");
    rss_feed.push_str(&format!("<description>全国の緊急車両出動情報を統一されたフォーマットで提供する。フィード生成日時: {}</description>", now.to_string()));
    rss_feed.push_str(&format!("<lastBuildDate>{}</lastBuildDate>", now.to_rfc2822()));
    rss_feed.push_str("<generator>emergency-dispatch</generator>");
    rss_feed.push_str("<language>ja</language>");

    for (time, title, address, source_url, _jisx0402, guid) in all_disasters {
        rss_feed.push_str("<item>");
        rss_feed.push_str(&format!("<title>{}</title>", title));
        rss_feed.push_str(&format!("<description>{}</description>", address));
        rss_feed.push_str(&format!("<link>{}</link>", source_url)); // ソースURLを含める
        rss_feed.push_str(&format!("<pubDate>{}</pubDate>", time));
        rss_feed.push_str(&format!("<guid isPermaLink=\"false\">{}</guid>", guid));
        rss_feed.push_str("</item>");
    }

    rss_feed.push_str("</channel></rss>");

    // RSSフィードをファイルに保存
    let mut file = fs::File::create("dist/all_feed.xml")?;
    file.write_all(rss_feed.as_bytes())?;

    println!("RSSフィードが生成されました: dist/all_feed.xml");
    Ok(())
}


/// 災害情報があるJSONファイルをjisx0402をキーとした統合JSONファイル(all.json)として生成する関数
pub fn generate_all_json() -> Result<(), Box<dyn std::error::Error>> {
    let files = get_all_json()?;
    let mut all_data = serde_json::Map::new();

    // 各JSONファイルを読み込む
    for file in files {
        let data = fs::read_to_string(&file)?;
        let json: Value = serde_json::from_str(&data)?;

        // 災害情報があるかチェック
        if let Some(disasters) = json["disasters"].as_array() {
            if !disasters.is_empty() {
                // jisx0402をキーとして使用
                if let Some(jisx0402) = json["jisx0402"].as_str() {
                    // jisx0402フィールドを除いた残りのデータを格納
                    let mut filtered_data = serde_json::Map::new();
                    if let Some(source) = json.get("source") {
                        filtered_data.insert("source".to_string(), source.clone());
                    }
                    filtered_data.insert("disasters".to_string(), Value::Array(disasters.clone()));
                    
                    all_data.insert(jisx0402.to_string(), Value::Object(filtered_data));
                }
            }
        }
    }

    // all.jsonファイルに保存
    let output = Value::Object(all_data);
    let mut file = fs::File::create("dist/all.json")?;
    file.write_all(serde_json::to_string_pretty(&output)?.as_bytes())?;

    println!("統合災害情報ファイルが生成されました: dist/all.json");
    Ok(())
}

/// 前回のRSSフィードから災害本質とguidのマッピングを読み込む関数
fn load_previous_guid_mapping() -> HashMap<String, String> {
    let mut guid_mapping = HashMap::new();

    if let Ok(rss_content) = fs::read_to_string("dist/all_feed.xml") {
        // 簡単なXML解析でdescription（住所）、title（災害種別）、guidを抽出
        let lines: Vec<&str> = rss_content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            if lines[i].trim().starts_with("<item>") {
                let mut title = "";
                let mut description = "";
                let mut guid = "";
                let mut j = i + 1;

                while j < lines.len() && !lines[j].trim().starts_with("</item>") {
                    let line = lines[j].trim();
                    if line.starts_with("<title>") {
                        title = line.trim_start_matches("<title>").trim_end_matches("</title>");
                    } else if line.starts_with("<description>") {
                        description = line.trim_start_matches("<description>").trim_end_matches("</description>");
                    } else if line.starts_with("<guid ") {
                        // <guid isPermaLink="false">GUID値</guid> から GUID値 を抽出
                        if let Some(start) = line.find('>') {
                            if let Some(end) = line.find("</guid>") {
                                guid = &line[start + 1..end];
                            }
                        }
                    }
                    j += 1;
                }

                if !title.is_empty() && !description.is_empty() && !guid.is_empty() {
                    // titleから災害種別を抽出（「災害種別（消防局名）」形式）
                    if let Some((disaster_type, _)) = title.split_once("（") {
                        // 災害の本質（時間以外の情報）
                        let disaster_essence = format!("{}-{}", description, disaster_type);
                        guid_mapping.insert(disaster_essence, guid.to_string());
                    }
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }

    guid_mapping
}

/// キャッシュをクリアする
pub fn clear_source_cache() {
    let mut cache = SOURCE_CACHE.lock().unwrap();
    let count = cache.len();
    cache.clear();
    println!("ソースキャッシュをクリアしました（{}エントリ）", count);
}
