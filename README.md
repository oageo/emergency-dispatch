# emergency_dispatch
![GitHub commit activity](https://img.shields.io/github/commit-activity/y/oageo/emergency-dispatch)
![GitHub License](https://img.shields.io/github/license/oageo/emergency-dispatch)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/oageo/emergency-dispatch)

全国の緊急車両出動情報を統一されたフォーマットで提供する

## フォーマット
出動情報は`dist`ディレクトリ以下へ、基本的に市区町村単位で生成される。

市区町村は6桁の地方公共団体コードによって区別される。例えば札幌市の地方公共団体コードは`011002`であり、その出動情報は`dist/011002.json`に出力される。

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

## 対応市区町村
* 北海道
    * 札幌市（札幌市消防局）
* 青森県
    * つがる市（つがる市消防本部）

## 作者
oageo（Osumi Akari）

* Website: https://www.osumiakari.jp/about/
* Fediverse: [@oageo@c.osumiakari.jp](https://c.osumiakari.jp/@oageo)
* Bluesky: [@osumiakari.jp](https://bsky.app/profile/osumiakari.jp)
