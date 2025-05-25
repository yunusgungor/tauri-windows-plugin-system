# Tauri Windows Plugin System - Developer Guide

## Introduction

This guide is intended for developers who want to create plugins for the Tauri Windows Plugin System. It covers the plugin development process, from setting up the development environment to creating and testing plugins.

## Prerequisites

Before you begin, ensure you have the following installed:

- Rust (2021 edition or newer)
- Cargo
- Visual Studio with C++ build tools (for Windows development)
- Tauri development environment

## Plugin Structure

A plugin for the Tauri Windows Plugin System consists of the following components:

1. **Plugin DLL**: A dynamic link library that implements the plugin interface
2. **Plugin Manifest**: A JSON file (plugin.json) that describes the plugin
3. **Resources** (optional): Additional files needed by the plugin

These components are packaged together in a ZIP file with the following structure:

```
plugin.zip
u251cu2500u2500 plugin.json    # Plugin manifest
u251cu2500u2500 plugin.dll     # Plugin binary
u2514u2500u2500 resources/     # Optional plugin resources
```

## Creating a Plugin

### 1. Create a New Rust Project

Start by creating a new Rust library project:

```bash
cargo new --lib my_plugin
cd my_plugin
```

### 2. Configure Cargo.toml

Modify your `Cargo.toml` to create a dynamic library:

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Add any dependencies your plugin needs
```

### 3. Implement the Plugin Interface

Implement the required C ABI interface in your `src/lib.rs` file:

```rust
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

// Plugin context structure matching the host-provided structure
#[repr(C)]
pub struct PluginContext {
    pub api_version: u32,
    pub host_data: *mut c_void,
    pub plugin_data: *mut c_void,
    pub register_callback: Option<
        unsafe extern "C" fn(
            context: *mut PluginContext,
            event_name: *const c_char,
            callback: Option<CallbackFn>,
        ) -> c_int,
    >,
    pub log: Option<
        unsafe extern "C" fn(context: *mut PluginContext, level: u32, message: *const c_char),
    >,
}

// Callback function type
pub type CallbackFn = unsafe extern "C" fn(
    context: *mut PluginContext,
    event_data: *const c_char,
    data_len: u32,
) -> c_int;

// Log levels
pub const LOG_DEBUG: u32 = 0;
pub const LOG_INFO: u32 = 1;
pub const LOG_WARN: u32 = 2;
pub const LOG_ERROR: u32 = 3;

// Plugin data structure - customize based on your plugin's needs
pub struct PluginData {
    // Add fields your plugin needs to store
    initialized: bool,
}

// Helper function to log messages
unsafe fn log(context: *mut PluginContext, level: u32, message: &str) {
    if !context.is_null() {
        let context_ref = &*context;
        if let Some(log_fn) = context_ref.log {
            let c_message = CString::new(message).unwrap_or_default();
            log_fn(context, level, c_message.as_ptr());
        }
    }
}

// Helper function to get plugin data
unsafe fn get_plugin_data(context: *mut PluginContext) -> Option<&'static mut PluginData> {
    if context.is_null() {
        return None;
    }
    
    let context_ref = &mut *context;
    if context_ref.plugin_data.is_null() {
        return None;
    }
    
    Some(&mut *(context_ref.plugin_data as *mut PluginData))
}

// Example callback function
unsafe extern "C" fn example_callback(
    context: *mut PluginContext,
    event_data: *const c_char,
    _data_len: u32,
) -> c_int {
    if context.is_null() || event_data.is_null() {
        return -1;
    }
    
    let data_str = match CStr::from_ptr(event_data).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    log(context, LOG_INFO, &format!("Received event data: {}", data_str));
    
    // Process the event data here
    
    0 // Success
}

#[no_mangle]
pub extern "C" fn plugin_init(context: *mut PluginContext) -> c_int {
    // Check for null context
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        let context_ref = &mut *context;
        
        // Check API version compatibility
        if context_ref.api_version != 1 {
            return -2; // Version mismatch
        }
        
        // Initialize plugin data
        let plugin_data = Box::new(PluginData {
            initialized: true,
        });
        
        // Store plugin data in context
        context_ref.plugin_data = Box::into_raw(plugin_data) as *mut c_void;
        
        // Register callbacks
        if let Some(register_fn) = context_ref.register_callback {
            let event_name = CString::new("example_event").unwrap_or_default();
            let result = register_fn(
                context,
                event_name.as_ptr(),
                Some(example_callback),
            );
            
            if result != 0 {
                log(context, LOG_ERROR, "Failed to register callback");
                return -3;
            }
        } else {
            return -4; // Missing register_callback function
        }
        
        log(context, LOG_INFO, "Plugin initialized successfully");
        
        0 // Success
    }
}

#[no_mangle]
pub extern "C" fn plugin_teardown(context: *mut PluginContext) -> c_int {
    // Check for null context
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        let context_ref = &mut *context;
        
        // Clean up plugin data
        if !context_ref.plugin_data.is_null() {
            // Convert back to Box and drop
            let _ = Box::from_raw(context_ref.plugin_data as *mut PluginData);
            context_ref.plugin_data = ptr::null_mut();
        }
        
        log(context, LOG_INFO, "Plugin teardown completed");
        
        0 // Success
    }
}
```

### 4. Create the Plugin Manifest

Create a `plugin.json` file in your project directory:

```json
{
  "name": "my_plugin",
  "version": "0.1.0",
  "entry": "plugin.dll",
  "api_version": "1.0.0",
  "permissions": [
    {
      "type": "file_system",
      "read": true,
      "write": false,
      "paths": ["data/"]
    }
  ],
  "description": "My first Tauri plugin",
  "author": "Your Name",
  "homepage": "https://example.com"
}
```

### 5. Build the Plugin

Build your plugin using Cargo:

```bash
cargo build --release
```

The compiled DLL will be in the `target/release` directory.

### 6. Package the Plugin

Create a ZIP file containing your plugin.json, the compiled DLL, and any resources your plugin needs:

```bash
# Copy the DLL to your project directory
cp target/release/my_plugin.dll plugin.dll

