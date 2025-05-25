//! Unit tests for the plugin manager functionality

use tauri_windows_plugin_system::plugin_manager::{PluginManager, PluginInfo, PluginStatus, PluginManagerError};
use tauri_windows_plugin_system::plugin_loader::PluginLoader;
use tauri_windows_plugin_system::permission_system::PermissionSystem;
use tauri_windows_plugin_system::plugin_host::PluginHost;
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_plugin_installation_from_file() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new()
        .with_granted_permission("read_file")
        .with_granted_permission("write_file");
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Act
    let result = manager.install_plugin(&package_path);
    
    // Assert
    assert!(result.is_ok());
    let plugin_id = result.unwrap();
    assert!(manager.get_plugin_info(&plugin_id).is_some());
}

#[test]
fn test_plugin_enabling_and_disabling() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin directly to the manager
    let plugin_id = "test-plugin";
    manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Installed,
        path: plugins_dir.join(plugin_id).to_str().unwrap().to_string(),
    });
    
    // Act - Enable plugin
    let enable_result = manager.enable_plugin(plugin_id);
    
    // Assert - Enable
    assert!(enable_result.is_ok());
    let plugin_info = manager.get_plugin_info(plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Enabled);
    
    // Act - Disable plugin
    let disable_result = manager.disable_plugin(plugin_id);
    
    // Assert - Disable
    assert!(disable_result.is_ok());
    let plugin_info = manager.get_plugin_info(plugin_id).unwrap();
    assert_eq!(plugin_info.status, PluginStatus::Disabled);
}

#[test]
fn test_plugin_uninstallation() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin directly to the manager
    let plugin_id = "test-plugin";
    let plugin_path = plugins_dir.join(plugin_id);
    fs::create_dir_all(&plugin_path).expect("Failed to create plugin directory");
    
    manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Installed,
        path: plugin_path.to_str().unwrap().to_string(),
    });
    
    // Act
    let result = manager.uninstall_plugin(plugin_id);
    
    // Assert
    assert!(result.is_ok());
    assert!(manager.get_plugin_info(plugin_id).is_none());
    assert!(!plugin_path.exists());
}

#[test]
fn test_plugin_updating() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new()
        .with_granted_permission("read_file")
        .with_granted_permission("write_file");
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin directly to the manager
    let plugin_id = "test-plugin";
    manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "0.9.0".to_string(), // Older version
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Installed,
        path: plugins_dir.join(plugin_id).to_str().unwrap().to_string(),
    });
    
    // Act
    let result = manager.update_plugin(plugin_id, &package_path);
    
    // Assert
    assert!(result.is_ok());
    let plugin_info = manager.get_plugin_info(plugin_id).unwrap();
    assert_eq!(plugin_info.version, "1.0.0"); // Updated version
}

#[test]
fn test_plugin_registry_persistence() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    let registry_path = temp_dir.path().join("plugin_registry.json");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add some test plugins
    manager.add_plugin_info(PluginInfo {
        id: "plugin1".to_string(),
        name: "Plugin 1".to_string(),
        version: "1.0.0".to_string(),
        description: "First test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Enabled,
        path: plugins_dir.join("plugin1").to_str().unwrap().to_string(),
    });
    
    manager.add_plugin_info(PluginInfo {
        id: "plugin2".to_string(),
        name: "Plugin 2".to_string(),
        version: "1.0.0".to_string(),
        description: "Second test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Disabled,
        path: plugins_dir.join("plugin2").to_str().unwrap().to_string(),
    });
    
    // Act - Save registry
    let save_result = manager.save_registry(&registry_path);
    assert!(save_result.is_ok());
    
    // Create a new manager and load the registry
    let mock_loader2 = mocks::MockPluginLoader::new();
    let mock_permission_system2 = mocks::MockPermissionSystem::new();
    let mock_host2 = mocks::MockPluginHost::new();
    
    let mut new_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader2),
        Box::new(mock_permission_system2),
        Box::new(mock_host2)
    );
    
    // Act - Load registry
    let load_result = new_manager.load_registry(&registry_path);
    
    // Assert
    assert!(load_result.is_ok());
    assert!(new_manager.get_plugin_info("plugin1").is_some());
    assert!(new_manager.get_plugin_info("plugin2").is_some());
    assert_eq!(new_manager.get_plugin_info("plugin1").unwrap().status, PluginStatus::Enabled);
    assert_eq!(new_manager.get_plugin_info("plugin2").unwrap().status, PluginStatus::Disabled);
}

#[test]
fn test_error_handling_during_installation() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("corrupted_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_corrupted_zip(&package_path)
        .expect("Failed to create corrupted zip file");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::with_failure();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Act
    let result = manager.install_plugin(&package_path);
    
    // Assert
    assert!(result.is_err());
}

#[test]
fn test_plugin_querying_functions() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add some test plugins
    manager.add_plugin_info(PluginInfo {
        id: "plugin1".to_string(),
        name: "Plugin 1".to_string(),
        version: "1.0.0".to_string(),
        description: "First test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Enabled,
        path: plugins_dir.join("plugin1").to_str().unwrap().to_string(),
    });
    
    manager.add_plugin_info(PluginInfo {
        id: "plugin2".to_string(),
        name: "Plugin 2".to_string(),
        version: "1.0.0".to_string(),
        description: "Second test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Disabled,
        path: plugins_dir.join("plugin2").to_str().unwrap().to_string(),
    });
    
    // Act & Assert - Get all plugins
    let all_plugins = manager.get_all_plugins();
    assert_eq!(all_plugins.len(), 2);
    
    // Act & Assert - Get enabled plugins
    let enabled_plugins = manager.get_enabled_plugins();
    assert_eq!(enabled_plugins.len(), 1);
    assert_eq!(enabled_plugins[0].id, "plugin1");
    
    // Act & Assert - Get disabled plugins
    let disabled_plugins = manager.get_disabled_plugins();
    assert_eq!(disabled_plugins.len(), 1);
    assert_eq!(disabled_plugins[0].id, "plugin2");
    
    // Act & Assert - Find plugin by name
    let found_plugin = manager.find_plugin_by_name("Plugin 1");
    assert!(found_plugin.is_some());
    assert_eq!(found_plugin.unwrap().id, "plugin1");
    
    // Act & Assert - Plugin exists
    assert!(manager.plugin_exists("plugin1"));
    assert!(!manager.plugin_exists("nonexistent"));
}
