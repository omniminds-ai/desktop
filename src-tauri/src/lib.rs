use app_finder::{AppCommon, AppFinder};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use display_info::DisplayInfo;
use log::{error, info};
use serde_json;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
};
use tauri::{Emitter, Manager};
use window_vibrancy::*;
use xcap::{image::ImageFormat, Monitor};
mod axtree;
mod ffmpeg;
mod input;
mod logger;
#[cfg(target_os = "macos")]
mod macos_screencapture;
#[cfg(target_os = "macos")]
mod permissions;
mod pipeline;
mod record;

#[cfg(target_os = "macos")]
use permissions::{has_ax_perms, has_record_perms, request_ax_perms, request_record_perms};
use record::{
    create_recording_zip, get_app_data_dir, get_recording_file, list_recordings,
    open_recording_folder, process_recording, start_recording, stop_recording, write_file,
    QuestState,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn take_screenshot() -> Result<String, String> {
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
        info!("[App List] Using App List cache.");
        let app_cache = File::open(path).map_err(|e| format!("Could not open file. {}", e))?;

        let app_cache_reader = BufReader::new(app_cache);

        let json: Vec<serde_json::Value> = serde_json::from_reader(app_cache_reader)
            .map_err(|e| format!("Error parsing JSON: {}", e))?;
        Ok(json)
    } else {
        info!("[App List] No App List cache found. Gathering application data...");
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
            error!("Failed to initialize FFmpeg: {}", e);
            std::process::exit(1);
        }
    }

    if let Err(e) = axtree::init_dump_tree() {
        error!("Failed to initialize dump-tree: {}", e);
        std::process::exit(1);
    }

    if let Err(e) = pipeline::init_pipeline() {
        error!("Failed to initialize pipeline: {}", e);
        std::process::exit(1);
    }

    let _app = tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level_for("app_finder::platform::platform", log::LevelFilter::Error)
                .level_for("tao::platform_impl::platform", log::LevelFilter::Error)
                .build(),
        )
        .plugin(tauri_plugin_process::init())
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
            #[cfg(target_os = "macos")]
            has_record_perms,
            #[cfg(target_os = "macos")]
            request_record_perms,
            #[cfg(target_os = "macos")]
            has_ax_perms,
            #[cfg(target_os = "macos")]
            request_ax_perms,
            list_recordings,
            get_recording_file,
            get_app_data_dir,
            write_file,
            open_recording_folder,
            process_recording,
            create_recording_zip,
        ])
        .setup(|app| {
            #[cfg(any(windows, target_os = "linux"))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            };

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
            // set overlay locations for specific platforms

            let mut overlay_x = primary.x as f64;
            let mut overlay_y = primary.y as f64;

            if cfg!(target_os = "macos") {
                overlay_y += 35.0;
                overlay_x -= 10.0;
            }

            if cfg!(target_os = "linux") {
                // do this becuase some linux distros have top bars
                overlay_y += 25.0;
                overlay_x -= 10.0;
            }

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
            .position(overlay_x, overlay_y)
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
