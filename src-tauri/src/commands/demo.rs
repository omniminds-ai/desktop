use crate::core::demo::create_demo_window;

#[tauri::command]
pub async fn start_demo(app: tauri::AppHandle) -> Result<(), String> {
    create_demo_window(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn run_demo_request(request: &str) -> Result<(), String> {
    log::info!("[Demo] New Request: {}", request);
    //todo: execute the demo CLI tool or whatever
    Ok(())
}
