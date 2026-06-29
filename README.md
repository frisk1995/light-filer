# harness

Claude Code のサブエージェント機能を使った自動開発パイプライン。短いプロダクトアイデアを入力するだけで、企画 → デザイン → 実装 → 検証のサイクルが自動で回り続ける。

## 概要

```
あなた（1〜4行のアイデア）
    ↓
Planner  ── /docs/spec.md を生成
    ↓
UIDesigner（提案モード） ── 2〜3パターンのデザイン案を生成・プレビュー
    ↓
あなた（デザイン案を選択・承認）
    ↓
UIDesigner（確定モード） ── /docs/design.md にデザイン仕様を記録
    ↓
Generator ── スプリント単位で実装 ── /docs/progress.md に記録
    ↓
Evaluator ── Playwright で実動作テスト ── /docs/feedback/sprint-N.md に結果出力
    ↓ 合格
次のスプリントへ（Generator に戻る）
    ↓ 不合格
Generator に差し戻し（修正後に再テスト）
```

## エージェント

| エージェント | 役割 | モデル |
|---|---|---|
| **Planner** | アイデアを製品仕様書（`/docs/spec.md`）に展開する | Opus |
| **UIDesigner** | AIDesigner MCP でデザイン案を生成し、ユーザー承認後に `/docs/design.md` を確定する | Opus |
| **Generator** | 仕様書・デザイン仕様を読み、1スプリントずつ実装する | Opus |
| **Evaluator** | Playwright MCP でアプリを実際に動かしてテストする | Opus |

## 使い方

### 1. 新しいプロジェクトを始める

Claude Code を起動し、Planner エージェントにアイデアを渡す：

```
planner エージェントを使って以下のアイデアを仕様書にしてください：
「シンプルなタスク管理アプリ。チームで共有できて、期限と優先度を設定できる」
```

### 2. デザイン案を生成・承認する

Planner が `/docs/spec.md` を生成したら、UIDesigner に提案を依頼する：

```
ui-designer エージェントでデザイン案を提案してください
```

UIDesigner が 2〜3パターンのデザインをプレビュー表示する。気に入った案を選んで確定する：

```
ui-designer エージェントで案Bを確定してください
（修正がある場合）案Aをベースに、カラーをダーク系に変更して確定してください
```

### 3. 実装・検証を繰り返す

デザインが確定したら Generator でスプリントを実装する：

```
generator エージェントでスプリント 1 を実装してください
```

Generator の実装が終わったら Evaluator でテストする：

```
evaluator エージェントでスプリント 1 を評価してください
```

合格後は Generator → Evaluator を繰り返す。

### 4. ファイル構成（自動生成）

```
docs/
  spec.md               # Planner が生成する製品仕様書
  design.md             # UIDesigner が確定するデザイン仕様書
  progress.md           # Generator が記録する実装進捗
  feedback/
    sprint-1.md         # Evaluator が出力する評価結果
    sprint-2.md
    ...
.aidesigner/
  proposals.json        # UIDesigner が提案モードで保存するデザイン案一覧
  runs/                 # AIDesigner が生成した HTML アーティファクト
```

## 評価基準

| 基準 | 合格閾値 |
|---|---|
| 機能完全性 | 4/5 以上 |
| 動作安定性 | 4/5 以上 |
| UI/UX品質 | 3/5 以上 |
| エラーハンドリング | 3/5 以上 |
| 回帰なし | 5/5（必須） |
| デザイン仕様適合 | 3/5 以上（`design.md` がない場合は評価対象外） |

1つでも閾値を下回ればスプリント不合格、Generator に自動差し戻し。

## 必要な MCP サーバ

- **AIDesigner MCP** — UIDesigner がデザイン生成に使用（`.mcp.json` に設定済み）
- **Playwright MCP** — Evaluator がブラウザ操作テストに使用（`.claude/agents/evaluator.md` に設定済み）

## 絶対ルール

- **Planner は実装しない。UIDesigner はコードを書かない。Generator は仕様・デザインを変更しない。Evaluator はコードを修正しない。**
- スプリントは Sprint 1 → 2 → 3 の順に実装する（スキップ禁止）。
- **ユーザーがデザインを承認してから Generator を起動する。**
- 各スプリント完了時にアプリが正常に起動・動作していること。
- Generator は新スプリント着手前に、前スプリントの不合格フィードバックを先に修正する。
