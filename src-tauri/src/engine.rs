use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::HashMap;
use std::process::{Command, Output};
use tauri::{AppHandle, Manager, State, Emitter};
use tauri::async_runtime::spawn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EngineType {
    Native,
    WSL,
    SSH,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
}

impl Default for SSHConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 22,
            username: "".to_string(),
            password: None,
            private_key_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WSLConfig {
    pub distro: String,
}

impl Default for WSLConfig {
    fn default() -> Self {
        Self {
            distro: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub engine_type: EngineType,
    pub wsl_config: Option<WSLConfig>,
    pub ssh_config: Option<SSHConfig>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine_type: EngineType::Native,
            wsl_config: Some(WSLConfig::default()),
            ssh_config: Some(SSHConfig::default()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum JobStatus {
    Running,
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandJob {
    pub id: String,
    pub command: String,
    pub status: JobStatus,
    pub output: String,
    pub error: String,
}

impl CommandJob {
    pub fn new(id: String, command: String) -> Self {
        Self {
            id,
            command,
            status: JobStatus::Running,
            output: String::new(),
            error: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineStatus {
    pub running: bool,
    pub system_info: Option<String>,
    pub gpu_info: Option<String>,
    pub error: Option<String>,
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self {
            running: false,
            system_info: None,
            gpu_info: None,
            error: None,
        }
    }
}

#[derive(Default)]
pub struct EngineState {
    pub status: Mutex<EngineStatus>,
    pub jobs: Mutex<HashMap<String, CommandJob>>,
}

/// Helper function to safely convert command output to string, removing null bytes
fn safe_string_from_bytes(bytes: &[u8]) -> String {
    // First use from_utf8_lossy to handle invalid UTF-8
    let string = String::from_utf8_lossy(bytes).into_owned();
    // Then explicitly replace null bytes
    string.replace('\0', "")
}

/// Helper function to sanitize strings by removing null bytes
fn sanitize_string(s: &str) -> String {
    s.replace('\0', "")
}

/// A robust cross-platform function to run shell commands with bash as the backend
/// 
/// This function handles all the platform-specific details and provides a unified interface
/// for executing shell commands across Windows, macOS, and Linux, using bash when possible.
/// 
/// # Arguments
/// 
/// * `cmd` - The command to execute
/// * `engine_type` - The type of engine to use (Native, WSL, SSH)
/// * `wsl_config` - Configuration for WSL execution (if applicable)
/// * `ssh_config` - Configuration for SSH execution (if applicable)
/// 
/// # Returns
/// 
/// A Result containing either the command Output or an error message
pub fn run_shell_command(
    cmd: &str, 
    engine_type: &EngineType,
    wsl_config: Option<&WSLConfig>,
    ssh_config: Option<&SSHConfig>,
) -> Result<Output, String> {
    info!("Running command: {}", cmd);

    match engine_type {
        EngineType::Native => {
            // Use cmd.exe on Windows and sh on other platforms
            #[cfg(target_os = "windows")]
            let output = Command::new("cmd")
                .args(["/C", &sanitize_string(cmd)])
                .output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;
                
            #[cfg(not(target_os = "windows"))]
            let output = Command::new("bash")
                .args(["-c", cmd])
                .output()
                .map_err(|e| format!("Failed to execute command: {}", e))?;
                
            Ok(output)
        },
        EngineType::WSL => {
            let wsl_config = wsl_config.ok_or("WSL config is missing")?;
            let distro = sanitize_string(&wsl_config.distro);
            
            #[cfg(target_os = "windows")]
            let mut command = Command::new("wsl");
            
            // Add distro argument if specified
            #[cfg(target_os = "windows")]
            if !distro.is_empty() {
                command.arg("-d");
                command.arg(&distro);
            }
            
            // Add -e flag and the command
            #[cfg(target_os = "windows")]
            {
                command.arg("-e");
                
                // Instead of splitting the command ourselves, 
                // pass it as a single argument to bash
                command.arg("bash");
                command.arg("-c");
                command.arg(cmd);
            }
            
            // Execute the command
            #[cfg(target_os = "windows")]
            {
                info!("Executing WSL command with bash: {}", cmd);
                let output = command
                    .output()
                    .map_err(|e| format!("Failed to execute WSL command: {}", e))?;
                
                info!("WSL command executed successfully");
                Ok(output)
            }
                
            #[cfg(not(target_os = "windows"))]
            return Err("WSL is only supported on Windows".to_string());
            
            #[cfg(not(target_os = "windows"))]
            { unreachable!() }
        },
        EngineType::SSH => {
            let ssh_config = ssh_config.ok_or("SSH config is missing")?;
            
            // For SSH we need to handle it differently to properly escape the command
            let ssh_user_host = format!("{}@{}", 
                sanitize_string(&ssh_config.username),
                sanitize_string(&ssh_config.host)
            );
            
            #[cfg(target_os = "windows")]
            let output = Command::new("cmd")
                .args(["/C", &format!("ssh {} -p {} \"{}\"", 
                    ssh_user_host,
                    ssh_config.port,
                    cmd.replace("\"", "\\\"")
                )])
                .output()
                .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
                
            #[cfg(not(target_os = "windows"))]
            let output = Command::new("ssh")
                .arg(ssh_user_host)
                .arg("-p")
                .arg(ssh_config.port.to_string())
                .arg(cmd)
                .output()
                .map_err(|e| format!("Failed to execute SSH command: {}", e))?;
            
            info!("SSH command executed");
                
            Ok(output)
        }
    }
}

#[tauri::command]
pub async fn start_engine(
    app: AppHandle,
    engine_state: State<'_, EngineState>,
    config: EngineConfig,
) -> Result<EngineStatus, String> {
    let mut status = engine_state.status.lock().map_err(|e| e.to_string())?;
    
    if status.running {
        return Ok(status.clone());
    }
    
    // Select appropriate commands based on engine type
    let (sys_info_cmd, gpu_info_cmd) = match config.engine_type {
        EngineType::Native => {
            // Use OS-specific commands for native mode
            #[cfg(target_os = "windows")]
            let sys_cmd = "ver";
            #[cfg(not(target_os = "windows"))]
            let sys_cmd = "uname -a";
            
            #[cfg(target_os = "windows")]
            let gpu_cmd = "where nvidia-smi && nvidia-smi || echo No GPU info available";
            #[cfg(not(target_os = "windows"))]
            let gpu_cmd = "which nvidia-smi && nvidia-smi || echo 'No GPU info available'";
            
            (sys_cmd, gpu_cmd)
        },
        EngineType::WSL => {
            // For WSL we need simpler commands without pipes or shell operators
            ("uname -a", "nvidia-smi")
        },
        EngineType::SSH => {
            ("uname -a", "which nvidia-smi && nvidia-smi || echo 'No GPU info available'")
        }
    };
    
    // Execute system info command
    let sys_cmd_result = run_shell_command(
        sys_info_cmd,
        &config.engine_type,
        config.wsl_config.as_ref(),
        config.ssh_config.as_ref()
    );
    
    let system_info = match sys_cmd_result {
        Ok(output) => {
            if output.status.success() {
                Some(safe_string_from_bytes(&output.stdout))
            } else {
                return Err(format!(
                    "System info command failed: {}",
                    safe_string_from_bytes(&output.stderr)
                ));
            }
        },
        Err(e) => {
            return Err(e);
        }
    };
    
    // Execute GPU info command
    let gpu_cmd_result = run_shell_command(
        gpu_info_cmd,
        &config.engine_type,
        config.wsl_config.as_ref(),
        config.ssh_config.as_ref()
    );
    
    let gpu_info = match gpu_cmd_result {
        Ok(output) => {
            if output.status.success() {
                Some(safe_string_from_bytes(&output.stdout))
            } else {
                Some(format!(
                    "GPU info command failed: {}",
                    safe_string_from_bytes(&output.stderr)
                ))
            }
        },
        Err(e) => {
            Some(format!("Failed to execute GPU info command: {}", e))
        }
    };
    
    // Update engine status
    *status = EngineStatus {
        running: true,
        system_info,
        gpu_info,
        error: None,
    };
    
    // Emit engine started event
    let _ = app.emit("engine-status", &*status);
    
    Ok(status.clone())
}

#[tauri::command]
pub async fn stop_engine(
    app: AppHandle,
    engine_state: State<'_, EngineState>,
) -> Result<EngineStatus, String> {
    let mut status = engine_state.status.lock().map_err(|e| e.to_string())?;
    
    if !status.running {
        return Ok(status.clone());
    }
    
    // Clear any running jobs
    let mut jobs = engine_state.jobs.lock().map_err(|e| e.to_string())?;
    jobs.clear();
    
    // Update engine status
    *status = EngineStatus::default();
    
    // Emit engine stopped event
    let _ = app.emit("engine-status", &*status);
    
    Ok(status.clone())
}

#[tauri::command]
pub async fn get_engine_status(
    engine_state: State<'_, EngineState>,
) -> Result<EngineStatus, String> {
    let status = engine_state.status.lock().map_err(|e| e.to_string())?;
    Ok(status.clone())
}

#[tauri::command]
pub async fn run_command(
    app: AppHandle,
    engine_state: State<'_, EngineState>,
    command: String,
) -> Result<String, String> {
    let status = engine_state.status.lock().map_err(|e| e.to_string())?;
    
    if !status.running {
        return Err("Engine is not running".to_string());
    }
    
    // Load settings directly instead of trying to access SettingsState
    let settings = crate::settings::Settings::load(&app);
    
    // Create engine config from current settings with null checks
    let engine_settings = settings.engines.unwrap_or_default();
    
    let engine_type = match engine_settings.backend.as_str() {
        "wsl" => EngineType::WSL,
        "ssh" => EngineType::SSH,
        _ => EngineType::Native
    };
    
    let wsl_config = if engine_type == EngineType::WSL {
        engine_settings.wsl_config.as_ref().map(|wsl_cfg| WSLConfig {
            distro: wsl_cfg.distro.clone()
        })
    } else {
        None
    };
    
    let ssh_config = if engine_type == EngineType::SSH {
        engine_settings.ssh_config.as_ref().map(|ssh_cfg| SSHConfig {
            host: ssh_cfg.host.clone(),
            port: ssh_cfg.port,
            username: ssh_cfg.username.clone(),
            private_key_path: ssh_cfg.private_key_path.clone(),
            password: None
        })
    } else {
        None
    };
    
    let job_id = uuid::Uuid::new_v4().to_string();
    let mut jobs = engine_state.jobs.lock().map_err(|e| e.to_string())?;
    
    // Create job and add to state
    let job = CommandJob::new(job_id.clone(), command.clone());
    jobs.insert(job_id.clone(), job.clone());
    
    // Emit job created event
    let _ = app.emit("job-created", &job);
    
    // Run the command in a separate thread
    let app_handle = app.clone();
    let job_id_clone = job_id.clone();
    let engine_type_clone = engine_type.clone();
    let wsl_config_clone = wsl_config.clone();
    let ssh_config_clone = ssh_config.clone();
    spawn(async move {
        // Execute the command using our robust cross-platform function
        let output = run_shell_command(
            &command,
            &engine_type_clone,
            wsl_config_clone.as_ref(),
            ssh_config_clone.as_ref()
        );
        
        // Get the engine state
        let engine_state = app_handle.state::<EngineState>();
        let mut jobs = match engine_state.jobs.lock() {
            Ok(jobs) => jobs,
            Err(e) => {
                error!("Failed to get jobs lock: {}", e);
                return;
            }
        };
        
        // Get the job
        if let Some(job) = jobs.get_mut(&job_id_clone) {
            match output {
                Ok(output) => {
                    if output.status.success() {
                        job.status = JobStatus::Success;
                        job.output = safe_string_from_bytes(&output.stdout);
                    } else {
                        job.status = JobStatus::Failed;
                        job.error = safe_string_from_bytes(&output.stderr);
                    }
                },
                Err(e) => {
                    job.status = JobStatus::Failed;
                    job.error = format!("Failed to execute command: {}", e);
                }
            }
            
            // Emit job updated event
            let _ = app_handle.emit("job-updated", &job);
        }
    });
    
    Ok(job_id)
}

#[tauri::command]
pub async fn get_job(
    engine_state: State<'_, EngineState>,
    job_id: String,
) -> Result<CommandJob, String> {
    let jobs = engine_state.jobs.lock().map_err(|e| e.to_string())?;
    jobs.get(&job_id)
        .cloned()
        .ok_or_else(|| format!("Job not found: {}", job_id))
}

#[tauri::command]
pub async fn get_all_jobs(
    engine_state: State<'_, EngineState>,
) -> Result<Vec<CommandJob>, String> {
    let jobs = engine_state.jobs.lock().map_err(|e| e.to_string())?;
    Ok(jobs.values().cloned().collect())
}

#[tauri::command]
pub async fn clear_all_jobs(
    app: AppHandle,
    engine_state: State<'_, EngineState>
) -> Result<(), String> {
    // Create a new jobs map with only running jobs
    let running_jobs: HashMap<String, CommandJob>;
    
    {
        // Limited scope for the jobs lock to avoid lifetime issues
        let mut jobs = engine_state.jobs.lock().map_err(|e| e.to_string())?;
        
        // Filter out completed jobs, keep only running ones
        running_jobs = jobs.iter()
            .filter(|(_, job)| job.status == JobStatus::Running)
            .map(|(id, job)| (id.clone(), job.clone()))
            .collect();
        
        // Replace the jobs map with only running jobs
        *jobs = running_jobs;
    }
    
    // Emit an event to notify clients that jobs were cleared
    let _ = app.emit("jobs-cleared", ());
    
    Ok(())
}
