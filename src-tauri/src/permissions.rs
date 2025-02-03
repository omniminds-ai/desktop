use std::process::Command;
use tauri::Runtime;

#[derive(Debug, Clone, Copy)]
pub enum PermissionStatus {
    Granted,
    Denied,
    Unknown,
    SystemDialogNeeded,
}

fn get_ffmpeg_test_args() -> (String, String, String) {
    if cfg!(target_os = "windows") {
        ("gdigrab".into(), "desktop".into(), String::new())
    } else if cfg!(target_os = "macos") {
        ("avfoundation".into(), "1:none".into(), String::new())
    } else {
        let display = std::env::var("DISPLAY").unwrap_or_else(|_| String::from(":0.0"));
        ("x11grab".into(), format!("{}+0,0", display), String::new())
    }
}

pub fn check_screen_capture_permission() -> PermissionStatus {
    let (format, input, _extra_args) = get_ffmpeg_test_args();

    // Use the path from your FFmpeg initialization if possible
    let ffmpeg_path = match crate::ffmpeg::FFMPEG_PATH.get() {
        Some(path) => path,
        None => return PermissionStatus::Unknown,
    };

    let test = Command::new(&ffmpeg_path)
        .args([
            "-f",
            &format,
            "-i",
            &input,
            "-frames:v",
            "1",
            "-f",
            "null",
            "-",
        ])
        .output();

    match test {
        Ok(output) if output.status.success() => PermissionStatus::Granted,
        Ok(_) => {
            if cfg!(target_os = "windows") {
                PermissionStatus::SystemDialogNeeded
            } else {
                PermissionStatus::Denied
            }
        }
        Err(_) => PermissionStatus::Unknown,
    }
}

pub fn request_screen_capture_permission() -> PermissionStatus {
    match check_screen_capture_permission() {
        PermissionStatus::SystemDialogNeeded if cfg!(target_os = "windows") => {
            // On Windows, try with elevation
            let status = Command::new("powershell")
                .args([
                    "Start-Process",
                    "ffmpeg",
                    "-ArgumentList",
                    "\"-f gdigrab -i desktop -frames:v 1 -f null -\"",
                    "-Verb",
                    "RunAs",
                    "-Wait",
                ])
                .status();

            match status {
                Ok(exit) if exit.success() => PermissionStatus::Granted,
                _ => PermissionStatus::Denied,
            }
        }
        other => other,
    }
}

#[tauri::command]
pub async fn check_screen_permission<R: Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<String, String> {
    match check_screen_capture_permission() {
        PermissionStatus::Granted => Ok("granted".to_string()),
        PermissionStatus::Denied => Ok("denied".to_string()),
        PermissionStatus::Unknown => Ok("unknown".to_string()),
        PermissionStatus::SystemDialogNeeded => Ok("system_dialog_needed".to_string()),
    }
}

#[tauri::command]
pub async fn request_screen_permission<R: Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<String, String> {
    match request_screen_capture_permission() {
        PermissionStatus::Granted => Ok("granted".to_string()),
        PermissionStatus::Denied => Ok("denied".to_string()),
        PermissionStatus::Unknown => Ok("unknown".to_string()),
        PermissionStatus::SystemDialogNeeded => Ok("system_dialog_needed".to_string()),
    }
}
