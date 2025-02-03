use std::process::Command;
use tauri::Runtime;

#[cfg(target_os = "macos")]
use core_graphics::display::CGDisplay;

pub enum PermissionStatus {
    Granted,
    Denied,
    Unknown,
    SystemDialogNeeded,
}

pub struct ScreenPermissions;

impl ScreenPermissions {
    #[cfg(target_os = "macos")]
    pub fn check() -> PermissionStatus {
        // On macOS, attempting to access screen recording will trigger system permission dialog
        match CGDisplay::main() {
            Ok(_) => PermissionStatus::Granted,
            Err(_) => PermissionStatus::Denied,
        }
    }

    #[cfg(target_os = "windows")]
    pub fn check() -> PermissionStatus {
        // Windows doesn't require explicit screen recording permission
        // but we can test if we can actually capture
        let test = Command::new("ffmpeg")
            .args([
                "-f",
                "gdigrab",
                "-i",
                "desktop",
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .output();

        match test {
            Ok(output) if output.status.success() => PermissionStatus::Granted,
            Ok(_) => PermissionStatus::SystemDialogNeeded, // Might need elevation
            Err(_) => PermissionStatus::Unknown,
        }
    }

    #[cfg(target_os = "linux")]
    pub fn check() -> PermissionStatus {
        // For X11, check if we can access the display
        if let Ok(display) = std::env::var("DISPLAY") {
            let test = Command::new("ffmpeg")
                .args([
                    "-f",
                    "x11grab",
                    "-i",
                    &format!("{}+0,0", display),
                    "-frames:v",
                    "1",
                    "-f",
                    "null",
                    "-",
                ])
                .output();

            match test {
                Ok(output) if output.status.success() => PermissionStatus::Granted,
                _ => PermissionStatus::Denied,
            }
        } else {
            PermissionStatus::Unknown
        }
    }

    pub fn request() -> PermissionStatus {
        #[cfg(target_os = "macos")]
        {
            // Attempting to capture will trigger macOS permission dialog
            Self::check()
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we might need to request elevation
            if let PermissionStatus::SystemDialogNeeded = Self::check() {
                // Try running an elevated test capture
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
            } else {
                Self::check()
            }
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux/X11, permissions are typically handled by the window manager
            Self::check()
        }
    }
}

// Tauri command to check permissions
#[tauri::command]
pub async fn check_screen_permission<R: Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<String, String> {
    match ScreenPermissions::check() {
        PermissionStatus::Granted => Ok("granted".to_string()),
        PermissionStatus::Denied => Ok("denied".to_string()),
        PermissionStatus::Unknown => Ok("unknown".to_string()),
        PermissionStatus::SystemDialogNeeded => Ok("system_dialog_needed".to_string()),
    }
}

// Tauri command to request permissions
#[tauri::command]
pub async fn request_screen_permission<R: Runtime>(
    _app: tauri::AppHandle<R>,
) -> Result<String, String> {
    match ScreenPermissions::request() {
        PermissionStatus::Granted => Ok("granted".to_string()),
        PermissionStatus::Denied => Ok("denied".to_string()),
        PermissionStatus::Unknown => Ok("unknown".to_string()),
        PermissionStatus::SystemDialogNeeded => Ok("system_dialog_needed".to_string()),
    }
}
