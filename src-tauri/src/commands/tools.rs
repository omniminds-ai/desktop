use crate::tools::{axtree, ffmpeg, pipeline};
use log::error;
use serde_json;
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::Emitter;

#[tauri::command]
pub async fn init_tools(app: tauri::AppHandle) -> Result<(), String> {
    // Create a vector to store thread handles
    let mut handles = Vec::new();

    // Create shared error storage
    let errors = Arc::new(Mutex::new(Vec::new()));

    // Spawn thread for FFmpeg initialization
    {
        let errors = Arc::clone(&errors);
        let handle = thread::spawn(move || {
            if let Err(e) = ffmpeg::init_ffmpeg_and_ffprobe() {
                let mut errors = errors.lock().unwrap();
                errors.push(format!("Failed to initialize FFmpeg/FFprobe: {}", e));
            }
        });
        handles.push(handle);
    }

    // Spawn thread for dump-tree initialization
    {
        let errors = Arc::clone(&errors);
        let handle = thread::spawn(move || {
            if let Err(e) = axtree::init_dump_tree() {
                let mut errors = errors.lock().unwrap();
                errors.push(format!("Failed to initialize dump-tree: {}", e));
            }
        });
        handles.push(handle);
    }

    // Spawn thread for pipeline initialization
    {
        let errors = Arc::clone(&errors);
        let handle = thread::spawn(move || {
            if let Err(e) = pipeline::init_pipeline() {
                let mut errors = errors.lock().unwrap();
                errors.push(format!("Failed to initialize pipeline: {}", e));
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        if let Err(e) = handle.join() {
            error!("Thread panicked: {:?}", e);
        }
    }

    // Check if there were any errors
    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        for err in errors.iter() {
            error!("{}", err);
        }
        let _ = app.emit(
            "init_tools_errors",
            serde_json::json!({
                "errors": errors.to_vec()
            }),
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn check_tools() -> Result<serde_json::Value, String> {
    let temp_dir = std::env::temp_dir().join("viralmind-desktop");

    // Check for ffmpeg
    let ffmpeg_path = temp_dir.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });
    let ffmpeg_exists = ffmpeg_path.exists();

    // Check for ffprobe
    let ffprobe_path = temp_dir.join(if cfg!(windows) {
        "ffprobe.exe"
    } else {
        "ffprobe"
    });
    let ffprobe_exists = ffprobe_path.exists();

    // Check for dump-tree
    let dump_tree_path = temp_dir.join(if cfg!(windows) {
        "dump-tree-windows-x64.exe"
    } else if cfg!(target_os = "macos") {
        "dump-tree-macos-arm64"
    } else {
        "dump-tree-linux-x64-arm64.js"
    });
    let dump_tree_exists = dump_tree_path.exists();

    // Check for pipeline
    let pipeline_path = temp_dir.join(if cfg!(windows) {
        "pipeline-win-x64.exe"
    } else if cfg!(target_os = "macos") {
        "pipeline-macos-arm64"
    } else {
        "pipeline-linux-x64"
    });
    let pipeline_exists = pipeline_path.exists();

    // Return the status of each tool
    Ok(serde_json::json!({
        "ffmpeg": ffmpeg_exists,
        "ffprobe": ffprobe_exists,
        "dump_tree": dump_tree_exists,
        "pipeline": pipeline_exists
    }))
}
