use std::fs;
use std::io::Write;
use std::collections::HashMap;
use serde_json::Value;
use chrono::{Local, NaiveTime, DateTime, Utc, Datelike, Timelike};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use encoding_rs::SHIFT_JIS;

pub mod parse;

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

    let res = client.get(&config.url)
        .headers(headers)
        .send()?;

    if config.use_shift_jis {
        let body_bytes = res.bytes()?;
        let (body, _, _) = SHIFT_JIS.decode(&body_bytes);
        Ok(body.into_owned())
    } else {
        let body = res.text()?;
        Ok(body)
    }
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
use crate::parse::parse_062103::return_062103;
use crate::parse::parse_082031::return_082031;
use crate::parse::parse_083020::return_083020;
use crate::parse::parse_122033::return_122033;
use crate::parse::parse_122173::return_122173;
use crate::parse::parse_122190::return_122190;
use crate::parse::parse_122297::return_122297;
use crate::parse::parse_141003::return_141003;
use crate::parse::parse_151009::return_151009;
use crate::parse::parse_152021::return_152021;
use crate::parse::parse_172031::return_172031;
use crate::parse::parse_231002::return_231002;
use crate::parse::parse_232068::return_232068;
use crate::parse::parse_261009::return_261009;
use crate::parse::parse_282189::return_282189;
use crate::parse::parse_292095::return_292095;
use crate::parse::parse_322016::return_322016;
use crate::parse::parse_401005::return_401005;
use crate::parse::parse_401307::return_401307;

pub fn get_all() -> Result<(), Box<dyn std::error::Error>> {
    return_011002()?;
    return_012025()?;
    return_012131()?;
    return_012173()?;
    return_012246()?;
    return_012319()?;
    return_012343()?;
    return_022098()?;
    return_062103()?;
    return_082031()?;
    return_083020()?;
    return_122033()?;
    return_122173()?;
    return_122190()?;
    return_122297()?;
    return_141003()?;
    return_151009()?;
    return_152021()?;
    return_172031()?;
    return_231002()?;
    return_232068()?;
    return_261009()?;
    return_282189()?;
    return_292095()?;
    return_322016()?;
    return_401005()?;
    return_401307()?;
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
