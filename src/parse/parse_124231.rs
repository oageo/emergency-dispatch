use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, to_half_width, HttpRequestConfig};

const HOST: &str = "chb1018.hs.plala.or.jp";
const GET_SOURCE: &str = "http://chb1018.hs.plala.or.jp/chiba119/Web/chosei/annai_list.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_124231() -> Result<(), Box<dyn std::error::Error>> {
    println!("124231, 長生村（長生郡市広域市町村圏組合消防本部）");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    let selector = scraper::Selector::parse("strong").unwrap();
    let mut disaster_data = vec![];

    for element in document.select(&selector) {
        let text = element.text().collect::<String>();
        let text = to_half_width(&text);

        if text.contains("情報") {
            if let Some((_type_part, rest)) = text.split_once("情報") {

                if let Some(date_location) = rest.split_once("頃、") {
                    let date_time_str = date_location.0.trim();
                    let location_info = date_location.1;

                    if location_info.contains("長生村") {
                        if let Some(time_start) = date_time_str.rfind("日") {
                            let time_part = &date_time_str[time_start + "日".len()..];
                            let time = time_part
                                .replace("時", ":")
                                .replace("分", "")
                                .trim()
                                .to_string();

                            if let Some((address_part, disaster_detail)) = location_info.split_once("付近で") {
                                let address = if address_part.contains("　") {
                                    address_part
                                        .split("　")
                                        .next()
                                        .map(|s| format!("千葉県{}", s.trim()))
                                        .unwrap_or_else(|| format!("千葉県{}", address_part.trim()))
                                } else {
                                    format!("千葉県{}", address_part.trim())
                                };

                                let disaster_type_detail = disaster_detail
                                    .split("が発生")
                                    .next()
                                    .unwrap_or("")
                                    .trim();

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
        "jisx0402": "124231",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "長生郡市広域市町村圏組合消防本部"
            }
        ],
        "disasters": disaster_data
    });

    let mut file = File::create("dist/124231.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 124231.json （長生村・長生郡市広域市町村圏組合消防本部）");
    Ok(())
}
