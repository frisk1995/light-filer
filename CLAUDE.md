# プロジェクトルール

このプロジェクトは6つのサブエージェント（Planner → UIDesigner → Generator → Evaluator → AppStoreValidator → Releaser）による自動開発パイプラインで構築される。
**ワークフロー開始時にアプリ種別（Web アプリ / スマホアプリ）を選択し、リリースフローを切り替える。**

## エージェント構成

**【共通】**
```
Planner ──→ UIDesigner ──→ Generator ──→ Evaluator
 (企画)       (デザイン)      (実装)        (検証)
                               ↑               │
                               └── 不合格時 ──┘
                                               │ 全スプリント合格
                                               ▼
                               ┌───────────────────────────┐
                               │  アプリ種別で分岐           │
                               └───────────────────────────┘
```

**【Web アプリ】**
```
                                           Releaser
                                    (Firebase デプロイ・公開)
```

**【スマホアプリ】**
```
                                       AppStoreValidator
                                       (申請パラメータ生成・審査前チェック)
                                               │ チェック通過
                                               ▼
                                           Releaser
                                          (ストア提出準備)
```

## ファイル規約

| パス | 用途 | 書き込み権限 |
|------|------|-------------|
| `/docs/spec.md` | 製品仕様書 | Planner のみ |
| `/docs/design.md` | UIデザイン仕様書（確定済み） | UIDesigner のみ（確定モード） |
| `.aidesigner/proposals.json` | デザイン提案一覧 | UIDesigner のみ（提案モード） |
| `/docs/progress.md` | 実装進捗・自己評価 | Generator のみ |
| `/docs/feedback/sprint-N.md` | スプリント評価結果 | Evaluator のみ |
| `/docs/app-type.md` | アプリ種別（web / mobile）の記録 | オーケストレーターのみ |
| `/docs/release/` | リリース準備成果物一式 | Releaser のみ |
| `docs/privacy-policy.html` | GitHub Pages 公開用プライバシーポリシー | Releaser のみ |
| `docs/release/firebase-url.md` | Firebase Hosting の公開 URL（Web アプリのみ） | Releaser のみ |
| `docs/terms-of-use.html` | GitHub Pages 公開用利用規約 | Releaser のみ |
| `docs/release/metadata/ios/ja/release_notes.txt` | バージョンリリースノート | Releaser のみ |
| `docs/release/submit/asc-app-info.txt` | ASC アプリ情報の申請パラメータ | AppStoreValidator のみ |
| `docs/release/submit/asc-metadata.txt` | ASC バージョンメタデータの申請パラメータ | AppStoreValidator のみ |
| `docs/release/submit/asc-iap-{product_id}.txt` | ASC IAP 商品ごとの申請パラメータ | AppStoreValidator のみ |
| `docs/release/submit/revenuecat-setup.txt` | RevenueCat 設定パラメータ | AppStoreValidator のみ |
| `docs/release/review-checklist.md` | Apple Review 提出前チェックリスト | AppStoreValidator のみ |

- **仕様書は Planner だけが書く。** UIDesigner・Generator・Evaluator・Releaser・AppStoreValidator は読み取り専用。
- **デザイン仕様書は UIDesigner だけが書く。** Generator・Evaluator・Releaser・AppStoreValidator は読み取り専用。
- **進捗は Generator だけが書く。** Evaluator・Releaser・AppStoreValidator は読み取り専用。
- **フィードバックは Evaluator だけが書く。** Generator・Releaser・AppStoreValidator は読み取り専用。
- **リリース成果物は Releaser だけが書く。** 他エージェントは読み取り専用。（ただし `docs/release/submit/` および `docs/release/review-checklist.md` は AppStoreValidator が書く）

## ワークフロー

### Step 0: アプリ種別選択・環境確認

**【アプリ種別選択】（最初に必ず実行）**
- ユーザーに「Web アプリ」か「スマホアプリ」かを確認する
- 選択結果を `/docs/app-type.md` に記録する（`web` または `mobile`）
- 以降のワークフロー（特に Step 3.5・Step 4・Step 4.5）はこの選択に従って切り替える

