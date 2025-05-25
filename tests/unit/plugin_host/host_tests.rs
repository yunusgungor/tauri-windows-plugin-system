//! Unit tests for the plugin host functionality

use tauri_windows_plugin_system::plugin_host::{PluginHost, PluginContext, PluginError};
use std::path::Path;
use std::ptr::null_mut;
use tempfile::tempdir;
use std::fs;

/// Mock plugin initialization function for testing
#[no_mangle]
extern "C" fn mock_plugin_init(context: *mut std::ffi::c_void) -> i32 {
    if context.is_null() {
        return -1;
    }
    0 // Success
}

/// Mock plugin teardown function for testing
#[no_mangle]
extern "C" fn mock_plugin_teardown() -> i32 {
    0 // Success
}

#[test]
fn test_plugin_initialization_with_valid_context() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    let context = PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0,
    };
    
    // Mock the DLL loading - in a real test we'd need to use dynamic library loading
    // For now, we're just testing the logic around it
    
    // Act
    let result = host.initialize_plugin(plugin_id, &context, mock_plugin_init);
    
    // Assert
    assert!(result.is_ok());
    assert!(host.has_plugin(plugin_id));
}

#[test]
fn test_plugin_initialization_with_null_context() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    
    // This would be a function that passes null as the context
    let null_context_init = |_: *mut std::ffi::c_void| -> i32 { -1 };
    
    // Act
    let result = host.initialize_plugin(plugin_id, &PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0,
    }, null_context_init);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PluginError::InitializationFailed(_)) => {
            // Expected error
        },
        _ => panic!("Expected InitializationFailed error, got {:?}", result),
    }
    assert!(!host.has_plugin(plugin_id));
}

#[test]
fn test_plugin_teardown() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    let context = PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0,
    };
    
    // Initialize plugin first
    host.initialize_plugin(plugin_id, &context, mock_plugin_init)
        .expect("Failed to initialize plugin");
    
    // Act
    let result = host.teardown_plugin(plugin_id, mock_plugin_teardown);
    
    // Assert
    assert!(result.is_ok());
    assert!(!host.has_plugin(plugin_id));
}

#[test]
fn test_plugin_teardown_not_loaded() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    
    // Act
    let result = host.teardown_plugin(plugin_id, mock_plugin_teardown);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PluginError::PluginNotFound(_)) => {
            // Expected error
        },
        _ => panic!("Expected PluginNotFound error, got {:?}", result),
    }
}

#[test]
fn test_callback_registration() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    let context = PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0,
    };
    
    // Initialize plugin first
    host.initialize_plugin(plugin_id, &context, mock_plugin_init)
        .expect("Failed to initialize plugin");
    
    // Act
    let result = host.register_callback(plugin_id, "test_event", |data| {
        // Test callback function
        println!("Callback called with data: {:?}", data);
        Ok(())
    });
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_callback_registration_plugin_not_found() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    
    // Act
    let result = host.register_callback(plugin_id, "test_event", |data| {
        // Test callback function
        println!("Callback called with data: {:?}", data);
        Ok(())
    });
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PluginError::PluginNotFound(_)) => {
            // Expected error
        },
        _ => panic!("Expected PluginNotFound error, got {:?}", result),
    }
}

#[test]
fn test_event_triggering() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    let context = PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0,
    };
    
    // Initialize plugin first
    host.initialize_plugin(plugin_id, &context, mock_plugin_init)
        .expect("Failed to initialize plugin");
    
    // Register a callback
    host.register_callback(plugin_id, "test_event", |data| {
        // Test callback function
        assert_eq!(data, "test_data");
        Ok(())
    }).expect("Failed to register callback");
    
    // Act
    let result = host.trigger_event("test_event", "test_data");
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_event_triggering_no_subscribers() {
    // Arrange
    let mut host = PluginHost::new();
    
    // Act
    let result = host.trigger_event("test_event", "test_data");
    
    // Assert
    assert!(result.is_ok()); // Should not fail even if no subscribers
}

#[test]
fn test_logging_functionality() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    let context = PluginContext {
        plugin_id: plugin_id.to_string(),
        app_data_dir: "/tmp/app_data".to_string(),
        log_level: 0, // Debug level
    };
    
    // Initialize plugin first
    host.initialize_plugin(plugin_id, &context, mock_plugin_init)
        .expect("Failed to initialize plugin");
    
    // Act
    let result = host.log(plugin_id, 0, "Test log message");
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_logging_plugin_not_found() {
    // Arrange
    let plugin_id = "test-plugin";
    let mut host = PluginHost::new();
    
    // Act
    let result = host.log(plugin_id, 0, "Test log message");
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PluginError::PluginNotFound(_)) => {
            // Expected error
        },
        _ => panic!("Expected PluginNotFound error, got {:?}", result),
    }
}
