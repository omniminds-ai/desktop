use crate::axtree;
#[cfg(not(target_os = "macos"))]
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::input;
use crate::logger::Logger;
#[cfg(target_os = "macos")]
use crate::macos_screencapture::MacOSScreenRecorder;
use chrono::Local;
use display_info::DisplayInfo;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

enum Recorder {
    #[cfg(not(target_os = "macos"))]
    FFmpeg(FFmpegRecorder),
    #[cfg(target_os = "macos")]
    MacOS(MacOSScreenRecorder),
}

impl Recorder {
    fn start(&mut self) -> Result<(), String> {
        match self {
            #[cfg(not(target_os = "macos"))]
            Recorder::FFmpeg(recorder) => recorder.start(),
            #[cfg(target_os = "macos")]
            Recorder::MacOS(recorder) => recorder.start(),
        }
    }

    fn stop(&mut self) -> Result<(), String> {
        match self {
            #[cfg(not(target_os = "macos"))]
            Recorder::FFmpeg(recorder) => recorder.stop(),
            #[cfg(target_os = "macos")]
            Recorder::MacOS(recorder) => recorder.stop(),
        }
    }

    fn new(video_path: &PathBuf, primary: &DisplayInfo) -> Result<Self, String> {
        #[cfg(target_os = "macos")]
        {
            return Ok(Recorder::MacOS(MacOSScreenRecorder::new(
                video_path.to_path_buf(),
            )));
        }

        #[cfg(not(target_os = "macos"))]
        {
            let (input_format, input_device) = {
                #[cfg(target_os = "windows")]
                {
                    ("gdigrab", "desktop".to_string())
                }
                #[cfg(target_os = "linux")]
                {
                    ("x11grab", ":0.0".to_string())
                }
                #[cfg(not(any(target_os = "windows", target_os = "linux")))]
                {
                    return Err("Unsupported platform".to_string());
                }
            };

            Ok(Recorder::FFmpeg(FFmpegRecorder::new_with_input(
                primary.width,
                primary.height,
                30,
                video_path.to_path_buf(),
                input_format.to_string(),
                input_device,
            )))
        }
    }
}

#[derive(Default)]
pub struct QuestState {
    pub objectives_completed: Mutex<i32>,
}

// Global state for recording and logging
lazy_static::lazy_static! {
    static ref RECORDING_STATE: Arc<Mutex<Option<Recorder>>> = Arc::new(Mutex::new(None));
    static ref LOGGER_STATE: Arc<Mutex<Option<Logger>>> = Arc::new(Mutex::new(None));
}

fn get_session_path(app: &tauri::AppHandle) -> Result<(PathBuf, String), String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    std::fs::create_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    Ok((recordings_dir, timestamp))
}

#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    quest_state: State<'_, QuestState>,
) -> Result<(), String> {
    // Start screen recording
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if rec_state.is_some() {
        return Err("Recording already in progress".to_string());
    }

    // Initialize FFmpeg if not already done
    #[cfg(not(target_os = "macos"))]
    ffmpeg::init_ffmpeg()?;

    // Get paths for both video and log files
    let (recordings_dir, timestamp) = get_session_path(&app)?;
    let video_path = recordings_dir.join(format!("recording_{}.mp4", timestamp));

    // Get primary display info
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    // Reset quest state and emit recording started event
    *quest_state.objectives_completed.lock().unwrap() = 0;
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "recording"
        }),
    )
    .unwrap();

    let mut recorder = Recorder::new(&video_path, &primary)?;

    recorder.start()?;
    *rec_state = Some(recorder);

    // Start input logging and listening
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    if log_state.is_none() {
        *log_state = Some(Logger::new(&app)?);
    }

    // Start input listener
    input::start_input_listener(app.clone())?;

    // Start dump-tree polling
    axtree::start_dump_tree_polling(app.clone())?;

    Ok(())
}

#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    _quest_state: State<'_, QuestState>,
) -> Result<(), String> {
    // Emit recording stopping event
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "stopping"
        }),
    )
    .unwrap();

    // Stop input logging and listening first
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    *log_state = None;

    // Stop input listener
    input::stop_input_listener()?;

    // Stop dump-tree polling
    axtree::stop_dump_tree_polling()?;

    // Stop screen recording last since it might hang
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut recorder) = rec_state.take() {
        recorder.stop()?;
    }

    // Emit recording stopped event
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "stopped"
        }),
    )
    .unwrap();

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
#[cfg(not(target_os = "macos"))]
pub fn log_ffmpeg(output: &str, is_stderr: bool) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_ffmpeg(output, is_stderr)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn request_record_perms(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        //todo: save settings.permissions.screen.requested so we don't overdo this
        let output_path = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?
            .join("tmp/perm");

        println!(
            "[MacOS Recorder] Creating temp file {} for recording permissions.",
            output_path.to_str().unwrap()
        );
        let mut process = Command::new("screencapture");
        process.args(["-x", output_path.to_str().unwrap()]);
        let _ = process.spawn();
        // remove the temp file
        println!(
            "[MacoOS Recorder] Removing {}. Permissions dialog triggered.",
            output_path.to_str().unwrap()
        );
        let _ = fs::remove_file(output_path.to_str().unwrap());
    }
    Ok(())
}
