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

```json
{
    "disasters": [
        {
            "address": "例示県例示市なんちゃら1丁目", // 都道府県から始まる住所
            "time": "01:23", // 出動時刻（ソースとなる機関によって異なる場合がある）
            "type": "火災" // 出動種別
        },
        {
            "address": "例示県例示市大字ほにゃらら234",
            "time": "00:12",
            "type": "航空隊支援"
        } // このように複数の出動情報が存在する場合がある。何も無い場合は空配列が返される。
    ],
    "jisx0402": "999999", // 6桁の地方公共団体コード
    "source": [
        {
            "name": "例示市消防本部", // ソースとなる機関名
            "url": "https://example.com/index.html" //ソースページ
        }
    ]
}
```

あくまで統一されているのはフォーマットであって、**使用される語彙は統制されていない**。例えば`disasters.type`は、同様の事象であっても「火災」であったり「林野火災」であったり「大規模火災」であったりと、ソースとなる機関によって異なる表現がなされることがあり得る。データを活用される際は注意していただきたい。

## 対応市区町村
* 北海道
    * 札幌市（札幌市消防局）
* 青森県
    * つがる市（つがる市消防本部）
* 千葉県
    * 市川市（市川市消防局）
* 新潟県
    * 長岡市（長岡市消防本部）
* 奈良県
    * 生駒市（生駒市消防本部）

## 全量フィード
取得した際にフィード（RSS 2.0）を生成するようにしている。`dist/all_feed.xml`へ生成される。日付変換機構が不完全なため、使用する際は1日のずれが発生する場合があるが、留意して使用すること。そのうち直す。

## 作者
oageo（Osumi Akari）

* Website: https://www.osumiakari.jp/about/
* Fediverse: [@oageo@c.osumiakari.jp](https://c.osumiakari.jp/@oageo)
* Bluesky: [@osumiakari.jp](https://bsky.app/profile/osumiakari.jp)