# Create a ZIP file
zip -r my_plugin.zip plugin.json plugin.dll resources/
```

## Plugin Interface Reference

### Plugin Context

The `PluginContext` structure provides the interface between the plugin and the host application:

- `api_version`: The version of the plugin API
- `host_data`: Pointer to host-specific data (opaque to the plugin)
- `plugin_data`: Pointer to plugin-specific data (opaque to the host)
- `register_callback`: Function to register callbacks for events
- `log`: Function to log messages to the host application

### Required Exports

Your plugin must export the following functions with C linkage:

#### plugin_init

```c
int32_t plugin_init(PluginContext* context);
```

Called when the plugin is loaded. Initialize your plugin and register callbacks here.

Return values:
- 0: Success
- Negative values: Error (see error codes below)

#### plugin_teardown

```c
int32_t plugin_teardown(PluginContext* context);
```

Called when the plugin is unloaded. Clean up resources here.

Return values:
- 0: Success
- Negative values: Error (see error codes below)

### Error Codes

- `-1`: Null context
- `-2`: API version mismatch
- `-3`: Failed to register callback
- `-4`: Missing required function pointer

## Permissions

Plugins must declare the permissions they need in their manifest. The available permission types are:

### File System Permissions

```json
{
  "type": "file_system",
  "read": true,
  "write": false,
  "paths": ["data/", "logs/"]
}
```

### Network Permissions

```json
{
  "type": "network",
  "allowed_hosts": ["api.example.com", "cdn.example.com"]
}
```

### UI Permissions

```json
{
  "type": "ui",
  "show_notifications": true,
  "create_windows": false
}
```

### System Permissions

```json
{
  "type": "system",
  "read_clipboard": true,
  "write_clipboard": false,
  "read_system_info": true
}
```

## Best Practices

### Memory Management

- Always check pointers for null before dereferencing
- Clean up all resources in `plugin_teardown`
- Use RAII patterns where possible
- Avoid global/static mutable state

### Error Handling

- Return appropriate error codes
- Log detailed error messages
- Handle all possible error cases
- Fail gracefully

### Threading

- Assume callbacks may be called from different threads
- Use proper synchronization for shared data
- Don't block the main thread for long operations

### Security

- Request only the permissions you need
- Validate all input data
- Don't trust data from the host or other plugins
- Handle sensitive data securely

## Debugging Plugins

### Logging

Use the provided logging function to debug your plugin:

```rust
unsafe {
    log(context, LOG_DEBUG, "Debug message");
    log(context, LOG_INFO, "Info message");
    log(context, LOG_WARN, "Warning message");
    log(context, LOG_ERROR, "Error message");
}
```

### Debugging with Visual Studio

1. Open your Rust project in Visual Studio
2. Set breakpoints in your code
3. Attach the debugger to the host application process
4. Debug as usual

## Example Plugins

### Hello World Plugin

A minimal plugin that logs a message when initialized and registers a simple callback:

```rust
#[no_mangle]
pub extern "C" fn plugin_init(context: *mut PluginContext) -> c_int {
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        log(context, LOG_INFO, "Hello, World! Plugin initialized.");
        // Register callbacks, etc.
    }
    
    0
}

#[no_mangle]
pub extern "C" fn plugin_teardown(context: *mut PluginContext) -> c_int {
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        log(context, LOG_INFO, "Goodbye, World! Plugin teardown.");
        // Clean up resources
    }
    
    0
}
```

### File System Plugin

A plugin that demonstrates file system access:

```rust
// Example callback that reads a file
unsafe extern "C" fn read_file_callback(
    context: *mut PluginContext,
    event_data: *const c_char,
    _data_len: u32,
) -> c_int {
    if context.is_null() || event_data.is_null() {
        return -1;
    }
    
    let path_str = match CStr::from_ptr(event_data).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    
    // Read file content
    match std::fs::read_to_string(path_str) {
        Ok(content) => {
            log(context, LOG_INFO, &format!("Read file: {}", path_str));
            // Process content
            0
        },
        Err(e) => {
            log(context, LOG_ERROR, &format!("Failed to read file: {}", e));
            -1
        }
    }
}
```

## Troubleshooting

### Common Issues

#### Plugin fails to load

- Check that the DLL is compiled for the correct architecture
- Verify that all dependencies are available
- Check for API version mismatch
- Look for initialization errors in the logs

#### Memory access violations

- Check for null pointer dereferences
- Ensure proper ownership of resources
- Look for use-after-free bugs
- Verify that you're not accessing freed memory

#### Permission errors

- Verify that your plugin manifest declares all required permissions
- Check that permission requests are being approved by the user
- Ensure you're not attempting to access resources outside of granted permissions

## Conclusion

This guide covers the basics of developing plugins for the Tauri Windows Plugin System. For more advanced topics, refer to the API reference documentation and example plugins.
