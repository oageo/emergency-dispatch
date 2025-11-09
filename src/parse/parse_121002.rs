use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, to_half_width, HttpRequestConfig};

const HOST: &str = "chb1018.hs.plala.or.jp";
const GET_SOURCE: &str = "http://chb1018.hs.plala.or.jp/chiba119/Web/chiba/annai_list.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_121002() -> Result<(), Box<dyn std::error::Error>> {
    println!("121002, 千葉市（千葉市消防局）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // <strong>タグ内のテキストを取得
    let selector = scraper::Selector::parse("strong").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>();
        // 全角数字を半角数字に変換
        let text = to_half_width(&text);

        // "情報" で災害種別を判定
        if text.contains("情報") {
            // フォーマット: "火災情報 2025年11月08日23時48分頃、千葉市中央区出洲港７番　サンクタス千葉シーサイドアベニュー付近で車両火災が発生しています。"
            if let Some((_type_part, rest)) = text.split_once("情報") {

                // 日時と場所情報を分離
                if let Some(date_location) = rest.split_once("頃、") {
                    let date_time_str = date_location.0.trim();
                    let location_info = date_location.1;

                    // 千葉市の情報のみ処理
                    if location_info.contains("千葉市") {
                        // 時刻を抽出（"2025年11月08日23時48分" -> "23:48"）
                        if let Some(time_start) = date_time_str.rfind("日") {
                            let time_part = &date_time_str[time_start + "日".len()..];
                            let time = time_part
                                .replace("時", ":")
                                .replace("分", "")
                                .trim()
                                .to_string();

                            // 場所と災害詳細を分離（"千葉市中央区出洲港７番　サンクタス千葉シーサイドアベニュー付近で車両火災が発生しています。"）
                            if let Some((address_part, disaster_detail)) = location_info.split_once("付近で") {
                                // 目標地点を除去（全角空白2つ（　）で区切られた後の部分）
                                // 目標がない場合もあるので、全角空白がない場合はそのまま使用
                                let address = if address_part.contains("　") {
                                    // 全角空白2つで分割して最初の部分のみ使用
                                    address_part
                                        .split("　")
                                        .next()
                                        .map(|s| format!("千葉県{}", s.trim()))
                                        .unwrap_or_else(|| format!("千葉県{}", address_part.trim()))
                                } else {
                                    format!("千葉県{}", address_part.trim())
                                };

                                // 災害種別を抽出（"車両火災が発生しています。" -> "車両火災"）
                                let disaster_type_detail = disaster_detail
                                    .split("が発生")
                                    .next()
                                    .unwrap_or("")
                                    .trim();

                                // 災害種別として使用
                                let full_disaster_type = disaster_type_detail.to_string();

                                disaster_data.push(json!({
                                    "type": full_disaster_type,
                                    "address": address,
                                    "time": time
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "121002",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "千葉市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/121002.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 121002.json （千葉市・千葉市消防局）");
    Ok(())
}
