use crate::core::input;
use crate::tools::axtree;
use crate::tools::ffmpeg::{init_ffmpeg, FFmpegRecorder, FFMPEG_PATH, FFPROBE_PATH};
use crate::tools::pipeline;
use crate::utils::logger::Logger;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Local;
use display_info::DisplayInfo;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use zip::{write::FileOptions, ZipWriter};

#[derive(Serialize, Deserialize, Clone)]
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
    monitor: MonitorInfo,
    quest: Option<Quest>,
}

#[derive(Serialize, Deserialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    task_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QuestReward {
    time: i64,
    max_reward: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MonitorInfo {
    width: u32,
    height: u32,
    x: i32,
    y: i32
}

enum Recorder {
    // #[cfg(not(target_os = "macos"))]
    FFmpeg(FFmpegRecorder),
    // #[cfg(target_os = "macos")]
    // MacOS(MacOSScreenRecorder),
}

impl Recorder {
    fn start(&mut self) -> Result<(), String> {
        match self {
            // #[cfg(not(target_os = "macos"))]
            Recorder::FFmpeg(recorder) => recorder.start(),
            // #[cfg(target_os = "macos")]
            // Recorder::MacOS(recorder) => recorder.start(),
        }
    }

    fn stop(&mut self) -> Result<(), String> {
        match self {
            // #[cfg(not(target_os = "macos"))]
            Recorder::FFmpeg(recorder) => recorder.stop(),
            // #[cfg(target_os = "macos")]
            // Recorder::MacOS(recorder) => recorder.stop(),
        }
    }

