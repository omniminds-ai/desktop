use crate::tools::ffmpeg::{get_ffmpeg_dir, get_ffprobe_dir};
use crate::utils::github_release;
use log::info;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use tauri::{AppHandle, Manager, Url};

static PIPELINE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "windows")]
const PIPELINE_URL: &str =
    "https://github.com/omniminds-ai/analyze-training/releases/latest/download/analyze-training-win-x64.exe";

#[cfg(target_os = "linux")]
const PIPELINE_URL: &str =
    "https://github.com/omniminds-ai/analyze-training/releases/latest/download/analyze-training-linux-x64";

#[cfg(target_os = "macos")]
const PIPELINE_URL: &str =
    "https://github.com/omniminds-ai/analyze-training/releases/latest/download/analyze-training-macos-arm64";

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("omniminds-desktop");
    temp
}

pub fn init_pipeline() -> Result<(), String> {
    if PIPELINE_PATH.get().is_some() {
        info!("[Pipeline] Already initialized");
        return Ok(());
    }

    info!("[Pipeline] Initializing pipeline");

    // Extract repo owner and name from the URL
    let url_parser = Url::parse(PIPELINE_URL).map_err(|e| format!("Failed to parse URL: {}", e))?;
    let path_segments: Vec<&str> = url_parser.path_segments().unwrap().collect();

    // For URLs like "https://github.com/omniminds-ai/analyze-training/releases/latest/download/analyze-training-macos-arm64"
    // path_segments will be ["omniminds-ai", "releases", "latest", "download", "analyze-training-win-x64.exe"]
    let repo_owner = path_segments[0];
    let repo_name = path_segments[1];

    // Get the temp directory
    let temp_dir = get_temp_dir();

    // Use the github_release module to get the latest release
    let pipeline_path = github_release::get_latest_release(
        repo_owner,
        repo_name,
        PIPELINE_URL,
        &temp_dir,
        true, // Make executable on Linux/macOS
    )?;

    info!("[Pipeline] Using pipeline at {}", pipeline_path.display());
    PIPELINE_PATH.set(pipeline_path).unwrap();
    Ok(())
}

pub fn process_recording(app: &AppHandle, recording_id: &str) -> Result<(), String> {
    let pipeline = PIPELINE_PATH
        .get()
        .ok_or_else(|| "pipeline not initialized".to_string())?;

    // Get the recording folder path using app.path() like record.rs
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings")
        .join(recording_id);

    info!(
        "[Pipeline] Processing recording at {}",
        recordings_dir.display()
    );

    // Run the pipeline command from the temp directory so it can find ffmpeg/ffprobe
    let temp_dir = get_temp_dir();

    let mut command = Command::new(pipeline);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }

    //todo: check if ffmpeg and ffprobe exist in the path before getting the custom dir.
    let ffmpeg_dir = get_ffmpeg_dir();
    let ffprobe_dir = get_ffprobe_dir();
    let output = command
        .current_dir(temp_dir)
        .arg("-f")
        .arg("desktop")
        .arg("-i")
        .arg(&recordings_dir)
        .arg("--ffmpeg")
        .arg(ffmpeg_dir)
        .arg("--ffprobe")
        .arg(ffprobe_dir)
        .output()
        .map_err(|e| format!("Failed to execute pipeline: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pipeline failed: {}", error));
    }

    info!("[Pipeline] Successfully processed recording");
    Ok(())
}
