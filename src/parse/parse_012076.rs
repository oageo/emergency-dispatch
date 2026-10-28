use serde_json::json;
use std::fs::File;
use std::io::Write;
use scraper::{Html, Selector};
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "fire-tokachi.info";
const GET_SOURCE: &str = "http://fire-tokachi.info/";

pub fn return_012076() -> Result<(), Box<dyn std::error::Error>> {
    println!("012076, 帯広市（とかち広域消防局）");

    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    let body = get_source_with_config(&config)?;
    let document = Html::parse_document(&body);

    // 各事案に発生時刻が無いため、柏市・松江市と同様にページの更新時刻を代わりに使用する
    // 例: "2026年10月28日 16時29分現在" → "16:29"
    let time_selector = Selector::parse("font.TIME").unwrap();
    let Some(time) = document.select(&time_selector).next().and_then(|element| {
        let text = to_half_width(&element.text().collect::<String>());
        let (_, time_part) = text.split_once('日')?;
        let (hour, minute) = time_part.trim().split_once('時')?;
        let minute = minute.split('分').next()?;
        Some(format!("{:02}:{:02}", hour.trim().parse::<u32>().ok()?, minute.trim().parse::<u32>().ok()?))
    }) else {
        return Err(format!("更新時刻が見つかりません: {}", GET_SOURCE).into());
    };

    let selector = Selector::parse("table.SGINFO td").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        // 全角数字を半角数字に変換し、住所などに含まれる空白を詰める
        let text: String = to_half_width(&element.text().collect::<String>())
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();

        // 災害がない場合
        if text.contains("災害は発生しておりません") {
            continue;
        }

        // 終了・鎮火した事案はスキップ（現在進行中の災害ではないため）
        // 例: "幕別町字軍岡【軍岡】で発生した確認出動は、10月30日06時44分に、活動を終了しました。"
        if text.contains("終了") || text.contains("鎮火") {
            continue;
        }

        // 出動の文以外（「●」の欄や、火災予防運動などのお知らせ）はスキップ
        if !text.contains("消防車が出動") {
            continue;
        }

        // 例: "帯広市西四条南2丁目で風害警戒による警戒事案のため消防車が出動しています。…"
        //     "清水町南三条6丁目で自火報通報が発生し、消防車が出動しております。…"
        let Some((location, rest)) = text.split_once('で') else {
            continue;
        };
        let disaster_type = rest
            .split("のため消防車が出動")
            .next()
            .unwrap_or("")
            .split("が発生し")
            .next()
            .unwrap_or("");

        // 帯広市の事案のみ
        if !location.starts_with("帯広市") {
            continue;
        }
        let address = format!("北海道{}", location);

        if !disaster_type.is_empty() {
            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "012076",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "とかち広域消防局"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/012076.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 012076.json （帯広市・とかち広域消防局）");
    Ok(())
}
