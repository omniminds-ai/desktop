use std::{
    io::{Cursor, Write},
    path::Path,
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use zip::{write::FileOptions, ZipWriter};

#[tauri::command]
pub async fn export_recordings(app: tauri::AppHandle) -> Result<String, String> {
    let recordings_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("recordings");

    // Create a buffer to store the zip file
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add recordings folder contents to zip
    if recordings_dir.exists() {
        log::info!("Zipping files in {:?}", recordings_dir.to_string_lossy());

        // Helper function to recursively add files and directories to the zip
        fn add_dir_to_zip(
            zip: &mut ZipWriter<Cursor<Vec<u8>>>,
            options: FileOptions,
            src_dir: &Path,
            base_path: &Path,
        ) -> Result<(), String> {
            for entry in std::fs::read_dir(src_dir)
                .map_err(|e| format!("Failed to read directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();

                // Calculate relative path from base_path
                let relative_path = path
                    .strip_prefix(base_path)
                    .map_err(|e| format!("Failed to strip prefix: {}", e))?;
                let relative_path_str = relative_path.to_string_lossy();

                if path.is_file() {
                    // Add file to zip
                    zip.start_file(relative_path_str.as_ref(), options)
                        .map_err(|e| format!("Failed to add file to zip: {}", e))?;

                    let contents = std::fs::read(&path)
                        .map_err(|e| format!("Failed to read file contents: {}", e))?;

                    zip.write_all(&contents)
                        .map_err(|e| format!("Failed to write file contents to zip: {}", e))?;
                } else if path.is_dir() {
                    // Create directory entry in zip
                    let dir_path = format!("{}/", relative_path_str);
                    zip.add_directory(dir_path, options)
                        .map_err(|e| format!("Failed to add directory to zip: {}", e))?;

                    // Recursively add contents of this directory
                    add_dir_to_zip(zip, options, &path, base_path)?;
                }
            }
            Ok(())
        }

        // Start recursively adding files and directories
        add_dir_to_zip(&mut zip, options, &recordings_dir, &recordings_dir)?;
    }

    let buf = zip
        .finish()
        .map_err(|e| format!("Failed to finalize zip: {}", e))?
        .into_inner();

    let selected_dir = app.dialog().file().blocking_pick_folder();

    // If user cancels the dialog, selected_dir will be None
    if let Some(dir_path) = selected_dir {
        // Create the full path for history.zip
        let dir_path_str = dir_path.to_string();
        let file_path = Path::new(&dir_path_str).join("history.zip");

        // Write the buffer to the file
        std::fs::write(&file_path, buf).map_err(|e| format!("Failed to write zip file: {}", e))?;

        Ok(file_path.to_string_lossy().into_owned())
    } else {
        Ok("".to_string())
    }
}
