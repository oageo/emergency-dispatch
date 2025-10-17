# emergency_dispatch
![GitHub commit activity](https://img.shields.io/github/commit-activity/y/oageo/emergency-dispatch)
![GitHub License](https://img.shields.io/github/license/oageo/emergency-dispatch)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/oageo/emergency-dispatch)

全国の緊急車両出動情報を統一されたフォーマットで提供する

提供される緊急車両出動情報は**公式なものではなく、このデータを基に消防本部等へ連絡を行うことは避けられたい**。

## フォーマット
出動情報は`dist`ディレクトリ以下へ、基本的に市区町村単位で生成されるが、一部事務組合などの場合はこれに限らない。

市区町村等は6桁の[地方公共団体コード](https://www.soumu.go.jp/denshijiti/code.html)によって区別される。例えば札幌市の地方公共団体コードは`011002`であり、その出動情報は`dist/011002.json`に出力される。

各JSONファイルは、UTF-8形式である。

```yaml
{
    "disasters": [
        {
            "address": "例示県例示市なんちゃら1丁目", # 都道府県から始まる住所
            "time": "01:23", # 出動時刻（ソースとなる機関によって異なる場合がある）
            "type": "火災" # 出動種別
        },
        {
            "address": "例示県例示市大字ほにゃらら234",
            "time": "00:12",
            "type": "航空隊支援"
        } # このように複数の出動情報が存在する場合がある。何も無い場合は空配列が返される。
    ],
    "jisx0402": "999999", # 6桁の地方公共団体コード
    "source": [
        {
            "name": "例示市消防本部", # ソースとなる機関名
            "url": "https://example.com/index.html" # ソースページ
        }
    ]
}
```

あくまで統一されているのはフォーマットであって、**使用される語彙は統制されていない**。例えば`disasters.type`は、同様の事象であっても「火災」であったり「林野火災」であったり「大規模火災」であったりと、ソースとなる機関によって異なる表現がなされることがあり得る。データを活用される際は注意していただきたい。

## 対応市区町村
* 北海道
    * 札幌市（札幌市消防局） - 011002
    * 函館市（函館市消防本部） - 012025
    * 江別市（江別市消防本部） - 012173
    * 千歳市（千歳市消防本部） - 012246
    * 恵庭市（恵庭市消防本部） - 012319
    * 北広島市（北広島市消防本部） - 012343
* 青森県
    * つがる市（つがる市消防本部） - 022098
* 山形県
    * 天童市（天童市消防本部） - 062103
* 茨城県
    * 土浦市（土浦市消防本部） - 082031
    * 茨城町（茨城町消防本部） - 083020
* 千葉県
    * 市川市（市川市消防局） - 122033
    * 柏市（柏市消防局） - 122173
    * 市原市（市原市消防局） - 122190
    * 袖ケ浦市（袖ケ浦市消防本部） - 122297
* 神奈川県
    * 横浜市（横浜市消防局） - 141003
* 新潟県
    * 新潟市（新潟市消防局） - 151009
    * 長岡市（長岡市消防本部） - 152021
* 石川県
    * 小松市（小松市消防本部） - 172031
* 愛知県
    * 名古屋市（名古屋市消防局） - 231002
    * 春日井市（春日井市消防本部） - 232068
* 京都府
    * 京都市（京都市消防局） - 261009
* 兵庫県
    * 小野市（小野市消防本部） - 282189
* 奈良県
    * 生駒市（生駒市消防本部） - 292095
* 島根県
    * 松江市（松江市消防本部） - 322016
* 福岡県
    * 北九州市（北九州市消防局） - 401005
    * 福岡市（福岡市消防局） - 401307

このデータのうち、6桁の地方公共団体コードは、`dist/list.json`で配列として取得することが可能である。

## 全量フィード
ここの自治体の出動情報を取得した際に、全ての自治体をまとめたフィード（RSS 2.0）を生成するようにしている。`dist/all_feed.xml`へ生成される。日付変換機構が不完全なため、使用する際は1日のずれが発生する場合があるが、留意して使用すること。

フィード生成時と比較して10分以上未来の時間を指している場合は前日と扱うようにしている。この機能は、動作しているマシンがJSTを使用していることを前提としている。そのうちちゃんと直す。

### 全量JSON
フィードと同様に、全ての自治体の出動情報をまとめたJSONファイルを生成しており、`dist/all.json`へ生成される。これは地方公共団体コードをキーとし、各地方公共団体内は`6桁の地方公共団体コード.json`と内容に差が無いように配慮している。また、 **出動が無い地方公共団体は出力しないようにしている** ことに留意すること。

## 運用・開発用情報

### 開発環境
開発を行うためにはRust環境を整える必要がある。[Rustは公式サイトよりインストーラをダウンロードする](https://www.rust-lang.org/ja/tools/install)ことが可能である。

Rustの環境が整ったのであれば、適当な場所へとクローンを行い、`cargo run`を行うことによって開発・実行環境が整う。

```bash
git clone https://github.com/oageo/emergency-dispatch.git
cd emergency-dispatch
cargo run
```

### パーサー
取得先によって条件が異なるため、`src/parse`以下にある`parse_（6桁の数字）.rs`によってそれぞれパースが行われている。6桁の数字は当該の地方公共団体コードとなっている。

#### デフォルト値について
HTTPリクエスト時に使用されるヘッダー値は、以下のようにデフォルト値が設定されている。

* `Accept`: "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
* `Accept-Language`: "ja,en-US;q=0.7,en;q=0.3"
* `Connection`: "keep-alive"
* `Content-Type`: "application/x-www-form-urlencoded"
* `User-Agent`: ACCESS_UA定数で定義された値

これらのデフォルト値を変更する場合は、各パーサーファイル内で`HttpRequestConfig`の対応するメソッドを呼び出すことで上書き可能である。このデフォルト値そのものを変更する場合は、`lib.rs`内に定義されている

```rust
let config = HttpRequestConfig::new(HOST, GET_SOURCE)
    .with_accept("custom/accept")
    .with_accept_language("ja-JP")
    .with_connection("close")
    .with_content_type("application/json");
```

Shift_JISエンコーディング（内部的にはUTF-8に変換してから処理を行っています）が必要な場合は、`.with_shift_jis(true)`を追加する。

### 定期実行
Ubuntu環境において以下のような設定をすると定期的に実行できる。環境に合わせて適宜権限等の管理をする必要がある。

/opt/emergency-dispatch/run_edbot.sh
```
#!/bin/bash
export PATH="$HOME/.cargo/bin:$PATH"
cd /opt/emergency-dispatch
cargo run
```

/etc/systemd/system/edbot.service
```
[Unit]
Description=emergency-dispatchを実行

[Service]
ExecStart=/opt/emergency-dispatch/run_edbot.sh
```

/etc/systemd/system/edbot.timer
```
[Unit]
Description=15分ごとに実行

[Timer]
OnCalendar=*:0/15
Persistent=true

[Install]
WantedBy=timers.target
```

## 作者
oageo（Osumi Akari）

* Website: https://www.osumiakari.jp/about/
* Fediverse: [@oageo@c.osumiakari.jp](https://c.osumiakari.jp/@oageo)
* Bluesky: [@osumiakari.jp](https://bsky.app/profile/osumiakari.jp)
