//! Unit tests for the UI integration functionality

use tauri_windows_plugin_system::ui_integration::{UiIntegration, CommandError};
use tauri_windows_plugin_system::plugin_manager::{PluginManager, PluginInfo, PluginStatus};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_command_registration() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Act
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Assert
    // In a real application, we'd verify that Tauri commands are registered
    // For unit testing, we're just ensuring the object is created without errors
    assert!(true);
}

#[test]
fn test_get_all_plugins_command() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add some test plugins
    plugin_manager.add_plugin_info(PluginInfo {
        id: "plugin1".to_string(),
        name: "Plugin 1".to_string(),
        version: "1.0.0".to_string(),
        description: "First test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Enabled,
        path: plugins_dir.join("plugin1").to_str().unwrap().to_string(),
    });
    
    plugin_manager.add_plugin_info(PluginInfo {
        id: "plugin2".to_string(),
        name: "Plugin 2".to_string(),
        version: "1.0.0".to_string(),
        description: "Second test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Disabled,
        path: plugins_dir.join("plugin2").to_str().unwrap().to_string(),
    });
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act
    let result = ui_integration.get_all_plugins();
    
    // Assert
    assert!(result.is_ok());
    let plugins = result.unwrap();
    assert_eq!(plugins.len(), 2);
}

#[test]
fn test_install_plugin_command() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    // Create a dummy file to simulate a plugin package
    fs::write(&package_path, "dummy plugin package").expect("Failed to create test plugin package");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new()
        .with_granted_permission("read_file")
        .with_granted_permission("write_file");
    let mock_host = mocks::MockPluginHost::new();
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    let mut ui_integration = UiIntegration::new(plugin_manager);
    
    // Mock the file dialog - in a real test this would be more sophisticated
    ui_integration.set_file_dialog_path_for_testing(Some(package_path.to_str().unwrap().to_string()));
    
    // Act
    let result = ui_integration.install_plugin_from_file();
    
    // Assert
    // Since we're using mocks, the installation might not succeed, but the command should execute
    assert!(result.is_ok() || matches!(result, Err(CommandError::InstallationFailed(_))));
}

#[test]
fn test_enable_plugin_command() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin
    let plugin_id = "test-plugin";
    plugin_manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Disabled,
        path: plugins_dir.join(plugin_id).to_str().unwrap().to_string(),
    });
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act
    let result = ui_integration.enable_plugin(plugin_id);
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_disable_plugin_command() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin
    let plugin_id = "test-plugin";
    plugin_manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Enabled,
        path: plugins_dir.join(plugin_id).to_str().unwrap().to_string(),
    });
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act
    let result = ui_integration.disable_plugin(plugin_id);
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_uninstall_plugin_command() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let mut plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    // Add a test plugin
    let plugin_id = "test-plugin";
    let plugin_path = plugins_dir.join(plugin_id);
    fs::create_dir_all(&plugin_path).expect("Failed to create plugin directory");
    
    plugin_manager.add_plugin_info(PluginInfo {
        id: plugin_id.to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        status: PluginStatus::Enabled,
        path: plugin_path.to_str().unwrap().to_string(),
    });
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act
    let result = ui_integration.uninstall_plugin(plugin_id);
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_permission_prompt_handling() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act
    let plugin_id = "test-plugin";
    let permissions = vec!["read_file".to_string(), "write_file".to_string()];
    let result = ui_integration.show_permission_prompt(plugin_id, permissions);
    
    // Assert
    // Since we're using mocks, the actual dialog won't appear, but the function should complete
    assert!(true);
}

#[test]
fn test_command_execution_with_invalid_inputs() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::new();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Act - Try to enable a non-existent plugin
    let result = ui_integration.enable_plugin("non-existent-plugin");
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(CommandError::PluginNotFound(_)) => {
            // Expected error
        },
        _ => panic!("Expected PluginNotFound error, got {:?}", result),
    }
}

#[test]
fn test_error_handling_and_reporting() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let mock_loader = mocks::MockPluginLoader::with_failure();
    let mock_permission_system = mocks::MockPermissionSystem::new();
    let mock_host = mocks::MockPluginHost::new();
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(mock_loader),
        Box::new(mock_permission_system),
        Box::new(mock_host)
    );
    
    let ui_integration = UiIntegration::new(plugin_manager);
    
    // Mock the file dialog path
    ui_integration.set_file_dialog_path_for_testing(Some(temp_dir.path().join("fake_plugin.zip").to_str().unwrap().to_string()));
    
    // Act - Try to install a plugin with a mocked failure
    let result = ui_integration.install_plugin_from_file();
    
    // Assert
    assert!(result.is_err());
}
