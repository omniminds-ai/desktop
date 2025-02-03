use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::fs;
use std::io::Write;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();

const FFMPEG_URLS: &[(&str, &str)] = &[
    ("windows", "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"),
    ("linux", "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"),
    ("macos", "https://evermeet.cx/ffmpeg/ffmpeg-6.1.zip"),
];

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("gym-desktop");
    temp
}

fn download_file(url: &str, path: &Path) -> Result<(), String> {
    println!("[FFmpeg] Downloading file from {} to {}", url, path.display());
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url)
        .send()
        .map_err(|e| {
            println!("[FFmpeg] Error: Failed to download FFmpeg: {}", e);
            format!("Failed to download FFmpeg: {}", e)
        })?;
    
    let bytes = resp.bytes()
        .map_err(|e| {
            println!("[FFmpeg] Error: Failed to get response bytes: {}", e);
            format!("Failed to get response bytes: {}", e)
        })?;

    fs::write(path, bytes)
        .map_err(|e| {
            println!("[FFmpeg] Error: Failed to write file: {}", e);
            format!("Failed to write file: {}", e)
        })?;

    println!("[FFmpeg] Successfully downloaded file to {}", path.display());
    Ok(())
}

pub fn init_ffmpeg() -> Result<(), String> {
    if FFMPEG_PATH.get().is_some() {
        println!("[FFmpeg] Already initialized");
        return Ok(());
    }

    println!("[FFmpeg] Initializing FFmpeg");
    // First check if ffmpeg is in PATH
    if let Ok(output) = Command::new("ffmpeg").arg("-version").output() {
        if output.status.success() {
            println!("[FFmpeg] Found FFmpeg in system PATH");
            FFMPEG_PATH.set("ffmpeg".into()).unwrap();
            return Ok(());
        }
    }
    println!("[FFmpeg] FFmpeg not found in PATH, proceeding with download");

    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir)
        .map_err(|e| {
            println!("[FFmpeg] Error: Failed to create temp directory: {}", e);
            format!("Failed to create temp directory: {}", e)
        })?;

    let ffmpeg_path = temp_dir.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    
    if ffmpeg_path.exists() {
        // Try to verify the binary works by running version check
        match Command::new(&ffmpeg_path).arg("-version").output() {
            Ok(output) if output.status.success() => {
                println!("[FFmpeg] Using existing FFmpeg at {}", ffmpeg_path.display());
                FFMPEG_PATH.set(ffmpeg_path.clone()).unwrap();
                return Ok(());
            }
            _ => {
                println!("[FFmpeg] Warning: Existing FFmpeg binary appears corrupted, removing and downloading fresh copy");
                if let Err(e) = fs::remove_file(&ffmpeg_path) {
                    println!("[FFmpeg] Warning: Failed to remove corrupted binary: {}", e);
                }
            }
        }
    }

    // Download and extract FFmpeg
    let (os, url) = FFMPEG_URLS.iter()
        .find(|(os, _)| {
            match *os {
                "windows" => cfg!(windows),
                "linux" => cfg!(unix) && !cfg!(target_os = "macos"),
                "macos" => cfg!(target_os = "macos"),
                _ => false
            }
        })
        .ok_or_else(|| {
            println!("[FFmpeg] Error: Unsupported platform");
            "Unsupported platform".to_string()
        })?;
    
    println!("[FFmpeg] Downloading FFmpeg for {} from {}", os, url);
    
    let archive_path = temp_dir.join("ffmpeg.archive");
    download_file(url, &archive_path)?;

    println!("[FFmpeg] Extracting FFmpeg from archive");
    // Extract based on archive type
    if url.ends_with(".zip") {
        let file = fs::File::open(&archive_path)
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to open zip: {}", e);
                format!("Failed to open zip: {}", e)
            })?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to read zip: {}", e);
                format!("Failed to read zip: {}", e)
            })?;
        
        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| {
                    println!("[FFmpeg] Error: Failed to read zip entry: {}", e);
                    format!("Failed to read zip entry: {}", e)
                })?;
            
            // For Windows, look for bin/ffmpeg.exe in the release essentials package
            if file.name().contains("/bin/ffmpeg.exe") {
                println!("[FFmpeg] Found FFmpeg binary in zip: {}", file.name());
                let mut outfile = fs::File::create(&ffmpeg_path)
                    .map_err(|e| {
                        println!("[FFmpeg] Error: Failed to create ffmpeg: {}", e);
                        format!("Failed to create ffmpeg: {}", e)
                    })?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| {
                        println!("[FFmpeg] Error: Failed to extract ffmpeg: {}", e);
                        format!("Failed to extract ffmpeg: {}", e)
                    })?;
                found = true;
                break;
            }
        }
        
        if !found {
            println!("[FFmpeg] Error: Could not find ffmpeg.exe in zip archive");
            return Err("Could not find ffmpeg.exe in zip archive".to_string());
        }
    } else if url.ends_with(".tar.xz") {
        let file = fs::File::open(&archive_path)
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to open tar.xz: {}", e);
                format!("Failed to open tar.xz: {}", e)
            })?;
        let tar = xz2::read::XzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);
        
        let mut found = false;
        for entry in archive.entries()
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to read tar entries: {}", e);
                format!("Failed to read tar entries: {}", e)
            })? {
            let mut entry = entry.map_err(|e| {
                println!("[FFmpeg] Error: Failed to read tar entry: {}", e);
                format!("Failed to read tar entry: {}", e)
            })?;
            if entry.path()
                .map_err(|e| {
                    println!("[FFmpeg] Error: Failed to get entry path: {}", e);
                    format!("Failed to get entry path: {}", e)
                })?
                .to_string_lossy()
                .contains("ffmpeg") {
                println!("[FFmpeg] Found FFmpeg binary in tar.xz");
                let mut outfile = fs::File::create(&ffmpeg_path)
                    .map_err(|e| {
                        println!("[FFmpeg] Error: Failed to create ffmpeg: {}", e);
                        format!("Failed to create ffmpeg: {}", e)
                    })?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| {
                        println!("[FFmpeg] Error: Failed to extract ffmpeg: {}", e);
                        format!("Failed to extract ffmpeg: {}", e)
                    })?;
                found = true;
                break;
            }
        }
        
        if !found {
            println!("[FFmpeg] Error: Could not find ffmpeg binary in tar.xz archive");
            return Err("Could not find ffmpeg binary in tar.xz archive".to_string());
        }
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        println!("[FFmpeg] Setting executable permissions");
        fs::set_permissions(&ffmpeg_path, fs::Permissions::from_mode(0o755))
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to make ffmpeg executable: {}", e);
                format!("Failed to make ffmpeg executable: {}", e)
            })?;
    }

    println!("[FFmpeg] Cleaning up archive file");
    fs::remove_file(archive_path)
        .map_err(|e| {
            println!("[FFmpeg] Warning: Failed to cleanup archive: {}", e);
            format!("Failed to cleanup archive: {}", e)
        })?;

    // Verify the extracted binary works
    match Command::new(&ffmpeg_path).arg("-version").output() {
        Ok(output) if output.status.success() => {
            println!("[FFmpeg] Successfully verified binary");
            FFMPEG_PATH.set(ffmpeg_path).unwrap();
            Ok(())
        }
        _ => {
            println!("[FFmpeg] Error: Extracted binary verification failed");
            Err("Failed to verify extracted FFmpeg binary".to_string())
        }
    }
}

