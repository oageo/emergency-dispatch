use emergency_dispatch::get_all;
use emergency_dispatch::generate_list_json;
use emergency_dispatch::generate_rss_feed;
use std::fs;

fn main() {
    println!("This software is emergency_dispatch. Developed by oageo.");
    println!("This software is released under the Apache 2.0 license and source code is available at https://github.com/oageo/emergency-dispatch.");
    // distディレクトリが存在しない場合は作成
    if let Err(e) = fs::create_dir_all("dist") {
        eprintln!("「dist」ディレクトリの作成に失敗しました: {}", e);
        return;
    }
    get_all().expect("データの取得に失敗しました");
    generate_list_json().expect("対応している地方公共団体コードの一覧の生成に失敗しました");
    generate_rss_feed().expect("RSSフィードの生成に失敗しました");
}
