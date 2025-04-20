use std::sync::Mutex;

use display_info::DisplayInfo;

lazy_static::lazy_static! {
    static ref DEMO_WINDOW_STATE: Mutex<Option<tauri::WebviewWindow>> = Mutex::new(None);
}

pub fn create_demo_window(app: &tauri::AppHandle) -> Result<(), String> {
    log::info!("Starting to create demo window");

    // Get primary display info
    let displays = DisplayInfo::all().map_err(|e| format!("Failed to get display info: {}", e))?;
    let primary = displays
        .iter()
        .find(|d| d.is_primary)
        .or_else(|| displays.first())
        .ok_or_else(|| "No display found".to_string())?;

    log::info!(
        "Using primary display: {}x{} at position ({},{})",
        primary.width,
        primary.height,
        primary.x,
        primary.y
    );

    // set demo locations for specific platforms
    let demo_w = 1000.0;
    let demo_h = 70.0;
    let demo_x = primary.x as f64 + (primary.width as f64 - demo_w) / 2.0;
    let demo_y = primary.y as f64 + ((primary.height as f64 - demo_h) / 2.0) - 600.0;

    log::info!(
        "Demo window dimensions: {}x{} at position ({},{})",
        demo_w,
        demo_h,
        demo_x,
        demo_y
    );

    // create the demo window
    let demo_window =
        tauri::WebviewWindowBuilder::new(app, "demo", tauri::WebviewUrl::App("demo".into()))
            .transparent(true)
            .always_on_top(true)
            .decorations(false)
            .focused(false)
            .shadow(false)
            .position(demo_x, demo_y)
            .inner_size(demo_w, demo_h)
            .skip_taskbar(true)
            .resizable(false)
            .visible_on_all_workspaces(true)
            .build();

    match demo_window {
        Ok(window) => {
            log::info!("Successfully created demo window");

            // Check if the window is visible
            if let Ok(visible) = window.is_visible() {
                log::info!("demo window visibility: {}", visible);
            } else {
                log::warn!("Failed to check demo window visibility");
            }

            // Store the window in the global state
            if let Ok(mut demo_state) = DEMO_WINDOW_STATE.lock() {
                log::info!("Storing demo window in global state");
                *demo_state = Some(window);
            } else {
                log::error!("Failed to acquire lock for demo window state");
                return Err("Failed to store demo window: mutex lock failed".to_string());
            }

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to create demo window: {}", e);

            Err(format!("Failed to create demo window: {}", e))
        }
    }
}
