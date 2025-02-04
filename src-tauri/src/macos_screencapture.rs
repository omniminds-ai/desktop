use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

pub struct MacOSScreenRecorder {
    output_path: PathBuf,
    process: Option<Child>,
}

impl MacOSScreenRecorder {
    pub fn new(output_path: PathBuf) -> Self {
        println!(
            "[MacOS Recorder] Creating new recorder. -> {}",
            output_path.display()
        );
        Self {
            output_path,
            process: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        println!("[MacOS Recorder] Starting recording.");

        // Ensure output directory exists
        if let Some(parent) = self.output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
        }

        // Start recording using screencapture
        let process = Command::new("screencapture")
            .args([
                "-v", // Enable video capture
                "-C", // Capture cursor
                "-x", // Do not play sounds
                self.output_path.to_str().unwrap(),
            ])
            .spawn()
            .map_err(|e| format!("Failed to start screencapture: {}", e))?;

        self.process = Some(process);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        println!("[MacOS Recorder] Stopping recording");
        if let Some(mut process) = self.process.take() {
            // Send SIGTERM to screencapture
            if let Err(e) = process.kill() {
                println!("[MacOS Recorder] Warning: Failed to kill process: {}", e);
            }

            // Give it a moment to finish writing
            thread::sleep(Duration::from_secs(1));

            // Check if output file exists and has size
            if !self.output_path.exists() {
                return Err("Failed to create output file".to_string());
            }

            let file_size = std::fs::metadata(&self.output_path)
                .map_err(|e| format!("Failed to get output file metadata: {}", e))?
                .len();

            if file_size == 0 {
                return Err("Created empty output file".to_string());
            }

            println!(
                "[MacOS Recorder] Recording saved successfully: {} ({} bytes)",
                self.output_path.display(),
                file_size
            );
        }
        Ok(())
    }
}
