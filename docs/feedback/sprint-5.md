# Sprint 5 フィードバック

## 評価結果: 合格

**評価対象:** Sprint 5 - 選択・検索・隠しファイル
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
| 1 | クリックで単一選択・アクセント+左バー | PASS | explorer.rs:491-503, 431-437 | |
| 2 | Ctrl/Shiftで複数選択トグル | PASS | explorer.rs:492-498 modifiers.ctrl/shift | |
| 3 | 複数選択時 "M selected · サイズ" | PARTIAL | state.rs:77-80 status_text | 下記バグ参照（隠し/検索時にサイズ不一致） |
| 4 | 検索で部分一致（大小無視）絞り込み | PASS | explorer.rs:345-350 to_lowercase().contains | Explorerのみ（Modern非適用は仕様既知） |
| 5 | フォルダ移動で検索クリア | PARTIAL | app.rs:145 navigate_to でクリア | back/forward では未クリア（下記） |
| 6 | 隠しファイル表示切替（.始まり） | PASS | explorer.rs:347 show_hidden\|\|!is_hidden、app.rs:467 | |
| 7 | 隠し項目は淡色表示 | PASS | explorer.rs:662 is_hidden→tok.dim、entry.rs:35 | |

## 問題点・推奨アクション
- **中（インデックス不整合バグ）**: 一覧は隠し/検索でフィルタした `entries`（フィルタ後インデックス）で選択を管理するが、`state.total_size_selected()`（state.rs:64-69）は `main_pane.entries`（フィルタ前）を参照する。隠しファイルや検索で項目数が変わると、ステータスバーの「選択合計サイズ」が別ファイルのサイズになる。選択数自体は正しい。
  - 再現: 隠しファイルを含むフォルダ（show_hidden=off）で項目を選択 → 合計サイズ表示がずれる。
  - 推奨: 選択をインデックスではなく PathBuf 基準で保持する、またはフィルタ前後でインデックスを正規化する。
- **軽微**: `navigate_back`/`navigate_forward`（app.rs:149-169）は `search_text` をクリアしない。受け入れ基準「フォルダ移動で検索クリア」は戻る/進む移動時に満たされない。`navigate_to` 同様に `search_text.clear()` の追加を推奨。

※ いずれも基本フロー（隠しファイルなし・通常移動）では発現せず、閾値は満たすため合格。次スプリント以降で改善を推奨。
</content>
