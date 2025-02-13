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

        // Ensure the path ends with .mp4
        let output_path = if self
            .output_path
            .extension()
            .map_or(true, |ext| ext != "mp4")
        {
            let mut new_path = self.output_path.with_extension("");
            new_path.set_extension("mp4");
            new_path
        } else {
            self.output_path.clone()
        };

        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
        }

        // Start recording using screencapture
        let process = Command::new("screencapture")
            .args([
                "-v", // Video recording mode
                "-C", // Capture cursor
                "-x", // Do not play sounds
                output_path.to_str().unwrap(),
            ])
            .stdin(std::process::Stdio::piped()) // Add this to handle stdin
            .stdout(std::process::Stdio::piped()) // Add this to capture output
            .stderr(std::process::Stdio::piped()) // Add this to capture errors
            .spawn()
            .map_err(|e| format!("Failed to start screencapture: {}", e))?;

        self.process = Some(process);

        // Give it a moment to initialize
        thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), String> {
        println!("[MacOS Recorder] Stopping recording");
        if let Some(mut process) = self.process.take() {
            // Try to write a newline to stdin to stop the recording
            if let Some(mut stdin) = process.stdin.take() {
                use std::io::Write;
                if let Err(e) = stdin.write_all(b"\n") {
                    println!("[MacOS Recorder] Warning: Failed to write to stdin: {}", e);
                }
            }

            // Wait briefly for the process to exit naturally
            thread::sleep(Duration::from_secs(2));

            // If it hasn't exited, force kill it
            match process.try_wait() {
                Ok(None) => {
                    // Process still running, kill it
                    if let Err(e) = process.kill() {
                        println!("[MacOS Recorder] Warning: Failed to kill process: {}", e);
                    }
                    // Wait for it to actually terminate
                    if let Err(e) = process.wait() {
                        println!("[MacOS Recorder] Warning: Error waiting for process: {}", e);
                    }
                }
                Ok(Some(_)) => {
                    // Process already exited
                }
                Err(e) => {
                    println!(
                        "[MacOS Recorder] Warning: Error checking process status: {}",
                        e
                    );
                }
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
