use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use log::info;
use tauri::{AppHandle, Manager, Url};

static PIPELINE_PATH: OnceLock<PathBuf> = OnceLock::new();

#[cfg(target_os = "windows")]
const PIPELINE_URL: &str =
    "https://github.com/viralmind-ai/vm-pipeline/releases/latest/download/pipeline-win-x64.exe";

#[cfg(target_os = "linux")]
const PIPELINE_URL: &str =
    "https://github.com/viralmind-ai/vm-pipeline/releases/latest/download/pipeline-linux-x64";

#[cfg(target_os = "macos")]
const PIPELINE_URL: &str =
    "https://github.com/viralmind-ai/vm-pipeline/releases/latest/download/pipeline-macos-arm64";

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("viralmind-desktop");
    temp
}

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    info!(
        "[Pipeline] Downloading file from {} to {}",
        url,
        path.display()
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send().map_err(|e| {
        info!("[Pipeline] Error: Failed to download pipeline: {}", e);
        format!("Failed to download pipeline: {}", e)
    })?;

    let bytes = resp.bytes().map_err(|e| {
        info!("[Pipeline] Error: Failed to get response bytes: {}", e);
        format!("Failed to get response bytes: {}", e)
    })?;

    fs::write(path, bytes).map_err(|e| {
        info!("[Pipeline] Error: Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    info!(
        "[Pipeline] Successfully downloaded file to {}",
        path.display()
    );
    Ok(())
}

pub fn init_pipeline() -> Result<(), String> {
    if PIPELINE_PATH.get().is_some() {
        info!("[Pipeline] Already initialized");
        return Ok(());
    }

    info!("[Pipeline] Initializing pipeline");

    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| {
        info!("[Pipeline] Error: Failed to create temp directory: {}", e);
        format!("Failed to create temp directory: {}", e)
    })?;

    let url_parser = Url::parse(PIPELINE_URL).map_err(|e| format!("Failed to parse URL: {}", e))?;

    let pipeline_filename = url_parser
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or_else(|| "Invalid URL format".to_string())?;

    let pipeline_path = temp_dir.join(pipeline_filename);

    if !pipeline_path.exists() {
        info!("[Pipeline] Downloading pipeline binary");
        download_file(PIPELINE_URL, &pipeline_path)?;

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&pipeline_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        }
    }

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
    let output = Command::new(pipeline)
        .current_dir(temp_dir)
        .arg("-f")
        .arg("desktop")
        .arg("-i")
        .arg(&recordings_dir)
        .output()
        .map_err(|e| format!("Failed to execute pipeline: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Pipeline failed: {}", error));
    }

    info!("[Pipeline] Successfully processed recording");
    Ok(())
}
