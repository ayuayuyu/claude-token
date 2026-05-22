---
name: claude-usage
description: Claude Code の利用上限（5時間枠 / 7日枠）の使用率を表示する。「トークン使用量」「あとどれくらい使える」「rate limit」「使用率」「/usage 相当」を聞かれたとき、または上限に近づいているか確認したいときに使う。macOS 専用。
---

# Claude Usage（利用上限の確認）

Claude Code の OAuth usage API を直接叩いて、現在の利用上限の使用率を表示するスキル。
`/usage` を実行したり Web コンソールを開いたりせずに、ターミナルから即座に確認できる。

## 仕組み

1. macOS Keychain に保存された OAuth アクセストークンを取得する。
2. usage API を呼び出し、5時間枠 / 7日枠の使用率とリセット時刻を取得する。

## 手順

以下の bash を実行し、出力をユーザーに整形して報告する。

```bash
TOKEN=$(security find-generic-password -s "Claude Code-credentials" -w 2>/dev/null | jq -r '.claudeAiOauth.accessToken')
if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "ERROR: Keychain に認証情報が見つかりません。Claude Code でログイン済みか確認してください。"
  exit 1
fi
curl -s "https://api.anthropic.com/api/oauth/usage" \
  -H "Authorization: Bearer $TOKEN" \
  -H "anthropic-beta: oauth-2025-04-20" \
| jq -r '
    "5h: \(.five_hour.utilization)%  (resets \(.five_hour.resets_at))",
    "7d: \(.seven_day.utilization)%  (resets \(.seven_day.resets_at))"
  '
```

## 出力例

```
5h: 32%  (resets 2026-05-22T15:30:00+00:00)
7d: 49%  (resets 2026-05-26T15:00:00+00:00)
```

これを次のように読みやすく報告する:

> **Claude Code 使用率**
> - 5時間枠: 32%（15:30 にリセット）
> - 7日枠: 49%（5/26 にリセット）

リセット時刻は UTC で返るので、報告時はローカル時刻（JST など）に直すと親切。

## 注意

- ⚠️ **アクセストークンは機密情報**。`$TOKEN` の中身を echo したり、ログ・出力に残したりしない。
- macOS の `security` / `jq` / `curl` に依存する。`jq` が無ければ `brew install jq`。
- 使用率が 90% を超えていたら、上限が近いことをユーザーに警告する。
