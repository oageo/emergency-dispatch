// ビルド時に Let's Encrypt の中間証明書を取得し、OUT_DIR/lets_encrypt_gen_y.rs として書き出す。
//
// 中間証明書を送ってこないサーバー（例: 三重中央消防指令センター mcfcc119.jp）は、
// 実行環境によって証明書の検証に失敗する場合があるため、不足分をビルド時に取得して src/lib.rs に埋め込む。
// 取得に失敗してもビルドは止めず、取得できた分だけを書き出して警告を出す（次回のビルドで再取得を試みる）。
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

// Let's Encrypt の Gen Y（RSA）階層: ISRG Root X1 が署名した Root YR と、中間証明書 YR1〜YR3。
// 証明書の Authority Information Access に記載された配布元で、HTTP でのみ提供されている
const CERT_HOSTS: [&str; 4] = [
    "yr.i.lencr.org",
    "yr1.i.lencr.org",
    "yr2.i.lencr.org",
    "yr3.i.lencr.org",
];

fn fetch_der(host: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let address = (host, 80)
        .to_socket_addrs()?
        .next()
        .ok_or("名前解決に失敗しました")?;
    let mut stream = TcpStream::connect_timeout(&address, Duration::from_secs(20))?;
    stream.set_read_timeout(Some(Duration::from_secs(20)))?;

    // チャンク転送を避けるため HTTP/1.0 で要求する
    write!(stream, "GET / HTTP/1.0\r\nHost: {}\r\n\r\n", host)?;
    let mut response = Vec::new();
    stream.read_to_end(&mut response)?;

    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("HTTPヘッダーの終端が見つかりません")?;
    let status_line = String::from_utf8_lossy(&response[..header_end])
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    if status_line.split_whitespace().nth(1) != Some("200") {
        return Err(format!("想定外の応答: {}", status_line).into());
    }

    let body = response[header_end + 4..].to_vec();
    // DER形式の証明書は SEQUENCE（0x30）で始まる
    if body.first() != Some(&0x30) {
        return Err("DER形式の証明書ではありません".into());
    }
    Ok(body)
}

fn main() {
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("lets_encrypt_gen_y.rs");

    let mut certificates = vec![];
    let mut all_fetched = true;
    for host in CERT_HOSTS {
        match fetch_der(host) {
            Ok(der) => certificates.push(der),
            Err(e) => {
                println!("cargo:warning=証明書の取得に失敗しました（{}）: {}", host, e);
                all_fetched = false;
            }
        }
    }

    let mut source = String::from("pub const LETS_ENCRYPT_GEN_Y_CERTS: &[&[u8]] = &[\n");
    for der in &certificates {
        source.push_str(&format!("    &{:?},\n", der));
    }
    source.push_str("];\n");
    fs::write(&out_path, source).unwrap();

    // すべて取得できた場合は build.rs が変更されたときだけ再実行する（ビルドのたびに取得しない）。
    // 失敗した場合は指定しないことで、次回以降のビルドで再取得を試みる
    if all_fetched {
        println!("cargo:rerun-if-changed=build.rs");
    }
}
