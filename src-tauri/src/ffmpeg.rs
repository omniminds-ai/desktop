use crate::downloader::download_file;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

// #[cfg(not(target_os = "macos"))]
use {std::io::Write, std::process::Stdio, std::thread, std::time::Duration};

pub static FFMPEG_PATH: OnceLock<PathBuf> = OnceLock::new();
pub static FFPROBE_PATH: OnceLock<PathBuf> = OnceLock::new();

const FFMPEG_URLS: &[(&str, &str)] = &[
    ("windows", "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"),
    ("linux", "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz"),
    ("macos", "https://www.osxexperts.net/ffmpeg71intel.zip")

];

#[cfg(target_os = "macos")]
const FFPROBE_MACOS: &str = "https://www.osxexperts.net/ffprobe71intel.zip";

fn get_temp_dir() -> PathBuf {
    let mut temp = std::env::temp_dir();
    temp.push("viralmind-desktop");
    temp
}

/// Checks for ffmpeg in the PATH and in the temp directory
/// Returns a PathBuf containing the full file path if found, or an empty PathBuf if not found
pub fn get_ffmpeg_dir() -> PathBuf {
    // First check if ffmpeg is in PATH
    let mut command = Command::new("ffmpeg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }
    if let Ok(output) = command.arg("-version").output() {
        if output.status.success() {
            log::info!("[FFmpeg] Found FFmpeg in system PATH");
            return "ffmpeg".into();
        }
    }
    log::info!("[FFmpeg] FFmpeg not found in PATH, checking temp directory");

    // Check if ffmpeg exists in temp directory
    let temp_dir = get_temp_dir();
    if !temp_dir.exists() {
        return PathBuf::new();
    }

    let ffmpeg_path = temp_dir.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    if ffmpeg_path.exists() {
        return ffmpeg_path;
    }

    // Not found or not working
    PathBuf::new()
}

/// Checks for ffprobe in the PATH and in the temp directory
/// Returns a PathBuf containing the full file path if found, or an empty PathBuf if not found
pub fn get_ffprobe_dir() -> PathBuf {
    // First check if ffprobe is in PATH
    let mut command = Command::new("ffprobe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
    }
    if let Ok(output) = command.arg("-version").output() {
        if output.status.success() {
            log::info!("[FFmpeg] Found FFprobe in system PATH");
            return "ffprobe".into();
        }
    }
    log::info!("[FFmpeg] FFprobe not found in PATH, checking temp directory");

    // Check if ffprobe exists in temp directory
    let temp_dir = get_temp_dir();
    if !temp_dir.exists() {
        return PathBuf::new();
    }

    let ffprobe_path = temp_dir.join(if cfg!(windows) {
        "ffprobe.exe"
    } else {
        "ffprobe"
    });

    if ffprobe_path.exists() {
        // Verify the binary works
        return ffprobe_path;
    }

    // Not found or not working
    PathBuf::new()
}

/// Downloads and extracts a binary from an archive
fn download_and_extract_binary(
    url: &str,
    archive_path: &Path,
    binary_path: &Path,
    binary_name: &str,
    os_type: &str,
) -> Result<(), String> {
    log::info!(
        "[FFmpeg] Downloading {} for {} from {}",
        binary_name,
        os_type,
        url
    );

    // Download the archive
    download_file(url, archive_path)?;

    log::info!("[FFmpeg] Extracting {} from archive", binary_name);

    let mut found_binary = false;

    // Extract based on OS type
    if os_type.starts_with("windows") {
        let file = fs::File::open(archive_path).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to open zip: {}", e);
            format!("Failed to open zip: {}", e)
        })?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to read zip: {}", e);
            format!("Failed to read zip: {}", e)
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                log::info!("[FFmpeg] Error: Failed to read zip entry: {}", e);
                format!("Failed to read zip entry: {}", e)
            })?;

            let name = file.name();
            // For Windows, look for bin/ffmpeg.exe or bin/ffprobe.exe in the release essentials package
            if name.contains(&format!("/bin/{}.exe", binary_name)) {
                log::info!("[FFmpeg] Found {} binary in zip: {}", binary_name, name);
                let mut outfile = fs::File::create(binary_path).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to create {}: {}", binary_name, e);
                    format!("Failed to create {}: {}", binary_name, e)
                })?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to extract {}: {}", binary_name, e);
                    format!("Failed to extract {}: {}", binary_name, e)
                })?;
                found_binary = true;
                break;
            }
        }
    } else if os_type.starts_with("macos") {
        let file = fs::File::open(archive_path).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to open zip: {}", e);
            format!("Failed to open zip: {}", e)
        })?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to read zip: {}", e);
            format!("Failed to read zip: {}", e)
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                log::info!("[FFmpeg] Error: Failed to read zip entry: {}", e);
                format!("Failed to read zip entry: {}", e)
            })?;
            let name = file.name();

            if name.contains(binary_name) {
                log::info!("[FFmpeg] Found {} binary in zip: {}", binary_name, name);
                let mut outfile = fs::File::create(binary_path).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to create {}: {}", binary_name, e);
                    format!("Failed to create {}: {}", binary_name, e)
                })?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to extract {}: {}", binary_name, e);
                    format!("Failed to extract {}: {}", binary_name, e)
                })?;
                found_binary = true;
                break;
            }
        }
    } else if os_type.starts_with("linux") {
        let file = fs::File::open(archive_path).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to open tar.xz: {}", e);
            format!("Failed to open tar.xz: {}", e)
        })?;
        let tar = xz2::read::XzDecoder::new(file);
        let mut archive = tar::Archive::new(tar);

        for entry in archive.entries().map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to read tar entries: {}", e);
            format!("Failed to read tar entries: {}", e)
        })? {
            let mut entry = entry.map_err(|e| {
                log::info!("[FFmpeg] Error: Failed to read tar entry: {}", e);
                format!("Failed to read tar entry: {}", e)
            })?;
            let path = entry.path().map_err(|e| {
                log::info!("[FFmpeg] Error: Failed to get entry path: {}", e);
                format!("Failed to get entry path: {}", e)
            })?;
            let path_str = path.to_string_lossy();

            // For ffmpeg, we want to match "ffmpeg" but not "ffprobe"
            // For ffprobe, we just match "ffprobe"
            let is_match = if binary_name == "ffmpeg" {
                path_str.contains("ffmpeg") && !path_str.contains("ffprobe")
            } else {
                path_str.contains(binary_name)
            };

            if is_match {
                log::info!("[FFmpeg] Found {} binary in tar.xz", binary_name);
                let mut outfile = fs::File::create(binary_path).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to create {}: {}", binary_name, e);
                    format!("Failed to create {}: {}", binary_name, e)
                })?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to extract {}: {}", binary_name, e);
                    format!("Failed to extract {}: {}", binary_name, e)
                })?;
                found_binary = true;
                break;
            }
        }
    }

    if !found_binary {
        let error_msg = format!("Could not find {} binary in archive", binary_name);
        log::info!("[FFmpeg] Error: {}", error_msg);
        return Err(error_msg);
    }

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        log::info!(
            "[FFmpeg] Setting executable permissions for {}",
            binary_name
        );
        fs::set_permissions(binary_path, fs::Permissions::from_mode(0o755)).map_err(|e| {
            log::info!("[FFmpeg] Error: Failed to make binary executable: {}", e);
            format!("Failed to make binary executable: {}", e)
        })?;
    }

    // Clean up archive file
    log::info!("[FFmpeg] Cleaning up archive file");
    fs::remove_file(archive_path).map_err(|e| {
        log::info!("[FFmpeg] Warning: Failed to cleanup archive: {}", e);
        format!("Failed to cleanup archive: {}", e)
    })?;

    Ok(())
}

