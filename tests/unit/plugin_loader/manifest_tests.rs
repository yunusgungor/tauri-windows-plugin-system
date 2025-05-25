//! Unit tests for the plugin manifest validation functionality

use crate::plugin_loader::manifest::{PluginManifest, ManifestError};
use std::path::PathBuf;

#[test]
fn test_valid_manifest() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin for unit testing",
        "author": "Test Author",
        "permissions": ["read_file", "write_file"],
        "min_host_version": "1.0.0",
        "entry_point": "plugin.dll"
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json);
    
    // Assert
    assert!(result.is_ok());
    let manifest = result.unwrap();
    assert_eq!(manifest.name, "test-plugin");
    assert_eq!(manifest.version, "1.0.0");
    assert_eq!(manifest.description, "A test plugin for unit testing");
    assert_eq!(manifest.author, "Test Author");
    assert_eq!(manifest.permissions, vec!["read_file", "write_file"]);
    assert_eq!(manifest.min_host_version, "1.0.0");
    assert_eq!(manifest.entry_point, "plugin.dll");
}

#[test]
fn test_missing_required_fields() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "1.0.0"
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(ManifestError::MissingField(field)) => {
            // One of the required fields should be missing
            assert!(field == "description" || field == "author" || 
                   field == "permissions" || field == "min_host_version" || 
                   field == "entry_point");
        },
        _ => panic!("Expected MissingField error, got {:?}", result),
    }
}

#[test]
fn test_invalid_version_format() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "invalid-version",
        "description": "A test plugin for unit testing",
        "author": "Test Author",
        "permissions": ["read_file", "write_file"],
        "min_host_version": "1.0.0",
        "entry_point": "plugin.dll"
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(ManifestError::InvalidVersionFormat(field)) => {
            assert_eq!(field, "version");
        },
        _ => panic!("Expected InvalidVersionFormat error, got {:?}", result),
    }
}

#[test]
fn test_invalid_min_host_version_format() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin for unit testing",
        "author": "Test Author",
        "permissions": ["read_file", "write_file"],
        "min_host_version": "invalid-version",
        "entry_point": "plugin.dll"
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(ManifestError::InvalidVersionFormat(field)) => {
            assert_eq!(field, "min_host_version");
        },
        _ => panic!("Expected InvalidVersionFormat error, got {:?}", result),
    }
}

#[test]
fn test_invalid_json_format() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "1.0.0",
        INVALID_JSON
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(ManifestError::InvalidJson(_)) => {
            // Error message should contain details about the JSON parsing error
        },
        _ => panic!("Expected InvalidJson error, got {:?}", result),
    }
}

#[test]
fn test_permissions_validation() {
    // Arrange
    let manifest_json = r#"{
        "name": "test-plugin",
        "version": "1.0.0",
        "description": "A test plugin for unit testing",
        "author": "Test Author",
        "permissions": ["read_file", "invalid_permission"],
        "min_host_version": "1.0.0",
        "entry_point": "plugin.dll"
    }"#;
    
    // Act
    let result = PluginManifest::from_json(manifest_json).and_then(|manifest| {
        manifest.validate_permissions()
    });
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(ManifestError::InvalidPermission(permission)) => {
            assert_eq!(permission, "invalid_permission");
        },
        _ => panic!("Expected InvalidPermission error, got {:?}", result),
    }
}
