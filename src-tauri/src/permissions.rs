use core_foundation::base::Boolean;
use core_graphics::access::ScreenCaptureAccess;

// External ApplicationServices declarations
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> Boolean;
}

#[tauri::command]
pub fn has_ax_perms() -> bool {
    unsafe {
        let result = AXIsProcessTrusted() != 0;
        result
    }
}

#[tauri::command]
pub fn request_ax_perms() {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
pub fn has_record_perms() -> bool {
    return ScreenCaptureAccess.preflight();
}

#[tauri::command]
pub fn request_record_perms() {
    ScreenCaptureAccess.request();
}
