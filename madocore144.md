1.44MB以下ゲームコンテスト向けに、Rustで MadoCore 144 というミニゲームコアを作ってください。



目的:

\- Unity/Godot/Defold/GDevelopの代替ではなく、1.44MB以下の単独実行ファイルゲームを作るための最小骨格です。

\- Windows端末で動くことを優先してください。

\- 外部crateは禁止です。

\- 画像ファイル、音声ファイル、外部データファイルは禁止です。

\- すべてコード内に収めてください。

\- releaseビルド後のexeサイズを表示してください。

\- 目標サイズは 1,474,560 bytes 以下です。



作ってほしい構成:

\- Cargo.toml

\- src/main.rs

\- README.md

\- docs/madocore-144.md



MadoCore 144 v0.1 の機能:

1\. 画面バッファ描画

&#x20;  - 毎フレーム標準出力に描画

&#x20;  - 画面クリア

&#x20;  - 枠線

&#x20;  - グリッド描画



2\. 入力

&#x20;  - WASD

&#x20;  - 矢印キー

&#x20;  - Enter

&#x20;  - Esc

&#x20;  - R: reset

&#x20;  - H: help

&#x20;  - U: undo

&#x20;  - Q: quit



3\. Scene管理

&#x20;  - Title

&#x20;  - Help

&#x20;  - Game

&#x20;  - Clear

&#x20;  - GameOver



4\. グリッドゲーム用データ構造

&#x20;  - Tile

&#x20;  - Pos

&#x20;  - GameState

&#x20;  - Stage

&#x20;  - Direction



5\. ステージ読み込み

&#x20;  - コード内の文字列配列から読み込む

&#x20;  - 例:

&#x20;    #######

&#x20;    #P...G#

&#x20;    #..#..#

&#x20;    #######

&#x20;  - P = player

&#x20;  - G = goal

&#x20;  - # = wall

&#x20;  - . = floor



6\. undo

&#x20;  - GameStateの履歴をVecに保存

&#x20;  - Uで1手戻す



7\. reset

&#x20;  - 現在ステージを初期状態に戻す



8\. デモゲーム

&#x20;  - プレイヤーをゴールまで動かすだけの最小パズル

&#x20;  - 3ステージ内蔵

&#x20;  - 全ステージクリアで終了画面



9\. テスト

&#x20;  - ステージパース

&#x20;  - 壁に移動できない

&#x20;  - ゴール到達判定

&#x20;  - undo

&#x20;  - reset



10\. README

&#x20;  - 操作方法

&#x20;  - ビルド方法

&#x20;  - サイズ確認方法

&#x20;  - コンテスト制約

&#x20;  - 将来このコアから作れるゲーム例



注意:

\- 汎用化しすぎないでください。

\- 未使用の抽象化、trait、巨大な設計は避けてください。

\- まずは小さく、読みやすく、改造しやすい実装にしてください。

\- cargo fmt

\- cargo test

\- cargo build --release

まで実行してください。



MadoCore 144を v0.2 Pixel Layer に拡張してください。



目的:

1.44MB以下コンテスト向けに、端末表示だけでなく、8bit風の低解像度グラフィックと簡易サウンドを扱えるようにしたいです。



制約:

\- 最終的なrelease exe + 必要ファイル総量が 1,474,560 bytes 以下

\- 外部crateは原則禁止

\- 画像ファイル、音声ファイルに依存しない

\- タイル、スプライト、曲データはコード内データとして持つ

\- 未使用機能は削りやすい構造にする



実装したいもの:

\- 160x144または256x144の低解像度描画

\- 4色または16色パレット

\- 8x8タイル描画

\- 16x16スプライト描画

\- タイルマップ描画

\- 簡易アニメーション

\- 矩形波ベースの効果音

\- MML風の短いBGMシーケンサー

\- デモとして、プレイヤーがドット絵で歩く1画面パズル



重要:

\- まず空のPixel Layerを作った時点でexeサイズを測定

\- 次にタイル描画追加後のサイズを測定

\- 次にサウンド追加後のサイズを測定

\- サイズ増加をREADMEに記録



以下のskillsを必要なら取り込んでください。

https://github.com/0x0funky/agent-sprite-forge

https://github.com/Hugo-Dz/spritefusion-pixel-snapper

https://github.com/wuyoscar/GPT-Image2-Skill

https://github.com/tachikomared/TachiSnap

https://github.com/ace-step/ace-step-skills/tree/main

https://github.com/kodustech/awesome-agent-skills

https://mcpmarket.com/ja/tools/skills/pixel-art-professional

