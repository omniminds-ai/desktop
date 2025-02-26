use crate::axtree;
#[cfg(not(target_os = "macos"))]
use crate::ffmpeg::{self, FFmpegRecorder};
use crate::input;
use crate::logger::Logger;
#[cfg(target_os = "macos")]
use crate::macos_screencapture::MacOSScreenRecorder;
use crate::pipeline;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Local;
use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::io::{BufReader, Cursor, Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;
use zip::{write::FileOptions, ZipWriter};

#[derive(Serialize, Deserialize)]
pub struct RecordingMeta {
    id: String,
    timestamp: String,
    duration_seconds: u64,
    status: String,
    reason: Option<String>,
    title: String,
    description: String,
    platform: String,
    arch: String,
    version: String,
    locale: String,
    primary_monitor: MonitorInfo,
    quest: Option<Quest>,
}

#[derive(Serialize, Deserialize)]
pub struct Quest {
    title: String,
    app: String,
    icon_url: String,
    objectives: Vec<String>,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reward: Option<QuestReward>,
}

#[derive(Serialize, Deserialize)]
pub struct QuestReward {
    time: i64,
    max_reward: i64,
}

#[derive(Serialize, Deserialize)]
pub struct MonitorInfo {
    width: u32,
    height: u32,
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
    pub recording_start_time: Mutex<Option<chrono::DateTime<chrono::Local>>>,
    pub current_recording_id: Mutex<Option<String>>,
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
    quest: Option<Quest>,
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

    // Store the recording ID
    *quest_state.current_recording_id.lock().unwrap() = Some(timestamp.clone());

    let video_path = session_dir.join("recording.mp4");

    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    // Create and save initial meta file
    let meta = RecordingMeta {
        id: timestamp.clone(),
        timestamp: Local::now().to_rfc3339(),
        duration_seconds: 0,
        status: "recording".to_string(),
        title: if let Some(q) = &quest {
            q.title.clone()
        } else {
            "Recording Session".to_string()
        },
        description: "".to_string(),
        platform: tauri_plugin_os::platform().to_string(),
        arch: tauri_plugin_os::arch().to_string(),
        version: tauri_plugin_os::version().to_string(),
        locale: tauri_plugin_os::locale().unwrap_or_default(),
        primary_monitor: MonitorInfo {
            width: primary.width,
            height: primary.height,
        },
        reason: None,
        quest,
    };

    fs::write(
        session_dir.join("meta.json"),
        serde_json::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize meta: {}", e))?,
    )
    .map_err(|e| format!("Failed to write meta file: {}", e))?;

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
    reason: Option<String>,
) -> Result<String, String> {
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
                meta.reason = reason;

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

    // Find the most recent recording directory to get its ID
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    let mut entries: Vec<_> = fs::read_dir(&recordings_dir)
        .map_err(|e| format!("Failed to read recordings directory: {}", e))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Failed to read directory entries: {}", e))?;

    entries.sort_by_key(|entry| std::cmp::Reverse(entry.metadata().unwrap().modified().unwrap()));

    // Get the recording ID from state
    if let Some(recording_id) = quest_state.current_recording_id.lock().unwrap().take() {
        Ok(recording_id)
    } else {
        Err("No recording ID found".to_string())
    }
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
pub async fn process_recording(app: tauri::AppHandle, recording_id: String) -> Result<(), String> {
    pipeline::process_recording(&app, &recording_id)
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
    let mut recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");
    // only add the ID if requested
    if !recording_id.is_empty() {
        recordings_dir = recordings_dir.join(&recording_id);
    }

    if !recordings_dir.exists() {
        return Err(format!("Recording folder not found: {}", recording_id));
    }

    app.opener()
        .open_path(recordings_dir.to_string_lossy().to_string(), None::<&str>)
        .map_err(|e| format!("Failed to open folder: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn create_recording_zip(
    app: tauri::AppHandle,
    recording_id: String,
) -> Result<Vec<u8>, String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    // Create a buffer to store the zip file
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add required files to zip
    for filename in ["input_log.jsonl", "meta.json", "recording.mp4"] {
        let file_path = recordings_dir.join(filename);
        if !file_path.exists() {
            return Err(format!("File not found: {}", filename));
        }

        let mut file =
            File::open(&file_path).map_err(|e| format!("Failed to open {}: {}", filename, e))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", filename, e))?;

        zip.start_file(filename, options)
            .map_err(|e| format!("Failed to add {} to zip: {}", filename, e))?;
        zip.write_all(&contents)
            .map_err(|e| format!("Failed to write {} to zip: {}", filename, e))?;
    }

    // Finish zip file
    let buf = zip
        .finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?
        .into_inner();

    Ok(buf)
}

#[tauri::command]
pub async fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}
