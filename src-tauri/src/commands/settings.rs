use tauri::AppHandle;
use crate::utils::settings::Settings;

#[tauri::command]
pub fn get_upload_data_allowed(app: AppHandle) -> bool {
    Settings::load(&app).upload_confirmed
}

#[tauri::command]
pub fn get_onboarding_complete(app: AppHandle) -> bool {
    Settings::load(&app).onboarding_complete
}

#[tauri::command]
pub fn set_upload_data_allowed(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.upload_confirmed = confirmed;
    settings.save(&app)
}

#[tauri::command]
pub fn set_onboarding_complete(app: AppHandle, confirmed: bool) -> Result<(), String> {
    let mut settings = Settings::load(&app);
    settings.onboarding_complete = confirmed;
    settings.save(&app)
}
