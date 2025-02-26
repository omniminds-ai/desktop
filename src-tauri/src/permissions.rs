use core_graphics::access::ScreenCaptureAccess;
use rdev::listen;
use std::ffi::c_void;
use std::ptr;

use core_foundation::base::Boolean;

// Type definitions
type CFDictionaryRef = *const c_void;
type CFStringRef = *const c_void;
type CFAllocatorRef = *mut c_void;
type CFTypeRef = *const c_void;

// External ApplicationServices declarations
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> Boolean;
}

// CoreFoundation external declarations
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFAllocatorDefault: CFAllocatorRef;
    static kCFRunLoopDefaultMode: CFStringRef;

    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFNumberCreate(
        allocator: CFAllocatorRef,
        type_id: u32,
        value_ptr: *const c_void,
    ) -> CFTypeRef;
    fn CFStringCreateWithCString(
        alloc: CFAllocatorRef,
        c_str: *const i8,
        encoding: u32,
    ) -> CFStringRef;
    fn CFDictionaryCreate(
        allocator: CFAllocatorRef,
        keys: *const CFTypeRef,
        values: *const CFTypeRef,
        count: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> CFDictionaryRef;
    fn CFRelease(cf: CFTypeRef);
}

// IOKit framework constants
const K_IOHIDOPTIONS_TYPE_NONE: u32 = 0;
const K_HIDPAGE_GENERIC_DESKTOP: i64 = 0x01;
const K_HIDUSAGE_GD_KEYBOARD: i64 = 0x06;

// Type definitions for IOKit
type IOHIDManagerRef = *mut c_void;
type IOHIDDeviceRef = *mut c_void;
type CFRunLoopRef = *const c_void;
type IOOptionBits = u32;
type IOReturn = i32;

// External IOKit function declarations
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDManagerCreate(allocator: CFAllocatorRef, options: IOOptionBits) -> IOHIDManagerRef;
    fn IOHIDManagerSetDeviceMatching(manager: IOHIDManagerRef, matching: CFDictionaryRef);
    fn IOHIDManagerScheduleWithRunLoop(
        manager: IOHIDManagerRef,
        runLoop: CFRunLoopRef,
        mode: CFStringRef,
    );
    fn IOHIDManagerCopyDevices(manager: IOHIDManagerRef) -> *const c_void; // Returns CFSetRef
    fn IOHIDManagerRegisterDeviceMatchingCallback(
        manager: IOHIDManagerRef,
        callback: extern "C" fn(
            context: *mut c_void,
            result: IOReturn,
            sender: *mut c_void,
            device: IOHIDDeviceRef,
        ),
        context: *mut c_void,
    );
    fn IOHIDManagerUnscheduleFromRunLoop(
        manager: IOHIDManagerRef,
        runLoop: CFRunLoopRef,
        mode: CFStringRef,
    );
}

#[tauri::command]
pub fn has_ax_perms() -> bool {
    unsafe {
        // Use AXIsProcessTrusted() directly which is more reliable for checking permissions
        let result = AXIsProcessTrusted() != 0;
        println!("AX permissions: {}", result);
        result
    }
}

#[tauri::command]
pub fn request_ax_perms() {
    #[cfg(target_os = "macos")]
    {
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt();
    }
}

#[tauri::command]
pub fn has_record_perms() -> bool {
    return ScreenCaptureAccess.preflight();
}

#[tauri::command]
pub fn request_record_perms() {
    ScreenCaptureAccess.request();
}

// Global variable to store the result of the device check
static mut HAS_INPUT_PERMS_RESULT: Option<bool> = None;

// Callback function for IOHIDManagerRegisterDeviceMatchingCallback
extern "C" fn device_matching_callback(
    context: *mut c_void,
    _result: IOReturn,
    _sender: *mut c_void,
    _device: IOHIDDeviceRef,
) {
    unsafe {
        // We got a device, which means we have permissions
        HAS_INPUT_PERMS_RESULT = Some(true);

        // Cast context back to CFRunLoopRef and stop the run loop
        let _run_loop = context as CFRunLoopRef;
        // We would need to call CFRunLoopStop here, but it's not necessary for our check
    }
}
