use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;

pub struct Logger {
    file: File,
}

impl Logger {
    pub fn new(session_dir: PathBuf) -> Result<Self, String> {
        let log_path = session_dir.join("input_log.jsonl");

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to create log file: {}", e))?;

        Ok(Logger { file })
    }

    pub fn log_event(&mut self, event: serde_json::Value) -> Result<(), String> {
        let json = serde_json::to_string(&event)
            .map_err(|e| format!("Failed to serialize event: {}", e))?;

        writeln!(self.file, "{}", json)
            .map_err(|e| format!("Failed to write to log file: {}", e))?;

        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn log_ffmpeg(&mut self, output: &str, is_stderr: bool) -> Result<(), String> {
        let event = serde_json::json!({
            "event": if is_stderr { "ffmpeg_stderr" } else { "ffmpeg_stdout" },
            "data": {
                "output": output
            },
            "time": chrono::Local::now().timestamp_millis()
        });

        self.log_event(event)
    }
}
