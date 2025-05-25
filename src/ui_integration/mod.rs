//! UI Integration Module
//!
//! Integrates the plugin system with the Tauri UI via commands and events.
//! Provides the interface for the frontend to interact with the plugin system.

use std::sync::Arc;
use serde::Serialize;
use tauri::{command, State, AppHandle, Runtime, Manager};

use crate::plugin_manager::{PluginManager, PluginInfo, PluginStatus, PluginSource};
use crate::permission_system::{Permission, PermissionSystem, PermissionPromptHandler, PermissionPromptResult, PermissionError};

/// Plugin system state for Tauri
pub struct PluginSystemState(pub Arc<PluginManager>);

/// Get the plugin manager from the plugin system state
impl PluginSystemState {
    /// Get a reference to the plugin manager
    pub fn manager(&self) -> &Arc<PluginManager> {
        &self.0
    }
}

/// Plugin status changed event
#[derive(Clone, Serialize)]
pub struct PluginStatusChangedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// New status of the plugin
    pub status: String,
    
    /// Error message if status is Error
    pub error: Option<String>,
}

/// Plugin installed event
#[derive(Clone, Serialize)]
pub struct PluginInstalledEvent {
    /// Information about the installed plugin
    pub plugin: PluginInfo,
}

/// Plugin uninstalled event
#[derive(Clone, Serialize)]
pub struct PluginUninstalledEvent {
    /// ID of the uninstalled plugin
    pub plugin_id: String,
}

/// Plugin updated event
#[derive(Clone, Serialize)]
pub struct PluginUpdatedEvent {
    /// Information about the updated plugin
    pub plugin: PluginInfo,
    
    /// Previous version of the plugin
    pub previous_version: String,
}

/// Permission granted event
#[derive(Clone, Serialize)]
pub struct PermissionGrantedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// Permissions that were granted
    pub permissions: Vec<String>,
}

/// Permission denied event
#[derive(Clone, Serialize)]
pub struct PermissionDeniedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// Permissions that were denied
    pub permissions: Vec<String>,
}

/// Command result type
type CommandResult<T> = Result<T, String>;

/// Tauri permission prompt handler
pub struct TauriPermissionPromptHandler<R: Runtime> {
    /// Tauri app handle
    app: AppHandle<R>,
}

