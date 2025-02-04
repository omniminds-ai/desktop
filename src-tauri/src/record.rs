use crate::axtree;
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::logger::Logger;
use chrono::Local;
use display_info::DisplayInfo;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};

#[derive(Default)]
pub struct QuestState {
    pub objectives_completed: Mutex<i32>,
}

// Global state for recording and logging
lazy_static::lazy_static! {
    static ref RECORDING_STATE: Arc<Mutex<Option<FFmpegRecorder>>> = Arc::new(Mutex::new(None));
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
    // Check permission before starting
    // match check_screen_capture_permission() {
    //     PermissionStatus::Granted => {
    //         // proceed with recording
    //     }
    //     PermissionStatus::SystemDialogNeeded => {
    //         match request_screen_capture_permission() {
    //             PermissionStatus::Granted => {
    //                 // proceed with recording
    //             }
    //             _ => return Err("Failed to obtain screen recording permission.".to_string()),
    //         }
    //     }
    //     _ => return Err("Screen recording permission denied.".to_string()),
    // }

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

    println!("{}", video_path.display());

    // Get primary display info
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    let mut width: u32 = primary.width;
    let mut height: u32 = primary.height;

    // Handle retina macos screens
    if cfg!(target_os = "macos") {
        let (w, h) = get_screen_resolution();
        if w == 0 && h == 0 {
            width = primary.width;
            height = primary.height;
        } else {
            width = w;
            height = h;
        };
        println!("{}x{}", width, height);
    }

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
        width,
        height,
        30,
        video_path,
        input_format.to_string(),
        if cfg!(target_os = "macos") {
            "1:none".to_string() // Screen:audio on macOS
        } else if cfg!(target_os = "windows") {
            "desktop".to_string()
        } else {
            ":0.0".to_string() // X11 display
        },
    );
    recorder.start()?;
    *rec_state = Some(recorder);

    // Reset quest state and emit recording started event
    *quest_state.objectives_completed.lock().unwrap() = 0;
    app.emit(
        "recording-status",
        serde_json::json!({
            "state": "recording"
        }),
    )
    .unwrap();

    // Start input logging
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    if log_state.is_none() {
        *log_state = Some(Logger::new(&app)?);
    }

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

    // Stop screen recording
    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut recorder) = rec_state.take() {
        recorder.stop()?;
    }

    // Stop input logging
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    *log_state = None;

    // Stop dump-tree polling
    axtree::stop_dump_tree_polling()?;

    // Emit recording stopped event after FFmpeg finishes
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

pub fn log_ffmpeg(output: &str, is_stderr: bool) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_ffmpeg(output, is_stderr)?;
        }
    }
    Ok(())
}

fn get_screen_resolution() -> (u32, u32) {
    let output = match Command::new("system_profiler")
        .arg("SPDisplaysDataType")
        .output()
    {
        Ok(output) => output,
        Err(_) => return (0, 0),
    };

    let output_str = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };

    for line in output_str.lines() {
        if line.contains("Resolution:") && line.contains("Retina") {
            if let Some(resolution_part) = line.split(':').nth(1) {
                let parts: Vec<&str> = resolution_part.trim().split(" x ").collect();

                if parts.len() >= 2 {
                    if let (Ok(w), Ok(h)) = (
                        parts[0].trim().parse(),
                        parts[1].split_whitespace().next().unwrap_or("0").parse(),
                    ) {
                        return (w, h);
                    }
                }
            }
        }
    }

    (0, 0)
}
