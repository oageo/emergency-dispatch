use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "docs.google.com";
// 大崎地域広域行政事務組合消防本部の災害情報案内（Google Sites）は本文が空の埋め込みで、
// 埋め込み内のJavaScriptがGoogleスプレッドシートの公開TSVを取得して表示している。
// そのため直接こちらを取得する。
const GET_SOURCE: &str = "https://docs.google.com/spreadsheets/d/e/2PACX-1vSxHvvk7rYZup-oWgYtdzIhSOS39ZJvliQ5dRa0j9cSUx6ca9ks1XT9Qr9yNzX2mOkhV5kMZmhyMxKz/pub?gid=0&single=true&output=tsv";
// 出力のsourceには、TSVではなく利用者向けの表示ページを載せる
const PAGE_URL: &str = "https://sites.google.com/view/sguide-osakikoiki/";

pub fn return_044458() -> Result<(), Box<dyn std::error::Error>> {
    println!("044458, 加美町（大崎地域広域行政事務組合消防本部）");

    // docs.google.com から *.googleusercontent.com へリダイレクトされるため、Hostヘッダーを送らない
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).without_host_header();
    let body = get_source_with_config(&config)?;

    // 1行目は見出し行。これが無い場合はエラーページ等を取得したとみなす
    if !body.starts_with("事案番号") {
        return Err(format!("TSVの見出し行が見つかりません: {}", GET_SOURCE).into());
    }

    let mut disaster_data = vec![];

    // 列: 0 事案番号 / 1 災害種別 / 2 場所 / 3 覚知時刻 / 4 状況 / 5 (空) / 6 最終更新 / 7 出動車両 / 8 災害種別（変更後）
    for line in body.lines().skip(1) {
        let columns: Vec<&str> = line.split('\t').map(|c| c.trim().trim_matches('"')).collect();
        if columns.len() < 5 {
            continue;
        }

        // 終了・鎮火した事案はスキップ（表にはしばらく残るため）
        let status = columns[4];
        if status.contains("終了") || status.contains("鎮火") {
            continue;
        }

        // 例: "大崎市 古川 諏訪 ２丁目 地内" → "大崎市古川諏訪2丁目"
        let location = to_half_width(columns[2])
            .replace(" ", "")
            .replace("　", "")
            .replace("地内", "");

        // 加美町の事案のみ
        if !location.starts_with("加美町") {
            continue;
        }
        let address = format!("宮城県{}", location);

        // 災害種別が変更されている場合は変更後を使用
        let disaster_type = match columns.get(8) {
            Some(changed) if !changed.is_empty() => changed.to_string(),
            _ => columns[1].to_string(),
        };

        // 例: "2:18" → "02:18"
        let Some((hour, minute)) = columns[3].split_once(':') else {
            continue;
        };
        let (Ok(hour), Ok(minute)) = (hour.parse::<u32>(), minute.parse::<u32>()) else {
            continue;
        };
        let time = format!("{:02}:{:02}", hour, minute);

        if !address.is_empty() && !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "044458",
        "source": [
            {
                "url": PAGE_URL,
                "name": "大崎地域広域行政事務組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/044458.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 044458.json （加美町・大崎地域広域行政事務組合消防本部）");
    Ok(())
}
