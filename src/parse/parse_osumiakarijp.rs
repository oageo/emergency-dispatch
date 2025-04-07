use reqwest::blocking::Client;
use reqwest::header::{HeaderMap};

// `ACCESS_UA`をlib.rsから参照
use super::super::ACCESS_UA;

const HOST: &str = "www.osumiakari.jp";
const ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
// const ACCEPT_ENCODING: &str = "gzip, deflate, br, zstd";
const ACCEPT_LANGUAGE: &str = "ja,en-US;q=0.7,en;q=0.3";
const CONNECTION: &str = "keep-alive";
const CONTENT_TYPE: &str = "application/x-www-form-urlencoded";
const USER_AGENT: &str = ACCESS_UA;
const GET_SOURCE: &str = "https://www.osumiakari.jp/about/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    // ヘッダーを設定
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::HOST, HOST.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT, ACCEPT.parse().unwrap());
    // headers.insert(reqwest::header::ACCEPT_ENCODING, ACCEPT_ENCODING.parse().unwrap());
    headers.insert(reqwest::header::ACCEPT_LANGUAGE, ACCEPT_LANGUAGE.parse().unwrap());
    headers.insert(reqwest::header::CONNECTION, CONNECTION.parse().unwrap());
    headers.insert(reqwest::header::CONTENT_TYPE, CONTENT_TYPE.parse().unwrap());
    headers.insert(reqwest::header::USER_AGENT, ACCESS_UA.parse()?);

    let client = Client::builder()
        .default_headers(headers.clone())
        .build()?;

    // リクエストを送信
    let res = client.get(GET_SOURCE)
        .headers(headers)
        .send()?;
    eprintln!("アクセス先: {}", GET_SOURCE);
    eprintln!("ステータス: {}", res.status());
    eprintln!("受取ヘッダー: {:?}", res.headers());
    let body = res.text()?;
    Ok(body) // bodyを返す
}

// 結果を返却
pub fn return_999999() -> Result<(), Box<dyn std::error::Error>> {
    println!("999999, テスト用");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("h2").unwrap();
    for element in document.select(&selector) {
        println!("{:?}", element.text().collect::<String>());
    }
    Ok(())
}