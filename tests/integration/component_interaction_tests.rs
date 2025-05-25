//! Integration tests for component interactions

use tauri_windows_plugin_system::plugin_loader::PluginLoader;
use tauri_windows_plugin_system::plugin_host::PluginHost;
use tauri_windows_plugin_system::permission_system::PermissionSystem;
use tauri_windows_plugin_system::plugin_manager::{PluginManager, PluginInfo, PluginStatus};
use tauri_windows_plugin_system::ui_integration::UiIntegration;
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_interaction_between_loader_and_host() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&extraction_dir).expect("Failed to create extraction directory");
    
    let mut loader = PluginLoader::new();
    let mut host = PluginHost::new();
    
    // Act
    // 1. Extract and validate the plugin package
    let extract_result = loader.extract_plugin_package(&package_path, &extraction_dir);
    assert!(extract_result.is_ok());
    
    // 2. Read and validate the manifest
    let manifest_result = loader.read_and_validate_manifest(&extraction_dir.join("plugin.json"));
    assert!(manifest_result.is_ok());
    let manifest = manifest_result.unwrap();
    
    // 3. Load the plugin DLL (in a real test, we'd need a real DLL)
    // For this integration test, we'll simulate the loading
    let plugin_id = manifest.name.clone();
    let context = host.create_plugin_context(&plugin_id);
    
    // 4. Initialize the plugin in the host
    // Since we can't actually load a DLL in the test, we'll simulate initialization
    let init_result = host.initialize_plugin_for_testing(&plugin_id, &context);
    
    // Assert
    assert!(init_result.is_ok());
    assert!(host.has_plugin(&plugin_id));
}

#[test]
fn test_interaction_between_permission_system_and_plugin_manager() {
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
    
    // Configure permission system to track prompts
    let mut permission_prompted = false;
    permission_system.set_prompt_handler(|plugin_id, permissions| {
        permission_prompted = true;
        println!("[TEST] Prompted for permissions: {:?} for plugin {}", permissions, plugin_id);
        true // Approve permissions
    });
    
    let mut manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
    );
    
    // Act - Install plugin
    let install_result = manager.install_plugin(&package_path);
    
    // Assert
    assert!(install_result.is_ok());
    assert!(permission_prompted, "Permission prompt should have been triggered");
    
    let plugin_id = install_result.unwrap();
    let plugin_permissions = manager.get_plugin_permissions(&plugin_id);
    assert!(!plugin_permissions.is_empty(), "Plugin should have permissions granted");
}

#[test]
fn test_interaction_between_plugin_manager_and_ui_integration() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let plugins_dir = temp_dir.path().join("plugins");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let loader = PluginLoader::new();
    let permission_system = PermissionSystem::new();
    let host = PluginHost::new();
    
    let mut plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
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
    let plugins_result = ui_integration.get_all_plugins();
    
    // Assert
    assert!(plugins_result.is_ok());
    let plugins = plugins_result.unwrap();
    assert_eq!(plugins.len(), 2, "UI integration should return all plugins from manager");
    
    // Act - Enable/disable through UI
    let disable_result = ui_integration.disable_plugin("plugin1");
    assert!(disable_result.is_ok());
    
    let enable_result = ui_integration.enable_plugin("plugin2");
    assert!(enable_result.is_ok());
    
    // Get updated plugins
    let updated_plugins = ui_integration.get_all_plugins().unwrap();
    
    // Assert - Status should be updated
    let plugin1 = updated_plugins.iter().find(|p| p.id == "plugin1").unwrap();
    let plugin2 = updated_plugins.iter().find(|p| p.id == "plugin2").unwrap();
    
    assert_eq!(plugin1.status, PluginStatus::Disabled, "Plugin1 should be disabled");
    assert_eq!(plugin2.status, PluginStatus::Enabled, "Plugin2 should be enabled");
}

#[test]
fn test_end_to_end_flow_from_ui_command_to_plugin_execution() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let plugins_dir = temp_dir.path().join("plugins");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&plugins_dir).expect("Failed to create plugins directory");
    
    let loader = PluginLoader::new();
    let permission_system = PermissionSystem::new();
    let mut host = PluginHost::new();
    
    // Configure the host to track event triggers
    let mut event_triggered = false;
    host.set_event_handler(|event_name, data| {
        if event_name == "test_event" {
            event_triggered = true;
            println!("[TEST] Event triggered: {} with data: {}", event_name, data);
        }
        Ok(())
    });
    
    let plugin_manager = PluginManager::new(
        plugins_dir.to_str().unwrap(),
        Box::new(loader),
        Box::new(permission_system),
        Box::new(host)
    );
    
    let mut ui_integration = UiIntegration::new(plugin_manager);
    
    // Configure UI integration to use the test package
    ui_integration.set_file_dialog_path_for_testing(Some(package_path.to_str().unwrap().to_string()));
    
    // Act - Install plugin through UI
    let install_result = ui_integration.install_plugin_from_file();
    assert!(install_result.is_ok());
    
    // Get the plugin ID
    let plugins = ui_integration.get_all_plugins().unwrap();
    assert!(!plugins.is_empty());
    let plugin_id = plugins[0].id.clone();
    
    // Enable the plugin
    let enable_result = ui_integration.enable_plugin(&plugin_id);
    assert!(enable_result.is_ok());
    
    // Trigger a test event (simulating user action in UI)
    let event_result = ui_integration.trigger_plugin_event("test_event", "test data");
    
    // Assert
    assert!(event_result.is_ok());
    assert!(event_triggered, "Event should have been triggered");
}
