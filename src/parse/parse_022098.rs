use serde_json::json;
use std::fs::File;
use std::io::Write;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "tsugaru-fd.jp";
const GET_SOURCE: &str = "http://tsugaru-fd.jp/saigai.html";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE).with_shift_jis(true);
    get_source_with_config(&config)
}

pub fn return_022098() -> Result<(), Box<dyn std::error::Error>> {
    println!("022098, つがる市消防本部");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);
    let selector = scraper::Selector::parse("html body center table tbody tr td table tbody tr td table tbody tr").unwrap();
    let mut disaster_data = vec![];

    // 各<tr>要素を解析
    for row in document.select(&selector) {
        let cells: Vec<String> = row
            .select(&scraper::Selector::parse("td").unwrap())
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        if cells.iter().any(|cell| cell.contains("現在発生中の事案はありません")) {
            disaster_data.clear(); // 配列を空にする
            break; // 処理を終了
        }
        else if cells.len() >= 5 {
            let time = cells[0].replace("/", "-").split_whitespace().nth(1).unwrap_or("").to_string();
            let disaster_type = cells[2].clone();
            let address = format!("青森県{}", cells[4].replace("　", "").trim());

            disaster_data.push(json!({
                "type": disaster_type,
                "address": address,
                "time": time
            }));
        }
    }

    let output = json!({
        "jisx0402": "022098",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "つがる市消防本部"
            }
        ],
        "disasters": disaster_data
    });

    // JSONファイルに書き出し
    let mut file = File::create("dist/022098.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 022098.json （つがる市消防本部）");
    Ok(())
}