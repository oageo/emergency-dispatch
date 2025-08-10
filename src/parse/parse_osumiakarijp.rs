use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "www.osumiakari.jp";
const GET_SOURCE: &str = "https://www.osumiakari.jp/about/index.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    let body = get_source_with_config(&config)?;
    eprintln!("アクセス先: {}", GET_SOURCE);
    Ok(body)
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