pub fn init_ffmpeg() -> Result<(), String> {
    if FFMPEG_PATH.get().is_some() {
        log::info!("[FFmpeg] FFmpeg already initialized");
        return Ok(());
    }

    log::info!("[FFmpeg] Initializing FFmpeg");

    // Check for existing ffmpeg
    let ffmpeg_path = get_ffmpeg_dir();

    // If ffmpeg binary is found and working, use it
    if !ffmpeg_path.as_os_str().is_empty() {
        log::info!(
            "[FFmpeg] Using existing FFmpeg binary at {}",
            ffmpeg_path.display()
        );
        FFMPEG_PATH.set(ffmpeg_path).unwrap();
        return Ok(());
    }

    // Need to download the binary
    log::info!("[FFmpeg] Need to download FFmpeg binary");

    // Ensure temp directory exists
    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| {
        log::info!("[FFmpeg] Error: Failed to create temp directory: {}", e);
        format!("Failed to create temp directory: {}", e)
    })?;

    // Define path for the binary we'll download
    let ffmpeg_path = temp_dir.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });

    // Remove any existing binary that might be corrupted
    if ffmpeg_path.exists() {
        log::info!("[FFmpeg] Removing existing FFmpeg binary for fresh download");
        let _ = fs::remove_file(&ffmpeg_path);
    }

    // Download and extract FFmpeg
    let (os, url) = FFMPEG_URLS
        .iter()
        .find(|(os, _)| match *os {
            "windows" => cfg!(windows),
            "linux" => cfg!(unix) && !cfg!(target_os = "macos"),
            "macos" => cfg!(target_os = "macos"),
            _ => false,
        })
        .ok_or_else(|| {
            log::info!("[FFmpeg] Error: Unsupported platform");
            "Unsupported platform".to_string()
        })?;

    let archive_path = temp_dir.join("ffmpeg.archive");
    download_and_extract_binary(url, &archive_path, &ffmpeg_path, "ffmpeg", os)?;

    // Set the path
    log::info!(
        "[FFmpeg] FFmpeg successfully initialized in {:?}",
        ffmpeg_path
    );
    FFMPEG_PATH.set(ffmpeg_path).unwrap();
    Ok(())
}

