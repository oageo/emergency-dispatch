use std::fs;
use std::io::Write;
use serde_json::Value;
use chrono::{Local, NaiveTime, NaiveDate, DateTime, Utc};
use regex::Regex;

pub mod parse;

pub const ACCESS_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:137.0) Gecko/20100101 Firefox/137.0 edbot v0.1.0(https://github.com/oageo/emergency-dispatch)";

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
use crate::parse::parse_022098::return_022098;
use crate::parse::parse_062103::return_062103;
use crate::parse::parse_122033::return_122033;
use crate::parse::parse_151009::return_151009;
use crate::parse::parse_152021::return_152021;
use crate::parse::parse_261009::return_261009;
use crate::parse::parse_292095::return_292095;
use crate::parse::parse_401307::return_401307;

pub fn get_all() -> Result<(), Box<dyn std::error::Error>> {
    return_011002()?; 
    return_022098()?;
    return_062103()?;
    return_122033()?;
    return_151009()?;
    return_152021()?;
    return_261009()?;
    return_292095()?;
    return_401307()?;
    Ok(())
}

/// RSSフィードを生成する関数
pub fn generate_rss_feed() -> Result<(), Box<dyn std::error::Error>> {
    let mut all_disasters = vec![];

    // distディレクトリ内の「6桁の数字.json」ファイルを取得
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

    // 現在の日時を取得
    let now = Local::now();

    // 各JSONファイルを読み込む
    for file in files {
        let data = fs::read_to_string(&file)?;
        let json: Value = serde_json::from_str(&data)?;

        if let (Some(source), Some(disasters)) = (json["source"].as_array(), json["disasters"].as_array()) {
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
                                disaster_date = disaster_date.pred(); // 1日前の日付に変更
                            }

                            let iso8601_time = format!("{}T{}", disaster_date, parsed_time);
                            all_disasters.push((
                                iso8601_time,
                                format!("{}（{}）", disaster_type, source_name), // タイトルに「disaster_type（source.name）」を表示
                                address.to_string(),
                                source_url.to_string(), // ソースURLを含める
                            ));
                        }
                    }
                }
            }
        }
    }

    // 時間順にソート
    all_disasters.sort_by_key(|(time, _, _, _)| {
        DateTime::parse_from_rfc3339(time).unwrap_or_else(|_| Utc::now().into())
    });

    // RSSフィードを生成
    let mut rss_feed = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    rss_feed.push_str(r#"<rss version="2.0"><channel>"#);
    rss_feed.push_str("<title>日本の緊急車両出動フィード（非公式） by oageo</title>");
    rss_feed.push_str("<link>https://github.com/oageo/emergency-dispatch</link>");
    rss_feed.push_str(&format!("<description>全国の緊急車両出動情報を統一されたフォーマットで提供する。フィード生成日時: {}</description>", now.to_string()));
    rss_feed.push_str(&format!("<lastBuildDate>{}</lastBuildDate>", now.to_rfc2822()));
    rss_feed.push_str("<generator>emergency-dispatch</generator>");
    rss_feed.push_str("<language>ja</language>");

    for (time, title, address, source_url) in all_disasters {
        rss_feed.push_str("<item>");
        rss_feed.push_str(&format!("<title>{}</title>", title));
        rss_feed.push_str(&format!("<description>{}</description>", address));
        rss_feed.push_str(&format!("<link>{}</link>", source_url)); // ソースURLを含める
        rss_feed.push_str(&format!("<pubDate>{}</pubDate>", time));
        rss_feed.push_str("</item>");
    }

    rss_feed.push_str("</channel></rss>");

    // RSSフィードをファイルに保存
    let mut file = fs::File::create("dist/all_feed.xml")?;
    file.write_all(rss_feed.as_bytes())?;

    println!("RSSフィードが生成されました: dist/all_feed.xml");
    Ok(())
}
