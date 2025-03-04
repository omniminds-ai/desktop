use chrono::DateTime;
use log::info;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static DUMP_TREE_PATH: OnceLock<PathBuf> = OnceLock::new();
static POLLING_ACTIVE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

#[cfg(target_os = "windows")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-windows-x64.exe";

#[cfg(target_os = "linux")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-linux-x64-arm64.js";

#[cfg(target_os = "macos")]
const DUMP_TREE_URL: &str = "https://github.com/viralmind-ai/ax-tree-parsers/releases/latest/download/dump-tree-macos-arm64";

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/viralmind-ai/ax-tree-parsers/releases/latest";

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("viralmind-desktop");
    temp
}

struct BinaryMetadata {
    version: String,
    build_timestamp: u64,
}

impl BinaryMetadata {
    fn new(version: String, build_timestamp: u64) -> Self {
        Self {
            version,
            build_timestamp,
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "build_timestamp": self.build_timestamp,
        })
    }

    fn from_json(json: &Value) -> Option<Self> {
        if let (Some(version), Some(build_timestamp)) = (
            json.get("version").and_then(Value::as_str),
            json.get("build_timestamp").and_then(Value::as_u64),
        ) {
            Some(Self::new(version.to_string(), build_timestamp))
        } else {
            None
        }
    }
}

fn save_metadata(path: &Path, metadata: &BinaryMetadata) -> Result<(), String> {
    let json = metadata.to_json();
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write metadata file: {}", e))?;

    info!("[AxTree] Saved metadata to {}", path.display());
    Ok(())
}

fn load_metadata(path: &Path) -> Result<Option<BinaryMetadata>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read metadata file: {}", e))?;

    let json: Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse metadata: {}", e))?;

    Ok(BinaryMetadata::from_json(&json))
}

fn fetch_latest_release_metadata() -> Result<BinaryMetadata, String> {
    info!("[AxTree] Fetching latest release metadata from GitHub API");

    let client = reqwest::blocking::Client::builder()
        .user_agent("viralmind-desktop")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(GITHUB_API_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    let json: Value = response
        .json()
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let version = json
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| "No tag name in release".to_string())?
        .to_string();

    // Use published_at timestamp from GitHub API
    let published_at = json
        .get("published_at")
        .and_then(Value::as_str)
        .ok_or_else(|| "No published_at in release".to_string())?;

    // Convert ISO 8601 date string to Unix timestamp
    let timestamp = {
        let dt = DateTime::parse_from_rfc3339(published_at)
            .map_err(|e| format!("Failed to parse published_at date: {}", e))?;

        dt.timestamp() as u64
    };

    info!(
        "[AxTree] Latest release: version={}, published_at={} ({})",
        version, published_at, timestamp
    );

    Ok(BinaryMetadata::new(version, timestamp))
}

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    info!(
        "[AxTree] Downloading file from {} to {}",
        url,
        path.display()
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send().map_err(|e| {
        info!("[AxTree] Error: Failed to download dump-tree: {}", e);
        format!("Failed to download dump-tree: {}", e)
    })?;

    let bytes = resp.bytes().map_err(|e| {
        info!("[AxTree] Error: Failed to get response bytes: {}", e);
        format!("Failed to get response bytes: {}", e)
    })?;

    fs::write(path, bytes).map_err(|e| {
        info!("[AxTree] Error: Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    info!(
        "[AxTree] Successfully downloaded file to {}",
        path.display()
    );
    Ok(())
}

pub fn init_dump_tree() -> Result<(), String> {
    if DUMP_TREE_PATH.get().is_some() {
        println!("[AxTree] Already initialized");
        return Ok(());
    }

    println!("[AxTree] Initializing dump-tree");

    // Initialize polling state
    POLLING_ACTIVE.get_or_init(|| Arc::new(Mutex::new(false)));

    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| {
        info!("[AxTree] Error: Failed to create temp directory: {}", e);
        format!("Failed to create temp directory: {}", e)
    })?;

    // Get the filename from the URL for proper version tracking
    let dump_tree_split: Vec<&str> = DUMP_TREE_URL.split("/").collect();
    let dump_tree_filename = dump_tree_split[DUMP_TREE_URL.split("/").count() - 1];

    let dump_tree_path = temp_dir.join(dump_tree_filename);
    let metadata_path = temp_dir.join(format!("{}.metadata.json", dump_tree_filename));

    // Fetch latest release metadata from GitHub
    let latest_metadata = fetch_latest_release_metadata()?;

    // Check if we need to download the binary
    let should_download = if !dump_tree_path.exists() {
        println!("[AxTree] Binary does not exist, downloading");
        true
    } else {
        // Load existing metadata
        let current_metadata = load_metadata(&metadata_path)?;

        match current_metadata {
            Some(metadata) => {
                // Compare build timestamps
                if metadata.build_timestamp < latest_metadata.build_timestamp {
                    println!(
                        "[AxTree] New version available: current={} ({}), latest={} ({})",
                        metadata.version,
                        metadata.build_timestamp,
                        latest_metadata.version,
                        latest_metadata.build_timestamp
                    );
                    true
                } else {
                    println!("[AxTree] Binary is up to date");
                    false
                }
            }
            None => {
                println!("[AxTree] No metadata found, downloading latest version");
                true
            }
        }
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

        // Save the metadata
        save_metadata(&metadata_path, &latest_metadata)?;
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

    info!("[AxTree] Starting dump-tree polling");

    thread::spawn(move || {
        info!("[AxTree] Polling thread started");
        while *POLLING_ACTIVE.get().unwrap().lock().unwrap() {
            info!("[AxTree] Starting new dump-tree process");

            // Run dump-tree and capture output
            let process = Command::new(&dump_tree)
                .arg("-e")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match process {
                Ok(mut child) => {
                    // Create separate threads for handling stdout and stderr
                    let stdout_thread = if let Some(stdout) = child.stdout.take() {
                        let stdout_handle = thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    info!("[AxTree] STDOUT line: {}", line);
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
                        });
                        Some(stdout_handle)
                    } else {
                        None
                    };

                    let stderr_thread = if let Some(stderr) = child.stderr.take() {
                        let stderr_handle = thread::spawn(move || {
                            let reader = BufReader::new(stderr);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    info!("[AxTree] STDERR line: {}", line);
                                }
                            }
                        });
                        Some(stderr_handle)
                    } else {
                        None
                    };

                    // Wait for process to finish
                    match child.wait() {
                        Ok(status) => info!("[AxTree] Process {}", status),
                        Err(e) => info!("[AxTree] Error waiting for process: {}", e),
                    }

                    // Wait for output processing to complete
                    if let Some(handle) = stdout_thread {
                        if let Err(e) = handle.join() {
                            info!("[AxTree] Error joining stdout thread: {:?}", e);
                        }
                    }

                    if let Some(handle) = stderr_thread {
                        if let Err(e) = handle.join() {
                            info!("[AxTree] Error joining stderr thread: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    info!("[AxTree] Error running dump-tree: {}", e);
                }
            }

            // Only sleep if we're still supposed to be polling
            if *POLLING_ACTIVE.get().unwrap().lock().unwrap() {
                info!("[AxTree] Sleeping for 2 seconds before next poll");
                thread::sleep(Duration::from_secs(2));
            }
        }
        info!("[AxTree] Stopped dump-tree polling");
    });

    Ok(())
}

pub fn stop_dump_tree_polling() -> Result<(), String> {
    info!("[AxTree] Stopping dump-tree polling");

    if let Some(polling_active) = POLLING_ACTIVE.get() {
        *polling_active.lock().unwrap() = false;
    }
    Ok(())
}
