use serde::Serialize;
use std::thread;
use tauri::{Runtime, Emitter};
use rdev::{listen, Event as RdevEvent, EventType as RdevEventType};
use crate::record;

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

pub fn start_input_listener<R: Runtime>(app_handle: tauri::AppHandle<R>) {
    // Start rdev for mouse absolute position tracking across all platforms
    let position_app_handle = app_handle.clone();
    thread::spawn(move || {
        let callback = move |event: RdevEvent| {
            if let RdevEventType::MouseMove { x, y } = event.event_type {
                let input_event = InputEvent::new("mousemove", serde_json::json!({
                    "x": x,
                    "y": y
                }));
                if let Err(e) = position_app_handle.emit("input-event", &input_event) {
                    eprintln!("Failed to emit mouse position event: {}", e);
                }
                // Log the mouse move event
                let _ = record::log_input(input_event.to_log_entry());
            }
        };

        if let Err(error) = listen(callback) {
            println!("Error: {:?}", error)
        }
    });

    // Platform-specific input handling
    // we use multiinput to get windows rawinput (mouse, kb, joystick)
    // rawinput is needed for capturing input on games and other apps
    #[cfg(target_os = "windows")]
    {
        use multiinput::*;
        let windows_app_handle = app_handle.clone();
        thread::spawn(move || {
            let mut manager = RawInputManager::new().unwrap();
            manager.register_devices(DeviceType::Joysticks(XInputInclude::True));
            manager.register_devices(DeviceType::Keyboards);
            manager.register_devices(DeviceType::Mice);

            loop {
                if let Some(event) = manager.get_event() {
                    let input_event = match event {
                        RawEvent::KeyboardEvent(_device_id, key, state) => match state {
                            State::Pressed => Some(InputEvent::new("keydown", serde_json::json!({
                                "key": format!("{:?}", key)
                            }))),
                            State::Released => Some(InputEvent::new("keyup", serde_json::json!({
                                "key": format!("{:?}", key)
                            }))),
                        },
                        RawEvent::MouseMoveEvent(_device_id, x, y) => Some(InputEvent::new("mousedelta", serde_json::json!({
                            "x": x,
                            "y": y
                        }))),
                        RawEvent::MouseButtonEvent(_device_id, button, state) => Some(InputEvent::new(
                            match state {
                                State::Pressed => "mousedown",
                                State::Released => "mouseup",
                            },
                            serde_json::json!({
                                "button": format!("{:?}", button)
                            })
                        )),
                        RawEvent::MouseWheelEvent(_device_id, delta) => Some(InputEvent::new("mousewheel", serde_json::json!({
                            "delta": delta
                        }))),
                        RawEvent::JoystickButtonEvent(device_id, button, state) => Some(InputEvent::new(
                            match state {
                                State::Pressed => "joystickdown",
                                State::Released => "joystickup",
                            },
                            serde_json::json!({
                                "id": device_id,
                                "button": button
                            })
                        )),
                        RawEvent::JoystickAxisEvent(device_id, axis, value) => Some(InputEvent::new("joystickaxis", serde_json::json!({
                            "id": device_id,
                            "axis": format!("{:?}", axis),
                            "value": value
                        }))),
                        _ => None,
                    };

                    if let Some(event) = input_event {
                        if let Err(e) = windows_app_handle.emit("input-event", &event) {
                            eprintln!("Failed to emit input event: {}", e);
                        }
                        // Log the input event
                        let _ = record::log_input(event.to_log_entry());
                    }
                }
            }
        });
    }

    // we fallback to rdev for all other input events & operating systems
    #[cfg(not(target_os = "windows"))]
    {
        let other_app_handle = app_handle.clone();
        thread::spawn(move || {
            let callback = move |event: RdevEvent| {
                let input_event = match event.event_type {
                    RdevEventType::KeyPress(key) => Some(InputEvent::new("keydown", serde_json::json!({
                        "key": format!("{:?}", key)
                    }))),
                    RdevEventType::KeyRelease(key) => Some(InputEvent::new("keyup", serde_json::json!({
                        "key": format!("{:?}", key)
                    }))),
                    RdevEventType::ButtonPress(button) => Some(InputEvent::new("mousedown", serde_json::json!({
                        "button": format!("{:?}", button)
                    }))),
                    RdevEventType::ButtonRelease(button) => Some(InputEvent::new("mouseup", serde_json::json!({
                        "button": format!("{:?}", button)
                    }))),
                    RdevEventType::Wheel { delta_x: _, delta_y } => Some(InputEvent::new("mousewheel", serde_json::json!({
                        "delta": delta_y as f32
                    }))),
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

            if let Err(error) = listen(callback) {
                println!("Error: {:?}", error)
            }
        });
    }
}
