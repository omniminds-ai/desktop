use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use log::info;
use serde_json;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Write},
};
use tauri::Manager;
use xcap::{image::ImageFormat, Monitor};

#[cfg(not(target_os = "linux"))]
use app_finder::{AppCommon, AppFinder};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub async fn take_screenshot() -> Result<String, String> {
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
pub async fn list_apps(
    app: tauri::AppHandle,
    include_icons: Option<bool>,
) -> Result<Vec<serde_json::Value>, String> {
    #[cfg(not(target_os = "linux"))]
    {
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
