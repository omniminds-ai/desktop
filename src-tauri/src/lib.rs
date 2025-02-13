use app_finder::{AppCommon, AppFinder};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use display_info::DisplayInfo;
use serde_json;
use std::{
    fmt::format,
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
    thread,
};
use tauri::{Emitter, Manager};
use window_vibrancy::*;
use xcap::{image::ImageFormat, Monitor};
mod axtree;
mod ffmpeg;
mod input;
mod logger;
mod macos_screencapture;
mod record;

use input::request_input_perms;
use record::{
    get_app_data_dir, get_recording_file, list_recordings, open_recording_folder,
    request_record_perms, start_recording, stop_recording, write_file, QuestState,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn take_screenshot() -> Result<String, String> {
    // Get primary monitor
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let primary = monitors
        .into_iter()
        .next()
        .ok_or_else(|| "No monitor found".to_string())?;

    // Capture image
    let xcap_image = primary.capture_image().map_err(|e| e.to_string())?;

    // Convert to PNG bytes
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    xcap_image
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    // Convert to base64
    Ok(format!("data:image/png;base64,{}", BASE64.encode(&buffer)))
}

#[tauri::command]
async fn list_apps(
    app: tauri::AppHandle,
    include_icons: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("app_list.json");

    let exists = path.exists();
    if exists {
        println!("[App List] Using App List cache.");
        let app_cache = File::open(path).map_err(|e| format!("Could not open file. {}", e))?;

        let app_cache_reader = BufReader::new(app_cache);

        let json: Vec<serde_json::Value> = serde_json::from_reader(app_cache_reader)
            .map_err(|e| format!("Error parsing JSON: {}", e))?;
        Ok(json)
    } else {
        println!("[App List] No App List cache found. Gathering application data...");
        let apps = AppFinder::list();
        let filtered: Vec<_> = apps
            .into_iter()
            .filter(|item| !item.path.contains("Frameworks"))
            .collect();

        let results = filtered
            .into_iter()
            .map(|app| {
                let mut json = serde_json::json!({
                    "name": app.name,
                    "path": app.path,
                });

                if include_icons.unwrap_or(false) {
                    if let Ok(icon) = app.get_app_icon_base64(64) {
                        json.as_object_mut()
                            .unwrap()
                            .insert("icon".to_string(), serde_json::Value::String(icon));
                    }
                }
                json
            })
            .collect();

        let file = File::create(path).map_err(|e| format!("Error creating file: {}", e))?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &results)
            .map_err(|e| format!("Error writing JSON: {}", e))?;
        writer
            .flush()
            .map_err(|e| format!("Failed to flush buffer: {}", e))?;
        Ok(results)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize FFmpeg and dump-tree synchronously before starting Tauri on windows and linux
    if !cfg!(target_os = "macos") {
        if let Err(e) = ffmpeg::init_ffmpeg() {
            eprintln!("Failed to initialize FFmpeg: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = axtree::init_dump_tree() {
        eprintln!("Failed to initialize dump-tree: {}", e);
        std::process::exit(1);
    }

    let _app = tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .manage(QuestState::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_recording,
            stop_recording,
            take_screenshot,
            list_apps,
            request_record_perms,
            request_input_perms,
            list_recordings,
            get_recording_file,
            get_app_data_dir,
            write_file,
            open_recording_folder
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None)
                .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

            #[cfg(target_os = "windows")]
            apply_mica(&window, Some(true))
                .expect("Unsupported platform! 'apply_mica' is only supported on Windows");

            // TODO: multimonitor support
            // Get primary display info
            let displays =
                DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
            let primary = displays
                .iter()
                .find(|d| d.is_primary)
                .or_else(|| displays.first())
                .ok_or_else(|| "No display found".to_string())?;

            // Create transparent overlay window
            let overlay_window = tauri::WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("overlay".into()),
            )
            .transparent(true)
            .always_on_top(true)
            .decorations(false)
            .focused(false)
            .shadow(false)
            .position(primary.x as f64, primary.y as f64)
            .inner_size(primary.width as f64, primary.height as f64)
            .skip_taskbar(true)
            .visible_on_all_workspaces(true)
            .build()?;

            overlay_window.set_ignore_cursor_events(true)?;

            // Emit initial recording status
            app.emit(
                "recording-status",
                serde_json::json!({
                    "state": "stopped"
                }),
            )
            .unwrap();

            // Set up window close handler after all other operations
            let window_handle = window.clone();
            let overlay_handle = overlay_window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Destroyed = event {
                    let _ = overlay_handle.close();
                    window_handle.app_handle().exit(0);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
