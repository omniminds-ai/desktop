use std::sync::{Arc, Mutex};
use chrono::Local;
use std::path::PathBuf;
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::logger::Logger;
use display_info::DisplayInfo;
use tauri::Manager;

// Global state for recording and logging
lazy_static::lazy_static! {
    static ref RECORDING_STATE: Arc<Mutex<Option<FFmpegRecorder>>> = Arc::new(Mutex::new(None));
    static ref LOGGER_STATE: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
}

fn get_session_path(app: &tauri::AppHandle) -> Result<(PathBuf, String), String> {
    let recordings_dir = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");
    
    std::fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;
    
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    Ok((recordings_dir, timestamp))
}

#[tauri::command]
pub async fn start_recording(app: tauri::AppHandle) -> Result<(), String> {
    // Start screen recording
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if rec_state.is_some() {
        return Err("Recording already in progress".to_string());
    }

    // Initialize FFmpeg if not already done
    ffmpeg::init_ffmpeg()?;

    // Get paths for both video and log files
    let (recordings_dir, timestamp) = get_session_path(&app)?;
    let video_path = recordings_dir.join(format!("recording_{}.mp4", timestamp));
    
    // Get primary display info
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays.iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;
    
    // Create FFmpeg recorder with screen capture settings
    let input_format = if cfg!(target_os = "windows") {
        "gdigrab"
    } else if cfg!(target_os = "macos") {
        "avfoundation"
    } else if cfg!(target_os = "linux") {
        "x11grab"
    } else {
        return Err("Unsupported platform".to_string());
    };

    let mut recorder = FFmpegRecorder::new_with_input(
        primary.width as u32,
        primary.height as u32,
        30,
        video_path,
        input_format.to_string(),
        if cfg!(target_os = "macos") {
            "1:none".to_string() // Screen:audio on macOS
        } else if cfg!(target_os = "windows") {
            "desktop".to_string()
        } else {
            ":0.0".to_string() // X11 display
        }
    );
    recorder.start()?;
    *rec_state = Some(recorder);

    // Start input logging
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    if log_state.is_none() {
        *log_state = Some(Logger::new(&app)?);
    }

    Ok(())
}

#[tauri::command]
pub async fn stop_recording() -> Result<(), String> {
    // Stop screen recording
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut recorder) = rec_state.take() {
        recorder.stop()?;
    }

    // Stop input logging
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    *log_state = None;

    Ok(())
}

pub fn log_input(event: serde_json::Value) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_event(event)?;
        }
    }
    Ok(())
}

pub fn log_ffmpeg(output: &str, is_stderr: bool) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_ffmpeg(output, is_stderr)?;
        }
    }
    Ok(())
}
