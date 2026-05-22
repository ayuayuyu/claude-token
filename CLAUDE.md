# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 概要

Claude Code の利用上限（rate limit）の使用率を、デスクトップに常駐するウィジェットと
macOS メニューバーの両方で常時表示する Tauri v2 アプリ。`/usage` を打たずに使用率を一目で確認できる。

参考: https://zenn.dev/usshi3560/articles/0ef5882c2bde37

## 技術スタック

- **Tauri v2** (Rust バックエンド + WebView)
- **React 19 + TypeScript + Vite 7** (フロントエンド)
- **reqwest** (async, rustls-tls) で usage API を呼び出し
- パッケージマネージャ: **pnpm**

## コマンド

```bash
pnpm install        # 依存インストール (初回。Tauri CLI もここで入る)
pnpm tauri dev      # 開発実行 (ウィジェット表示 + メニューバー常駐)
pnpm tauri build    # .app / .dmg を生成 (src-tauri/target/release/bundle/)
```

## アーキテクチャ

### データ源（重要）

- **Keychain**: `security find-generic-password -s "Claude Code-credentials" -w`
  → JSON の `.claudeAiOauth.accessToken` が OAuth アクセストークン。
- **API**: `GET https://api.anthropic.com/api/oauth/usage`
  - ヘッダー: `Authorization: Bearer <token>` ＋ `anthropic-beta: oauth-2025-04-20`
  - レスポンス: `five_hour` / `seven_day` それぞれ `{ utilization (0-100), resets_at (ISO8601) }`。

> ⚠️ **セキュリティ**: アクセストークンは機密。ログ・UI・イベントに**生の文字列を絶対に出さない**。
> Rust 側はトークンを API ヘッダーにしか使わず、フロントへ渡す `Usage` には含めない。

### Rust バックエンド (`src-tauri/src/`)

- `keychain.rs` — `security` CLI 経由でトークンを取得・JSON 解析。
- `usage.rs` — usage API クライアントと `Usage` / `UsageWindow` モデル。`fetch_usage()` を公開。
- `lib.rs` — エントリ。次を担う:
  - `get_usage` コマンド（フロントの初期取得用）。
  - トレイ生成（`TRAY_ID = "main-tray"`）＋ `set_title()` でメニューバーに `5h 32% · 7d 49%` を表示。
  - トレイメニュー: 表示 / 今すぐ更新 / 終了。
  - 起動時 + 60 秒ごとに `refresh_once()` → トレイ更新 ＋ `usage-updated` イベント送信。
- `main.rs` — `claude_token_lib::run()` を呼ぶだけ。

### フロントエンド (`src/`)

- `hooks/useUsage.ts` — 初回 `invoke("get_usage")`、以降 `listen("usage-updated")` で更新。
- `components/RingGauge.tsx` — SVG リングゲージ。使用率で色が変化。
- `components/UsageCard.tsx` — 5h / 7d の 2 リングを並べたグラスモーフィズムのカード。
- `types.ts` — `Usage` 型、しきい値 → 色/表情のマッピング、リセット時刻整形。
- `App.tsx` / `styles.css` — 透明ウィンドウ。`data-tauri-drag-region` でドラッグ移動可。

### ウィンドウ設定 (`src-tauri/tauri.conf.json`)

`app.macOSPrivateApi: true`（透明化に必須）。window: `transparent` / `decorations:false` /
`alwaysOnTop` / `skipTaskbar` / `resizable:false` / `shadow:false`。label は `"main"`
（`capabilities/default.json` の対象ウィンドウと一致させること）。

## しきい値（色・表情）

| 使用率 | レベル | 色 |
|--------|--------|-----|
| 0–49%  | calm    | 緑 |
| 50–74% | normal  | 青 |
| 75–89% | warn    | 黄 |
| 90%+   | critical| 赤 |

## 配布 (Homebrew Cask)

GUI アプリなので Formula ではなく **Cask** で配布する。**未署名**のため Gatekeeper 対策が要る。

```bash
# 利用者側
brew tap ayuayuyu/tap
brew install --cask --no-quarantine claude-token   # 未署名なので --no-quarantine が必須
```

### リリースフロー（CI 完全自動）

`v*` タグを push すると [.github/workflows/release.yml](.github/workflows/release.yml) が実行され:

1. macOS ランナーで `pnpm tauri build --target universal-apple-darwin` (Intel/Apple Silicon 両対応の universal .dmg)。
2. `claude-token-<version>-universal.dmg` を GitHub Release に添付。
3. `packaging/claude-token.rb`（Cask テンプレート）の `__VERSION__` / `__SHA256__` を置換し、
   `ayuayuyu/homebrew-tap` の `Casks/claude-token.rb` を自動更新・push。

```bash
# リリース手順
# 1) package.json と src-tauri/tauri.conf.json の version を更新
# 2) タグを打って push
git tag v0.1.0 && git push origin v0.1.0
```

**前提**: claude-token リポジトリに `HOMEBREW_TAP_TOKEN` シークレット（homebrew-tap への push 権限を持つ PAT）が必要。

### ローカルビルド（手動確認用）

```bash
pnpm tauri build                              # arm64 の .app / .dmg
# 出力: src-tauri/target/release/bundle/{macos/claude-token.app, dmg/*.dmg}
```

## 注意点

- 透明化に private API を使うため **App Store 配布は不可**（個人利用前提）。
- **未署名**配布のため、利用者は `--no-quarantine` での install か、初回のみ右クリック→開く / `xattr -dr com.apple.quarantine` が必要。
- usage 値の更新間隔は `lib.rs` の `REFRESH_INTERVAL_SECS`（既定 60 秒）。
