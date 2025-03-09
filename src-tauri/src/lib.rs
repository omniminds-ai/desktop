use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use display_info::DisplayInfo;
use log::{error, info};
use serde_json;
use std::{
    io::{Cursor, Write},
    path::Path,
};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use window_vibrancy::*;
use xcap::{image::ImageFormat, Monitor};
use zip::{write::FileOptions, ZipWriter};
mod axtree;
mod ffmpeg;
mod input;
mod logger;
#[cfg(not(target_os = "linux"))]
use app_finder::{AppCommon, AppFinder};
#[cfg(target_os = "macos")]
mod macos_screencapture;
#[cfg(target_os = "macos")]
mod permissions;
mod pipeline;
mod record;
mod settings;
mod engine;

#[cfg(target_os = "macos")]
use permissions::{has_ax_perms, has_record_perms, request_ax_perms, request_record_perms};
use record::{
    create_recording_zip, get_app_data_dir, get_directory_size, get_recording_file,
    list_recordings, open_folder, open_recording_folder, process_recording, start_recording,
    stop_recording, write_file, QuestState, delete_recording
};
use settings::{get_upload_data_allowed, set_upload_confirmed, get_settings, save_settings, set_onboarding_complete, get_onboarding_complete};
#[cfg(target_os = "windows")]
use settings::{list_wsl_distros, get_default_wsl_distro};
use engine::{EngineState, start_engine, stop_engine, get_engine_status, run_command, get_job, get_all_jobs, clear_all_jobs};

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
    #[cfg(not(target_os = "linux"))]
    {
        use std::{
            fs::File,
            io::{BufReader, BufWriter},
        };

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

    #[cfg(target_os = "linux")]
    {
        // Return empty list for Linux
        Ok(Vec::new())
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
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(QuestState::default())
        .manage(EngineState::default())
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
            get_onboarding_complete,
            set_onboarding_complete,
            get_app_data_dir,
            write_file,
            open_recording_folder,
            open_folder,
            process_recording,
            create_recording_zip,
            get_upload_data_allowed,
            set_upload_confirmed,
            get_settings,
            save_settings,
            #[cfg(target_os = "windows")]
            list_wsl_distros,
            #[cfg(target_os = "windows")]
            get_default_wsl_distro,
            export_recordings,
            get_directory_size,
            start_engine,
            stop_engine,
            get_engine_status,
            run_command,
            get_job,
            get_all_jobs,
            clear_all_jobs,
            delete_recording
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

#[tauri::command]
async fn export_recordings(app: tauri::AppHandle) -> Result<String, String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    // Create a buffer to store the zip file
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add recordings folder contents to zip
    if recordings_dir.exists() {
        println!("Zipping files in {:?}", recordings_dir.to_string_lossy());

        // Helper function to recursively add files and directories to the zip
        fn add_dir_to_zip(
            zip: &mut ZipWriter<Cursor<Vec<u8>>>,
            options: FileOptions,
            src_dir: &Path,
            base_path: &Path,
        ) -> Result<(), String> {
            for entry in std::fs::read_dir(src_dir)
                .map_err(|e| format!("Failed to read directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                // Calculate relative path from base_path
                let relative_path = path
                    .strip_prefix(base_path)
                    .map_err(|e| format!("Failed to strip prefix: {}", e))?;
                let relative_path_str = relative_path.to_string_lossy();

                if path.is_file() {
                    // Add file to zip
                    zip.start_file(relative_path_str.as_ref(), options)
                        .map_err(|e| format!("Failed to add file to zip: {}", e))?;

                    let contents = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read file contents: {}", e))?;

                    zip.write_all(&contents)
                        .map_err(|e| format!("Failed to write file contents to zip: {}", e))?;
                } else if path.is_dir() {
                    // Create directory entry in zip
                    let dir_path = format!("{}/", relative_path_str);
                    zip.add_directory(dir_path, options)
                        .map_err(|e| format!("Failed to add directory to zip: {}", e))?;

                    // Recursively add contents of this directory
                    add_dir_to_zip(zip, options, &path, base_path)?;
                }
            }
            Ok(())
        }

        // Start recursively adding files and directories
        add_dir_to_zip(&mut zip, options, &recordings_dir, &recordings_dir)?;
    }

    let buf = zip
        .finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?
        .into_inner();

    let selected_dir = app.dialog().file().blocking_pick_folder();

    // If user cancels the dialog, selected_dir will be None
    if let Some(dir_path) = selected_dir {
        // Create the full path for history.zip
        let dir_path_str = dir_path.to_string();
        let file_path = Path::new(&dir_path_str).join("history.zip");

        // Write the buffer to the file
        std::fs::write(&file_path, buf).map_err(|e| format!("Failed to write zip file: {}", e))?;

        Ok(file_path.to_string_lossy().into_owned())
    } else {
        Ok("".to_string())
    }
}
