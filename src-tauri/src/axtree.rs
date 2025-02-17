use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Url;

static DUMP_TREE_PATH: OnceLock<PathBuf> = OnceLock::new();
static POLLING_ACTIVE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

#[cfg(target_os = "windows")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-0.0.1-windows-x64.exe";

#[cfg(target_os = "linux")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-0.0.1-linux-x64-arm64.js";

#[cfg(target_os = "macos")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-0.0.1-macos-arm64";

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("viralmind-desktop");
    temp
}

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    println!(
        "[AxTree] Downloading file from {} to {}",
        url,
        path.display()
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send().map_err(|e| {
        println!("[AxTree] Error: Failed to download dump-tree: {}", e);
        format!("Failed to download dump-tree: {}", e)
    })?;

    let bytes = resp.bytes().map_err(|e| {
        println!("[AxTree] Error: Failed to get response bytes: {}", e);
        format!("Failed to get response bytes: {}", e)
    })?;

    fs::write(path, bytes).map_err(|e| {
        println!("[AxTree] Error: Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    println!(
        "[AxTree] Successfully downloaded file to {}",
        path.display()
    );
    Ok(())
}

pub fn init_dump_tree() -> Result<(), String> {
    // todo: should somehow version the scripts in case theres a new release for the ax parsing
    if DUMP_TREE_PATH.get().is_some() {
        println!("[AxTree] Already initialized");
        return Ok(());
    }

    println!("[AxTree] Initializing dump-tree");

    // Initialize polling state
    POLLING_ACTIVE.get_or_init(|| Arc::new(Mutex::new(false)));

    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| {
        println!("[AxTree] Error: Failed to create temp directory: {}", e);
        format!("Failed to create temp directory: {}", e)
    })?;

    // Get the filename from the URL for proper version tracking
    let _response = reqwest::blocking::get(DUMP_TREE_URL)
        .map_err(|e| format!("Failed to check latest version: {}", e))?;

    let url_parser =
        Url::parse(DUMP_TREE_URL).map_err(|e| format!("Failed to parse URL: {}", e))?;

    let dump_tree_filename = url_parser
        .path_segments()
        .and_then(|segments| segments.last())
        .ok_or_else(|| "Invalid URL format".to_string())?;

    let dump_tree_path = temp_dir.join(dump_tree_filename);

    println!("{}", dump_tree_filename);

    let should_download = if !dump_tree_path.exists() {
        true
    } else {
        // Extract versions and compare
        let current_version = dump_tree_path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|name| name.split('-').nth(2))
            .unwrap_or("0.0.0");

        let new_version = dump_tree_filename.split('-').nth(2).unwrap_or("0.0.0");

        current_version < new_version
    };

    if should_download {
        println!("[AxTree] Downloading new version: {}", dump_tree_filename);
        download_file(DUMP_TREE_URL, &dump_tree_path)?;

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&dump_tree_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        }
    }

    println!("[AxTree] Using dump-tree at {}", dump_tree_path.display());
    DUMP_TREE_PATH.set(dump_tree_path).unwrap();
    Ok(())
}

pub fn start_dump_tree_polling(_: tauri::AppHandle) -> Result<(), String> {
    let dump_tree = DUMP_TREE_PATH
        .get()
        .ok_or_else(|| "dump-tree not initialized".to_string())?
        .clone();

    let polling_active = POLLING_ACTIVE
        .get()
        .ok_or_else(|| "Polling state not initialized".to_string())?;
    *polling_active.lock().unwrap() = true;

    println!("[AxTree] Starting dump-tree polling");

    thread::spawn(move || {
        while *POLLING_ACTIVE.get().unwrap().lock().unwrap() {
            // Run dump-tree and capture output
            let process = Command::new(&dump_tree)
                .arg("-e")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match process {
                Ok(mut child) => {
                    let stdout = child.stdout.take();
                    let mut child_owned = child;

                    if let Some(stdout) = stdout {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                // Try to parse as JSON
                                if let Ok(mut json) = serde_json::from_str::<Value>(&line) {
                                    // Modify the event field
                                    if let Some(obj) = json.as_object_mut() {
                                        obj.insert("event".to_string(), json!("axtree"));
                                        // Log the modified event
                                        let _ = crate::record::log_input(json!(obj));
                                    }
                                }
                            }
                        }
                    }

                    // Wait for process to finish
                    let _ = child_owned.wait();
                }
                Err(e) => {
                    println!("[AxTree] Error running dump-tree: {}", e);
                }
            }

            // Sleep for 5 seconds before next poll
            thread::sleep(Duration::from_secs(5));
        }
        println!("[AxTree] Stopped dump-tree polling");
    });

    Ok(())
}

pub fn stop_dump_tree_polling() -> Result<(), String> {
    println!("[AxTree] Stopping dump-tree polling");

    if let Some(polling_active) = POLLING_ACTIVE.get() {
        *polling_active.lock().unwrap() = false;
    }
    Ok(())
}
