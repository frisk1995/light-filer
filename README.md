# filox

## ダウンロード

**インストール不要**。exe をダウンロードしてそのまま実行できます。

[filox.exe をダウンロード (v0.4.2)](https://github.com/frisk1995/light-filer/releases/latest/download/filox.exe)

または [Releases ページ](https://github.com/frisk1995/light-filer/releases) から最新版を取得してください。

Rust + egui で実装した Windows 向け軽量ファイルマネージャー。

## 特徴

- **高速起動** — egui の即時描画モードにより低オーバーヘッドで動作
- **2 ビューモード** — リスト形式の Explorer とグリッド形式の Modern を切り替え可能
- **ダーク / ライトテーマ** — ツールバーのボタン1つで切り替え
- **Material Symbols アイコン** — ファイル種別に応じたアイコン表示
- **クイックアクセス** — よく使うフォルダをサイドバーに登録・永続保存
- **全ドライブ対応** — C: 以外のドライブも自動検出し、使用量バーを表示
- **隠しファイル表示切替** — ツールバーのアイコンで即時フィルタリング
- **右クリックメニュー** — 開く / パスをコピー / ターミナルで開く / 名前の変更 / 削除
- **単一 exe 配布** — 静的 CRT リンクで追加ランタイム不要

## 動作環境

- Windows 10 / 11 (x86_64)
- 日本語環境: Meiryo / Yu Gothic などの Windows システムフォントを自動使用

## ビルド方法

### 事前準備

Rust toolchain (stable) と Visual Studio Build Tools (MSVC) が必要です。

```powershell
# フォントをダウンロード（初回のみ）
powershell -ExecutionPolicy Bypass -File scripts/download_fonts.ps1
```

### ビルド

```powershell
# 開発ビルド
cargo build

# リリースビルド（最適化・コンソール非表示）
cargo build --release
```

ビルド成果物: `target/release/filox.exe`

## フォント

以下のフォントを `assets/fonts/` に配置します（`download_fonts.ps1` で自動取得）:

| ファイル | 用途 |
|---|---|
| `MaterialSymbolsRounded.ttf` | UI アイコン (Material Symbols) |
| `IBMPlexSans-Regular.ttf` | UI 本文フォント |
| `JetBrainsMono-Regular.ttf` | パス・モノスペース表示 |

日本語グリフは起動時に Windows システムフォント (`meiryo.ttc` 等) を自動ロードします。

## 設定ファイル

クイックアクセスの登録内容は再起動後も保持されます。

- 保存先: `%APPDATA%\filox\quick_access.txt`
- 形式: `表示名<TAB>パス` の1行1エントリ（手動編集可）

## 操作方法

| 操作 | 方法 |
|---|---|
| ディレクトリを開く | ダブルクリック |
| ファイルを開く | ダブルクリック（既定アプリで起動） |
| 複数選択 | Ctrl + クリック |
| コンテキストメニュー | 右クリック |
| 戻る / 進む | ツールバーの ‹ › ボタン |
| 上の階層へ | ツールバーの ↑ ボタン |
| 更新 | ツールバーの ↻ ボタン |
| 隠しファイル表示切替 | ツールバーの 👁 ボタン |
| クイックアクセスに追加 | ファイル一覧またはフォルダツリーで右クリック |

## 技術スタック

| 項目 | 詳細 |
|---|---|
| 言語 | Rust 2021 edition |
| GUI | [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) 0.29 / [egui](https://github.com/emilk/egui) 0.29 |
| スレッド間通信 | [crossbeam-channel](https://github.com/crossbeam-rs/crossbeam) |
| ファイルを開く | [open](https://github.com/Byron/open-rs) |
| 既知フォルダ取得 | [dirs](https://github.com/dirs-dev/dirs-rs) |
| リンク | 静的 CRT (`+crt-static`) |

## ライセンス

MIT
