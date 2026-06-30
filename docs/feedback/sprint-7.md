# Sprint 7 フィードバック

## 評価結果: 合格

**評価対象:** Sprint 7 - ビューモード・プレビュー
**評価方法:** 静的コード解析

## スコア
| 基準 | スコア | 閾値 | 判定 |
|------|--------|------|------|
| 機能完全性 | 4/5 | 4/5 | PASS |
| 動作安定性 | 4/5 | 4/5 | PASS |
| UI/UX品質 | 4/5 | 3/5 | PASS |
| エラーハンドリング | 3/5 | 3/5 | PASS |
| 回帰なし | 5/5 | 5/5 | PASS |

## 受け入れ基準チェック
| # | 基準 | 結果 | 確認方法 | 備考 |
|---|------|------|----------|------|
| 1 | セグメントでリスト/グリッド切替 | PASS | app.rs:649-657 segment_button→view_mode | |
| 2 | Modernでタイル表示（大アイコン+名前、14文字超省略） | PASS | modern.rs:266-346、335 chars().count()>14→take(12)+… | |
| 3 | タイルクリックでプレビューに情報表示 | PARTIAL | modern.rs:348-350 preview_idx=Some(i) | 下記インデックスバグ |
| 4 | テキスト系(rs/toml/md/txt/json/yaml/yml/lock)先頭20行 | PASS | modern.rs:191-216, 399-406 read_preview(20) | |
| 5 | Openボタンで既定アプリ起動 | PASS | modern.rs:243-245 open::that | |
| 6 | フォルダタイルDClickで移動 | PASS | modern.rs:351-357 | |
| 7 | 未選択時「Select a file to preview」 | PASS | modern.rs:144-152 | |

## 問題点・推奨アクション
- **中（インデックス不整合バグ）**: グリッドは隠しファイルでフィルタした `entries`（modern.rs:285-288）のインデックス `i` を `preview_idx` に保存するが、プレビューパネルは `app.main_pane.entries.get(idx)`（modern.rs:154、フィルタ前）を参照する。`show_hidden=off` かつ隠しファイルが存在するフォルダでは、クリックしたタイルと別のファイルがプレビューされる。
  - 再現: 隠しファイルを含むフォルダを Modern ビューで開き、隠し項目より後ろのタイルをクリック → 別ファイルの情報・プレビューが表示される。
  - 推奨: クリック時にフィルタ後 entry の実体（PathBuf）を保持する、または `main_pane.entries` 上の実インデックスへ変換する。
- 軽微: Modern サイドバーの "Projects"=`C:\dev` / "Recent"=`C:\` はハードコード（仕様の既知制限）。

※ 隠しファイルのないフォルダでは正常動作し、基本受け入れ基準は満たすため合格。早期修正を推奨。
</content>
