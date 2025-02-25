use crate::record;
use rdev::{listen, Event as RdevEvent, EventType as RdevEventType};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use tauri::{Emitter, Runtime};

fn has_input_permissions() -> bool {
    //todo: get input perm status from storage
    #[cfg(target_os = "macos")]
    {
        false
    }

    #[cfg(not(target_os = "macos"))]
    {
        // On non-macOS platforms, we don't need special permissions
        true
    }
}

/// Requests Input Monitoring permissions by opening System Settings
fn request_input_permissions() {
    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .args(&["x-apple.systempreferences:com.apple.preference.security"])
        .spawn()
        .expect("Failed to open System Settings");
    //todo: save input perm status to storage
}

pub struct InputListener {
    running: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl InputListener {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            threads: Vec::new(),
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        // Don't wait for threads since they might be blocked in rdev listen()
        self.threads.clear();
    }
}

impl Drop for InputListener {
    fn drop(&mut self) {
        self.stop();
    }
}

// Global state for input listening
lazy_static::lazy_static! {
    static ref INPUT_LISTENER_STATE: Arc<Mutex<Option<InputListener>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone, Serialize)]
pub struct InputEvent {
    pub event: String,
    pub data: serde_json::Value,
}

impl InputEvent {
    pub fn new(event: &str, data: serde_json::Value) -> Self {
        Self {
            event: event.to_string(),
            data,
        }
    }

    pub fn to_log_entry(&self) -> serde_json::Value {
        serde_json::json!({
            "event": self.event,
            "data": self.data,
            "time": chrono::Local::now().timestamp_millis()
        })
    }
}

