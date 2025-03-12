use chrono::DateTime;
use log::info;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;

/// Metadata for a binary file downloaded from GitHub
pub struct BinaryMetadata {
    pub version: String,
    pub build_timestamp: u64,
}

impl BinaryMetadata {
    pub fn new(version: String, build_timestamp: u64) -> Self {
        Self {
            version,
            build_timestamp,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "version": self.version,
            "build_timestamp": self.build_timestamp,
        })
    }

    pub fn from_json(json: &Value) -> Option<Self> {
        if let (Some(version), Some(build_timestamp)) = (
            json.get("version").and_then(Value::as_str),
            json.get("build_timestamp").and_then(Value::as_u64),
        ) {
            Some(Self::new(version.to_string(), build_timestamp))
        } else {
            None
        }
    }
}

/// Save metadata to a file
fn save_metadata(path: &Path, metadata: &BinaryMetadata) -> Result<(), String> {
    let json = metadata.to_json();
    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    fs::write(path, content).map_err(|e| format!("Failed to write metadata file: {}", e))?;

    info!("[GitHub Release] Saved metadata to {}", path.display());
    Ok(())
}

/// Load metadata from a file
fn load_metadata(path: &Path) -> Result<Option<BinaryMetadata>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read metadata file: {}", e))?;

    let json: Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse metadata: {}", e))?;

    Ok(BinaryMetadata::from_json(&json))
}

/// Fetch latest release metadata from GitHub API
fn fetch_latest_release_metadata(repo_owner: &str, repo_name: &str) -> Result<BinaryMetadata, String> {
    info!("[GitHub Release] Fetching latest release metadata for {}/{}", repo_owner, repo_name);

    let github_api_url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        repo_owner, repo_name
    );

    let client = reqwest::blocking::Client::builder()
        .user_agent("viralmind-desktop")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&github_api_url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    let json: Value = response
        .json()
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let version = json
        .get("tag_name")
        .and_then(Value::as_str)
        .ok_or_else(|| "No tag name in release".to_string())?
        .to_string();

    // Use published_at timestamp from GitHub API
    let published_at = json
        .get("published_at")
        .and_then(Value::as_str)
        .ok_or_else(|| "No published_at in release".to_string())?;

    // Convert ISO 8601 date string to Unix timestamp
    let timestamp = {
        let dt = DateTime::parse_from_rfc3339(published_at)
            .map_err(|e| format!("Failed to parse published_at date: {}", e))?;

        dt.timestamp() as u64
    };

    info!(
        "[GitHub Release] Latest release: version={}, published_at={} ({})",
        version, published_at, timestamp
    );

    Ok(BinaryMetadata::new(version, timestamp))
}

/// Download a file from a URL to a local path
fn download_file(url: &str, path: &Path) -> Result<(), String> {
    info!(
        "[GitHub Release] Downloading file from {} to {}",
        url,
        path.display()
    );
    let client = reqwest::blocking::Client::new();
    let resp = client.get(url).send().map_err(|e| {
        info!("[GitHub Release] Error: Failed to download file: {}", e);
        format!("Failed to download file: {}", e)
    })?;

    let bytes = resp.bytes().map_err(|e| {
        info!("[GitHub Release] Error: Failed to get response bytes: {}", e);
        format!("Failed to get response bytes: {}", e)
    })?;

    fs::write(path, bytes).map_err(|e| {
        info!("[GitHub Release] Error: Failed to write file: {}", e);
        format!("Failed to write file: {}", e)
    })?;

    info!(
        "[GitHub Release] Successfully downloaded file to {}",
        path.display()
    );
    Ok(())
}

/// Get the latest release of a binary from GitHub
/// 
/// # Arguments
/// 
/// * `repo_owner` - The owner of the GitHub repository
/// * `repo_name` - The name of the GitHub repository
/// * `asset_url` - The URL of the asset to download
/// * `target_dir` - The directory to store the downloaded file
/// * `make_executable` - Whether to make the file executable (for Linux/macOS)
/// 
/// # Returns
/// 
/// The path to the downloaded file
pub fn get_latest_release(
    repo_owner: &str,
    repo_name: &str,
    asset_url: &str,
    target_dir: &Path,
    make_executable: bool,
) -> Result<PathBuf, String> {
    info!("[GitHub Release] Getting latest release for {}/{}", repo_owner, repo_name);

    // Create target directory if it doesn't exist
    fs::create_dir_all(target_dir).map_err(|e| {
        info!("[GitHub Release] Error: Failed to create target directory: {}", e);
        format!("Failed to create target directory: {}", e)
    })?;

    // Get the filename from the URL for proper version tracking
    let asset_split: Vec<&str> = asset_url.split('/').collect();
    let asset_filename = asset_split[asset_url.split('/').count() - 1];

    let asset_path = target_dir.join(asset_filename);
    let metadata_path = target_dir.join(format!("{}.metadata.json", asset_filename));

    // Fetch latest release metadata from GitHub
    let latest_metadata = fetch_latest_release_metadata(repo_owner, repo_name)?;

    // Check if we need to download the binary
    let should_download = if !asset_path.exists() {
        info!("[GitHub Release] Asset does not exist, downloading");
        true
    } else {
        // Load existing metadata
        let current_metadata = load_metadata(&metadata_path)?;

        match current_metadata {
            Some(metadata) => {
                // Compare build timestamps
                if metadata.build_timestamp < latest_metadata.build_timestamp {
                    info!(
                        "[GitHub Release] New version available: current={} ({}), latest={} ({})",
                        metadata.version,
                        metadata.build_timestamp,
                        latest_metadata.version,
                        latest_metadata.build_timestamp
                    );
                    true
                } else {
                    info!("[GitHub Release] Asset is up to date");
                    false
                }
            }
            None => {
                info!("[GitHub Release] No metadata found, downloading latest version");
                true
            }
        }
    };

    if should_download {
        info!("[GitHub Release] Downloading new version: {}", asset_filename);
        download_file(asset_url, &asset_path)?;

        // Set executable permissions if needed
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if make_executable {
            fs::set_permissions(&asset_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        }

        // Save the metadata
        save_metadata(&metadata_path, &latest_metadata)?;
    }

    info!("[GitHub Release] Using asset at {}", asset_path.display());
    Ok(asset_path)
}
