use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use encoding_rs::SHIFT_JIS; // Shift_JISエンコーディング用

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "nara119.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const GET_SOURCE: &str = "https://nara119.jp/fire/saigai/saigaipcIkoma.html";

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
    let body_bytes = res.bytes()?; // バイト列として取得
    let (body, _, _) = SHIFT_JIS.decode(&body_bytes); // Shift_JISからUTF-8に変換
    Ok(body.into_owned())
}

pub fn return_292095() -> Result<(), Box<dyn std::error::Error>> {
    println!("292095, 生駒市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body div#WRAPPER div#WRAPPERINNER ul").unwrap();
    let mut disaster_data = vec![];

    // 最初の<ul>要素のみを取得
    if let Some(ul_element) = document.select(&selector).next() {
        let li_selector = scraper::Selector::parse("li span").unwrap();

        // 各<li>要素を解析
        for element in ul_element.select(&li_selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if text.contains("現在、火災等の災害は発生していません") {
                break;
            } else if let Some((date_time, rest)) = text.split_once("頃、生駒市") {
                if let Some((address, disaster_type)) = rest.split_once("付近で、") {
                    let time = date_time.split_whitespace().nth(1).unwrap_or("").replace("時", ":").replace("分", "");
                    let disaster_type = if disaster_type.trim() == "その他警戒が発生" {
                        "その他警戒".to_string() // 「その他警戒が発生」の場合は「その他警戒」のみを出力
                    } else if disaster_type.contains("事案が発生") { // 「事案が発生」が含まれている場合は「事案が発生」を省く
                        disaster_type.trim().replace("事案が発生", "") 
                    } else {
                        disaster_type.trim().replace("が発生", "") // 「事案が発生」ではなく単に「が発生」の場合においては「が発生」を省く
                    };
                    let address = format!("奈良県生駒市{}", address.trim());

                    disaster_data.push(json!({
                        "type": disaster_type,
                        "address": address,
                        "time": time
                    }));
                }
            }
        }
    }

    let output = json!({
        "jisx0402": "292095",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "生駒市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/292095.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 292095.json （生駒市消防本部）");
    Ok(())
}