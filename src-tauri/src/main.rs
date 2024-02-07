// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{
    AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let attendance = CustomMenuItem::new("attendance".to_string(), "出退勤");
    let tray_menu = SystemTrayMenu::new()
        .add_item(attendance.clone()) // Clone attendance item for toggling its title
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);
    let system_tray = SystemTray::new().with_menu(tray_menu);
    let is_working = Arc::new(AtomicBool::new(false)); // Arc と AtomicBool を使用してスレッドセーフに

    tauri::Builder::default()
        .setup(|app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(move |app, event| {
            // is_workingをクロージャ内で共有
            let is_working = Arc::clone(&is_working);

            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    handle_attendance(app, &is_working);
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "attendance" => {
                        handle_attendance(app, &is_working);
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 出退勤ボタンが押された時の処理
fn handle_attendance(app: &AppHandle, is_working: &Arc<AtomicBool>) {
    // is_working フラグを反転させる
    let new_value = !is_working.load(Ordering::Relaxed);
    is_working.store(new_value, Ordering::Relaxed);

    // 出勤ボタンが押されたらタイマーを開始
    if new_value {
        timer(app, is_working);
    } else {
        let _ = app.tray_handle().set_title("");
    }
}

// タイマーを開始する関数
fn timer(app: &AppHandle, is_working: &Arc<AtomicBool>) {
    let app_clone = app.clone();
    let is_working_clone = Arc::clone(&is_working);
    thread::spawn(move || {
        let start_time = Instant::now();
        loop {
            if !is_working_clone.load(Ordering::Relaxed) {
                break;
            }
            let elapsed = start_time.elapsed();
            let formatted_duration = format_duration(elapsed);
            let _ = app_clone.tray_handle().set_title(&formatted_duration);

            thread::sleep(Duration::from_secs(1));
        }
    });
}

// 経過時間を hh:mm:ss のフォーマットに整形する関数
fn format_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
