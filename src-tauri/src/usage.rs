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

/// usage API 呼び出しで発生し得るエラーをカテゴリ別に表す。
/// 429 (レート制限) を呼び出し側で識別してバックオフできるよう独立列挙する。
#[derive(Debug)]
pub enum UsageError {
    Token(String),
    Request(String),
    /// 429 Too Many Requests。
    RateLimited,
    /// その他の HTTP ステータスエラー。
    Status(u16),
    Parse(String),
}

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageError::Token(s) => write!(f, "{s}"),
            UsageError::Request(s) => write!(f, "usage API へのリクエストに失敗: {s}"),
            UsageError::RateLimited => {
                write!(f, "レート制限中 (429)。次回更新まで待機します")
            }
            UsageError::Status(s) => write!(f, "usage API がエラーを返しました: {s}"),
            UsageError::Parse(s) => write!(f, "usage レスポンスの解析に失敗: {s}"),
        }
    }
}

impl UsageError {
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, UsageError::RateLimited)
    }
}

/// usage API を呼び出して使用率を取得する。
pub async fn fetch_usage() -> Result<Usage, UsageError> {
    let token = keychain::get_access_token().map_err(UsageError::Token)?;

    let client = reqwest::Client::new();
    let resp = client
        .get(USAGE_URL)
        .header("Authorization", format!("Bearer {token}"))
        .header("anthropic-beta", OAUTH_BETA)
        .send()
        .await
        .map_err(|e| UsageError::Request(e.to_string()))?;

    let status = resp.status();
    if status.as_u16() == 429 {
        return Err(UsageError::RateLimited);
    }
    if !status.is_success() {
        return Err(UsageError::Status(status.as_u16()));
    }

    resp.json::<Usage>()
        .await
        .map_err(|e| UsageError::Parse(e.to_string()))
}
