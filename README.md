# claude-token

Claude Code の利用上限（rate limit）の使用率を、macOS のメニューバーと
常駐ウィジェットで**常時可視化**する Tauri v2 アプリ。

`/usage` を打たなくても、5 時間枠と 7 日枠の残量がメニューバーに常に表示されます。

参考記事: <https://zenn.dev/usshi3560/articles/0ef5882c2bde37>

---

## 特徴

- **メニューバー常駐**: `5h 32% · 7d 49%` のような形式で使用率を表示
- **デスクトップウィジェット**: 透明・常時最前面のリングゲージで一目で確認
- **自動更新**: 通常 90 秒間隔、429（レート制限）を踏んだ後は 15 分にバックオフ
- **色 / 表情で警告**: 使用率に応じて緑 → 青 → 黄 → 赤に変化
- **トークン漏洩防止**: アクセストークンは Keychain から取得し、フロントや UI には**一切渡さない**
- **Dock を占有しない**: macOS では Accessory モードで起動し、Dock アイコンを表示しない

## しきい値（色 / 状態）

| 使用率 | レベル | 色 |
|--------|--------|-----|
| 0–49%  | calm    | 緑 |
| 50–74% | normal  | 青 |
| 75–89% | warn    | 黄 |
| 90%+   | critical| 赤 |

---

## インストール（Homebrew Cask）

GUI アプリのため Formula ではなく **Cask** で配布しています。**未署名**なので
Gatekeeper 対策として `--no-quarantine` が必要です。

```bash
brew tap ayuayuyu/tap
brew install --cask --no-quarantine claude-token
```

通常インストール後に Gatekeeper でブロックされた場合は、次のいずれかで解除できます。

```bash
# 方法 1: 隔離属性を一括解除
xattr -dr com.apple.quarantine "/Applications/claude-token.app"

# 方法 2: Finder で右クリック →「開く」を一度だけ実行
```

## アンインストール

```bash
brew uninstall --cask claude-token
brew untap ayuayuyu/tap   # tap を残したくない場合
```

Cask には `zap` も定義されているため、設定・キャッシュごと消したい場合は:

```bash
brew uninstall --cask --zap claude-token
```

---

## 必要環境

- **macOS** (Apple Silicon / Intel どちらも universal バイナリで動作)
- **Claude Code** がインストール済みで、ログイン状態（Keychain にトークンが入っている状態）
- 透明化に Apple private API を使用しているため **App Store 配布は不可**（個人利用前提）

---

## 仕組み

### データ源

1. **macOS Keychain** からアクセストークンを取得

   ```bash
   security find-generic-password -s "Claude Code-credentials" -w
   ```

   JSON の `.claudeAiOauth.accessToken` フィールドが OAuth アクセストークン。

2. **usage API** に問い合わせる

   ```http
   GET https://api.anthropic.com/api/oauth/usage
     Authorization: Bearer <token>
     anthropic-beta: oauth-2025-04-20
   ```

   レスポンスから `five_hour` / `seven_day` の `utilization`（0–100）と `resets_at` を取り出して表示する。

### 自動更新の挙動

`src-tauri/src/lib.rs` の定数で制御:

| 定数 | 既定 | 用途 |
|------|------|------|
| `REFRESH_INTERVAL_SECS` | 90 秒 | 通常時の更新間隔 |
| `RATE_LIMIT_BACKOFF_SECS` | 900 秒 (15 分) | 429 後のバックオフ |

usage API は短時間に連投すると 429 を返すため、レート制限を踏んだら自動的に
長めの間隔に切り替え、トレイには `Claude · 429` と表示します。

### トレイ操作

- **左クリック**: ウィンドウを表示 / 非表示トグル
- **右クリック**: メニュー（表示 / 非表示・今すぐ更新・終了）

ウィンドウは `data-tauri-drag-region` でドラッグして自由に動かせます。
閉じるボタンを押しても終了せず、隠れるだけです（メニューバーから「終了」で終了）。

---

## 開発

```bash
pnpm install           # 依存インストール（Tauri CLI もここで入る）
pnpm tauri dev         # 開発実行（ウィジェット表示 + メニューバー常駐）
```

### ビルド

```bash
pnpm tauri build       # 現在のアーキの .app / .dmg
# 出力: src-tauri/target/release/bundle/{macos/claude-token.app, dmg/*.dmg}

# universal バイナリ（CI と同じ Intel/AS 両対応）
pnpm tauri build --target universal-apple-darwin
```

### 技術スタック

- **Tauri v2**（Rust バックエンド + WebView）
- **React 19 + TypeScript + Vite 7**（フロントエンド）
- **reqwest**（async, rustls-tls）で usage API を呼び出し
- パッケージマネージャ: **pnpm**

---

## アーキテクチャ

