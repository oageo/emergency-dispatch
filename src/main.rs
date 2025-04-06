use emergency_dispatch::parse::parse_011002::return_011002;
use emergency_dispatch::parse::parse_osumiakarijp::return_999999;
use std::fs;

fn main() {
    println!("This software is emergency_dispatch. Developed by oageo.");
    println!("This software is released under the Apache 2.0 license and source code is available at https://github.com/oageo/emergency-dispatch.");
    // distディレクトリが存在しない場合は作成
    if let Err(e) = fs::create_dir_all("dist") {
        eprintln!("「dist」ディレクトリの作成に失敗しました: {}", e);
        return;
    }
    return_011002();
    // return_999999();
}
