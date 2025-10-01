use serde_json::json;
use std::fs::File;
use std::io::Write;
use crate::to_half_width;

use super::super::{get_source_with_config, HttpRequestConfig};

const HOST: &str = "cgi.city.yokohama.lg.jp";
const GET_SOURCE: &str = "https://cgi.city.yokohama.lg.jp/shobo/disaster/";

fn getsource() -> Result<String, Box<dyn std::error::Error>> {
    let config = HttpRequestConfig::new(HOST, GET_SOURCE);
    get_source_with_config(&config)
}

pub fn return_141003() -> Result<(), Box<dyn std::error::Error>> {
    println!("141003, 横浜市消防局");
    let body = getsource()?;
    let document = scraper::Html::parse_document(&body);

    // Get the entire body text and split by lines
    let body_text = document.root_element().text().collect::<String>();
    let mut disaster_data = vec![];

    // Process each line
    for line in body_text.lines() {
        let text = line.trim();

        // Only process rows containing time and minute markers
        if !text.contains("時") || !text.contains("分頃") {
            continue;
        }

        // Only process rows with disaster dispatch message
        if !text.contains("で発生した災害に、消防隊等が出場しています。") {
            continue;
        }

        // Convert full-width to half-width numbers
        let text = to_half_width(text);

        // Extract time (e.g., "17時47分頃" -> "17:47")
        let time = text
            .split("分頃")
            .next()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("")
            .replace("時", ":")
            .replace("分", "");

        // Validate time format
        if time.is_empty() || !time.contains(":") {
            continue;
        }

        // Extract address (e.g., "旭区今宿西町付近で発生した" -> "旭区今宿西町")
        let address = if let Some(addr_part) = text.split("分頃").nth(1) {
            addr_part
                .split("で発生した")
                .next()
                .unwrap_or("")
                .trim()
                .replace("付近", "")
                .trim()
                .to_string()
        } else {
            continue;
        };

        // Skip if address is empty
        if address.is_empty() {
            continue;
        }

        // Add prefecture and city prefix
        let full_address = format!("神奈川県横浜市{}", address);

        // Disaster type is fixed as "災害"
        let disaster_type = "災害".to_string();

        disaster_data.push(json!({
            "type": disaster_type,
            "address": full_address,
            "time": time
        }));
    }

    let output = json!({
        "jisx0402": "141003",
        "source": [
            {
                "url": GET_SOURCE,
                "name": "横浜市消防局"
            }
        ],
        "disasters": disaster_data
    });

    // Write to JSON file
    let mut file = File::create("dist/141003.json")?;
    file.write_all(output.to_string().as_bytes())?;
    eprintln!("{:?}", output);
    println!("JSONファイルが出力されました: 141003.json （横浜市消防局）");
    Ok(())
}