```text
┌─────────────────────────────────────────────────┐
│  macOS Keychain                                 │
│   └─ "Claude Code-credentials" (accessToken)    │
└─────────────────────────────────────────────────┘
              │ security find-generic-password
              ▼
┌─────────────────────────────────────────────────┐
│  Rust backend (src-tauri/src)                   │
│   keychain.rs ──► usage.rs ──► api.anthropic.com│
│        │              │                          │
│        │              ▼                          │
│        │         Usage { five_hour, seven_day } │
│        ▼                                         │
│   lib.rs                                         │
│    ├─ Tray icon: set_title("5h 32% · 7d 49%")   │
│    └─ emit("usage-updated", usage) ──┐          │
└──────────────────────────────────────┼──────────┘
                                       │ Tauri IPC
                                       ▼
┌─────────────────────────────────────────────────┐
│  Frontend (src/)                                │
│   useUsage.ts ──► UsageCard ──► RingGauge × 2  │
└─────────────────────────────────────────────────┘
```

### 主なファイル

**Rust 側 (`src-tauri/src/`)**

| ファイル | 役割 |
|----------|------|
| `keychain.rs` | `security` CLI 経由でトークンを取得・JSON 解析 |
| `usage.rs` | usage API クライアントと `Usage` / `UsageWindow` モデル |
| `lib.rs` | エントリ。トレイ生成、自動更新ループ、`get_usage` コマンド |
| `main.rs` | `claude_token_lib::run()` を呼ぶだけ |

**フロントエンド (`src/`)**

| ファイル | 役割 |
|----------|------|
| `hooks/useUsage.ts` | 初回 `invoke("get_usage")`、以降 `listen("usage-updated")` |
| `components/RingGauge.tsx` | SVG リングゲージ。使用率で色が変化 |
| `components/UsageCard.tsx` | 5h / 7d の 2 リングを並べたグラスモーフィズムのカード |
| `types.ts` | `Usage` 型、しきい値 → 色マッピング、リセット時刻整形 |

### ウィンドウ設定（`src-tauri/tauri.conf.json`）

- `app.macOSPrivateApi: true`（透明化に必須）
- window: `transparent` / `decorations:false` / `alwaysOnTop` / `skipTaskbar` / `resizable:false` / `shadow:false`
- label は `"main"`（`capabilities/default.json` の対象ウィンドウと一致させること）

---

## セキュリティ

- アクセストークンは **Keychain から実行時に取得**し、ソースには含まれません
- Rust 側はトークンを **API ヘッダーにしか使わず**、フロント／UI／イベントペイロード／ログには出力しません
- HTTPS（rustls）で Anthropic API と直接通信。中継サーバーは存在しません
- CI が使うのは `HOMEBREW_TAP_TOKEN`（homebrew-tap への push 権限）のみで、リポジトリには値は入っていません

---

## リリース（CI 完全自動）

`v*` タグを push すると [.github/workflows/release.yml](.github/workflows/release.yml) が走り、以下が自動で行われます。

1. macOS ランナーで `pnpm tauri build --target universal-apple-darwin`
2. `claude-token-<version>-universal.dmg` を GitHub Release に添付
3. [packaging/claude-token.rb](packaging/claude-token.rb) の `__VERSION__` / `__SHA256__` を置換し、
   [ayuayuyu/homebrew-tap](https://github.com/ayuayuyu/homebrew-tap) の `Casks/claude-token.rb` を自動更新・push

```bash
# リリース手順
# 1) package.json と src-tauri/tauri.conf.json の version を更新
# 2) タグを打って push
git tag v0.1.0 && git push origin v0.1.0
```

> **前提**: 本リポジトリに `HOMEBREW_TAP_TOKEN`（homebrew-tap への push 権限を持つ PAT）が GitHub Secrets として登録されている必要があります。

---

## トラブルシューティング

### メニューバーに `Claude ⚠` と表示される

トークンが取得できないか、API 呼び出しに失敗しています。確認手順:

```bash
# 1) Keychain にトークンが存在するか
security find-generic-password -s "Claude Code-credentials" -w >/dev/null && echo OK

# 2) Claude Code に再ログインする（必要に応じて）
```

### メニューバーに `Claude · 429` と表示される

usage API のレート制限です。15 分後に自動で再試行されるので、放置すれば直ります。

### 「壊れているため開けません」と Gatekeeper に拒否される

未署名アプリのため、隔離属性を外す必要があります:

```bash
xattr -dr com.apple.quarantine "/Applications/claude-token.app"
```

または `brew install --cask --no-quarantine claude-token` を使ってください。

### ウィンドウが透明にならない

`tauri.conf.json` の `app.macOSPrivateApi` が `true` であること、
ビルド時に `macos-private-api` feature が有効であることを確認してください。

---

## ライセンス

個人利用前提のため特にライセンスは設定していません。フォーク・改変は自由にどうぞ。

## クレジット

- 着想元の Zenn 記事: [Claude Code の利用上限を可視化する](https://zenn.dev/usshi3560/articles/0ef5882c2bde37)
- 利用 API: Anthropic OAuth usage API（`anthropic-beta: oauth-2025-04-20`）