impl<R: Runtime> TauriPermissionPromptHandler<R> {
    /// Create a new Tauri permission prompt handler
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime> PermissionPromptHandler for TauriPermissionPromptHandler<R> {
    fn prompt_for_permissions(
        &self,
        plugin_id: &str,
        _plugin_name: &str,
        permissions: &[Permission],
    ) -> Result<PermissionPromptResult, PermissionError> {
        // Convert permissions to strings for display
        let permission_strings: Vec<String> = permissions.iter()
            .map(|p| p.to_string())
            .collect();
        
        // In a real implementation, this would show a UI dialog
        // For now, we'll just automatically allow all permissions
        // This should be replaced with actual UI interaction
        
        // Emit permission granted event
        let _ = self.app.emit_all(
            "plugin-permission-granted",
            PermissionGrantedEvent {
                plugin_id: plugin_id.to_owned(),
                permissions: permission_strings,
            },
        );
        
        Ok(PermissionPromptResult::Allowed(permissions.to_vec()))
    }
}

/// Convert plugin status to string
fn status_to_string(status: &PluginStatus) -> String {
    match status {
        PluginStatus::Enabled => "enabled".to_owned(),
        PluginStatus::Disabled => "disabled".to_owned(),
        PluginStatus::Error(_) => "error".to_owned(),
        PluginStatus::Incompatible(_) => "incompatible".to_owned(),
    }
}

/// Command to install a plugin from a file
#[command]
pub async fn install_plugin_from_file(
    state: State<'_, PluginSystemState>,
    path: String,
) -> CommandResult<PluginInfo> {
    let source = PluginSource::File(path.into());
    
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.install_plugin(source).await {
        Ok(plugin_info) => {
            // Log plugin installation (event emission removed)
            println!("Plugin installed: {}", plugin_info.name);
            Ok(plugin_info)
        },
        Err(e) => Err(format!("Failed to install plugin: {}", e)),
    }
}

/// Command to install a plugin from a URL
#[command]
pub async fn install_plugin_from_url(
    state: State<'_, PluginSystemState>,
    url: String,
) -> CommandResult<PluginInfo> {
    let source = PluginSource::Url(url);
    
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.install_plugin(source).await {
        Ok(plugin_info) => {
            // Log plugin installation (event emission removed)
            println!("Plugin installed: {}", plugin_info.name);
            Ok(plugin_info)
        },
        Err(e) => Err(format!("Failed to install plugin: {}", e)),
    }
}

/// Command to get all installed plugins
#[command]
pub fn get_all_plugins(state: State<'_, PluginSystemState>) -> CommandResult<Vec<PluginInfo>> {
    // Access manager through the accessor method
    let manager = state.manager();
    Ok(manager.get_all_plugins())
}

/// Command to get a specific plugin by ID
#[command]
pub fn get_plugin(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
) -> CommandResult<Option<PluginInfo>> {
    // Access manager through the accessor method
    let manager = state.manager();
    Ok(manager.get_plugin(&plugin_id))
}

/// Command to enable a plugin
#[command]
pub async fn enable_plugin(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
) -> CommandResult<()> {
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.enable_plugin(&plugin_id).await {
        Ok(()) => {
            // Log plugin status change (event emission removed)
            if let Some(plugin) = manager.get_plugin(&plugin_id) {
                println!("Plugin enabled: {} - Status: {}", plugin_id, status_to_string(&plugin.status));
            }
            
            Ok(())
        },
        Err(e) => Err(format!("Failed to enable plugin: {}", e)),
    }
}

/// Command to disable a plugin
#[command]
pub async fn disable_plugin(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
) -> CommandResult<()> {
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.disable_plugin(&plugin_id).await {
        Ok(()) => {
            // Log plugin status change (event emission removed)
            if let Some(plugin) = manager.get_plugin(&plugin_id) {
                println!("Plugin disabled: {} - Status: {}", plugin_id, status_to_string(&plugin.status));
            }
            
            Ok(())
        },
        Err(e) => Err(format!("Failed to disable plugin: {}", e)),
    }
}

/// Command to uninstall a plugin
#[command]
pub async fn uninstall_plugin(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
) -> CommandResult<()> {
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.uninstall_plugin(&plugin_id).await {
        Ok(()) => {
            // Log plugin uninstallation (event emission removed)
            println!("Plugin uninstalled: {}", plugin_id);
            Ok(())
        },
        Err(e) => Err(format!("Failed to uninstall plugin: {}", e)),
    }
}

/// Command to update a plugin
#[command]
pub async fn update_plugin(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
    path: Option<String>,
) -> CommandResult<PluginInfo> {
    let source = match path {
        Some(p) => Some(PluginSource::File(p.into())),
        None => None,
    };
    
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.update_plugin(&plugin_id, source).await {
        Ok(plugin_info) => {
            // Get previous version
            let previous_version = match manager.get_plugin(&plugin_id) {
                Some(old_info) => old_info.version,
                None => "unknown".to_owned(),
            };
            
            // Log plugin update (event emission removed)
            println!("Plugin updated: {} - Previous: {}, New: {}", plugin_id, previous_version, plugin_info.version);
            
            Ok(plugin_info)
        },
        Err(e) => Err(format!("Failed to update plugin: {}", e)),
    }
}

/// Command to trigger a plugin event
#[command]
pub async fn trigger_plugin_event(
    state: State<'_, PluginSystemState>,
    plugin_id: String,
    event_name: String,
    event_data: String,
) -> CommandResult<i32> {
    // Access manager through the accessor method
    let manager = state.manager();
    match manager.trigger_plugin_event(&plugin_id, &event_name, &event_data).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Failed to trigger plugin event: {}", e)),
    }
}

/// Register all plugin system commands
pub fn register_commands<R: Runtime>(
    app: &mut tauri::App<R>,
    plugin_manager: Arc<PluginManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create and register the plugin system state
    let plugin_system_state = PluginSystemState(plugin_manager);
    
    app.manage(plugin_system_state);
    
    Ok(())
}

/// Setup a Tauri permission prompt handler
pub fn setup_permission_handler<R: Runtime>(
    app: &mut tauri::App<R>,
    _permission_system: Arc<PermissionSystem>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_handle = app.handle().clone();
    let _handler = TauriPermissionPromptHandler::new(app_handle);
    
    // Note: In a real implementation, we'd need to clone and modify the permission system
    // Since we're using an Arc, we'd need interior mutability or other mechanism
    // For simplicity, we're just showing the concept here
    
    Ok(())
}

/// Plugin system setup for Tauri
pub fn setup<R: Runtime>(
    app: &mut tauri::App<R>,
    plugin_manager: Arc<PluginManager>,
    permission_system: Arc<PermissionSystem>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Register commands
    register_commands(app, plugin_manager)?;
    
    // Setup permission handler
    setup_permission_handler(app, permission_system)?;
    
    Ok(())
}
