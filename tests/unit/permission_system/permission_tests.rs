//! Unit tests for the permission system functionality

use crate::permission_system::{PermissionSystem, Permission, PermissionError};
use std::collections::HashSet;

#[test]
fn test_permission_validation_with_valid_permissions() {
    // Arrange
    let permission_system = PermissionSystem::new();
    let permissions = vec!["read_file".to_string(), "write_file".to_string()];
    
    // Act
    let result = permission_system.validate_permissions(&permissions);
    
    // Assert
    assert!(result.is_ok());
}

#[test]
fn test_permission_validation_with_invalid_permissions() {
    // Arrange
    let permission_system = PermissionSystem::new();
    let permissions = vec!["read_file".to_string(), "invalid_permission".to_string()];
    
    // Act
    let result = permission_system.validate_permissions(&permissions);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PermissionError::InvalidPermission(permission)) => {
            assert_eq!(permission, "invalid_permission");
        },
        _ => panic!("Expected InvalidPermission error, got {:?}", result),
    }
}

#[test]
fn test_permission_validation_with_overly_broad_permissions() {
    // Arrange
    let permission_system = PermissionSystem::new();
    let permissions = vec!["all".to_string()];
    
    // Act
    let result = permission_system.validate_permissions(&permissions);
    
    // Assert
    assert!(result.is_err());
    match result {
        Err(PermissionError::OverlyBroadPermission(permission)) => {
            assert_eq!(permission, "all");
        },
        _ => panic!("Expected OverlyBroadPermission error, got {:?}", result),
    }
}

#[test]
fn test_permission_prompting() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permissions = vec!["read_file".to_string(), "write_file".to_string()];
    
    // Mock user response - in a real test this would use a mock UI
    let user_response = true; // User grants permission
    
    // Act
    let result = permission_system.prompt_for_permissions(plugin_id, &permissions, |plugin, perms| {
        // This would show a UI prompt in a real application
        assert_eq!(plugin, plugin_id);
        assert_eq!(perms, &permissions);
        user_response
    });
    
    // Assert
    assert!(result.is_ok());
    assert!(result.unwrap()); // User granted permission
}

#[test]
fn test_permission_prompting_denied() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permissions = vec!["read_file".to_string(), "write_file".to_string()];
    
    // Mock user response - in a real test this would use a mock UI
    let user_response = false; // User denies permission
    
    // Act
    let result = permission_system.prompt_for_permissions(plugin_id, &permissions, |plugin, perms| {
        // This would show a UI prompt in a real application
        assert_eq!(plugin, plugin_id);
        assert_eq!(perms, &permissions);
        user_response
    });
    
    // Assert
    assert!(result.is_ok());
    assert!(!result.unwrap()); // User denied permission
}

#[test]
fn test_permission_granting_and_checking() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permission = "read_file";
    
    // Act - Grant permission
    permission_system.grant_permission(plugin_id, permission)
        .expect("Failed to grant permission");
    
    // Assert - Check permission
    let has_permission = permission_system.is_permission_granted(plugin_id, permission)
        .expect("Failed to check permission");
    assert!(has_permission);
}

#[test]
fn test_permission_revocation() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permission = "read_file";
    
    // Grant permission first
    permission_system.grant_permission(plugin_id, permission)
        .expect("Failed to grant permission");
    
    // Act - Revoke permission
    permission_system.revoke_permission(plugin_id, permission)
        .expect("Failed to revoke permission");
    
    // Assert - Check permission
    let has_permission = permission_system.is_permission_granted(plugin_id, permission)
        .expect("Failed to check permission");
    assert!(!has_permission);
}

#[test]
fn test_permission_serialization_and_deserialization() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permissions = vec!["read_file", "write_file"];
    
    // Grant permissions
    for permission in &permissions {
        permission_system.grant_permission(plugin_id, permission)
            .expect("Failed to grant permission");
    }
    
    // Act - Serialize
    let serialized = permission_system.serialize()
        .expect("Failed to serialize permissions");
    
    // Create a new permission system and deserialize
    let mut new_permission_system = PermissionSystem::new();
    new_permission_system.deserialize(&serialized)
        .expect("Failed to deserialize permissions");
    
    // Assert - Check permissions in the new system
    for permission in &permissions {
        let has_permission = new_permission_system.is_permission_granted(plugin_id, permission)
            .expect("Failed to check permission");
        assert!(has_permission);
    }
}

#[test]
fn test_bulk_permission_operations() {
    // Arrange
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "test-plugin";
    let permissions = vec!["read_file".to_string(), "write_file".to_string()];
    
    // Act - Grant multiple permissions
    permission_system.grant_permissions(plugin_id, &permissions)
        .expect("Failed to grant permissions");
    
    // Assert - Check all permissions
    for permission in &permissions {
        let has_permission = permission_system.is_permission_granted(plugin_id, permission)
            .expect("Failed to check permission");
        assert!(has_permission);
    }
    
    // Act - Revoke all permissions
    permission_system.revoke_all_permissions(plugin_id)
        .expect("Failed to revoke all permissions");
    
    // Assert - Check all permissions are revoked
    for permission in &permissions {
        let has_permission = permission_system.is_permission_granted(plugin_id, permission)
            .expect("Failed to check permission");
        assert!(!has_permission);
    }
}
