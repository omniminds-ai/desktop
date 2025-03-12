use crate::github_release;
use log::info;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
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

pub fn init_dump_tree() -> Result<(), String> {
    if DUMP_TREE_PATH.get().is_some() {
        println!("[AxTree] Already initialized");
        return Ok(());
    }

    println!("[AxTree] Initializing dump-tree");

    // Initialize polling state
    POLLING_ACTIVE.get_or_init(|| Arc::new(Mutex::new(false)));

    // Extract repo owner and name from the GitHub API URL
    let url_parts: Vec<&str> = GITHUB_API_URL.split('/').collect();
    let repo_owner = url_parts[4];
    let repo_name = url_parts[5];

    // Get the temp directory
    let temp_dir = get_temp_dir();

    // Use the github_release module to get the latest release
    let dump_tree_path = github_release::get_latest_release(
        repo_owner,
        repo_name,
        DUMP_TREE_URL,
        &temp_dir,
        true, // Make executable on Linux/macOS
    )?;

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
            let mut command = Command::new(&dump_tree);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
            }
            let proc = command
                .arg("-e")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match proc {
                Ok(mut child) => {
                    // Create separate threads for handling stdout and stderr
                    let stdout_thread = if let Some(stdout) = child.stdout.take() {
                        let stdout_handle = thread::spawn(move || {
                            let reader = BufReader::new(stdout);
                            for line in reader.lines() {
                                if let Ok(line) = line {
                                    // info!("[AxTree] STDOUT line: {}", line);
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
