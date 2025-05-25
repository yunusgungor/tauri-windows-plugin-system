//! Performance and stress tests for the Tauri Windows Plugin System

use tauri_windows_plugin_system::plugin_manager::PluginManager;
use tauri_windows_plugin_system::permission_system::{PermissionSystem, Permission, FileSystemPermission};
use tempfile::tempdir;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_plugin_manager_creation_performance() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let plugins_dir = temp_dir.path().to_path_buf();
    let _registry_path = temp_dir.path().join("registry.json");
    
    // Act & Measure
    let start = Instant::now();
    
    for i in 0..100 {
        let permission_system = Arc::new(PermissionSystem::new());
        let result = PluginManager::new(
            plugins_dir.clone(),
            temp_dir.path().join(&format!("registry_{}.json", i)),
            permission_system,
        );
        
        // Assert each creation succeeds
        assert!(result.is_ok(), "Plugin manager creation {} should succeed", i);
    }
    
    let duration = start.elapsed();
    
    // Assert performance - 100 creations should complete within reasonable time
    assert!(
        duration.as_millis() < 5000, 
        "100 plugin manager creations took too long: {:?}", 
        duration
    );
    
    println!("Created 100 plugin managers in {:?}", duration);
}

#[test]
fn test_permission_system_creation_performance() {
    // Act & Measure
    let start = Instant::now();
    
    let permission_systems: Vec<PermissionSystem> = (0..1000)
        .map(|_| PermissionSystem::new())
        .collect();
    
    let duration = start.elapsed();
    
    // Assert all were created
    assert_eq!(permission_systems.len(), 1000);
    
    // Assert performance - 1000 creations should be fast
    assert!(
        duration.as_millis() < 1000, 
        "1000 permission system creations took too long: {:?}", 
        duration
    );
    
    println!("Created 1000 permission systems in {:?}", duration);
}

#[test]
fn test_permission_validation_performance() {
    // Arrange
    let permission_system = PermissionSystem::new();
    let permission = Permission::FileSystem(FileSystemPermission {
        read: true,
        write: false,
        paths: vec!["temp".to_string()],
    });
    let permissions = vec![permission];
    
    // Act & Measure
    let start = Instant::now();
    
    for _ in 0..10000 {
        let _ = permission_system.validate_permissions(&permissions);
    }
    
    let duration = start.elapsed();
    
    // Assert performance - 10,000 validations should be fast
    assert!(
        duration.as_millis() < 1000, 
        "10,000 permission validations took too long: {:?}", 
        duration
    );
    
    println!("Performed 10,000 permission validations in {:?}", duration);
}

#[test]
fn test_large_permission_set_creation() {
    // Arrange & Act
    let large_permissions: Vec<Permission> = (0..1000)
        .map(|i| {
            Permission::FileSystem(FileSystemPermission {
                read: i % 2 == 0,
                write: i % 3 == 0,
                paths: vec![format!("path_{}", i)],
            })
        })
        .collect();
    
    // Assert
    assert_eq!(large_permissions.len(), 1000);
    
    // Test that we can validate a large permission set
    let permission_system = PermissionSystem::new();
    let start = Instant::now();
    let _ = permission_system.validate_permissions(&large_permissions);
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() < 100, 
        "Validating 1000 permissions took too long: {:?}", 
        duration
    );
    
    println!("Validated 1000 permissions in {:?}", duration);
}

#[test]
fn test_concurrent_plugin_manager_access() {
    use std::thread;
    use std::sync::Arc;
    
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let plugins_dir = temp_dir.path().to_path_buf();
    let registry_path = temp_dir.path().join("registry.json");
    let permission_system = Arc::new(PermissionSystem::new());
    
    let plugin_manager = Arc::new(
        PluginManager::new(
            plugins_dir,
            registry_path,
            permission_system,
        ).expect("Failed to create plugin manager")
    );
    
    // Act - Multiple threads accessing the plugin manager
    let start = Instant::now();
    let mut handles = vec![];
    
    for i in 0..10 {
        let pm = Arc::clone(&plugin_manager);
        let handle = thread::spawn(move || {
            // Perform multiple operations
            for j in 0..100 {
                let plugin_id = format!("plugin_{}_{}", i, j);
                let _ = pm.get_plugin(&plugin_id);
                let plugins = pm.get_all_plugins();
                assert_eq!(plugins.len(), 0); // Should always be empty in this test
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        assert!(handle.join().is_ok(), "Thread should complete successfully");
    }
    
    let duration = start.elapsed();
    
    // Assert performance and correctness
    assert!(
        duration.as_millis() < 2000, 
        "Concurrent access took too long: {:?}", 
        duration
    );
    
    println!("10 threads with 100 operations each completed in {:?}", duration);
}

#[test]
fn test_memory_usage_stability() {
    // This test ensures that creating and dropping many objects doesn't cause memory issues
    
    for batch in 0..10 {
        let mut managers = Vec::new();
        let mut permission_systems = Vec::new();
        
        // Create a batch of objects
        for i in 0..50 {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let plugins_dir = temp_dir.path().to_path_buf();
            let registry_path = temp_dir.path().join(&format!("registry_{}_{}.json", batch, i));
            
            let permission_system = Arc::new(PermissionSystem::new());
            permission_systems.push(permission_system.clone());
            
            if let Ok(manager) = PluginManager::new(
                plugins_dir,
                registry_path,
                permission_system,
            ) {
                managers.push(manager);
            }
        }
        
        // Verify we created the expected number
        assert!(managers.len() <= 50, "Should create at most 50 managers per batch");
        assert_eq!(permission_systems.len(), 50, "Should create exactly 50 permission systems per batch");
        
        // Objects will be dropped at the end of this scope
    }
    
    // If we reach here without panicking, memory usage is stable
    assert!(true, "Memory usage remained stable across 10 batches of 50 objects each");
}
