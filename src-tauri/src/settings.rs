use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub upload_confirmed: bool,
    pub onboarding_complete: bool,
}

impl Settings {
    pub fn load(app: &AppHandle) -> Self {
        let path = get_settings_path(app);

        if path.exists() {
            match File::open(&path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    match serde_json::from_reader(reader) {
                        Ok(settings) => {
                            info!("[Settings] Loaded settings from {}", path.display());
                            settings
                        }
                        Err(e) => {
                            error!("[Settings] Error parsing settings JSON: {}", e);
                            
                            // Try to parse the file again with more lenient options
                            // This will handle missing fields by using default values
                            if let Ok(file) = File::open(&path) {
                                let reader = BufReader::new(file);
                                let json_value: Result<serde_json::Value, _> = serde_json::from_reader(reader);
                                
                                if let Ok(json) = json_value {
                                    if let Ok(partial_settings) = serde_json::from_value::<Settings>(json.clone()) {
                                        info!("[Settings] Successfully recovered settings with defaults for missing fields");
                                        return partial_settings;
                                    }
                                    
                                    // If the above fails, try to manually extract the fields that are present
                                    let mut settings = Settings::default();
                                    
                                    if let Some(upload_confirmed) = json.get("upload_confirmed") {
                                        if let Some(value) = upload_confirmed.as_bool() {
                                            settings.upload_confirmed = value;
                                        }
                                    }
                                    
                                    if let Some(onboarding_complete) = json.get("onboarding_complete") {
                                        if let Some(value) = onboarding_complete.as_bool() {
                                            settings.onboarding_complete = value;
                                        }
                                    }
                                    
                                    info!("[Settings] Manually recovered partial settings");
                                    return settings;
                                }
                            }
                            
                            // If all recovery attempts fail, use defaults
                            Settings::default()
                        }
                    }
                }
                Err(e) => {
                    error!("[Settings] Could not open settings file: {}", e);
                    Settings::default()
                }
            }
        } else {
            info!("[Settings] No settings file found, using defaults");
            Settings::default()
        }
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let path = get_settings_path(app);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let file =
            File::create(&path).map_err(|e| format!("Failed to create settings file: {}", e))?;

        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self)
            .map_err(|e| format!("Failed to write settings JSON: {}", e))?;

        writer
            .flush()
            .map_err(|e| format!("Failed to flush settings file: {}", e))?;

        info!("[Settings] Saved settings to {}", path.display());
        Ok(())
    }
}

fn get_settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_local_data_dir()
        .expect("Failed to get app data directory")
        .join("settings.json")
}

#[tauri::command]
pub fn get_upload_data_allowed(app: AppHandle) -> bool {
    Settings::load(&app).upload_confirmed
}

#[tauri::command]
pub fn get_onboarding_complete(app: AppHandle) -> bool {
    Settings::load(&app).onboarding_complete
}

#[tauri::command]
pub fn set_upload_data_allowed(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.upload_confirmed = confirmed;
    settings.save(&app)
}

#[tauri::command]
pub fn set_onboarding_complete(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.onboarding_complete = confirmed;
    settings.save(&app)
}
