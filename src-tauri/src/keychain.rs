//! macOS Keychain から Claude Code の OAuth アクセストークンを読み出す。
//!
//! Claude Code は認証情報を generic password として
//! service 名 "Claude Code-credentials" に JSON 形式で保存している。
//! `security find-generic-password -s "Claude Code-credentials" -w` で取り出せる。
//!
//! ⚠️ 取得したトークンは機密情報。ログや UI に生の文字列を出してはならない。

use std::process::Command;

use serde::Deserialize;

#[derive(Deserialize)]
struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: OAuth,
}

#[derive(Deserialize)]
struct OAuth {
    #[serde(rename = "accessToken")]
    access_token: String,
}

/// Keychain から Claude Code のアクセストークンを取得する。
pub fn get_access_token() -> Result<String, String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .map_err(|e| format!("security コマンドの実行に失敗: {e}"))?;

    if !output.status.success() {
        return Err(
            "Keychain に認証情報が見つかりません。Claude Code でログイン済みか確認してください。"
                .to_string(),
        );
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let creds: Credentials = serde_json::from_str(raw.trim())
        .map_err(|e| format!("認証情報の JSON 解析に失敗: {e}"))?;

    Ok(creds.claude_ai_oauth.access_token)
}
