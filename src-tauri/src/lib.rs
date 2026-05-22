mod keychain;
mod usage;

use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

use usage::Usage;

const TRAY_ID: &str = "main-tray";
const REFRESH_INTERVAL_SECS: u64 = 60;

/// フロントエンドから呼ばれる手動取得コマンド。
#[tauri::command]
async fn get_usage() -> Result<Usage, String> {
    usage::fetch_usage().await
}

/// メニューバーに出すタイトル文字列を組み立てる。例: "5h 32% · 7d 49%"
fn format_title(u: &Usage) -> String {
    format!(
        "5h {:.0}% · 7d {:.0}%",
        u.five_hour_pct(),
        u.seven_day_pct()
    )
}

/// 使用率を 1 回取得し、トレイのタイトル更新とフロントへのイベント送信を行う。
async fn refresh_once(app: &AppHandle) {
    match usage::fetch_usage().await {
        Ok(u) => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_title(Some(format_title(&u)));
            }
            // フロントへ最新値を通知 (生トークンは含まれない)。
            let _ = app.emit("usage-updated", &u);
        }
        Err(e) => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_title(Some("Claude ⚠".to_string()));
            }
            // 失敗内容のみログ。トークンは出力しない。
            eprintln!("usage refresh error: {e}");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_usage])
        .setup(|app| {
            // --- トレイメニュー ---
            let open_i = MenuItem::with_id(app, "open", "ウィジェットを表示", true, None::<&str>)?;
            let refresh_i = MenuItem::with_id(app, "refresh", "今すぐ更新", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &refresh_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .title("Claude …")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "refresh" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move { refresh_once(&app).await });
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            // --- 起動時取得 + 60 秒ごとの自動更新 ---
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    refresh_once(&handle).await;
                    tokio::time::sleep(Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