**【環境確認】**
- 動作に必要なコンポーネントが全て揃っているかを確認
- エージェント動作に必要なMCPサーバ全てConnectedしているかを確認
- **AIDesigner MCP が未接続の場合**: Step 1.5a はフォールバックフロー（claude.ai Design）に切り替える。他のステップへの影響はない。
- 不足している場合は報告する

### Step 1: 企画（Planner）
- ユーザーの短いプロンプトを受け取り `/docs/spec.md` を生成
- 技術的な実装詳細（DB設計、API設計、技術スタック）には踏み込まない
- 各スプリントに Evaluator がテスト可能な受け入れ基準を必ず記述する

### Step 1.5a: デザイン提案（UIDesigner 提案モード）

**【通常フロー】AIDesigner MCP が接続されている場合**
- `/docs/spec.md` を読み、UI/UX 要件とユーザーフローを把握する
- AIDesigner MCP で **2〜3パターンのデザイン案** を生成し、`.aidesigner/` に保存する
- 各案をプレビュー表示し、`.aidesigner/proposals.json` に案の一覧を記録する
- **この時点では `/docs/design.md` は書かない**

**【フォールバックフロー】AIDesigner MCP が未接続の場合**
- UIDesigner は `/docs/spec.md` からアプリの目的・ターゲット・主要画面を要約し、オーケストレーターに報告する
- オーケストレーターはユーザーに以下を案内する：
  1. [claude.ai/design](https://claude.ai) を開く（Design 機能）
  2. 要約した仕様を貼り付け、2〜3パターンのデザイン案を生成してもらう
  3. 気に入ったデザインのスクリーンショットまたは HTML を会話に貼り付けてもらう
- ユーザーが選択・フィードバックを返したら Step 1.5b へ進む（Step 1.5b の手順は共通）

### Step 1.5b: ユーザー承認（オーケストレーター）
- UIDesigner の提案サマリーをユーザーに提示する
- ユーザーに案の選択・修正フィードバックを求める
- 承認を得てから Step 1.5c に進む

### Step 1.5c: デザイン確定（UIDesigner 確定モード）
- ユーザーの選択・フィードバックを UIDesigner に渡す
- UIDesigner が必要に応じてデザインを修正し、`/docs/design.md` に最終仕様を書き込む
- Generator が迷わず実装できる粒度（色コード・数値込み）でデザイン仕様を記述する
- **フォールバックフロー時**: ユーザーが貼り付けたスクリーンショット・HTML を解析し、色・レイアウト・コンポーネント仕様を抽出して `/docs/design.md` に書き込む（AIDesigner MCP なしで完結）

### Step 2: 実装（Generator）
- `/docs/spec.md`・`/docs/design.md`・`/docs/progress.md` を読み、次のスプリントを特定
- **`/docs/design.md` が存在する場合は必ず参照し、デザイン仕様に従って実装する**
- **1回の呼び出しで1スプリントのみ実装する**
- 完了時に `/docs/progress.md` へ自己評価と引き渡し事項（起動方法、テストURL、テストシナリオ）を記録
- 前スプリントのフィードバック（`/docs/feedback/sprint-N.md`）があれば、修正を先に行う
- デザイン仕様と実装に乖離が生じた場合は `/docs/progress.md` にその理由を明記する

### Step 3: 検証（Evaluator）
- `/docs/spec.md` の受け入れ基準と `/docs/progress.md` の引き渡し事項を読む
- Playwright MCP でアプリを実際に操作してテスト
- 結果を `/docs/feedback/sprint-N.md` に出力
- 不合格の場合 → Generator に戻って修正（Step 2 へ）
- 合格の場合 → 次のスプリントへ（Step 2 へ）
- **全スプリント合格** → オーケストレーターに報告し Step 4 へ進む

### Step 3.5（IAP 事前準備確認 / オーケストレーター）【スマホアプリのみ】

**Web アプリの場合はこのステップをスキップし、直接 Step 4 へ進む。**
アプリ内課金（サブスクリプション）を実装する場合のみ実行する。
**このステップが完了してから AppStoreValidator → Releaser を呼び出すこと。**

- [ ] App Store Connect で **有料アプリ契約（Paid Apps Agreement）** に署名済みか
  - 未締結の場合、サンドボックスでも StoreKit がプロダクト情報を返さない
  - 契約締結後、反映まで数時間〜半日かかる場合がある
- [ ] ASC でサブスクリプション商品（各プラン）が「承認済み」になっているか
- [ ] RevenueCat に Products・Offerings・Entitlements が設定済みか
- [ ] 税務情報（W-8BEN 等）が提出済みか
- [ ] サンドボックステストで購入フローが正常に動作することを確認済みか

未完了項目があればユーザーに対応を促し、完了後に次へ進む。

### Step 4: リリース準備（Releaser）
- 全スプリントが合格してから呼び出す
- `/docs/app-type.md` を読み、アプリ種別（web / mobile）を確認する
- `/docs/spec.md` と `package.json` / `app.json` を読み、アプリ情報を把握する

#### 【Web アプリの場合】以下の3タスクを順番に実行する：
  1. **Firebase プロジェクトデプロイ**:
     - `firebase.json` / `.firebaserc` を確認・生成し Firebase Hosting を設定する
     - `firebase deploy` でアプリを公開する
     - 公開 URL を `/docs/release/firebase-url.md` に記録する
  2. **プライバシーポリシー公開**: `docs/privacy-policy.html` を生成し GitHub Pages または Firebase Hosting で公開、URL を `/docs/release/privacy-policy-url.md` に記録
  3. **利用規約公開**: `docs/terms-of-use.html` を生成し公開、URL を `/docs/release/terms-of-use-url.md` に記録
- 完了後、公開 URL をユーザーに案内する（Step 4.5 はスキップ）

#### 【スマホアプリの場合】以下の6タスクを順番に実行する：
  1. **EAS 証明書自動管理**: `eas.json` / `app.json` を設定し `/docs/release/certificates.md` に記録
  2. **プライバシーポリシー公開**: `docs/privacy-policy.html` を生成し GitHub Pages で公開、URL を `/docs/release/privacy-policy-url.md` に記録
  3. **利用規約公開**: `docs/terms-of-use.html` を生成し GitHub Pages で公開、URL を `/docs/release/terms-of-use-url.md` に記録
     - サブスクリプションを含む場合は必ず以下を記載すること（Apple Guideline 3.1.2(c)）：
       サブスクリプション名・期間・価格、自動更新の説明、キャンセル手順、Apple 標準 EULA へのリンク
  4. **ストアメタデータ作成**: iOS・Android 向けの説明文・キーワード・プロモーションテキスト・リリースノートを `/docs/release/metadata/` に生成
     - 説明文末尾に利用規約URL・プライバシーポリシーURLを必ず含めること
     - サブスクリプション情報ブロック（価格・期間・更新ルール・キャンセル手順）を説明文に含めること
  5. **スクリーンショット準備**: 必要サイズ一覧と撮影・リサイズ手順を `/docs/release/screenshots/` に作成
     - 提出サイズ: `1242×2688`（6.5"）または `1320×2868`（6.9"）
     - リサイズコマンド: `sips -z 2688 1242 input.png --out output.png`
     - IAP 用プロモーション画像は価格表記を含めないこと・テキストを十分大きくすること（Guideline 2.3.2）
  6. **申請パラメータ・リリースノート生成**: AppStoreValidator に引き渡すため、`/docs/release/submit/` ディレクトリを作成し `asc-metadata.txt` を生成しておく
- 完了後、残りの手動作業（実際の撮影・申請）をユーザーに案内する

### Step 4.5: 申請パラメータ生成・審査前チェック（AppStoreValidator）【スマホアプリのみ】

**Web アプリの場合はこのステップ全体をスキップする。**
全スプリント合格・Releaser 完了後に呼び出す。
`/docs/spec.md`・`app.json`・`/docs/release/` 以下を読み込み、以下の2タスクを実行する。

#### タスク 1: 申請パラメータファイルの生成

`/docs/release/submit/` に以下のファイルを出力する。
各ファイルは「ASC / RevenueCat の入力欄にそのまま貼り付けられる」形式にすること。

**`asc-app-info.txt`** — アプリ名・サブタイトル・カテゴリ・年齢制限・著作権・URL 類

**`asc-metadata.txt`** — バージョン番号・プロモーションテキスト・説明文・キーワード・リリースノート・プライバシーポリシーURL

**`asc-iap-{product_id}.txt`**（IAP を含む場合・商品ごとに生成）— 参照名・製品ID・サブスクリプショングループ・期間・価格・ローカリゼーション（表示名30文字以内、説明45文字以内）・審査用メモ

**`revenuecat-setup.txt`** — Bundle ID・Public API Key・Products 一覧・Entitlements 設定・Offerings 構成・App Store Connect API キー登録手順

#### タスク 2: Apple Review 提出前チェックリスト

`/docs/release/review-checklist.md` を生成し確認する。
未チェック項目があればユーザーに対応を促す。全項目通過後にオーケストレーターへ「提出準備完了」を報告する。

チェック項目:
```
【メタデータ】
- 説明文に利用規約URLが含まれているか
- 「完全オフライン」「無料」など虚偽の表現がないか
- 罫線文字（U+2500系）を使っていないか（em dash — を使う）
- プロモーション画像に価格表記がないか
- プロモーション画像のテキストが十分に大きいか（画像幅の80%以上）
- スクリーンショットが規定サイズになっているか

【IAP / サブスクリプション（該当する場合）】
- 有料アプリ契約が「有効」ステータスになっているか
- ASC でサブスクリプション商品が「承認済み」になっているか
- RevenueCat の Default Offering に全プランが含まれているか
- サンドボックステストで購入フローが正常に動作することを確認済みか
- 購入エラー時に適切なエラーメッセージが表示されるか
- ペイウォールに購入中のローディング表示があるか

【iPad 対応（supportsTablet: true の場合）】
- iPad 実機またはシミュレーターで主要画面の表示を確認済みか
- iPad での IAP 購入ボタンがサンドボックスで正常に動作することを確認済みか

【ビルド】
- EAS ビルド残枠を確認済みか（eas build:list で確認）
- eas build --profile production でビルドできたか
- eas submit でビルドが ASC にアップロードされたか
- ASC でビルドの「輸出コンプライアンス」に回答済みか
```

## 評価基準と閾値

| 基準 | 閾値 | 不合格時の扱い |
|------|------|---------------|
| 機能完全性 | 4/5 以上 | Generator に差し戻し |
| 動作安定性 | 4/5 以上 | Generator に差し戻し |
| UI/UX品質 | 3/5 以上 | Generator に差し戻し |
| エラーハンドリング | 3/5 以上 | Generator に差し戻し |
| 回帰なし | 5/5 必須 | Generator に差し戻し |
| デザイン仕様適合 | 3/5 以上 | Generator に差し戻し（`/docs/design.md` がない場合は評価対象外） |

**1つでも閾値を下回ればスプリント不合格。**

## 絶対ルール

1. **責務を越境しない** - Planner は実装しない。UIDesigner はコードを書かない。Generator は仕様・デザインを変更しない。Evaluator はコードを修正しない。Releaser はコード実装・仕様変更を行わない。
2. **スプリント順序を守る** - Sprint 1 → 2 → 3 と順番に実装する。スキップ禁止。
3. **動作する状態を維持する** - 各スプリント完了時にアプリが正常に起動・動作すること。
4. **フィードバックを最優先で処理する** - Generator は新スプリント着手前に、前スプリントの不合格フィードバックを修正すること。
5. **起動手順を必ず記載する** - Generator は `/docs/progress.md` にアプリの起動コマンドを毎回明記する。Evaluator はそれに従って起動する。
6. **デザイン承認なしで Generator を起動しない** - UIDesigner 確定モードが完了し `/docs/design.md` が存在するか、またはユーザーがデザインスキップを明示した場合にのみ Generator を呼び出す。
7. **全スプリント合格前に Releaser を起動しない** - Evaluator が全スプリントの合格を報告した後にのみ Releaser を呼び出す。
8. **アプリ種別を最初に確定する** - Step 0 でユーザーに Web / スマホを確認し `/docs/app-type.md` に記録する。種別未確定のまま Planner 以降を進めない。
9. **IAP を含むアプリは Step 3.5 を必ず通過してから AppStoreValidator を呼び出す** — 有料アプリ契約や RevenueCat 未設定のままビルドしても審査でリジェクトされる。
10. **利用規約は必ず公開 URL で提供する** — 説明文内にインラインで書くのではなく、GitHub Pages 等でホストした URL を ASC の「EULA」欄と説明文末尾の両方に記載する（Guideline 3.1.2(c)）。
11. **EAS 無料プランのビルド上限に注意する** — 無料プランは月ごとにリセット。Releaser は本番ビルド前に `eas build:list` で残り枠を確認し、上限到達時はローカル Xcode ビルドをユーザーに案内する。

## 知見ナレッジベース

iOS App Store リリース経験から得た既知の問題と解決策。Generator・Releaser・AppStoreValidator はこのセクションを参照して作業すること。

### EAS / Expo

| 事象 | 原因 | 解決策 |
|------|------|--------|
| `react-native-purchases` を plugins に追加すると PluginError | RC プラグインの互換性問題 | `app.json` の plugins から除外する |
| `ios/` ディレクトリがあるとアイコン設定が反映されない | EAS が `ios/` を優先参照 | `npx expo prebuild --clean` で再生成 |
| Metro 接続がシミュレーターで繋がらない | Xcode ビルドは localhost 固定 | `npx expo run:ios --device` または `--tunnel` を使う |
| EAS ビルドにデバイスが含まれない | ビルド後にデバイス登録した | `--clear-cache` 付きで再ビルド |

### Apple Review リジェクト事例

| リジェクト理由 | Guideline | 解決策 |
|---------------|-----------|--------|
| 購入ボタンが反応しない | 2.1(b) | 有料アプリ契約を締結・RevenueCat の Offerings 設定を確認 |
| 購入時にエラーが表示される | 2.1(b) | 有料アプリ契約の反映を待つ・購入エラー時の Alert 表示を追加 |
| プロモーション画像のテキストが小さい | 2.3.2 | テキストを大きく（画像幅の 80% 以上） |
| プロモーション画像に価格が含まれている | 2.3.2 | 価格表記を全て削除する |
| 説明文に利用規約 URL がない | 3.1.2(c) | 説明文末尾に ToU URL を追加する |

### サブスクリプション ローカリゼーション文字数制限

| 項目 | 上限 | 例 |
|------|------|----|
| 表示名 | 30文字 | `プレミアムプラン（年額）` |
| 説明 | 45文字 | `月換算¥233。グループ・メンバー無制限、テーマカラー全10色、CSVエクスポート` |

- 価格は ASC が自動表示するため、説明欄への価格記載は不要（記載すると Guideline 2.3.2 でリジェクトされる場合がある）
- 同一サブスクリプショングループ内のプランは表示名の命名を統一する（例: `プレミアムプラン（月額）` / `プレミアムプラン（年額）`）

### スクリーンショット・プロモーション画像

- App Store Connect 提出サイズ: `1242×2688`（6.5"）または `1320×2868`（6.9"）
- 外部ツール（ChatGPT / Gemini 等）で生成した画像は多くの場合サイズ不足 → `sips` でリサイズ必須
- `sips -z 2688 1242 input.png --out output.png`（macOS 標準、追加インストール不要）