pub struct FFmpegRecorder {
    width: u32,
    height: u32,
    fps: u32,
    output_path: PathBuf,
    process: Option<std::process::Child>,
    input_format: Option<String>,
    input_device: Option<String>,
}

impl FFmpegRecorder {
    pub fn new_with_input(width: u32, height: u32, fps: u32, output_path: PathBuf, 
                         input_format: String, input_device: String) -> Self {
        println!("[FFmpeg] Creating new recorder with input format {}: {}x{} @ {} fps -> {}", 
            input_format, width, height, fps, output_path.display());
        Self {
            width,
            height,
            fps,
            output_path,
            process: None,
            input_format: Some(input_format),
            input_device: Some(input_device),
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        println!("[FFmpeg] Starting recording: {}x{} @ {} fps", self.width, self.height, self.fps);
        let ffmpeg = FFMPEG_PATH.get()
            .ok_or_else(|| {
                println!("[FFmpeg] Error: FFmpeg not initialized");
                "FFmpeg not initialized".to_string()
            })?;
        
        let mut args: Vec<String> = Vec::new();

        // Input format args
        if let (Some(format), Some(device)) = (&self.input_format, &self.input_device) {
            args.extend([
                "-f".to_string(),
                format.clone(),
                "-video_size".to_string(),
                format!("{}x{}", self.width, self.height),
                "-framerate".to_string(),
                self.fps.to_string(),
            ]);

            // Platform specific options
            if format == "gdigrab" {
                args.extend([
                    "-draw_mouse".to_string(),
                    "1".to_string(),
                    "-offset_x".to_string(),
                    "0".to_string(),
                    "-offset_y".to_string(),
                    "0".to_string(),
                    "-probesize".to_string(),
                    "10M".to_string(),
                    "-thread_queue_size".to_string(),
                    "1024".to_string(),
                ]);
            } else if format == "avfoundation" {
                args.extend([
                    "-capture_cursor".to_string(),
                    "1".to_string(),
                ]);
            }

            args.extend([
                "-i".to_string(),
                device.clone(),
            ]);
        } else {
            // Fallback to raw video input
            args.extend([
                "-f".to_string(),
                "rawvideo".to_string(),
                "-pixel_format".to_string(),
                "rgb24".to_string(),
                "-video_size".to_string(),
                format!("{}x{}", self.width, self.height),
                "-framerate".to_string(),
                self.fps.to_string(),
                "-i".to_string(),
                "-".to_string(), // Read from stdin
            ]);
        }

        // Output encoding args
        args.extend([
            "-c:v".to_string(),
            "libx264".to_string(),
            "-preset".to_string(),
            "ultrafast".to_string(),
            "-crf".to_string(),
            "23".to_string(), // Balance between quality and file size
            "-pix_fmt".to_string(),
            "yuv420p".to_string(), // Required for compatibility
            "-movflags".to_string(),
            "+faststart".to_string(), // Enable streaming playback
            "-profile:v".to_string(),
            "high".to_string(),
            "-tune".to_string(),
            "zerolatency".to_string(), // Reduce encoding latency
            "-y".to_string(), // Overwrite output file
            self.output_path.to_str().unwrap().to_string()
        ]);

        println!("[FFmpeg] Command: {} {}", ffmpeg.display(), args.join(" "));

        let mut process = Command::new(ffmpeg)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                println!("[FFmpeg] Error: Failed to start process: {}", e);
                format!("Failed to start FFmpeg: {}", e)
            })?;

