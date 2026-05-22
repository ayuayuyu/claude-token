//! Claude Code の利用上限 (rate limit) を OAuth usage API から取得する。
//!
//! エンドポイント: GET https://api.anthropic.com/api/oauth/usage
//! 認証: Authorization: Bearer <token> ＋ anthropic-beta: oauth-2025-04-20

use serde::{Deserialize, Serialize};

use crate::keychain;

const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const OAUTH_BETA: &str = "oauth-2025-04-20";

/// 1 つの利用枠 (5時間 / 7日) の使用率とリセット時刻。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageWindow {
    /// 使用率 (0.0 - 100.0)。
    #[serde(default)]
    pub utilization: f64,
    /// リセット時刻 (ISO 8601)。
    #[serde(default)]
    pub resets_at: Option<String>,
}

/// usage API のレスポンス。必要な枠だけを取り出し、他フィールドは無視する。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// 5 時間枠。
    #[serde(default)]
    pub five_hour: Option<UsageWindow>,
    /// 7 日間枠 (全モデル)。
    #[serde(default)]
    pub seven_day: Option<UsageWindow>,
}

impl Usage {
    /// 5 時間枠の使用率 (取得できなければ 0.0)。
    pub fn five_hour_pct(&self) -> f64 {
        self.five_hour.as_ref().map_or(0.0, |w| w.utilization)
    }

    /// 7 日間枠の使用率 (取得できなければ 0.0)。
    pub fn seven_day_pct(&self) -> f64 {
        self.seven_day.as_ref().map_or(0.0, |w| w.utilization)
    }
}

/// usage API を呼び出して使用率を取得する。
pub async fn fetch_usage() -> Result<Usage, String> {
    let token = keychain::get_access_token()?;

    let client = reqwest::Client::new();
    let resp = client
        .get(USAGE_URL)
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", OAUTH_BETA)
        .send()
        .await
        .map_err(|e| format!("usage API へのリクエストに失敗: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("usage API がエラーを返しました: {status}"));
    }

    resp.json::<Usage>()
        .await
        .map_err(|e| format!("usage レスポンスの解析に失敗: {e}"))
}
