//! Unit tests for the plugin package loading functionality

use tauri_windows_plugin_system::plugin_loader::package::{PluginPackage, PackageError};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

#[test]
fn test_extract_valid_package() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    // Act
    let result = PluginPackage::extract_package(&package_path, &extraction_dir);
    
    // Assert
    assert!(result.is_ok());
    assert!(extraction_dir.exists());
    assert!(extraction_dir.join("plugin.json").exists());
    assert!(extraction_dir.join("plugin.dll").exists());
    assert!(extraction_dir.join("resources").exists());
    assert!(extraction_dir.join("resources/test.txt").exists());
}

#[test]
fn test_extract_corrupted_package() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("corrupted_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_corrupted_zip(&package_path)
        .expect("Failed to create corrupted zip file");
    
    // Act
    let result = PluginPackage::extract_package(&package_path, &extraction_dir);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PackageError::ExtractionFailed(_)) => {
            // Error message should contain details about the extraction failure
        },
        _ => panic!("Expected ExtractionFailed error, got {:?}", result),
    }
}

#[test]
fn test_read_manifest_from_package() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&extraction_dir).expect("Failed to create extraction directory");
    PluginPackage::extract_package(&package_path, &extraction_dir)
        .expect("Failed to extract package");
    
    // Act
    let result = PluginPackage::read_manifest_from_directory(&extraction_dir);
    
    // Assert
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.name, "test-plugin");
    assert_eq!(manifest.version, "1.0.0");
}

#[test]
fn test_read_manifest_missing_file() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let extraction_dir = temp_dir.path().join("extracted");
    
    fs::create_dir_all(&extraction_dir).expect("Failed to create extraction directory");
    
    // Act
    let result = PluginPackage::read_manifest_from_directory(&extraction_dir);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PackageError::ManifestNotFound) => {
            // Expected error
        },
        _ => panic!("Expected ManifestNotFound error, got {:?}", result),
    }
}

#[test]
fn test_validate_package_structure() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    PluginPackage::extract_package(&package_path, &extraction_dir)
        .expect("Failed to extract package");
    
    // Act
    let result = PluginPackage::validate_package_structure(&extraction_dir);
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_validate_package_structure_missing_dll() {
    // Arrange
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    PluginPackage::extract_package(&package_path, &extraction_dir)
        .expect("Failed to extract package");
    
    // Remove the DLL file to simulate a missing DLL
    fs::remove_file(extraction_dir.join("plugin.dll"))
        .expect("Failed to remove DLL file");
    
    // Act
    let result = PluginPackage::validate_package_structure(&extraction_dir);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PackageError::MissingEntryPoint(_)) => {
            // Expected error
        },
        _ => panic!("Expected MissingEntryPoint error, got {:?}", result),
    }
}
