//! Permission System Module
//!
//! Manages plugin permissions with validation and user prompting.
//! Ensures that plugins only access resources they are explicitly permitted to use.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    /// File system access permission
    FileSystem(FileSystemPermission),
    
    /// Network access permission
    Network(NetworkPermission),
    
    /// UI permission
    UI(UIPermission),
    
    /// System access permission
    System(SystemPermission),
}

/// File system access permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FileSystemPermission {
    /// Whether read access is granted
    pub read: bool,
    
    /// Whether write access is granted
    pub write: bool,
    
    /// Paths that can be accessed
    pub paths: Vec<String>,
}

/// Network access permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkPermission {
    /// Hosts that can be accessed
    pub allowed_hosts: Vec<String>,
}

/// UI permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct UIPermission {
    /// Whether the plugin can show notifications
    pub show_notifications: bool,
    
    /// Whether the plugin can create windows
    pub create_windows: bool,
}

/// System access permission
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SystemPermission {
    /// Whether the plugin can read the clipboard
    pub read_clipboard: bool,
    
    /// Whether the plugin can write to the clipboard
    pub write_clipboard: bool,
    
    /// Whether the plugin can read system information
    pub read_system_info: bool,
}

/// Permission grant status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionStatus {
    /// Permission is granted
    Granted,
    
    /// Permission is denied
    Denied,
    
    /// Permission is pending user approval
    Pending,
}

/// Permission-related error
#[derive(Error, Debug)]
pub enum PermissionError {
    /// Permission denied
    #[error("Permission denied: {0}")]
    Denied(String),
    
    /// Failed to save permission settings
    #[error("Failed to save permission settings: {0}")]
    SaveFailed(#[from] std::io::Error),
    
    /// Failed to prompt for permissions
    #[error("Failed to prompt for permissions: {0}")]
    PromptFailed(String),
}

/// Error during permission validation
#[derive(Error, Debug)]
pub enum PermissionValidationError {
    /// Unsupported permission
    #[error("Unsupported permission: {0}")]
    UnsupportedPermission(String),
    
    /// Permission scope too broad
    #[error("Permission scope too large: {0}")]
    ScopeTooLarge(String),
    
    /// Other validation error
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Permission prompt result
#[derive(Debug, Clone)]
pub enum PermissionPromptResult {
    /// User allowed the permissions
    Allowed(Vec<Permission>),
    
    /// User denied the permissions
    Denied(Vec<Permission>),
    
    /// User allowed some permissions and denied others
    Partial {
        /// Permissions that were allowed
        allowed: Vec<Permission>,
        
        /// Permissions that were denied
        denied: Vec<Permission>,
    },
}

/// Permission settings for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermissionSettings {
    /// Plugin ID
    pub plugin_id: String,
    
    /// Granted permissions
    pub granted_permissions: Vec<Permission>,
    
    /// Whether to remember this decision
    pub remember: bool,
}

/// Permission system for managing plugin permissions
pub struct PermissionSystem {
    /// Default permissions
    default_permissions: Vec<Permission>,
    
    /// Granted permissions for each plugin
    permissions: Arc<Mutex<HashMap<String, PluginPermissionSettings>>>,
    
    /// Permission prompt handler
    prompt_handler: Option<Box<dyn PermissionPromptHandler>>,
}

/// Permission prompt handler trait
pub trait PermissionPromptHandler: Send + Sync {
    /// Prompt the user for permissions
    fn prompt_for_permissions(
        &self,
        plugin_id: &str,
        plugin_name: &str,
        permissions: &[Permission],
    ) -> Result<PermissionPromptResult, PermissionError>;
}

impl PermissionSystem {
    /// Create a new permission system with default settings
    pub fn new() -> Self {
        Self {
            default_permissions: Vec::new(),
            permissions: Arc::new(Mutex::new(HashMap::new())),
            prompt_handler: None,
        }
    }
    
