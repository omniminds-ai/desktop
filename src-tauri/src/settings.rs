use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WSLConfig {
    pub distro: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub private_key_path: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSettings {
    pub backend: String,
    pub wsl_config: Option<WSLConfig>,
    pub ssh_config: Option<SSHConfig>,
    pub downloads_path: String,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            backend: String::new(),
            wsl_config: None,
            ssh_config: None,
            downloads_path: "~/.viralmind/downloads".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StorageSettings {
    pub recordings_path: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub upload_confirmed: bool,
    pub engines: Option<EngineSettings>,
    pub storage: Option<StorageSettings>,
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
pub fn set_upload_confirmed(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.upload_confirmed = confirmed;
    settings.save(&app)
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    Settings::load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    settings.save(&app)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_wsl_distros() -> Result<String, String> {
    use std::process::Command;
    
    let output = Command::new("wsl")
        .args(["--list", "--quiet"])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "Command returned with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    // Clean the output by removing carriage returns and empty lines
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let cleaned = stdout
        .lines()
        .map(|line| line.trim().replace("\r", ""))
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join("\n");
    
    Ok(cleaned)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_default_wsl_distro() -> Result<String, String> {
    use std::process::Command;
    
    // Get the default distribution by parsing the output of 'wsl -l -v'
    // The default distribution has a '*' next to it
    let output = Command::new("wsl")
        .args(["-l", "-v"])
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;
    
    if !output.status.success() {
        return Err(format!(
            "Command returned with error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    // Find the line with '*' indicating default distro
    for line in stdout.lines().skip(1) { // Skip header line
        // Clean the line by removing carriage returns
        let cleaned = line.replace("\r", "");
        let trimmed = cleaned.trim();
        
        if trimmed.contains('*') {
            // Extract distro name (2nd column)
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                // Return the cleaned distro name
                return Ok(parts[1].to_string().trim().to_string());
            }
        }
    }
    
    // If no default was found, return empty string
    Ok("".to_string())
}

#[tauri::command]
pub fn select_directory() -> Result<String, String> {
    // Note: This function will be replaced by using dialog plugin from the frontend
    Err("Use dialog plugin from frontend".into())
}

#[tauri::command]
pub fn set_onboarding_complete(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.onboarding_complete = confirmed;
    settings.save(&app)
}
