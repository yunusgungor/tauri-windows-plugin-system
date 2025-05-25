//! Plugin Manager Module
//!
//! Coordinates plugin lifecycle operations such as installation, loading, enabling,
//! disabling, and uninstallation. Acts as the central coordinator for the plugin system.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use thiserror::Error;
use log::{info, warn, error};
use tokio::sync::RwLock;

use crate::plugin_loader::{PluginLoader, PluginMetadata, PluginLoadError};
use crate::plugin_host::{PluginHost, PluginHostError};
use crate::permission_system::{PermissionSystem, Permission, PermissionError, PermissionValidationError};

/// Error type for plugin operations
#[derive(Error, Debug)]
pub enum PluginError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    /// Plugin already exists
    #[error("Plugin already exists: {0}")]
    AlreadyExists(String),
    
    /// Plugin is in an invalid state
    #[error("Invalid plugin state: {0}")]
    InvalidState(String),
    
    /// Permission error
    #[error("Permission error: {0}")]
    Permission(#[from] PermissionError),
    
    /// I/O error
    #[error("I/O error: {0}")]
    IO(#[from] io::Error),
    
    /// Plugin load error
    #[error("Plugin load error: {0}")]
    LoadError(#[from] PluginLoadError),
    
    /// Plugin host error
    #[error("Plugin host error: {0}")]
    HostError(#[from] PluginHostError),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Error type for plugin installation
#[derive(Error, Debug)]
pub enum PluginInstallError {
    /// Failed to download plugin
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    /// Failed to load plugin
    #[error("Load failed: {0}")]
    LoadFailed(#[from] PluginLoadError),
    
    /// Permission validation failed
    #[error("Permission validation failed: {0}")]
    PermissionFailed(#[from] PermissionValidationError),
    
    /// Failed to install plugin files
    #[error("Install failed: {0}")]
    InstallFailed(#[from] io::Error),
    
    /// Failed to update registry
    #[error("Registry update failed: {0}")]
    RegistryFailed(String),
    
    /// Plugin already installed
    #[error("Plugin already installed: {0}")]
    AlreadyInstalled(String),
}

/// Error type for plugin updates
#[derive(Error, Debug)]
pub enum PluginUpdateError {
    /// Plugin not found
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    /// Failed to download update
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    /// Failed to load update
    #[error("Load failed: {0}")]
    LoadFailed(#[from] PluginLoadError),
    
    /// No update available
    #[error("No update available")]
    NoUpdateAvailable,
    
    /// Failed to install update files
    #[error("Update installation failed: {0}")]
    InstallFailed(#[from] io::Error),
    
    /// Permission validation failed
    #[error("Permission validation failed: {0}")]
    PermissionFailed(#[from] PermissionValidationError),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Plugin information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique identifier of the plugin
    pub id: String,
    
    /// Name of the plugin
    pub name: String,
    
    /// Version of the plugin
    pub version: String,
    
    /// Description of the plugin
    pub description: String,
    
    /// Author of the plugin
    pub author: String,
    
    /// Homepage URL of the plugin
    pub homepage: Option<String>,
    
    /// Installation path of the plugin
    pub install_path: PathBuf,
    
    /// Status of the plugin (enabled, disabled, error)
    pub status: PluginStatus,
    
    /// Permissions granted to the plugin
    pub permissions: Vec<Permission>,
    
    /// Installation timestamp
    pub installed_at: DateTime<Utc>,
    
    /// Last update timestamp, if any
    pub updated_at: Option<DateTime<Utc>>,
}

/// Status of a plugin
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginStatus {
    /// Plugin is enabled and active
    Enabled,
    
    /// Plugin is disabled
    Disabled,
    
    /// Plugin is in an error state
    Error(String),
    
    /// Plugin is incompatible with the current system
    Incompatible(String),
}

/// Source of a plugin package
#[derive(Debug, Clone)]
pub enum PluginSource {
    /// Local file path
    File(PathBuf),
    
    /// Remote URL
    Url(String),
    
    /// Plugin store identifier
    Store(String),
}

/// Plugin registry for storing plugin metadata
#[derive(Debug, Serialize, Deserialize)]
struct PluginRegistry {
    /// Installed plugins
    plugins: HashMap<String, PluginInfo>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}

/// Plugin manager for coordinating plugin operations
pub struct PluginManager {
    /// Plugin loader
    plugin_loader: PluginLoader,
    
    /// Plugin host
    plugin_host: Arc<RwLock<PluginHost>>,
    
    /// Permission system
    permission_system: Arc<PermissionSystem>,
    
    /// Registry of installed plugins
    registry: Arc<Mutex<PluginRegistry>>,
    
    /// Base directory for plugins
    plugins_dir: PathBuf,
    
    /// Path to the registry file
    registry_path: PathBuf,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(
        plugins_dir: PathBuf,
        registry_path: PathBuf,
        permission_system: Arc<PermissionSystem>,
    ) -> Result<Self, PluginError> {

        // Create plugins directory if it doesn't exist
        fs::create_dir_all(&plugins_dir)?;
        
        // Create extract base directory
        let extract_dir = plugins_dir.join("extract");
        fs::create_dir_all(&extract_dir)?;
        
        // Create plugin loader
        let plugin_loader = PluginLoader::new(extract_dir);
        
        // Create plugin host
        let plugin_host = Arc::new(RwLock::new(PluginHost::new()));
        
        // Load registry if it exists
        let registry = if registry_path.exists() {
            let mut file = File::open(&registry_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            match serde_json::from_str::<PluginRegistry>(&contents) {
                Ok(reg) => reg,
                Err(e) => {
                    warn!("Failed to parse plugin registry: {}", e);
                    PluginRegistry::default()
                },
            }
        } else {
            PluginRegistry::default()
        };
        
        Ok(Self {
            plugin_loader,
            plugin_host,
            permission_system,
            registry: Arc::new(Mutex::new(registry)),
            plugins_dir,
            registry_path,
        })
    }

    /// Save the plugin registry to disk
    fn save_registry(&self) -> Result<(), PluginError> {
        let registry = self.registry.lock().unwrap();
        let contents = serde_json::to_string_pretty(&*registry)?;
        
        // Create parent directories if they don't exist
        if let Some(parent) = self.registry_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&self.registry_path)?;
        file.write_all(contents.as_bytes())?;
        
        Ok(())
    }
    
    /// Install a plugin from a package source
    pub async fn install_plugin(&self, source: PluginSource) -> Result<PluginInfo, PluginInstallError> {
        // Get the package path
        let package_path = match source {
            PluginSource::File(path) => path,
            PluginSource::Url(url) => {
                return Err(PluginInstallError::DownloadFailed(
                    format!("URL installation not yet implemented: {}", url)
                ));
            },
            PluginSource::Store(id) => {
                return Err(PluginInstallError::DownloadFailed(
                    format!("Store installation not yet implemented: {}", id)
                ));
            },
        };
        
        // Load and validate the package
        let metadata = self.plugin_loader.load_plugin_package(&package_path).await?;
        
        // Generate a unique plugin ID
        let plugin_id = format!("{}-{}", metadata.manifest.name.to_lowercase().replace(" ", "-"), metadata.manifest.version);
        
        // Check if plugin is already installed
        {
            let registry = self.registry.lock().unwrap();
            if registry.plugins.contains_key(&plugin_id) {
                return Err(PluginInstallError::AlreadyInstalled(plugin_id));
            }
        }
        
        // Validate permissions
        self.permission_system.validate_permissions(&metadata.manifest.permissions)?;
        
        // Create installation directory
        let install_dir = self.plugins_dir.join(&plugin_id);
        fs::create_dir_all(&install_dir)?;
        
        // Copy files from extraction directory to installation directory
        copy_dir_all(&metadata.install_path, &install_dir)?;
        
        // Create plugin info
        let plugin_info = PluginInfo {
            id: plugin_id.clone(),
            name: metadata.manifest.name.clone(),
            version: metadata.manifest.version.clone(),
            description: metadata.manifest.description.clone(),
            author: metadata.manifest.author.clone(),
            homepage: metadata.manifest.homepage.clone(),
            install_path: install_dir.clone(),
            status: PluginStatus::Disabled, // Start disabled by default
            permissions: metadata.manifest.permissions.clone(),
            installed_at: Utc::now(),
            updated_at: None,
        };
        
        // Update registry
        {
            let mut registry = self.registry.lock().unwrap();
            registry.plugins.insert(plugin_id.clone(), plugin_info.clone());
        }
        
        // Save registry
        if let Err(e) = self.save_registry() {
            error!("Failed to save plugin registry: {}", e);
        }
        
        info!("Plugin '{}' installed successfully", plugin_id);
        
        Ok(plugin_info)
    }
    
    /// Enable a plugin
    pub async fn enable_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        // Get plugin info
        let plugin_info = {
            let registry = self.registry.lock().unwrap();
            registry.plugins.get(plugin_id).cloned().ok_or_else(|| {
                PluginError::NotFound(plugin_id.to_owned())
            })?
        };
        
        // Check if already enabled
        if plugin_info.status == PluginStatus::Enabled {
            return Ok(());
        }
        
        // Check for incompatible status
        if let PluginStatus::Incompatible(reason) = &plugin_info.status {
            return Err(PluginError::InvalidState(
                format!("Cannot enable incompatible plugin: {}", reason)
            ));
        }
        
        // Load plugin DLL
        let dll_path = plugin_info.install_path.join("plugin.dll");
        let metadata = PluginMetadata {
            manifest: serde_json::from_slice(&fs::read(
                plugin_info.install_path.join("plugin.json")
            )?)?,
            install_path: plugin_info.install_path.clone(),
            dll_path,
            installed_at: plugin_info.installed_at,
        };
        
        let loaded_plugin = self.plugin_loader.load_plugin_dll(&metadata)?;
        
        // Check and prompt for permissions if needed
        let permissions = self.permission_system.get_granted_permissions(plugin_id);
        if permissions.is_empty() {
            // Prompt for permissions
            let granted_permissions = self.permission_system.prompt_for_permissions(
                plugin_id,
                &plugin_info.name,
                &metadata.manifest.permissions,
            ).await?;
            
            // Store granted permissions
            self.permission_system.grant_permissions(plugin_id, granted_permissions, true)?;
        }
        
        // Initialize plugin
        let mut plugin_host = self.plugin_host.write().await;
        plugin_host.init_plugin(plugin_id.to_owned(), loaded_plugin)?;
        
        // Update status
        {
            let mut registry = self.registry.lock().unwrap();
            if let Some(plugin) = registry.plugins.get_mut(plugin_id) {
                plugin.status = PluginStatus::Enabled;
            }
        }
        
        // Save registry
        self.save_registry()?;
        
        info!("Plugin '{}' enabled successfully", plugin_id);
        
        Ok(())
    }
    
    /// Disable a plugin
    pub async fn disable_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        // Get plugin info
        let plugin_info = {
            let registry = self.registry.lock().unwrap();
            registry.plugins.get(plugin_id).cloned().ok_or_else(|| {
                PluginError::NotFound(plugin_id.to_owned())
            })?
        };
        
        // Check if already disabled
        if plugin_info.status == PluginStatus::Disabled {
            return Ok(());
        }
        
        // Check if plugin is loaded
        let mut plugin_host = self.plugin_host.write().await;
        if plugin_host.has_plugin(plugin_id) {
            // Teardown plugin
            plugin_host.teardown_plugin(plugin_id)?;
        }
        
        // Update status
        {
            let mut registry = self.registry.lock().unwrap();
            if let Some(plugin) = registry.plugins.get_mut(plugin_id) {
                plugin.status = PluginStatus::Disabled;
            }
        }
        
        // Save registry
        self.save_registry()?;
        
        info!("Plugin '{}' disabled successfully", plugin_id);
        
        Ok(())
    }
    
    /// Uninstall a plugin
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<(), PluginError> {
        // Get plugin info
        let plugin_info = {
            let registry = self.registry.lock().unwrap();
            registry.plugins.get(plugin_id).cloned().ok_or_else(|| {
                PluginError::NotFound(plugin_id.to_owned())
            })?
        };
        
        // Disable the plugin if it's enabled
        if plugin_info.status == PluginStatus::Enabled {
            self.disable_plugin(plugin_id).await?;
        }
        
        // Remove the plugin files
        if plugin_info.install_path.exists() {
            fs::remove_dir_all(&plugin_info.install_path)?;
        }
        
        // Remove from registry
        {
            let mut registry = self.registry.lock().unwrap();
            registry.plugins.remove(plugin_id);
        }
        
        // Revoke permissions
        self.permission_system.revoke_permissions(plugin_id)?;
        
        // Save registry
        self.save_registry()?;
        
        info!("Plugin '{}' uninstalled successfully", plugin_id);
        
        Ok(())
    }
    
    /// Get all installed plugins
    pub fn get_all_plugins(&self) -> Vec<PluginInfo> {
        let registry = self.registry.lock().unwrap();
        registry.plugins.values().cloned().collect()
    }
    
    /// Get a specific plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginInfo> {
        let registry = self.registry.lock().unwrap();
        registry.plugins.get(plugin_id).cloned()
    }
    
    /// Get all enabled plugins
    pub fn get_enabled_plugins(&self) -> Vec<PluginInfo> {
        let registry = self.registry.lock().unwrap();
        registry.plugins.values()
            .filter(|p| p.status == PluginStatus::Enabled)
            .cloned()
            .collect()
    }
    
    /// Get all disabled plugins
    pub fn get_disabled_plugins(&self) -> Vec<PluginInfo> {
        let registry = self.registry.lock().unwrap();
        registry.plugins.values()
            .filter(|p| p.status == PluginStatus::Disabled)
            .cloned()
            .collect()
    }
    
    /// Update a plugin
    pub async fn update_plugin(
        &self,
        plugin_id: &str,
        source: Option<PluginSource>,
    ) -> Result<PluginInfo, PluginUpdateError> {
        // Get plugin info
        let plugin_info = {
            let registry = self.registry.lock().unwrap();
            registry.plugins.get(plugin_id).cloned().ok_or_else(|| {
                PluginUpdateError::NotFound(plugin_id.to_owned())
            })?
        };
        
        // Use provided source or try to get from original install
        let package_path = match source {
            Some(PluginSource::File(path)) => path,
            Some(PluginSource::Url(_)) | Some(PluginSource::Store(_)) => {
                return Err(PluginUpdateError::DownloadFailed(
                    "URL and Store updates not yet implemented".to_owned()
                ));
            },
            None => {
                return Err(PluginUpdateError::DownloadFailed(
                    "Automatic update source detection not yet implemented".to_owned()
                ));
            },
        };
        
        // Load and validate the package
        let metadata = self.plugin_loader.load_plugin_package(&package_path).await?;
        
        // Check if this is actually an update (version is different)
        if metadata.manifest.version == plugin_info.version {
            return Err(PluginUpdateError::NoUpdateAvailable);
        }
        
        // Validate permissions
        self.permission_system.validate_permissions(&metadata.manifest.permissions)?;
        
        // Disable the plugin if it's enabled
        let was_enabled = plugin_info.status == PluginStatus::Enabled;
        if was_enabled {
            self.disable_plugin(plugin_id).await
                .map_err(|e| PluginUpdateError::Other(format!("Failed to disable plugin: {}", e)))?;
        }
        
        // Backup the old version
        let backup_dir = self.plugins_dir.join(format!("{}-backup-{}", plugin_id, Utc::now().timestamp()));
        fs::rename(&plugin_info.install_path, &backup_dir)
            .map_err(|e| PluginUpdateError::InstallFailed(io::Error::new(
                e.kind(),
                format!("Failed to backup plugin: {}", e),
            )))?;
        
        // Create installation directory
        fs::create_dir_all(&plugin_info.install_path)?;
        
        // Copy files from extraction directory to installation directory
        copy_dir_all(&metadata.install_path, &plugin_info.install_path)?;
        
        // Update registry
        let updated_plugin_info = {
            let mut registry = self.registry.lock().unwrap();
            let plugin = registry.plugins.get_mut(plugin_id).ok_or_else(|| {
                PluginUpdateError::Other(format!("Plugin disappeared from registry: {}", plugin_id))
            })?;
            
            plugin.version = metadata.manifest.version.clone();
            plugin.description = metadata.manifest.description.clone();
            plugin.homepage = metadata.manifest.homepage.clone();
            plugin.permissions = metadata.manifest.permissions.clone();
            plugin.status = PluginStatus::Disabled;
            plugin.updated_at = Some(Utc::now());
            
            plugin.clone()
        };
        
        // Save registry
        self.save_registry()
            .map_err(|e| PluginUpdateError::Other(format!("Failed to save registry: {}", e)))?;
        
        // Re-enable the plugin if it was enabled before
        if was_enabled {
            self.enable_plugin(plugin_id).await
                .map_err(|e| PluginUpdateError::Other(format!("Failed to re-enable plugin: {}", e)))?;
        }
        
        // Remove backup if everything went well
        if let Err(e) = fs::remove_dir_all(&backup_dir) {
            warn!("Failed to remove backup directory: {}", e);
        }
        
        info!("Plugin '{}' updated successfully to version {}", plugin_id, updated_plugin_info.version);
        
        Ok(updated_plugin_info)
    }
    
    /// Trigger an event on a plugin
    pub async fn trigger_plugin_event(
        &self,
        plugin_id: &str,
        event_name: &str,
        event_data: &str,
    ) -> Result<i32, PluginError> {
        // Check if plugin exists
        {
            let registry = self.registry.lock().unwrap();
            if !registry.plugins.contains_key(plugin_id) {
                return Err(PluginError::NotFound(plugin_id.to_owned()));
            }
        }
        
        // Check if plugin is enabled
        let plugin_host = self.plugin_host.read().await;
        if !plugin_host.has_plugin(plugin_id) {
            return Err(PluginError::InvalidState(
                format!("Plugin is not enabled: {}", plugin_id)
            ));
        }
        
        // Trigger the event
        let result = plugin_host.trigger_event(plugin_id, event_name, event_data)?;
        
        Ok(result)
    }
}

/// Recursively copy a directory
fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    
    Ok(())
}
