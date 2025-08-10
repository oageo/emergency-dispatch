use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.119.city.sapporo.jp";
const GET_SOURCE: &str = "http://www.119.city.sapporo.jp/saigai/sghp.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_011002() -> Result<(), Box<dyn std::error::Error>> {
    println!("011002, 札幌市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body.format_free div#tmp_wrapper div#tmp_wrapper2 div#tmp_wrapper3 div#tmp_wrap_main.column_lnavi div#tmp_main div.wrap_col_main div.col_main div#tmp_contents").unwrap();
    let mut disaster_data = vec![];
    if let Some(element) = document.select(&selector).next() {
        let text = element.text().collect::<Vec<_>>().join(" ");
        if let Some(start) = text.find("現在の災害出動") {
            if let Some(end) = text[start..].find("出動中の災害は以上です") {
                let disaster_text = &text[start..start + end];
                for line in disaster_text.split('●').skip(1) {
                    let parts: Vec<&str> = line.split('・').collect();
                    if parts.len() > 1 {
                        let disaster_type = parts[0].trim().replace("出動", "");
                        let location_time = parts[1].trim();
                        if let Some((location, time)) = location_time.rsplit_once('（') {
                            let time = time.trim_end_matches('）').replace("時", ":").replace("分", "");
                            let address = format!("北海道札幌市{}", location.trim());
                            disaster_data.push(json!({
                                "type": disaster_type,
                                "address": address,
                                "time": time
                            }));
                        }
                    }
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "011002",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "札幌市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/011002.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 011002.json （札幌市消防局）");
    Ok(())
}