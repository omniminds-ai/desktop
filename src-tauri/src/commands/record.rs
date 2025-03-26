use crate::core::record::{self, Quest, QuestState, RecordingMeta};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    quest_state: State<'_, QuestState>,
    quest: Option<Quest>,
) -> Result<(), String> {
    record::start_recording(app, quest_state, quest).await
}

#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    quest_state: State<'_, QuestState>,
    reason: Option<String>,
) -> Result<String, String> {
    record::stop_recording(app, quest_state, reason).await
}

#[tauri::command]
pub async fn get_recording_state() -> Result<String, String> {
    record::get_recording_state().await
}

#[tauri::command]
pub async fn list_recordings(app: AppHandle) -> Result<Vec<RecordingMeta>, String> {
    record::list_recordings(app).await
}

#[tauri::command]
pub async fn get_recording_file(
    app: AppHandle,
    recording_id: String,
    filename: String,
    as_base64: Option<bool>,
    as_path: Option<bool>,
) -> Result<String, String> {
    record::get_recording_file(app, recording_id, filename, as_base64, as_path).await
}

#[tauri::command]
pub async fn process_recording(app: AppHandle, recording_id: String) -> Result<(), String> {
    record::process_recording(app, recording_id).await
}

#[tauri::command]
pub async fn write_file(app: AppHandle, path: String, content: String) -> Result<(), String> {
    record::write_file(app, path, content).await
}

#[tauri::command]
pub async fn write_recording_file(
    app: AppHandle,
    recording_id: String,
    filename: String,
    content: String,
) -> Result<(), String> {
    record::write_recording_file(app, recording_id, filename, content).await
}

#[tauri::command]
pub async fn open_recording_folder(app: AppHandle, recording_id: String) -> Result<(), String> {
    record::open_recording_folder(app, recording_id).await
}

#[tauri::command]
pub async fn delete_recording(app: AppHandle, recording_id: String) -> Result<(), String> {
    record::delete_recording(app, recording_id).await
}

#[tauri::command]
pub async fn create_recording_zip(app: AppHandle, recording_id: String) -> Result<Vec<u8>, String> {
    record::create_recording_zip(app, recording_id).await
}

#[tauri::command]
pub async fn export_recording_zip(id: String, app: AppHandle) -> Result<String, String> {
    record::export_recording_zip(id, app).await
}

#[tauri::command]
pub async fn get_app_data_dir(app: AppHandle) -> Result<String, String> {
    record::get_app_data_dir(app).await
}

#[tauri::command]
pub async fn get_current_quest(
    quest_state: State<'_, QuestState>,
) -> Result<Option<Quest>, String> {
    record::get_current_quest(quest_state).await
}