        // Spawn threads to handle stdout and stderr in real-time
        if let Some(stdout) = process.stdout.take() {
            let stdout_reader = std::io::BufReader::new(stdout);
            thread::spawn(move || {
                use std::io::BufRead;
                for line in stdout_reader.lines() {
                    if let Ok(line) = line {
                        println!("[FFmpeg] stdout: {}", line);
                        let _ = crate::record::log_ffmpeg(&line, false);
                    }
                }
            });
        }

        if let Some(stderr) = process.stderr.take() {
            let stderr_reader = std::io::BufReader::new(stderr);
            thread::spawn(move || {
                use std::io::BufRead;
                for line in stderr_reader.lines() {
                    if let Ok(line) = line {
                        println!("[FFmpeg] stderr: {}", line);
                        let _ = crate::record::log_ffmpeg(&line, true);
                    }
                }
            });
        }

        // Check if process died immediately and capture any error output
        match process.try_wait() {
            Ok(Some(status)) => {
                // Process exited immediately
                let mut error_msg = format!("FFmpeg process exited immediately with status: {}", status);
                
                // Try to capture any error output
                if let Some(mut stderr) = process.stderr.take() {
                    let mut error_output = String::new();
                    if std::io::Read::read_to_string(&mut stderr, &mut error_output).is_ok() && !error_output.is_empty() {
                        error_msg = format!("{}\nFFmpeg error output: {}", error_msg, error_output);
                        
                        // Check for common Windows-specific errors
                        if cfg!(windows) {
                            if error_output.contains("Could not find video device") {
                                error_msg = format!("{}\nHint: On Windows, make sure you have permission to access screen recording.", error_msg);
                            } else if error_output.contains("Permission denied") {
                                error_msg = format!("{}\nHint: Try running the application as administrator.", error_msg);
                            }
                        }
                    }
                }
                
                // Cleanup any partial output file
                if self.output_path.exists() {
                    if let Err(e) = fs::remove_file(&self.output_path) {
                        println!("[FFmpeg] Warning: Failed to cleanup partial output file: {}", e);
                    }
                }
                
                println!("[FFmpeg] Error: {}", error_msg);
                return Err(error_msg);
            }
            Ok(None) => {
                // Process is still running, which is what we want
                println!("[FFmpeg] Process started successfully");
                
                // On Windows, verify we can write to the output directory
                if cfg!(windows) {
                    if let Some(parent) = self.output_path.parent() {
                        if !parent.exists() {
                            if let Err(e) = fs::create_dir_all(parent) {
                                let error_msg = format!("Failed to create output directory: {}", e);
                                println!("[FFmpeg] Error: {}", error_msg);
                                return Err(error_msg);
                            }
                        }
                        // Try creating a test file to verify write permissions
                        let test_file = parent.join(".test_write");
                        if let Err(e) = fs::write(&test_file, b"test") {
                            let error_msg = format!("No write permission in output directory: {}", e);
                            println!("[FFmpeg] Error: {}", error_msg);
                            return Err(error_msg);
                        }
                        let _ = fs::remove_file(test_file); // Cleanup test file
                    }
                }
                
                self.process = Some(process);
                Ok(())
            }
            Err(e) => {
                println!("[FFmpeg] Error: Failed to check process status: {}", e);
                Err(format!("Failed to check FFmpeg process status: {}", e))
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        println!("[FFmpeg] Stopping recording");
        if let Some(mut process) = self.process.take() {
            // Send 'q' to FFmpeg to stop recording gracefully
            if let Some(mut stdin) = process.stdin.take() {
                let _ = stdin.write_all(b"q");
            }

            println!("[FFmpeg] Waiting for process to finish");
            // Give FFmpeg time to finish writing
            thread::sleep(Duration::from_secs(2));
            
            // Wait for process to finish and capture output
            match process.wait_with_output() {
                Ok(output) => {
            // Output is already handled by the reader threads
                }
                Err(e) => {
                    println!("[FFmpeg] Warning: Failed to get process output: {}", e);
                }
            }

            // Check if output file exists and has size
            if !self.output_path.exists() {
                println!("[FFmpeg] Error: Failed to create output file at {}", self.output_path.display());
                return Err("FFmpeg failed to create output file".to_string());
            }
            
            let file_size = fs::metadata(&self.output_path)
                .map_err(|e| {
                    println!("[FFmpeg] Error: Failed to get output file metadata: {}", e);
                    format!("Failed to get output file metadata: {}", e)
                })?
                .len();
                
            if file_size == 0 {
                println!("[FFmpeg] Error: Created empty output file at {}", self.output_path.display());
                return Err("FFmpeg created empty output file".to_string());
            }

            println!("[FFmpeg] Recording saved successfully: {} ({} bytes)", self.output_path.display(), file_size);
        } else {
            println!("[FFmpeg] No active process to stop");
        }
        Ok(())
    }
}
