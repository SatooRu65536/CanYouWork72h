// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "macos")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use tauri::{
    AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, ActivationPolicy,
};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let attendance = CustomMenuItem::new("attendance".to_string(), "出勤");
    let break_time = CustomMenuItem::new("break_time".to_string(), "休憩").disabled();

    let tray_menu = SystemTrayMenu::new()
        .add_item(attendance.clone()) // Clone attendance item for toggling its title
        .add_item(break_time.clone()) // Clone break_time item for toggling its title
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    let is_working = Arc::new(AtomicBool::new(false)); // 出勤状態のフラグ
    let is_on_break = Arc::new(AtomicBool::new(false)); // 休憩状態のフラグ

    tauri::Builder::default()
        .setup(|app| {
            app.set_activation_policy(ActivationPolicy::Accessory);
            Ok(())
        })
        .system_tray(system_tray)
        .enable_macos_default_menu(false)
        .on_system_tray_event(move |app, event| {
            // フラグをクロージャ内で共有
            let is_working = Arc::clone(&is_working);
            let is_on_break = Arc::clone(&is_on_break);

            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    handle_tray_left_click(app, &is_working, &is_on_break);
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "attendance" => {
                        handle_attendance(app, &is_working, &is_on_break);
                    }
                    "break_time" => {
                        handle_break_time(app, &is_on_break);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// タスクトレイ右クリックの処理
fn handle_tray_left_click(
    app: &AppHandle,
    is_working: &Arc<AtomicBool>,
    is_on_break: &Arc<AtomicBool>,
) {
    if is_on_break.load(Ordering::Relaxed) {
        handle_break_time(app, is_on_break);
    } else {
        handle_attendance(app, is_working, is_on_break);
    }
}

// "attendance" メニュー項目の処理
fn handle_attendance(app: &AppHandle, is_working: &Arc<AtomicBool>, is_on_break: &Arc<AtomicBool>) {
    // 出勤/退勤を切り替える
    let new_value = !is_working.load(Ordering::Relaxed);
    is_working.store(new_value, Ordering::Relaxed);

    // メニューアイテムのタイトルを更新
    let item_handle = app.tray_handle().get_item("attendance");
    let new_title = if new_value { "退勤" } else { "出勤" };
    let _ = item_handle.set_title(new_title);
    println!("出勤: {}", new_value);

    // タイマーを開始または停止
    if new_value {
        start_timer(app, is_working.clone(), is_on_break.clone());

        // "break_time" メニューアイテムを有効化
        let item_handle = app.tray_handle().get_item("break_time");
        let _ = item_handle.set_enabled(true);
    } else {
        // "break_time" メニューアイテムを有効化
        let item_handle = app.tray_handle().get_item("break_time");
        let _ = item_handle.set_enabled(false);

        let app_clone = app.clone();
        let _ = app_clone.tray_handle().set_title("");
    }
}

// "break_time" メニュー項目の処理
fn handle_break_time(app: &AppHandle, is_on_break: &Arc<AtomicBool>) {
    let new_value = !is_on_break.load(Ordering::Relaxed);
    is_on_break.store(new_value, Ordering::Relaxed);

    {
        // メニューアイテムのタイトルを更新
        let item_handle = app.tray_handle().get_item("break_time");
        let new_title = if new_value { "休憩解除" } else { "休憩" };
        let _ = item_handle.set_title(new_title);
        println!("休憩: {}", new_value);
    }

    if new_value {
        let app_clone = app.clone();
        let _ = app_clone.tray_handle().set_title("休憩中");

        // "attendance" メニューアイテムを無効化
        let item_handle = app.tray_handle().get_item("attendance");
        let _ = item_handle.set_enabled(false);
    } else {
        // "attendance" メニューアイテムを有効化
        let item_handle = app.tray_handle().get_item("attendance");
        let _ = item_handle.set_enabled(true);
    }
}

// タイマーを開始
fn start_timer(app: &AppHandle, is_working: Arc<AtomicBool>, is_on_break: Arc<AtomicBool>) {
    let app_clone = app.clone();
    thread::spawn(move || {
        let mut time = Duration::from_secs(0);
        loop {
            if !is_working.load(Ordering::Relaxed) {
                break;
            }
            if is_on_break.load(Ordering::Relaxed) {
                continue;
            }
            time += Duration::from_secs(1);
            let formatted_duration = format_duration(time);
            println!("タイマー: {}", formatted_duration);

            // アプリケーションのトレイハンドルを使ってタイトルを設定
            let _ = app_clone.tray_handle().set_title(&formatted_duration);

            thread::sleep(Duration::from_secs(1));
        }
    });
}

// 経過時間を hh:mm:ss のフォーマットに整形
fn format_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
