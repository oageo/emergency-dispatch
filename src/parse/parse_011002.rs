use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "www.119.city.sapporo.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "http://www.119.city.sapporo.jp/saigai/sghp.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::HOST, HOST.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT, ACCEPT.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT_LANGUAGE, ACCEPT_LANGUAGE.parse().unwrap());
    headers.insert(reqwest::header::CONNECTION, CONNECTION.parse().unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, CONTENT_TYPE.parse().unwrap());
    headers.insert(reqwest::header::USER_AGENT, ACCESS_UA.parse()?);

    let client = Client::builder()
        .default_headers(headers.clone())
        .build()?;

    let res = client.get(GET_SOURCE)
        .headers(headers)
        .send()?;
    let body = res.text()?;
    Ok(body)
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