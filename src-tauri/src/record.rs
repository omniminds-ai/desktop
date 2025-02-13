use crate::axtree;
#[cfg(not(target_os = "macos"))]
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::input;
use crate::logger::Logger;
#[cfg(target_os = "macos")]
use crate::macos_screencapture::MacOSScreenRecorder;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Local;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize, Deserialize)]
pub struct RecordingMeta {
    id: String,
    timestamp: String,
    duration_seconds: u64,
    status: String,
    title: String,
    description: String,
}

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
    pub recording_start_time: Mutex<Option<chrono::DateTime<chrono::Local>>>,
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
    let session_dir = recordings_dir.join(&timestamp);

    std::fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session directory: {}", e))?;

    Ok((session_dir, timestamp))
}

#[tauri::command]
pub async fn list_recordings(app: tauri::AppHandle) -> Result<Vec<RecordingMeta>, String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    if !recordings_dir.exists() {
        return Ok(Vec::new());
    }

    let mut recordings = Vec::new();
    for entry in fs::read_dir(&recordings_dir)
        .map_err(|e| format!("Failed to read recordings directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let meta_path = entry.path().join("meta.json");
        if meta_path.exists() {
            let meta_str = fs::read_to_string(&meta_path)
                .map_err(|e| format!("Failed to read meta file: {}", e))?;
            let meta: RecordingMeta = serde_json::from_str(&meta_str)
                .map_err(|e| format!("Failed to parse meta file: {}", e))?;
            recordings.push(meta);
        }
    }

    recordings.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(recordings)
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

    let (session_dir, timestamp) = get_session_path(&app)?;

    let video_path = session_dir.join("recording.mp4");

    // Create and save initial meta file
    let meta = RecordingMeta {
        id: timestamp.clone(),
        timestamp: Local::now().to_rfc3339(),
        duration_seconds: 0,
        status: "recording".to_string(),
        title: "Recording Session".to_string(),
        description: "".to_string(),
    };

    fs::write(
        session_dir.join("meta.json"),
        serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?,
    )
    .map_err(|e| format!("Failed to write meta file: {}", e))?;

    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    *quest_state.objectives_completed.lock().unwrap() = 0;
    *quest_state.recording_start_time.lock().unwrap() = Some(Local::now());

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
        *log_state = Some(Logger::new(session_dir.clone())?);
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
    quest_state: State<'_, QuestState>,
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

    let mut rec_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut recorder) = rec_state.take() {
        recorder.stop()?;
    }

    // Update meta file with duration
    if let Some(start_time) = *quest_state.recording_start_time.lock().unwrap() {
        let duration = Local::now().signed_duration_since(start_time).num_seconds() as u64;

        let recordings_dir = app
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?
            .join("recordings");

        // Find the most recent recording directory
        let mut entries: Vec<_> = fs::read_dir(&recordings_dir)
            .map_err(|e| format!("Failed to read recordings directory: {}", e))?
            .collect::<Result<_, _>>()
            .map_err(|e| format!("Failed to read directory entries: {}", e))?;

        entries
            .sort_by_key(|entry| std::cmp::Reverse(entry.metadata().unwrap().modified().unwrap()));

        if let Some(latest_dir) = entries.first() {
            let meta_path = latest_dir.path().join("meta.json");
            if meta_path.exists() {
                let meta_str = fs::read_to_string(&meta_path)
                    .map_err(|e| format!("Failed to read meta file: {}", e))?;
                let mut meta: RecordingMeta = serde_json::from_str(&meta_str)
                    .map_err(|e| format!("Failed to parse meta file: {}", e))?;

                meta.duration_seconds = duration;
                meta.status = "completed".to_string();

                fs::write(
                    &meta_path,
                    serde_json::to_string_pretty(&meta)
                        .map_err(|e| format!("Failed to serialize meta: {}", e))?,
                )
                .map_err(|e| format!("Failed to write meta file: {}", e))?;
            }
        }
    }

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
pub async fn get_recording_file(
    app: tauri::AppHandle,
    recording_id: String,
    filename: String,
    as_base64: Option<bool>,
) -> Result<String, String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    let file_path = recordings_dir.join(&filename);
    if !file_path.exists() {
        return Err(format!("File not found: {}", filename));
    }

    let mut file = File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    if as_base64 == Some(true) {
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(format!("data:video/mp4;base64,{}", BASE64.encode(&buffer)))
    } else {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(contents)
    }
}

#[tauri::command]
pub async fn write_file(
    app: tauri::AppHandle,
    path: String,
    content: String,
) -> Result<(), String> {
    // Create parent directories if they don't exist
    if let Some(parent) = std::path::Path::new(&path).parent() {
        create_dir_all(parent).map_err(|e| format!("Failed to create directories: {}", e))?;
    }

    fs::write(&path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn open_recording_folder(
    app: tauri::AppHandle,
    recording_id: String,
) -> Result<(), String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    if !recordings_dir.exists() {
        return Err(format!("Recording folder not found: {}", recording_id));
    }

    app.opener()
        .open_path(recordings_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("Failed to open folder: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(path.to_string_lossy().to_string())
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
