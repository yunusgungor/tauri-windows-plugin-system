//! Sample plugin for the tauri-windows-plugin-system

#[no_mangle]
pub extern "C" fn plugin_init() -> i32 {
    println!("Sample plugin initialized");
    0 // Success
}

#[no_mangle]
pub extern "C" fn plugin_execute(input: *const std::os::raw::c_char) -> *mut std::os::raw::c_char {
    println!("Sample plugin executed");
    std::ptr::null_mut() // Placeholder
}

#[no_mangle]
pub extern "C" fn plugin_cleanup() -> i32 {
    println!("Sample plugin cleaned up");
    0 // Success
}
