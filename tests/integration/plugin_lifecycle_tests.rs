//! Integration tests for complete plugin lifecycle

use tauri_windows_plugin_system::plugin_loader::PluginLoader;
use tauri_windows_plugin_system::plugin_host::PluginHost;
use tauri_windows_plugin_system::permission_system::PermissionSystem;
use tauri_windows_plugin_system::plugin_manager::{PluginManager, PluginStatus};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_complete_plugin_lifecycle() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let loader = PluginLoader::new();
    let permission_system = PermissionSystem::new();
    let host = PluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
    );
    
    // Act 1 - Install
    let install_result = manager.install_plugin(&package_path);
    assert!(install_result.is_ok());
    let plugin_id = install_result.unwrap();
    
    // Assert 1 - Check installed status
    let plugin_info = manager.get_plugin_info(&plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Installed);
    
    // Act 2 - Enable
    let enable_result = manager.enable_plugin(&plugin_id);
    assert!(enable_result.is_ok());
    
    // Assert 2 - Check enabled status
    let plugin_info = manager.get_plugin_info(&plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Enabled);
    
    // Act 3 - Disable
    let disable_result = manager.disable_plugin(&plugin_id);
    assert!(disable_result.is_ok());
    
    // Assert 3 - Check disabled status
    let plugin_info = manager.get_plugin_info(&plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Disabled);
    
    // Act 4 - Uninstall
    let uninstall_result = manager.uninstall_plugin(&plugin_id);
    assert!(uninstall_result.is_ok());
    
    // Assert 4 - Check plugin is removed
    assert!(manager.get_plugin_info(&plugin_id).is_none());
}

#[test]
fn test_plugin_installation_with_permission_prompting() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let loader = PluginLoader::new();
    let mut permission_system = PermissionSystem::new();
    let host = PluginHost::new();
    
    // Configure permission system to always prompt and approve
    permission_system.set_prompt_handler(|plugin_id, permissions| {
        // In a real test, this would interact with UI
        // For integration testing, we'll just auto-approve
        println!("[TEST] Prompted for permissions: {:?} for plugin {}", permissions, plugin_id);
        true
    });
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
    );
    
    // Act - Install with permission prompting
    let install_result = manager.install_plugin(&package_path);
    
    // Assert
    assert!(install_result.is_ok());
    let plugin_id = install_result.unwrap();
    
    // Verify permissions were granted
    let plugin_permissions = manager.get_plugin_permissions(&plugin_id);
    assert!(!plugin_permissions.is_empty());
}

#[test]
fn test_plugin_update_with_version_change() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let old_package_path = temp_dir.path().join("old_plugin_package.zip");
    let new_package_path = temp_dir.path().join("new_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    // Create an old version package
    helpers::create_test_plugin_package(&old_package_path, true)
        .expect("Failed to create old test plugin package");
    
    // Create a new version package (would have different version in real test)
    helpers::create_test_plugin_package(&new_package_path, true)
        .expect("Failed to create new test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let loader = PluginLoader::new();
    let permission_system = PermissionSystem::new();
    let host = PluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
    );
    
    // Act 1 - Install old version
    let install_result = manager.install_plugin(&old_package_path);
    assert!(install_result.is_ok());
    let plugin_id = install_result.unwrap();
    
    // Save the original version
    let original_version = manager.get_plugin_info(&plugin_id).unwrap().version.clone();
    
    // Modify the version info to simulate an older version
    let plugin_info = manager.get_plugin_info_mut(&plugin_id).unwrap();
    plugin_info.version = "0.9.0".to_string(); // Set to older version
    
    // Act 2 - Update to new version
    let update_result = manager.update_plugin(&plugin_id, &new_package_path);
    
    // Assert
    assert!(update_result.is_ok());
    let plugin_info = manager.get_plugin_info(&plugin_id).unwrap();
    assert_eq!(plugin_info.version, original_version); // Should be updated to the newer version
}

#[test]
fn test_plugin_persistence_across_application_restarts() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    let registry_path = temp_dir.path().join("plugin_registry.json");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    // First "application session"
    let loader1 = PluginLoader::new();
    let permission_system1 = PermissionSystem::new();
    let host1 = PluginHost::new();
    
    let mut manager1 = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader1),
        Box::new(permission_system1),
        Box::new(host1)
    );
    
    // Act 1 - Install and enable plugin
    let install_result = manager1.install_plugin(&package_path);
    assert!(install_result.is_ok());
    let plugin_id = install_result.unwrap();
    
    let enable_result = manager1.enable_plugin(&plugin_id);
    assert!(enable_result.is_ok());
    
    // Save registry state
    let save_result = manager1.save_registry(&registry_path);
    assert!(save_result.is_ok());
    
    // Simulate application restart by creating new manager
    let loader2 = PluginLoader::new();
    let permission_system2 = PermissionSystem::new();
    let host2 = PluginHost::new();
    
    let mut manager2 = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader2),
        Box::new(permission_system2),
        Box::new(host2)
    );
    
    // Act 2 - Load registry
    let load_result = manager2.load_registry(&registry_path);
    
    // Assert
    assert!(load_result.is_ok());
    assert!(manager2.plugin_exists(&plugin_id));
    let plugin_info = manager2.get_plugin_info(&plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Enabled);
}