pub fn init_ffprobe() -> Result<(), String> {
    if FFPROBE_PATH.get().is_some() {
        log::info!("[FFmpeg] FFprobe already initialized");
        return Ok(());
    }

    log::info!("[FFmpeg] Initializing FFprobe");

    // Check for existing ffprobe
    let ffprobe_path = get_ffprobe_dir();

    // If ffprobe binary is found and working, use it
    if !ffprobe_path.as_os_str().is_empty() {
        log::info!(
            "[FFmpeg] Using existing FFprobe binary at {}",
            ffprobe_path.display()
        );
        FFPROBE_PATH.set(ffprobe_path).unwrap();
        return Ok(());
    }

    // Need to download the binary
    log::info!("[FFmpeg] Need to download FFprobe binary");

    // Ensure temp directory exists
    let temp_dir = get_temp_dir();
    fs::create_dir_all(&temp_dir).map_err(|e| {
        log::info!("[FFmpeg] Error: Failed to create temp directory: {}", e);
        format!("Failed to create temp directory: {}", e)
    })?;

    // Define path for the binary we'll download
    let ffprobe_path = temp_dir.join(if cfg!(windows) {
        "ffprobe.exe"
    } else {
        "ffprobe"
    });

    // Remove any existing binary that might be corrupted
    if ffprobe_path.exists() {
        log::info!("[FFmpeg] Removing existing FFprobe binary for fresh download");
        let _ = fs::remove_file(&ffprobe_path);
    }

    // For macOS, FFprobe has a separate download URL
    #[cfg(target_os = "macos")]
    {
        let archive_path = temp_dir.join("ffprobe.archive");
        download_and_extract_binary(
            FFPROBE_MACOS,
            &archive_path,
            &ffprobe_path,
            "ffprobe",
            "macos",
        )?;
    }

    // For Windows and Linux, FFprobe is included in the same archive as FFmpeg
    #[cfg(not(target_os = "macos"))]
    {
        // We need to download the FFmpeg archive which contains FFprobe
        let (os, url) = FFMPEG_URLS
            .iter()
            .find(|(os, _)| match *os {
                "windows" => cfg!(windows),
                "linux" => cfg!(unix) && !cfg!(target_os = "macos"),
                _ => false,
            })
            .ok_or_else(|| {
                log::info!("[FFmpeg] Error: Unsupported platform");
                "Unsupported platform".to_string()
            })?;

        let archive_path = temp_dir.join("ffmpeg.archive");
        download_and_extract_binary(url, &archive_path, &ffprobe_path, "ffprobe", os)?;
    }

    // Set the path
    log::info!(
        "[FFmpeg] FFprobe successfully initialized in {:?}",
        ffprobe_path
    );
    FFPROBE_PATH.set(ffprobe_path).unwrap();
    Ok(())
}

// #[cfg(not(target_os = "macos"))]
pub struct FFmpegRecorder {
    width: u32,
    height: u32,
    fps: u32,
    output_path: PathBuf,
    process: Option<std::process::Child>,
    input_format: Option<String>,
    input_device: Option<String>,
}

// #[cfg(not(target_os = "macos"))]
impl FFmpegRecorder {
    pub fn new_with_input(
        width: u32,
        height: u32,
        fps: u32,
        output_path: PathBuf,
        input_format: String,
        input_device: String,
    ) -> Self {
        log::info!(
            "[FFmpeg] Creating new recorder with input format {}: {}x{} @ {} fps -> {}",
            input_format,
            width,
            height,
            fps,
            output_path.display()
        );

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
        log::info!(
            "[FFmpeg] Starting recording: {}x{} @ {} fps",
            self.width,
            self.height,
            self.fps
        );
        let ffmpeg = FFMPEG_PATH.get().ok_or_else(|| {
            log::info!("[FFmpeg] Error: FFmpeg not initialized");
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
                args.extend(["-capture_cursor".to_string(), "1".to_string()]);
            }

            args.extend(["-i".to_string(), device.clone()]);
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
            "-y".to_string(),          // Overwrite output file
            self.output_path.to_str().unwrap().to_string(),
        ]);

