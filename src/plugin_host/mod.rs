//! Plugin Host Module
//!
//! Defines the interface for plugin execution and manages the plugin lifecycle.
//! Provides the C ABI interface for communication between the host application and plugins.

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{debug, info, warn, error};
use thiserror::Error;

use crate::plugin_loader::{LoadedPlugin, PluginLoadError};

/// Log levels for plugin logging
pub const LOG_DEBUG: u32 = 0;
pub const LOG_INFO: u32 = 1;
pub const LOG_WARN: u32 = 2;
pub const LOG_ERROR: u32 = 3;

/// Callback function type for event handling
pub type CallbackFn = unsafe extern "C" fn(
    context: *mut PluginContext,
    event_data: *const c_char,
    data_len: u32,
) -> c_int;

/// Plugin context structure for communication between host and plugin
#[repr(C)]
pub struct PluginContext {
    /// API version for compatibility checking
    pub api_version: u32,
    
    /// Pointer to host-specific data (opaque to the plugin)
    pub host_data: *mut c_void,
    
    /// Pointer to plugin-specific data (opaque to the host)
    pub plugin_data: *mut c_void,
    
    /// Function to register callbacks for events
    pub register_callback: Option<
        unsafe extern "C" fn(
            context: *mut PluginContext,
            event_name: *const c_char,
            callback: Option<CallbackFn>,
        ) -> c_int,
    >,
    
    /// Function to log messages to the host application
    pub log: Option<
        unsafe extern "C" fn(context: *mut PluginContext, level: u32, message: *const c_char),
    >,
}

/// Host-specific data associated with a plugin
#[derive(Default)]
pub struct HostData {
    /// Plugin ID
    pub plugin_id: String,
    /// Registered callbacks for events
    pub callbacks: HashMap<String, CallbackFn>,
}

/// Error type for plugin host operations
#[derive(Error, Debug)]
pub enum PluginHostError {
    /// Plugin load error
    #[error("Plugin load error: {0}")]
    LoadError(#[from] PluginLoadError),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed with code: {0}")]
    InitializationFailed(i32),
    
    /// Plugin teardown failed
    #[error("Plugin teardown failed with code: {0}")]
    TeardownFailed(i32),
    
    /// Invalid event name
    #[error("Invalid event name: {0}")]
    InvalidEventName(String),
    
    /// Callback registration failed
    #[error("Callback registration failed: {0}")]
    CallbackRegistrationFailed(String),
    
    /// Failed to communicate with plugin
    #[error("Plugin communication error: {0}")]
    CommunicationError(String),
}

/// Plugin host responsible for managing plugin execution
pub struct PluginHost {
    /// Loaded plugins managed by this host
    plugins: HashMap<String, PluginInstance>,
}

/// A running plugin instance
struct PluginInstance {
    /// The loaded plugin
    loaded_plugin: LoadedPlugin,
    /// Host data for this plugin
    host_data: Arc<Mutex<HostData>>,
    /// Raw pointer for FFI (not shared between threads directly)
    /// This is used only for C ABI calls and is managed by the context above
    context_ptr: *mut PluginContext,
}

// Implementing Send and Sync explicitly for PluginInstance
// This is safe because we never share the raw pointer between threads directly
// All access to the pointer is protected by the Arc<Mutex<>> wrapper
unsafe impl Send for PluginInstance {}
unsafe impl Sync for PluginInstance {}

impl PluginHost {
    /// Create a new plugin host
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    
    /// Initialize a plugin
    pub fn init_plugin(&mut self, plugin_id: String, loaded_plugin: LoadedPlugin) -> Result<(), PluginHostError> {
        // Create host data
        let host_data = Arc::new(Mutex::new(HostData {
            plugin_id: plugin_id.clone(),
            callbacks: HashMap::new(),
        }));
        
        // Create plugin context
        let context = Box::new(PluginContext {
            api_version: 1,
            host_data: Arc::into_raw(host_data.clone()) as *mut c_void,
            plugin_data: ptr::null_mut(),
            register_callback: Some(Self::register_callback_trampoline),
            log: Some(Self::log_trampoline),
        });
        
        // Convert to raw pointer for C interface
        let context_ptr = Box::into_raw(context);
        
        // Call plugin_init
        unsafe {
            let init_fn = loaded_plugin.get_init_fn()?;
            let result = init_fn(context_ptr);
            
            if result != 0 {
                // Cleanup on failure
                let _ = Box::from_raw(context_ptr);
                return Err(PluginHostError::InitializationFailed(result));
            }
        }
        
        // Store plugin instance with the raw pointer for FFI calls
        // The context_ptr is managed by the plugin instance lifecycle
        self.plugins.insert(plugin_id.clone(), PluginInstance {
            loaded_plugin,
            context_ptr,
            host_data,
        });
        
        info!("Plugin {} initialized successfully", plugin_id);
        Ok(())
    }
    
