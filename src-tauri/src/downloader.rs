use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

pub fn download_file(url: &str, path: &Path) -> Result<(), String> {
    log::info!(
        "[Downloader] Downloading file from {} to {}",
        url,
        path.display()
    );

    let filename = path.components().last().unwrap().as_os_str();
    let client = reqwest::blocking::Client::new();
    let mut resp = client.get(url).send().map_err(|e| {
        log::info!(
            "[Downloader] Error: Failed to download {}: {}",
            filename.to_string_lossy(),
            e
        );
        format!("Failed to download {:?}: {}", filename, e)
    })?;

    // Create file for writing
    let mut file = fs::File::create(path).map_err(|e| {
        log::info!(
            "[Download] Error: Failed to create {}: {}",
            filename.to_string_lossy(),
            e
        );
        format!("Failed to create {}: {}", filename.to_string_lossy(), e)
    })?;

    // Download in chunks and write to file
    let mut buffer = [0; 8192]; // 8KB buffer
    let mut total_bytes = 0;
    let mut last_logged_bytes = 0;

    loop {
        let bytes_read = resp.read(&mut buffer).map_err(|e| {
            log::info!("[Downloader] Error: Failed to read from response: {}", e);
            format!("Failed to read from response: {}", e)
        })?;

        if bytes_read == 0 {
            break; // End of response
        }

        file.write_all(&buffer[..bytes_read]).map_err(|e| {
            log::info!(
                "[Downloader] Error: Failed to write to {}: {}",
                filename.to_string_lossy(),
                e
            );
            format!("Failed to write to {:?}: {}", filename, e)
        })?;

        total_bytes += bytes_read;

        // Log progress every 5mb
        if total_bytes - last_logged_bytes >= 5 * 1024 * 1024 {
            log::info!(
                "[Downloader] Downloaded {:.2} MB of {}",
                total_bytes as f64 / (1024.0 * 1024.0),
                filename.to_string_lossy()
            );
            last_logged_bytes = total_bytes;
        }
    }

    log::info!(
        "[Downloader] Successfully downloaded {:?} to {} ({:.2} MB total)",
        filename,
        path.display(),
        total_bytes as f64 / (1024.0 * 1024.0)
    );
    Ok(())
}