    fn new(video_path: &PathBuf, display: &DisplayInfo) -> Result<Self, String> {
        log::info!("[record] Starting new recorder");
        // #[cfg(target_os = "macos")]
        // {
        //     return Ok(Recorder::MacOS(MacOSScreenRecorder::new(
        //         video_path.to_path_buf(),
        //         primary,
        //     )));
        // }

        // #[cfg(not(target_os = "macos"))]
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
                #[cfg(target_os = "macos")]
                {
                    // Run ffmpeg to list availabkle devices
                    let ffmpeg = FFMPEG_PATH.get().ok_or_else(|| {
                        log::info!("[FFmpeg] Error: FFmpeg not initialized");
                        PathBuf::from("ffmpeg")
                    });
                    let output =
                        Command::new(ffmpeg.unwrap_or(&PathBuf::from("ffmpeg")).as_os_str())
                            .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
                            .output()
                            .map_err(|e| {
                                format!("Failed to execute ffmpeg to list devices: {}", e)
                            })?;

                    let output_str = String::from_utf8_lossy(&output.stderr);

                    log::info!("[record] FFmpeg screen devices output:\n{}", output_str);

                    // Find the screen capture device
                    let mut screen_device_index = None;

                    // Parse the output to find the screen capture device
                    for line in output_str.lines() {
                        if line.contains("Capture screen") {
                            // This is a screen capture device
                            log::info!("[record] Found screen capture line: {}", line);

                            // Find the opening bracket
                            if let Some(first_bracket) = line.find('[') {
                                // Find the second opening bracket
                                if let Some(start_idx) = line[first_bracket + 1..].find('[') {
                                    // Adjust index to be relative to the original string
                                    let start_idx = first_bracket + 1 + start_idx;

                                    // Find the closing bracket after the second opening bracket
                                    if let Some(end_idx) = line[start_idx + 1..].find(']') {
                                        // Extract the content between brackets
                                        let number_str =
                                            &line[start_idx + 1..start_idx + 1 + end_idx];

                                        // Parse as integer
                                        if let Ok(index) = number_str.parse::<i32>() {
                                            screen_device_index = Some(index);
                                            log::info!(
                                                "[record] Found screen capture device at index: {}",
                                                index
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Format the input device string - just the video device index with a colon
                    let input_device = if let Some(index) = screen_device_index {
                        format!("{}:", index)
                    } else {
                        log::info!("[record] No screen capture device found.");
                        log::info!("[record] Defualting to device [1].");
                        // Fallback to a default if no screen capture device found
                        "1".to_string() // Common default for screen capture
                    };

                    ("avfoundation", input_device)
                }
                #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
                {
                    return Err("Unsupported platform".to_string());
                }
            };

            Ok(Recorder::FFmpeg(FFmpegRecorder::new_with_input(
                display.x,
                display.y,
                display.width,
                display.height,
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
    pub current_quest: Mutex<Option<Quest>>,
}

// Global state for recording and logging and overlay
lazy_static::lazy_static! {
    static ref RECORDER_STATE: Arc<Mutex<Option<Recorder>>> = Arc::new(Mutex::new(None));
    static ref RECORDING_STATE: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(Some("off".to_string())));
    static ref OVERLAY_WINDOW_STATE: Mutex<Option<tauri::WebviewWindow>> = Mutex::new(None);
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

pub fn set_rec_state(
    app: &tauri::AppHandle,
    state: String,
    id: Option<String>,
) -> Result<(), String> {
    let mut recording_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    *recording_state = Some(state.clone());
    if id.is_some() {
        app.emit(
            "recording-status",
            serde_json::json!({
                "state": state,
                    "id": id
            }),
        )
        .unwrap();
    } else {
        app.emit(
            "recording-status",
            serde_json::json!({
                "state": state
            }),
        )
        .unwrap();
    }
    Ok(())
}

pub async fn get_recording_state() -> Result<String, String> {
    let recording_state = RECORDING_STATE.lock().map_err(|e| e.to_string())?;
    recording_state
        .as_ref()
        .map(|s| s.clone())
        .ok_or_else(|| "Recording state not initialized".to_string())
}

pub async fn start_recording(
    app: tauri::AppHandle,
    quest_state: State<'_, QuestState>,
    quest: Option<Quest>,
    display: Option<MonitorInfo>,
) -> Result<(), String> {
    // Start screen recording
    let mut recorder_state = RECORDER_STATE.lock().map_err(|e| e.to_string())?;
    if recorder_state.is_some() {
        set_rec_state(&app, "recording".to_string(), None)?;
        return Err("Recording already in progress".to_string());
    }

    set_rec_state(&app, "starting".to_string(), None)?;

    //processing screen recording
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let recorded_display = match display
    {
        Some(m) => {
            displays
                .iter()
                .find(|d| d.x == m.x && d.y == m.y)
                .or_else(|| displays.first())
                .ok_or_else(|| "No displays found".to_string())?
        },
        None => {
            displays
                .iter()
                .find(|d| d.is_primary)
                .or_else(|| displays.first())
                .ok_or_else(|| "No displays found".to_string())?
        }
    };

    // Initialize FFmpeg
    init_ffmpeg()?;

    create_overlay_window(&app, recorded_display)?;

    // Store quest data in state if available
    if let Some(quest_data) = &quest {
        // Store in QuestState for later retrieval
        *quest_state.current_quest.lock().unwrap() = Some(quest_data.clone());

        // Also emit the event for backward compatibility
        app.emit(
            "quest-overlay",
            serde_json::json!({
                "quest": quest_data
            }),
        )
        .map_err(|e| format!("Failed to emit quest data: {}", e))?;
    }

    let (session_dir, timestamp) = get_session_path(&app)?;

    // Store the recording ID
    *quest_state.current_recording_id.lock().unwrap() = Some(timestamp.clone());

    let video_path = session_dir.join("recording.mp4");

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
        monitor: MonitorInfo {
            width: recorded_display.width,
            height: recorded_display.height,
            x: recorded_display.x,
            y: recorded_display.y
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

    set_rec_state(&app, "recording".to_string(), None)?;

    let mut recorder = Recorder::new(&video_path, &recorded_display)?;
    recorder.start()?;
    *recorder_state = Some(recorder);

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

pub async fn stop_recording(
    app: tauri::AppHandle,
    quest_state: State<'_, QuestState>,
    reason: Option<String>,
) -> Result<String, String> {
    // Emit recording stopping event
    set_rec_state(&app, "stopping".to_string(), None)?;

    // Stop input logging and listening first
    let mut log_state = LOGGER_STATE.lock().map_err(|e| e.to_string())?;
    *log_state = None;

    // Stop input listener
    input::stop_input_listener()?;

    // Stop dump-tree polling
    axtree::stop_dump_tree_polling()?;

    let mut rec_state = RECORDER_STATE.lock().map_err(|e| e.to_string())?;
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

    // destroy the overlay window
    let mut overlay_state = OVERLAY_WINDOW_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(window) = overlay_state.take() {
        window
            .close()
            .map_err(|e| format!("Failed to close overlay window: {}", e))?;
    }

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

    // Clear the current quest
    *quest_state.current_quest.lock().unwrap() = None;

    // Get the recording ID from state
    if let Some(recording_id) = quest_state.current_recording_id.lock().unwrap().take() {
        set_rec_state(&app, "saved".to_string(), Some(recording_id.clone()))?;
        set_rec_state(&app, "off".to_string(), None)?;

        Ok(recording_id.to_string())
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

// #[cfg(not(target_os = "macos"))]
pub fn log_ffmpeg(output: &str, is_stderr: bool) -> Result<(), String> {
    if let Ok(mut state) = LOGGER_STATE.lock() {
        if let Some(logger) = state.as_mut() {
            logger.log_ffmpeg(output, is_stderr)?;
        }
    }
    Ok(())
}

pub async fn get_recording_file(
    app: tauri::AppHandle,
    recording_id: String,
    filename: String,
    as_base64: Option<bool>,
    as_path: Option<bool>,
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
    } else if as_path == Some(true) {
        Ok(file_path.to_str().ok_or("Invalid path")?.to_string())
    } else {
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(contents)
    }
}

pub async fn process_recording(app: tauri::AppHandle, recording_id: String) -> Result<(), String> {
    pipeline::process_recording(&app, &recording_id)
}

pub async fn write_file(
    _app: tauri::AppHandle,
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

pub async fn write_recording_file(
    app: tauri::AppHandle,
    recording_id: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    // Get the path to the recording directory
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    // Check if the recording directory exists
    if !recordings_dir.exists() {
        return Err(format!("Recording folder not found: {}", recording_id));
    }

    // Create the full file path
    let file_path = recordings_dir.join(&filename);

    // Write the content to the file
    fs::write(&file_path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

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

pub async fn delete_recording(app: tauri::AppHandle, recording_id: String) -> Result<(), String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    if !recordings_dir.exists() {
        return Err(format!("Recording folder not found: {}", recording_id));
    }

    fs::remove_dir_all(&recordings_dir)
        .map_err(|e| format!("Failed to delete recording: {}", e))?;

    Ok(())
}

#[derive(Deserialize, Debug)]
pub struct PrivateRange {
    start: f64,
    end: f64,
    #[allow(dead_code)]
    count: i32, // this is needed for processing, but not accessed in Rust
}

// Helper function to read and parse a JSON file
fn read_json_file<T: DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(|e| format!("Failed to parse JSON: {}", e))
}

// Helper function to process video with FFmpeg to black out private ranges
fn process_video_with_private_ranges(
    input_path: &PathBuf,
    output_path: &PathBuf,
    private_ranges: &[PrivateRange],
) -> Result<(), String> {
    // If no private ranges, just copy the file
    if private_ranges.is_empty() {
        fs::copy(input_path, output_path)
            .map_err(|e| format!("Failed to copy video file: {}", e))?;
        return Ok(());
    }

    // Get ffmpeg path
    let ffmpeg = FFMPEG_PATH
        .get()
        .ok_or_else(|| "FFmpeg not initialized".to_string())?;

    // Get video duration using ffprobe
    log::info!(
        "[process_video] Getting video duration for {}",
        input_path.display()
    );
    let ffprobe = FFPROBE_PATH
        .get()
        .ok_or_else(|| "FFprobe not initialized".to_string())?;

    let mut command = Command::new(ffprobe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }
    let duration_output = command
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            input_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe for duration: {}", e))?;

    let duration_str = String::from_utf8_lossy(&duration_output.stdout);
    log::info!("[process_video] Duration output: '{}'", duration_str.trim());

    if duration_str.trim().is_empty() {
        return Err(format!(
            "Failed to get video duration: empty output from ffprobe"
        ));
    }

    let duration: f64 = duration_str.trim().parse().map_err(|e| {
        format!(
            "Failed to parse video duration '{}': {}",
            duration_str.trim(),
            e
        )
    })?;
    log::info!("[process_video] Video duration: {} seconds", duration);

    // Get video resolution using ffprobe
    log::info!("[process_video] Getting video resolution");

    let mut r_o_command = Command::new(ffprobe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }

    let resolution_output = r_o_command
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=s=x:p=0",
            input_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe for resolution: {}", e))?;

    let resolution_str = String::from_utf8_lossy(&resolution_output.stdout);
    let resolution_parts: Vec<&str> = resolution_str.trim().split('x').collect();
    if resolution_parts.len() != 2 {
        return Err(format!(
            "Failed to parse video resolution: {}",
            resolution_str
        ));
    }

    let width: u32 = resolution_parts[0]
        .parse()
        .map_err(|e| format!("Failed to parse video width: {}", e))?;
    let height: u32 = resolution_parts[1]
        .parse()
        .map_err(|e| format!("Failed to parse video height: {}", e))?;

    // Convert milliseconds to seconds for FFmpeg
    let ranges: Vec<(f64, f64)> = private_ranges
        .iter()
        .map(|range| (range.start / 1000.0, range.end / 1000.0))
        .collect();

    // Sort ranges by start time
    let mut sorted_ranges = ranges.clone();
    sorted_ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Merge overlapping ranges
    let mut merged_ranges: Vec<(f64, f64)> = Vec::new();
    for range in sorted_ranges {
        if merged_ranges.is_empty() {
            // First range
            merged_ranges.push(range);
        } else {
            let last_idx = merged_ranges.len() - 1;
            let (last_start, last_end) = merged_ranges[last_idx];

            if range.0 <= last_end {
                // Ranges overlap, merge them
                merged_ranges[last_idx] = (last_start, f64::max(last_end, range.1));
            } else {
                // No overlap, add as new range
                merged_ranges.push(range);
            }
        }
    }

    // Create filter for each segment
    let mut filter_parts = Vec::new();
    let mut segment_inputs = Vec::new();
    let mut last_end = 0.0;
    let mut segment_index = 0;

    log::info!(
        "[process_video] Building filter graph for {} private ranges",
        merged_ranges.len()
    );

    for (start, end) in merged_ranges {
        // Ensure start and end are within video bounds
        let start = start.max(0.0).min(duration);
        let end = end.max(0.0).min(duration);

        if start >= end {
            log::info!(
                "[process_video] Skipping invalid range: start={}, end={}",
                start,
                end
            );
            continue;
        }

        if start > last_end {
            // Add normal segment before private range
            log::info!(
                "[process_video] Adding normal segment: {}s to {}s",
                last_end,
                start
            );
            filter_parts.push(format!(
                "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS[v{}]",
                last_end, start, segment_index
            ));
            segment_inputs.push(format!("[v{}]", segment_index));
            segment_index += 1;
        }

        // Add blacked out segment for private range
        let black_duration = end - start;
        log::info!(
            "[process_video] Adding black segment: {}s to {}s (duration: {}s)",
            start,
            end,
            black_duration
        );
        filter_parts.push(format!(
            "color=c=black:s={}x{}:d={}[v{}]",
            width, height, black_duration, segment_index
        ));
        segment_inputs.push(format!("[v{}]", segment_index));
        segment_index += 1;

        last_end = end;
    }

    // Add final segment after last private range if needed
    if last_end < duration {
        log::info!(
            "[process_video] Adding final segment: {}s to {}s",
            last_end,
            duration
        );
        filter_parts.push(format!(
            "[0:v]trim=start={}:end={},setpts=PTS-STARTPTS[v{}]",
            last_end, duration, segment_index
        ));
        segment_inputs.push(format!("[v{}]", segment_index));
        segment_index += 1;
    }

    // If no segments were created (e.g., all ranges were invalid), just copy the video
    if segment_index == 0 {
        log::info!("[process_video] No valid segments to process, copying video file");
        fs::copy(input_path, output_path)
            .map_err(|e| format!("Failed to copy video file: {}", e))?;
        return Ok(());
    }

    // Create concat filter
    let concat_filter = format!(
        "{}concat=n={}:v=1:a=0[outv]",
        segment_inputs.join(""),
        segment_index
    );

    // Build complete filter graph
    let filter_graph = format!("{};{}", filter_parts.join(";"), concat_filter);

    log::info!("[process_video] Filter graph: {}", filter_graph);

    // Execute FFmpeg command
    log::info!("[process_video] Executing FFmpeg command");

    let mut ffmpeg_command = Command::new(ffmpeg);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        ffmpeg_command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }

    let status = ffmpeg_command
        .args([
            "-i",
            input_path.to_str().unwrap(),
            "-filter_complex",
            &filter_graph,
            "-map",
            "[outv]",
            "-c:v",
            "libx264",
            "-preset",
            "fast",
            "-y", // Overwrite output file if it exists
            output_path.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("Failed to execute FFmpeg: {}", e))?;

    if !status.success() {
        return Err(format!("FFmpeg process failed with status: {}", status));
    }

    Ok(())
}

// Helper function to filter input log events based on private ranges
fn filter_input_log(
    input_path: &PathBuf,
    output_path: &PathBuf,
    private_ranges: &[PrivateRange],
) -> Result<(), String> {
    let input_file =
        File::open(input_path).map_err(|e| format!("Failed to open input log: {}", e))?;
    let mut reader = BufReader::new(input_file);

    // Read all lines into memory
    let mut lines = Vec::new();
    let mut line = String::new();
    while reader
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read line: {}", e))?
        > 0
    {
        if !line.trim().is_empty() {
            lines.push(line.clone());
        }
        line.clear();
    }

    // Get reference timestamp from first event
    let reference_timestamp = if !lines.is_empty() {
        let first_line = &lines[0];
        let first_event: serde_json::Value = serde_json::from_str(first_line)
            .map_err(|e| format!("Failed to parse first event JSON: {}", e))?;

        first_event
            .get("time")
            .and_then(|t| t.as_i64())
            .unwrap_or(0)
    } else {
        0 // Default if no lines
    };

    log::info!(
        "[filter_input_log] Using reference timestamp: {}",
        reference_timestamp
    );

    let output_file =
        File::create(output_path).map_err(|e| format!("Failed to create output file: {}", e))?;
    let mut writer = BufWriter::new(output_file);

    // Process each line
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON event
        let event: serde_json::Value = serde_json::from_str(&line)
            .map_err(|e| format!("Failed to parse event JSON: {}", e))?;

        // Check if the event has a timestamp
        if let Some(timestamp) = event.get("time").and_then(|t| t.as_i64()) {
            // Calculate relative timestamp by subtracting reference timestamp
            let relative_timestamp = timestamp - reference_timestamp;
            let relative_timestamp_f64 = relative_timestamp as f64;

            // Check if this relative timestamp falls within any private range
            let is_private = private_ranges.iter().any(|range| {
                relative_timestamp_f64 >= range.start && relative_timestamp_f64 <= range.end
            });

            // If not private, write to output
            if !is_private {
                writeln!(writer, "{}", line)
                    .map_err(|e| format!("Failed to write to output: {}", e))?;
            }
        } else {
            // If no timestamp, include the event
            writeln!(writer, "{}", line)
                .map_err(|e| format!("Failed to write to output: {}", e))?;
        }
    }

    writer
        .flush()
        .map_err(|e| format!("Failed to flush output: {}", e))?;
    Ok(())
}

pub async fn create_recording_zip(
    app: tauri::AppHandle,
    recording_id: String,
) -> Result<Vec<u8>, String> {
    log::info!(
        "[create_recording_zip] Starting to create zip for recording ID: {}",
        recording_id
    );

    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(&recording_id);

    log::info!(
        "[create_recording_zip] Recording directory: {}",
        recordings_dir.display()
    );

    // Create a buffer to store the zip file
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    log::info!("[create_recording_zip] Initialized zip writer with Stored compression method");

    // Check if private_ranges.json exists
    let private_ranges_path = recordings_dir.join("private_ranges.json");
    let has_private_ranges = private_ranges_path.exists();
    log::info!(
        "[create_recording_zip] Private ranges file exists: {}",
        has_private_ranges
    );

    // Create temp directory for processed files if needed
    let temp_dir = if has_private_ranges {
        let temp_path = recordings_dir.join("temp_private");
        log::info!(
            "[create_recording_zip] Creating temp directory for private ranges processing: {}",
            temp_path.display()
        );
        create_dir_all(&temp_path)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;
        Some(temp_path)
    } else {
        None
    };

    // Process files with private ranges if needed
    if let Some(temp_dir) = &temp_dir {
        // Read private ranges
        log::info!(
            "[create_recording_zip] Reading private ranges from: {}",
            private_ranges_path.display()
        );
        let private_ranges: Vec<PrivateRange> = read_json_file(&private_ranges_path)?;
        log::info!(
            "[create_recording_zip] Found {} private ranges to process",
            private_ranges.len()
        );

        // Filter input_log.jsonl
        let input_log_path = recordings_dir.join("input_log.jsonl");
        let temp_input_log_path = temp_dir.join("input_log.jsonl");
        log::info!(
            "[create_recording_zip] Filtering input log from {} to {}",
            input_log_path.display(),
            temp_input_log_path.display()
        );
        filter_input_log(&input_log_path, &temp_input_log_path, &private_ranges)?;

        // Process video (blackout frames in private ranges)
        let video_path = recordings_dir.join("recording.mp4");
        let temp_video_path = temp_dir.join("recording.mp4");
        log::info!(
            "[create_recording_zip] Processing video with private ranges from {} to {}",
            video_path.display(),
            temp_video_path.display()
        );
        process_video_with_private_ranges(&video_path, &temp_video_path, &private_ranges)?;
    }

    // Add files to zip
    let filenames = ["input_log.jsonl", "meta.json", "recording.mp4"];
    log::info!(
        "[create_recording_zip] Adding {} files to zip archive",
        filenames.len()
    );

    for filename in filenames {
        let file_path = if has_private_ranges && filename != "meta.json" && temp_dir.is_some() {
            // Use temp files for input_log and recording
            temp_dir.as_ref().unwrap().join(filename)
        } else {
            // Use original meta.json
            recordings_dir.join(filename)
        };

        log::info!(
            "[create_recording_zip] Processing file: {} from path: {}",
            filename,
            file_path.display()
        );

        if !file_path.exists() {
            log::info!(
                "[create_recording_zip] ERROR: File not found: {}",
                file_path.display()
            );
            return Err(format!("File not found: {}", filename));
        }

        // Get file size for logging
        let metadata = fs::metadata(&file_path)
            .map_err(|e| format!("Failed to get metadata for {}: {}", filename, e))?;
        let file_size = metadata.len();
        log::info!(
            "[create_recording_zip] Adding file to zip: {} (size: {} bytes)",
            filename,
            file_size
        );

        let mut file =
            File::open(&file_path).map_err(|e| format!("Failed to open {}: {}", filename, e))?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)
            .map_err(|e| format!("Failed to read {}: {}", filename, e))?;

        log::info!(
            "[create_recording_zip] Read {} bytes from {}",
            contents.len(),
            filename
        );

        zip.start_file(filename, options)
            .map_err(|e| format!("Failed to add {} to zip: {}", filename, e))?;
        zip.write_all(&contents)
            .map_err(|e| format!("Failed to write {} to zip: {}", filename, e))?;

        log::info!(
            "[create_recording_zip] Successfully added {} to zip archive",
            filename
        );
    }

    // Finish zip file
    log::info!("[create_recording_zip] Finalizing zip archive");
    let buf = zip
        .finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?
        .into_inner();

    let zip_size = buf.len();
    log::info!(
        "[create_recording_zip] Zip archive created successfully, size: {} bytes",
        zip_size
    );

    // Clean up temp directory if it was created
    // if let Some(temp_dir) = temp_dir {
    //     log::info!("[create_recording_zip] Cleaning up temp directory: {}", temp_dir.display());
    //     match fs::remove_dir_all(&temp_dir) {
    //         Ok(_) => log::info!("[create_recording_zip] Successfully removed temp directory"),
    //         Err(e) => log::info!("[create_recording_zip] Warning: Failed to remove temp directory: {}", e),
    //     }
    // }

    log::info!(
        "[create_recording_zip] Completed creating zip for recording ID: {}",
        recording_id
    );
    Ok(buf)
}

pub async fn export_recording_zip(id: String, app: tauri::AppHandle) -> Result<String, String> {
    let buf = create_recording_zip(app.clone(), id.clone()).await;
    let selected_dir = app.dialog().file().blocking_pick_folder();

    // If user cancels the dialog, selected_dir will be None
    if let Some(dir_path) = selected_dir {
        // Create the full path for history.zip
        let dir_path_str = dir_path.to_string();
        let file_path = Path::new(&dir_path_str).join(format!("export_recording_{}.zip", id));

        // Write the buffer to the file
        std::fs::write(&file_path, buf?).map_err(|e| format!("Failed to write zip file: {}", e))?;

        Ok(file_path.to_string_lossy().into_owned())
    } else {
        Ok("".to_string())
    }
}

pub async fn get_app_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    let path = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}

pub async fn get_current_quest(
    quest_state: State<'_, QuestState>,
) -> Result<Option<Quest>, String> {
    let current_quest = quest_state
        .current_quest
        .lock()
        .map_err(|e| e.to_string())?;
    Ok(current_quest.clone())
}

fn create_overlay_window(app: &tauri::AppHandle, recorded_monitor :&DisplayInfo) -> Result<(), String> {
    log::info!("Starting to create overlay window");

    // display info
    log::info!(
        "Using display: {}x{} at position ({},{})",
        recorded_monitor.width,
        recorded_monitor.height,
        recorded_monitor.x,
        recorded_monitor.y
    );

    // set overlay locations for specific platforms
    let overlay_w = 280.0;
    let overlay_h = 280.0;
    let overlay_x = recorded_monitor.x as f64 + recorded_monitor.width as f64 - overlay_w;
    let overlay_y = recorded_monitor.y as f64;

    log::info!(
        "Overlay window dimensions: {}x{} at position ({},{})",
        overlay_w,
        overlay_h,
        overlay_x,
        overlay_y
    );

    // create the overlay window
    let overlay_window =
        tauri::WebviewWindowBuilder::new(app, "overlay", tauri::WebviewUrl::App("overlay".into()))
            .transparent(true)
            .always_on_top(true)
            .decorations(false)
            .focused(false)
            .shadow(false)
            .position(overlay_x, overlay_y)
            .inner_size(overlay_w, overlay_h)
            .skip_taskbar(true)
            .resizable(false)
            .visible_on_all_workspaces(true)
            .build();

    match overlay_window {
        Ok(window) => {
            log::info!("Successfully created overlay window");

            // Check if the window is visible
            if let Ok(visible) = window.is_visible() {
                log::info!("Overlay window visibility: {}", visible);
            } else {
                log::warn!("Failed to check overlay window visibility");
            }

            // Store the window in the global state
            if let Ok(mut overlay_state) = OVERLAY_WINDOW_STATE.lock() {
                log::info!("Storing overlay window in global state");
                *overlay_state = Some(window);
            } else {
                log::error!("Failed to acquire lock for overlay window state");
                return Err("Failed to store overlay window: mutex lock failed".to_string());
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to create overlay window: {}", e);

            Err(format!("Failed to create overlay window: {}", e))
        }
    }
}