    /// Set the permission prompt handler
    pub fn set_prompt_handler<H: PermissionPromptHandler + 'static>(&mut self, handler: H) {
        self.prompt_handler = Some(Box::new(handler));
    }
    
    /// Set default permissions
    pub fn set_default_permissions(&mut self, permissions: Vec<Permission>) {
        self.default_permissions = permissions;
    }
    
    /// Load permission settings from disk
    pub fn load_permissions(&mut self, settings_path: &Path) -> Result<(), PermissionError> {
        if settings_path.exists() {
            let contents = std::fs::read_to_string(settings_path)?;
            let settings: Vec<PluginPermissionSettings> = serde_json::from_str(&contents)
                .map_err(|e| PermissionError::SaveFailed(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse permissions: {}", e),
                )))?;
            
            let mut permissions = self.permissions.lock().unwrap();
            for setting in settings {
                permissions.insert(setting.plugin_id.clone(), setting);
            }
        }
        
        Ok(())
    }
    
    /// Save permission settings to disk
    pub fn save_permissions(&self, settings_path: &Path) -> Result<(), PermissionError> {
        let permissions = self.permissions.lock().unwrap();
        let settings: Vec<PluginPermissionSettings> = permissions.values().cloned().collect();
        
        let contents = serde_json::to_string_pretty(&settings)
            .map_err(|e| PermissionError::SaveFailed(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize permissions: {}", e),
            )))?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = settings_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(settings_path, contents)?;
        
        Ok(())
    }
    
    /// Validate permissions against the allowed permissions
    pub fn validate_permissions(&self, permissions: &[Permission]) -> Result<(), PermissionValidationError> {
        for permission in permissions {
            match permission {
                Permission::FileSystem(fs_perm) => {
                    // Validate file system permissions
                    if fs_perm.paths.is_empty() {
                        return Err(PermissionValidationError::ScopeTooLarge(
                            "File system permission must specify paths".into()
                        ));
                    }
                    
                    // Check for overly broad paths
                    for path in &fs_perm.paths {
                        if path == "*" || path == "/**" || path == "/*" {
                            return Err(PermissionValidationError::ScopeTooLarge(
                                "File system permission too broad".into()
                            ));
                        }
                    }
                },
                Permission::Network(net_perm) => {
                    // Validate network permissions
                    if net_perm.allowed_hosts.is_empty() {
                        return Err(PermissionValidationError::ScopeTooLarge(
                            "Network permission must specify allowed hosts".into()
                        ));
                    }
                    
                    // Check for overly broad hosts
                    for host in &net_perm.allowed_hosts {
                        if host == "*" {
                            return Err(PermissionValidationError::ScopeTooLarge(
                                "Network permission too broad".into()
                            ));
                        }
                    }
                },
                Permission::UI(_) | Permission::System(_) => {
                    // These are generally fine as-is
                }
            }
        }
        
        Ok(())
    }
    
    /// Grant permissions to a plugin
    pub fn grant_permissions(
        &self,
        plugin_id: &str,
        permissions: Vec<Permission>,
        remember: bool,
    ) -> Result<(), PermissionError> {
        let mut permissions_lock = self.permissions.lock().unwrap();
        
        let settings = permissions_lock.entry(plugin_id.to_owned())
            .or_insert_with(|| PluginPermissionSettings {
                plugin_id: plugin_id.to_owned(),
                granted_permissions: Vec::new(),
                remember,
            });
        
        settings.granted_permissions = permissions;
        settings.remember = remember;
        
        Ok(())
    }
    
    /// Check if a specific permission is granted for a plugin
    pub fn is_permission_granted(&self, plugin_id: &str, permission: &Permission) -> bool {
        let permissions_lock = self.permissions.lock().unwrap();
        
        if let Some(settings) = permissions_lock.get(plugin_id) {
            settings.granted_permissions.contains(permission)
        } else {
            // Check default permissions
            self.default_permissions.contains(permission)
        }
    }
    
    /// Prompt the user for permissions
    pub async fn prompt_for_permissions(
        &self,
        plugin_id: &str,
        plugin_name: &str,
        permissions: &[Permission],
    ) -> Result<Vec<Permission>, PermissionError> {
        // Check if permissions are already granted
        let already_granted: Vec<Permission> = {
            let permissions_lock = self.permissions.lock().unwrap();
            
            if let Some(settings) = permissions_lock.get(plugin_id) {
                if settings.remember {
                    return Ok(settings.granted_permissions.clone());
                }
                
                settings.granted_permissions.clone()
            } else {
                Vec::new()
            }
        };
        
        // Filter out already granted permissions
        let permissions_to_request: Vec<Permission> = permissions
            .iter()
            .filter(|p| !already_granted.contains(p))
            .cloned()
            .collect();
        
        if permissions_to_request.is_empty() {
            return Ok(already_granted);
        }
        
        // Prompt the user
        if let Some(handler) = &self.prompt_handler {
            match handler.prompt_for_permissions(plugin_id, plugin_name, &permissions_to_request)? {
                PermissionPromptResult::Allowed(allowed) => {
                    // Combine with already granted permissions
                    let mut all_granted = already_granted.clone();
                    all_granted.extend(allowed);
                    
                    Ok(all_granted)
                },
                PermissionPromptResult::Denied(denied) => {
                    Err(PermissionError::Denied(format!("Permission denied: {:?}", denied)))
                },
                PermissionPromptResult::Partial { allowed, denied } => {
                    if !denied.is_empty() {
                        return Err(PermissionError::Denied(
                            format!("Some permissions were denied: {:?}", denied)
                        ));
                    }
                    
                    // Combine with already granted permissions
                    let mut all_granted = already_granted.clone();
                    all_granted.extend(allowed);
                    
                    Ok(all_granted)
                },
            }
        } else {
            Err(PermissionError::PromptFailed("No permission prompt handler set".into()))
        }
    }
    
    /// Get all granted permissions for a plugin
    pub fn get_granted_permissions(&self, plugin_id: &str) -> Vec<Permission> {
        let permissions_lock = self.permissions.lock().unwrap();
        
        if let Some(settings) = permissions_lock.get(plugin_id) {
            settings.granted_permissions.clone()
        } else {
            Vec::new()
        }
    }
    
    /// Revoke all permissions for a plugin
    pub fn revoke_permissions(&self, plugin_id: &str) -> Result<(), PermissionError> {
        let mut permissions_lock = self.permissions.lock().unwrap();
        permissions_lock.remove(plugin_id);
        
        Ok(())
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::FileSystem(fs_perm) => {
                write!(
                    f,
                    "File system access ({}{}) to: {}",
                    if fs_perm.read { "read" } else { "" },
                    if fs_perm.write { if fs_perm.read { "/write" } else { "write" } } else { "" },
                    fs_perm.paths.join(", ")
                )
            },
            Permission::Network(net_perm) => {
                write!(
                    f,
                    "Network access to: {}",
                    net_perm.allowed_hosts.join(", ")
                )
            },
            Permission::UI(ui_perm) => {
                let mut perms = Vec::new();
                if ui_perm.show_notifications {
                    perms.push("show notifications");
                }
                if ui_perm.create_windows {
                    perms.push("create windows");
                }
                
                write!(f, "UI access: {}", perms.join(", "))
            },
            Permission::System(sys_perm) => {
                let mut perms = Vec::new();
                if sys_perm.read_clipboard {
                    perms.push("read clipboard");
                }
                if sys_perm.write_clipboard {
                    perms.push("write clipboard");
                }
                if sys_perm.read_system_info {
                    perms.push("read system info");
                }
                
                write!(f, "System access: {}", perms.join(", "))
            },
        }
    }
}
