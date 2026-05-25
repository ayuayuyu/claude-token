mod keychain;
mod usage;

use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use usage::Usage;

const TRAY_ID: &str = "main-tray";
/// 通常時の自動更新間隔 (5 分)。usage API は短時間連投すると 429 になりやすい。
const REFRESH_INTERVAL_SECS: u64 = 90;
/// 429 (レート制限) を踏んだ後のバックオフ (15 分)。
const RATE_LIMIT_BACKOFF_SECS: u64 = 900;

/// フロントエンドから呼ばれる手動取得コマンド。
#[tauri::command]
async fn get_usage() -> Result<Usage, String> {
    usage::fetch_usage().await.map_err(|e| e.to_string())
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
/// 戻り値は次回までの待機時間 (429 のときだけ長めに伸ばす)。
async fn refresh_once(app: &AppHandle) -> Duration {
    match usage::fetch_usage().await {
        Ok(u) => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_title(Some(format_title(&u)));
            }
            // フロントへ最新値を通知 (生トークンは含まれない)。
            let _ = app.emit("usage-updated", &u);
            Duration::from_secs(REFRESH_INTERVAL_SECS)
        }
        Err(e) if e.is_rate_limited() => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_title(Some("Claude · 429".to_string()));
            }
            eprintln!("usage refresh rate-limited, backing off");
            Duration::from_secs(RATE_LIMIT_BACKOFF_SECS)
        }
        Err(e) => {
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                let _ = tray.set_title(Some("Claude ⚠".to_string()));
            }
            // 失敗内容のみログ。トークンは出力しない。
            eprintln!("usage refresh error: {e}");
            Duration::from_secs(REFRESH_INTERVAL_SECS)
        }
    }
}

/// メニューバーのアイコン左クリックでウィンドウを表示/非表示トグルする。
fn toggle_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        match w.is_visible() {
            Ok(true) => {
                let _ = w.hide();
            }
            _ => {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![get_usage])
        .setup(|app| {
            // --- macOS: Dock から消してメニューバー常駐アプリにする ---
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // --- ウィンドウ: 閉じるボタンで quit せず hide する ---
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let _ = w.hide();
                        api.prevent_close();
                    }
                });
            }

            // --- 初回起動時のみ右上に配置。以降は window-state プラグインが復元する ---
            let has_saved_state = app
                .path()
                .app_config_dir()
                .ok()
                .map(|p| p.join(".window-state.json").exists())
                .unwrap_or(false);

            if let Some(window) = app.get_webview_window("main") {
                if !has_saved_state {
                    if let Ok(Some(monitor)) = app.primary_monitor() {
                        let phys = monitor.size();
                        let scale = monitor.scale_factor();
                        let logical_w = phys.width as f64 / scale;
                        let window_w = 260.0;
                        let margin_x = 24.0;
                        let margin_y = 44.0;
                        let x = (logical_w - window_w - margin_x).max(0.0);
                        let _ = window.set_position(tauri::LogicalPosition::new(x, margin_y));
                    }
                }
                // 念のため明示的にも前面表示
                let _ = window.show();
                let _ = window.set_always_on_top(true);
            }

            // --- トレイメニュー (右クリック) ---
            let toggle_i = MenuItem::with_id(app, "toggle", "表示 / 非表示", true, None::<&str>)?;
            let refresh_i = MenuItem::with_id(app, "refresh", "今すぐ更新", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_i, &refresh_i, &quit_i])?;

            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(false)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .title("Claude …")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => toggle_window(app),
                    "refresh" => {
                        let app = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = refresh_once(&app).await;
                        });
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // 左クリック: ウィンドウ表示/非表示トグル (Docker for Mac 風)
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // --- 起動時取得 + 自動更新 (通常 5 分, 429 後は 15 分) ---
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let next = refresh_once(&handle).await;
                    tokio::time::sleep(next).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
