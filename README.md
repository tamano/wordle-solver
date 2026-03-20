# wordle-solver

Wordleの攻略を支援するRust製CLIツールです。
ユーザーが入力した単語と、Wordleからのフィードバックをもとに候補単語を絞り込み、次の一手を提案します。

## 機能

- **候補絞り込み** — これまでのすべてのヒントを組み合わせて、矛盾しない単語のみを候補として保持する
- **次の一手の提案** — 候補内の文字出現頻度スコアにより、情報量が最大になる単語を自動で推薦する
- **重複文字の正確な処理** — グレー×グリーン/イエローの混在する重複文字（例: `erase` で `e` がグリーンとグレー）を正確に扱う
- **ラウンド管理** — 正解またはラウンド6まで繰り返し進行する

## ビルドと実行

[Rust](https://www.rust-lang.org/ja/tools/install) (1.75以上) が必要です。

```bash
# ビルド
cargo build --release

# 実行
cargo run --release

# テスト
cargo test
```

## 使い方

起動後、ラウンドごとに以下の2つを入力します。

| 入力 | 形式 | 例 |
|------|------|----|
| 推測した単語 | 5文字の英字 | `crane` |
| Wordleのフィードバック | 5文字 (G / Y / \_) | `G_Y__` |

### フィードバック文字

| 文字 | Wordleの色 | 意味 |
|------|-----------|------|
| `G`  | 🟩 緑     | 正しい文字・正しい位置 |
| `Y`  | 🟨 黄     | 正しい文字・違う位置   |
| `_`  | ⬜ グレー | その文字は答えに含まれない |

### 実行例

```
╔══════════════════════════════════╗
║        Wordle Solver CLI         ║
╚══════════════════════════════════╝

Dictionary loaded: 8636 five-letter words

Suggested first guess: "AROSE"

--- Round 1 ---
Candidates remaining: 8636

Enter your guess (5 letters): crane
Enter feedback (G/Y/_ for each letter): ____G

Suggested next guess: "STOMP"

--- Round 2 ---
Candidates remaining: 156

Enter your guess (5 letters): stomp
Enter feedback (G/Y/_ for each letter): GGG__

Suggested next guess: "STOVE"

--- Round 3 ---
Candidates remaining: 4
Candidates: STONE, STOVE, STOKE, STOLE

Enter your guess (5 letters): stove
Enter feedback (G/Y/_ for each letter): GGGGG

Congratulations! Solved in 3 guesses!
```

## アルゴリズム概要

1. **候補フィルタリング** (`filter_candidates`)
   - グリーン: 該当位置の文字が一致する単語のみ残す
   - イエロー: 該当文字を含むが、その位置には置かれていない単語のみ残す
   - グレー: 重複文字を正確に処理し、文字の出現回数を確定する
     - グリーン/イエローが0件かつグレー → 文字が存在しない
     - グリーン/イエローがN件かつグレーも存在 → 文字がちょうどN文字

2. **次の一手の推薦** (`suggest_next`)
   - 残候補における文字出現頻度を集計する
   - 候補が多い場合は全辞書から探索し、より効果的な絞り込みができる単語を推薦する
   - 候補が6語以下の場合は候補内から選択する

## 辞書データについて

### データソース

`src/data/words.txt` は **[dolph/dictionary](https://github.com/dolph/dictionary)** リポジトリの `enable1.txt` から、5文字の単語 8,636 語を抽出したものです。

`enable1.txt` は **ENABLE (Enhanced North American Benchmark LExicon)** ワードリストです。Scrabbleプレイヤー向けに作られた英単語リストで、8文字以上の単語を含む点でOSPD（公式スクラブル辞典）より網羅的です。

### ライセンス

ENABLEワードリストは **パブリックドメイン** です。作者より、商用・非商用を問わずあらゆる法的用途への使用が自由に認められています。帰属表示や許諾申請は不要です。

> "WORD.LST is hereby placed in the Public Domain for anyone to use as they see fit."
> — ENABLE word list authors

`dolph/dictionary` リポジトリ自体には明示的なライセンスファイルは存在しませんが、収録している `enable1.txt` の素材はパブリックドメインです。

### 参考リンク

- [dolph/dictionary — GitHub](https://github.com/dolph/dictionary)
- [ENABLE word list について (NPL Wiki)](http://wiki.puzzlers.org/dokuwiki/doku.php?id=solving:wordlists:about:enable_readme)

## ライセンス

このプロジェクトのソースコードは MIT ライセンスのもとで公開します。辞書データ (`src/data/words.txt`) はパブリックドメインです。