        log::info!("[FFmpeg] Command: {} {}", ffmpeg.display(), args.join(" "));
        let mut command = Command::new(ffmpeg);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW constant
        }
        let mut process = command
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                log::info!("[FFmpeg] Error: Failed to start process: {}", e);
                format!("Failed to start FFmpeg: {}", e)
            })?;

        // Spawn threads to handle stdout and stderr in real-time
        if let Some(stdout) = process.stdout.take() {
            let stdout_reader = std::io::BufReader::new(stdout);
            thread::spawn(move || {
                use std::io::BufRead;
                for line in stdout_reader.lines() {
                    if let Ok(line) = line {
                        log::info!("[FFmpeg] stdout: {}", line);
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
                        log::info!("[FFmpeg] stderr: {}", line);
                        let _ = crate::record::log_ffmpeg(&line, true);
                    }
                }
            });
        }

        // Check if process died immediately and capture any error output
        match process.try_wait() {
            Ok(Some(status)) => {
                // Process exited immediately
                let mut error_msg =
                    format!("FFmpeg process exited immediately with status: {}", status);

                // Try to capture any error output
                if let Some(mut stderr) = process.stderr.take() {
                    let mut error_output = String::new();
                    if std::io::Read::read_to_string(&mut stderr, &mut error_output).is_ok()
                        && !error_output.is_empty()
                    {
                        error_msg = format!("{}\nFFmpeg error output: {}", error_msg, error_output);

                        // Check for common Windows-specific errors
                        if cfg!(windows) {
                            if error_output.contains("Could not find video device") {
                                error_msg = format!("{}\nHint: On Windows, make sure you have permission to access screen recording.", error_msg);
                            } else if error_output.contains("Permission denied") {
                                error_msg = format!(
                                    "{}\nHint: Try running the application as administrator.",
                                    error_msg
                                );
                            }
                        }
                    }
                }

                // Cleanup any partial output file
                if self.output_path.exists() {
                    if let Err(e) = fs::remove_file(&self.output_path) {
                        log::info!(
                            "[FFmpeg] Warning: Failed to cleanup partial output file: {}",
                            e
                        );
                    }
                }

                log::info!("[FFmpeg] Error: {}", error_msg);
                return Err(error_msg);
            }
            Ok(None) => {
                // Process is still running, which is what we want
                log::info!("[FFmpeg] Process started successfully");

                // On Windows, verify we can write to the output directory
                if cfg!(windows) {
                    if let Some(parent) = self.output_path.parent() {
                        if !parent.exists() {
                            if let Err(e) = fs::create_dir_all(parent) {
                                let error_msg = format!("Failed to create output directory: {}", e);
                                log::info!("[FFmpeg] Error: {}", error_msg);
                                return Err(error_msg);
                            }
                        }
                        // Try creating a test file to verify write permissions
                        let test_file = parent.join(".test_write");
                        if let Err(e) = fs::write(&test_file, b"test") {
                            let error_msg =
                                format!("No write permission in output directory: {}", e);
                            log::info!("[FFmpeg] Error: {}", error_msg);
                            return Err(error_msg);
                        }
                        let _ = fs::remove_file(test_file); // Cleanup test file
                    }
                }

                self.process = Some(process);
                Ok(())
            }
            Err(e) => {
                log::info!("[FFmpeg] Error: Failed to check process status: {}", e);
                Err(format!("Failed to check FFmpeg process status: {}", e))
            }
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        log::info!("[FFmpeg] Stopping recording");
        if let Some(mut process) = self.process.take() {
            // Send 'q' to FFmpeg to stop recording gracefully
            if let Some(mut stdin) = process.stdin.take() {
                let _ = stdin.write_all(b"q");
            }

            log::info!("[FFmpeg] Waiting for process to finish");
            // Give FFmpeg time to finish writing
            thread::sleep(Duration::from_secs(2));

            // Wait for process to finish and capture output
            match process.wait_with_output() {
                Ok(_output) => {
                    // Output is already handled by the reader threads
                }
                Err(e) => {
                    log::info!("[FFmpeg] Warning: Failed to get process output: {}", e);
                }
            }

            // Check if output file exists and has size
            if !self.output_path.exists() {
                log::info!(
                    "[FFmpeg] Error: Failed to create output file at {}",
                    self.output_path.display()
                );
                return Err("FFmpeg failed to create output file".to_string());
            }

            let file_size = fs::metadata(&self.output_path)
                .map_err(|e| {
                    log::info!("[FFmpeg] Error: Failed to get output file metadata: {}", e);
                    format!("Failed to get output file metadata: {}", e)
                })?
                .len();

            if file_size == 0 {
                log::info!(
                    "[FFmpeg] Error: Created empty output file at {}",
                    self.output_path.display()
                );
                return Err("FFmpeg created empty output file".to_string());
            }

            log::info!(
                "[FFmpeg] Recording saved successfully: {} ({} bytes)",
                self.output_path.display(),
                file_size
            );
        } else {
            log::info!("[FFmpeg] No active process to stop");
        }
        Ok(())
    }
}