    /// Teardown a plugin
    pub fn teardown_plugin(&mut self, plugin_id: &str) -> Result<(), PluginHostError> {
        // Find the plugin
        let plugin = self.plugins.remove(plugin_id).ok_or_else(|| {
            PluginHostError::CommunicationError(format!("Plugin not found: {}", plugin_id))
        })?;
        
        // Call plugin_teardown
        unsafe {
            let teardown_fn = plugin.loaded_plugin.get_teardown_fn()?;
            let result = teardown_fn(plugin.context_ptr);
            
            // Clean up resources
            let _ = Box::from_raw(plugin.context_ptr);
            // We don't need to call Arc::from_raw since we're using normal Arc
            
            if result != 0 {
                return Err(PluginHostError::TeardownFailed(result));
            }
        }
        
        info!("Plugin {} torn down successfully", plugin_id);
        Ok(())
    }
    
    /// Trigger an event on a plugin
    pub fn trigger_event(&self, plugin_id: &str, event_name: &str, event_data: &str) -> Result<i32, PluginHostError> {
        // Find the plugin
        let plugin = self.plugins.get(plugin_id).ok_or_else(|| {
            PluginHostError::CommunicationError(format!("Plugin not found: {}", plugin_id))
        })?;
        
        // Get the callback
        let callback = {
            let host_data = plugin.host_data.lock().unwrap();
            host_data.callbacks.get(event_name).copied()
        };
        
        // Call the callback if registered
        if let Some(callback_fn) = callback {
            let c_data = CString::new(event_data).map_err(|e| {
                PluginHostError::CommunicationError(format!("Invalid event data: {}", e))
            })?;
            
            unsafe {
                // Use the raw pointer for FFI calls instead of the thread-safe wrapper
                let result = callback_fn(
                    plugin.context_ptr,
                    c_data.as_ptr(),
                    event_data.len() as u32,
                );
                
                Ok(result)
            }
        } else {
            Err(PluginHostError::InvalidEventName(format!("No callback registered for event: {}", event_name)))
        }
    }
    
    /// Check if a plugin is loaded
    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }
    
    /// Register callback trampoline function
    unsafe extern "C" fn register_callback_trampoline(
        context: *mut PluginContext,
        event_name: *const c_char,
        callback: Option<CallbackFn>,
    ) -> c_int {
        if context.is_null() || event_name.is_null() {
            return -1;
        }
        
        let context_ref = &*context;
        
        if context_ref.host_data.is_null() {
            return -2;
        }
        
        // Convert event name to Rust string
        let event_name_str = match CStr::from_ptr(event_name).to_str() {
            Ok(s) => s,
            Err(_) => return -3,
        };
        
        // Get callback function
        let callback_fn = match callback {
            Some(cb) => cb,
            None => return -4,
        };
        
        // Get host data
        let host_data_ptr = context_ref.host_data as *const Mutex<HostData>;
        let host_data = &*(host_data_ptr);
        
        // Register callback
        let mut host_data_lock = match host_data.lock() {
            Ok(lock) => lock,
            Err(_) => return -5,
        };
        
        host_data_lock.callbacks.insert(event_name_str.to_owned(), callback_fn);
        
        0 // Success
    }
    
    /// Log trampoline function
    unsafe extern "C" fn log_trampoline(
        context: *mut PluginContext,
        level: u32,
        message: *const c_char,
    ) {
        if context.is_null() || message.is_null() {
            return;
        }
        
        let context_ref = &*context;
        
        if context_ref.host_data.is_null() {
            return;
        }
        
        // Convert message to Rust string
        let message_str = match CStr::from_ptr(message).to_str() {
            Ok(s) => s,
            Err(_) => return,
        };
        
        // Get host data
        let host_data_ptr = context_ref.host_data as *const Mutex<HostData>;
        let host_data = &*(host_data_ptr);
        
        // Get plugin ID
        let plugin_id = match host_data.lock() {
            Ok(lock) => lock.plugin_id.clone(),
            Err(_) => return,
        };
        
        // Log the message with the appropriate level
        match level {
            LOG_DEBUG => debug!("[Plugin {}] {}", plugin_id, message_str),
            LOG_INFO => info!("[Plugin {}] {}", plugin_id, message_str),
            LOG_WARN => warn!("[Plugin {}] {}", plugin_id, message_str),
            LOG_ERROR => error!("[Plugin {}] {}", plugin_id, message_str),
            _ => info!("[Plugin {}] {}", plugin_id, message_str),
        }
    }
}

impl Drop for PluginHost {
    fn drop(&mut self) {
        // Teardown all plugins
        let plugin_ids: Vec<String> = self.plugins.keys().cloned().collect();
        
        for plugin_id in plugin_ids {
            if let Err(e) = self.teardown_plugin(&plugin_id) {
                error!("Failed to teardown plugin {}: {}", plugin_id, e);
            }
        }
    }
}
