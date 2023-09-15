// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, Instant};

use tauri::{CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let attendance = CustomMenuItem::new("attendance".to_string(), "出勤");

    let tray_menu = SystemTrayMenu::new()
        .add_item(attendance.clone()) // Clone attendance item for toggling its title
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    let is_working = Arc::new(AtomicBool::new(false)); // Arc と AtomicBool を使用してスレッドセーフに

    tauri::Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(move |app, event| {
            // is_workingをクロージャ内で共有
            let is_working = Arc::clone(&is_working);

            match event {
                SystemTrayEvent::LeftClick { .. } => {
                    println!("system tray received a left click");
                }
                SystemTrayEvent::RightClick { .. } => {
                    println!("system tray received a right click");
                }
                SystemTrayEvent::DoubleClick { .. } => {
                    println!("system tray received a double click");
                }
                SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "attendance" => {
                        // is_working フラグを反転させる
                        let new_value = !is_working.load(Ordering::Relaxed);
                        is_working.store(new_value, Ordering::Relaxed);

                        // メニューアイテムのタイトルを更新
                        let item_handle = app.tray_handle().get_item(&id);
                        let new_title = if new_value { "退勤" } else { "出勤" };
                        let _ = item_handle.set_title(new_title);

                        println!("出勤: {}", new_value);

                        // 出勤ボタンが押されたらタイマーを開始
                        if new_value {
                            let app_clone = app.clone();
                            thread::spawn(move || {
                                let start_time = Instant::now();
                                loop {
                                    if !is_working.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    let elapsed = start_time.elapsed();
                                    let formatted_duration = format_duration(elapsed);
                                    println!("タイマー: {}", formatted_duration);

                                    // アプリケーションのトレイハンドルを使ってタイトルを設定
                                    let _ = app_clone.tray_handle().set_title(&formatted_duration);

                                    thread::sleep(Duration::from_secs(1));
                                }
                            });
                        } else {
                            // システムトレイのタイトルを戻す
                            let _ = app.tray_handle().set_title(new_title);
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 経過時間を hh:mm:ss のフォーマットに整形する関数
fn format_duration(duration: Duration) -> String {
    let hours = duration.as_secs() / 3600;
    let minutes = (duration.as_secs() % 3600) / 60;
    let seconds = duration.as_secs() % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