pub fn start_input_listener<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Result<(), String> {
    // Check if already listening
    let mut state = INPUT_LISTENER_STATE.lock().map_err(|e| e.to_string())?;
    if state.is_some() {
        return Ok(()); // Already listening
    }

    // request input permissions
    if !has_input_permissions() {
        request_input_permissions();
    }

    let mut input_listener = InputListener::new();
    let running = input_listener.running.clone();
    // Start rdev for mouse absolute position tracking across all platforms
    let running_clone = running.clone();
    let handle = thread::spawn(move || {
        let callback = move |event: RdevEvent| {
            if let RdevEventType::MouseMove { x, y } = event.event_type {
                let input_event = InputEvent::new(
                    "mousemove",
                    serde_json::json!({
                        "x": x,
                        "y": y
                    }),
                );
                // if let Err(e) = position_app_handle.emit("input-event", &input_event) {
                //     eprintln!("Failed to emit mouse position event: {}", e);
                // }
                // Log the mouse move event
                let _ = record::log_input(input_event.to_log_entry());
            }
        };

        if let Err(error) = listen(move |event| {
            if !running_clone.load(Ordering::SeqCst) {
                return;
            }
            callback(event);
        }) {
            println!("Error: {:?}", error)
        }
    });
    input_listener.threads.push(handle);

    // Platform-specific input handling
    // we use multiinput to get windows rawinput (mouse, kb, joystick)
    // rawinput is needed for capturing input on games and other apps
    #[cfg(target_os = "windows")]
    {
        use multiinput::*;
        let running_clone = running.clone();
        let handle = thread::spawn(move || {
            let mut manager = RawInputManager::new().unwrap();
            manager.register_devices(DeviceType::Joysticks(XInputInclude::True));
            manager.register_devices(DeviceType::Keyboards);
            manager.register_devices(DeviceType::Mice);

            while running_clone.load(Ordering::SeqCst) {
                if let Some(event) = manager.get_event() {
                    let input_event = match event {
                        RawEvent::KeyboardEvent(_device_id, key, state) => match state {
                            State::Pressed => Some(InputEvent::new(
                                "keydown",
                                serde_json::json!({
                                    "key": format!("{:?}", key)
                                }),
                            )),
                            State::Released => Some(InputEvent::new(
                                "keyup",
                                serde_json::json!({
                                    "key": format!("{:?}", key)
                                }),
                            )),
                        },
                        RawEvent::MouseMoveEvent(_device_id, x, y) => Some(InputEvent::new(
                            "mousedelta",
                            serde_json::json!({
                                "x": x,
                                "y": y
                            }),
                        )),
                        RawEvent::MouseButtonEvent(_device_id, button, state) => {
                            Some(InputEvent::new(
                                match state {
                                    State::Pressed => "mousedown",
                                    State::Released => "mouseup",
                                },
                                serde_json::json!({
                                    "button": format!("{:?}", button)
                                }),
                            ))
                        }
                        RawEvent::MouseWheelEvent(_device_id, delta) => Some(InputEvent::new(
                            "mousewheel",
                            serde_json::json!({
                                "delta": delta
                            }),
                        )),
                        RawEvent::JoystickButtonEvent(device_id, button, state) => {
                            Some(InputEvent::new(
                                match state {
                                    State::Pressed => "joystickdown",
                                    State::Released => "joystickup",
                                },
                                serde_json::json!({
                                    "id": device_id,
                                    "button": button
                                }),
                            ))
                        }
                        RawEvent::JoystickAxisEvent(device_id, axis, value) => {
                            Some(InputEvent::new(
                                "joystickaxis",
                                serde_json::json!({
                                    "id": device_id,
                                    "axis": format!("{:?}", axis),
                                    "value": value
                                }),
                            ))
                        }
                        _ => None,
                    };

                    if let Some(event) = input_event {
                        // if let Err(e) = windows_app_handle.emit("input-event", &event) {
                        //     eprintln!("Failed to emit input event: {}", e);
                        // }
                        // Log the input event
                        let _ = record::log_input(event.to_log_entry());
                    }
                }
            }
        });
        input_listener.threads.push(handle);
    }

    // we fallback to rdev for all other input events & operating systems
    #[cfg(not(target_os = "windows"))]
    {
        let other_app_handle = app_handle.clone();
        let running_clone = running.clone();
        let handle = thread::spawn(move || {
            let callback = move |event: RdevEvent| {
                let input_event = match event.event_type {
                    RdevEventType::KeyPress(key) => Some(InputEvent::new(
                        "keydown",
                        serde_json::json!({
                            "key": format!("{:?}", key)
                        }),
                    )),
                    RdevEventType::KeyRelease(key) => Some(InputEvent::new(
                        "keyup",
                        serde_json::json!({
                            "key": format!("{:?}", key)
                        }),
                    )),
                    RdevEventType::ButtonPress(button) => Some(InputEvent::new(
                        "mousedown",
                        serde_json::json!({
                            "button": format!("{:?}", button)
                        }),
                    )),
                    RdevEventType::ButtonRelease(button) => Some(InputEvent::new(
                        "mouseup",
                        serde_json::json!({
                            "button": format!("{:?}", button)
                        }),
                    )),
                    RdevEventType::Wheel {
                        delta_x: _,
                        delta_y,
                    } => Some(InputEvent::new(
                        "mousewheel",
                        serde_json::json!({
                            "delta": delta_y as f32
                        }),
                    )),
                    RdevEventType::MouseMove { .. } => None, // Handled by the cross-platform listener
                };

                if let Some(event) = input_event {
                    if let Err(e) = other_app_handle.emit("input-event", &event) {
                        eprintln!("Failed to emit input event: {}", e);
                    }
                    // Log the input event
                    let _ = record::log_input(event.to_log_entry());
                }
            };

            if let Err(error) = listen(move |event| {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }
                callback(event);
            }) {
                println!("Error: {:?}", error)
            }
        });
        input_listener.threads.push(handle);
    }

    *state = Some(input_listener);
    Ok(())
}

pub fn stop_input_listener() -> Result<(), String> {
    println!("[Input] Stopping input listener");
    let mut state = INPUT_LISTENER_STATE.lock().map_err(|e| e.to_string())?;
    if let Some(mut listener) = state.take() {
        listener.stop();
    }
    println!("[Input] Input listener stopped");
    Ok(())
}

#[tauri::command]
pub async fn request_input_perms() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        //todo: save settings.permissions.input.requested so we don't overdo this
        // println!("[Input] Listening for single input event for input permissions.",);
        // if let Err(error) = listen(|event| println!("{:#?}", event.time)) {
        //     println!("Error Requesting Perms: {:?}", error)
        // }
        // println!("[Input] Input event found. Permissions dialog triggered.",);
    }
    Ok(())
}
