use serde::Serialize;
use std::thread;
use tauri::{Manager, Runtime, Emitter};
use std::fmt;
use rdev::{listen, Event as RdevEvent, EventType as RdevEventType};

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum InputEvent {
    KeyPress { key: String },
    KeyRelease { key: String },
    MouseMove { x: f64, y: f64 },
    MouseDelta { x: i32, y: i32 },
    MouseClick { button: String, state: String },
    MouseWheel { delta: f32 },
    JoystickButton { id: usize, button: usize, state: String },
    JoystickAxis { id: usize, axis: String, value: f64 },
}

impl fmt::Display for InputEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputEvent::KeyPress { key } => write!(f, "Key press: {}", key),
            InputEvent::KeyRelease { key } => write!(f, "Key release: {}", key),
            InputEvent::MouseMove { x, y } => write!(f, "Mouse position: ({:.1}, {:.1})", x, y),
            InputEvent::MouseDelta { x, y } => write!(f, "Mouse delta: ({}, {})", x, y),
            InputEvent::MouseClick { button, state } => write!(f, "Mouse {}: {}", state, button),
            InputEvent::MouseWheel { delta } => write!(f, "Mouse wheel: {}", delta),
            InputEvent::JoystickButton { id, button, state } => write!(f, "Joystick {} button {}: {}", id, button, state),
            InputEvent::JoystickAxis { id, axis, value } => write!(f, "Joystick {} axis {}: {}", id, axis, value),
        }
    }
}

pub fn start_input_listener<R: Runtime>(app_handle: tauri::AppHandle<R>) {
    // Start rdev for mouse absolute position tracking across all platforms
    let position_app_handle = app_handle.clone();
    thread::spawn(move || {
        let callback = move |event: RdevEvent| {
            if let RdevEventType::MouseMove { x, y } = event.event_type {
                let input_event = InputEvent::MouseMove { x, y };
                if let Err(e) = position_app_handle.emit("input-event", input_event) {
                    eprintln!("Failed to emit mouse position event: {}", e);
                }
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
                            State::Pressed => Some(InputEvent::KeyPress {
                                key: format!("{:?}", key),
                            }),
                            State::Released => Some(InputEvent::KeyRelease {
                                key: format!("{:?}", key),
                            }),
                        },
                        RawEvent::MouseMoveEvent(_device_id, x, y) => Some(InputEvent::MouseDelta { x, y }),
                        RawEvent::MouseButtonEvent(_device_id, button, state) => Some(InputEvent::MouseClick {
                            button: format!("{:?}", button),
                            state: format!("{:?}", state),
                        }),
                        RawEvent::MouseWheelEvent(_device_id, delta) => Some(InputEvent::MouseWheel { delta }),
                        RawEvent::JoystickButtonEvent(device_id, button, state) => Some(InputEvent::JoystickButton {
                            id: device_id,
                            button,
                            state: format!("{:?}", state),
                        }),
                        RawEvent::JoystickAxisEvent(device_id, axis, value) => Some(InputEvent::JoystickAxis {
                            id: device_id,
                            axis: format!("{:?}", axis),
                            value,
                        }),
                        _ => None,
                    };

                    if let Some(event) = input_event {
                        if let Err(e) = windows_app_handle.emit("input-event", event) {
                            eprintln!("Failed to emit input event: {}", e);
                        }
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
                    RdevEventType::KeyPress(key) => Some(InputEvent::KeyPress {
                        key: format!("{:?}", key),
                    }),
                    RdevEventType::KeyRelease(key) => Some(InputEvent::KeyRelease {
                        key: format!("{:?}", key),
                    }),
                    RdevEventType::ButtonPress(button) => Some(InputEvent::MouseClick {
                        button: format!("{:?}", button),
                        state: "pressed".to_string(),
                    }),
                    RdevEventType::ButtonRelease(button) => Some(InputEvent::MouseClick {
                        button: format!("{:?}", button),
                        state: "released".to_string(),
                    }),
                    RdevEventType::Wheel { delta_x, delta_y } => Some(InputEvent::MouseWheel {
                        delta: delta_y as f32,
                    }),
                    RdevEventType::MouseMove { .. } => None, // Handled by the cross-platform listener
                };

                if let Some(event) = input_event {
                    if let Err(e) = other_app_handle.emit("input-event", event) {
                        eprintln!("Failed to emit input event: {}", e);
                    }
                }
            };

            if let Err(error) = listen(callback) {
                println!("Error: {:?}", error)
            }
        });
    }
}
