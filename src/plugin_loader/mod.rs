//! Plugin Loader Module
//!
//! Responsible for loading plugin packages, extracting them, and validating their manifests.
//! Handles dynamic loading of plugin DLLs and manages the plugin lifecycle.

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{self, Read};
use libloading::{Library, Symbol};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use zip::ZipArchive;
use thiserror::Error;

use crate::permission_system::Permission;
use crate::plugin_host::PluginContext;

/// Metadata about a loaded plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Installation path
    pub install_path: PathBuf,
    /// Path to the DLL file
    pub dll_path: PathBuf,
    /// Installation timestamp
    pub installed_at: DateTime<Utc>,
}

/// Plugin manifest information extracted from plugin.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Name of the plugin
    pub name: String,
    /// Version of the plugin
    pub version: String,
    /// Entry point (DLL filename)
    pub entry: String,
    /// API version the plugin is compatible with
    pub api_version: String,
    /// Permissions required by the plugin
    pub permissions: Vec<Permission>,
    /// Description of the plugin
    pub description: String,
    /// Author of the plugin
    pub author: String,
    /// Homepage URL of the plugin
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Error type for plugin loading operations
#[derive(Error, Debug)]
pub enum PluginLoadError {
    /// Failed to extract plugin package
    #[error("Failed to extract plugin package: {0}")]
    ExtractFailed(#[from] io::Error),
    
    /// Failed to read or parse plugin manifest
    #[error("Manifest error: {0}")]
    ManifestError(String),
    
    /// Plugin is not compatible
    #[error("Plugin incompatible: {0}")]
    Incompatible(String),
    
    /// Failed to load plugin DLL
    #[error("Failed to load plugin DLL: {0}")]
    DllLoadFailed(String),
    
    /// Required export function missing
    #[error("Missing export function: {0}")]
    MissingExport(String),
    
    /// Plugin initialization failed
    #[error("Plugin initialization failed with code: {0}")]
    InitializationFailed(i32),
    
    /// ZIP extraction error
    #[error("ZIP extraction error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// Function type for plugin initialization
pub type PluginInitFn = unsafe extern "C" fn(context: *mut PluginContext) -> i32;

/// Function type for plugin teardown
pub type PluginTeardownFn = unsafe extern "C" fn(context: *mut PluginContext) -> i32;

/// Represents a loaded plugin DLL
pub struct LoadedPlugin {
    /// The library handle
    library: Library,
    /// Plugin metadata
    metadata: PluginMetadata,
}

impl LoadedPlugin {
    /// Get the init function from the plugin DLL
    pub unsafe fn get_init_fn(&self) -> Result<Symbol<PluginInitFn>, PluginLoadError> {
        self.library.get(b"plugin_init")
            .map_err(|e| PluginLoadError::MissingExport(format!("plugin_init: {}", e)))
    }
    
    /// Get the teardown function from the plugin DLL
    pub unsafe fn get_teardown_fn(&self) -> Result<Symbol<PluginTeardownFn>, PluginLoadError> {
        self.library.get(b"plugin_teardown")
            .map_err(|e| PluginLoadError::MissingExport(format!("plugin_teardown: {}", e)))
    }
    
    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

/// Plugin loader responsible for loading and validating plugins
pub struct PluginLoader {
    /// Base directory for extracting plugins
    extract_base_dir: PathBuf,
}

impl PluginLoader {
    /// Create a new plugin loader with the specified extract directory
    pub fn new(extract_base_dir: PathBuf) -> Self {
        Self { extract_base_dir }
    }
    
    /// Load a plugin package from a path
    pub async fn load_plugin_package(&self, package_path: &Path) -> Result<PluginMetadata, PluginLoadError> {
        // Extract ZIP package
        let extract_dir = self.extract_plugin_package(package_path)?;
        
        // Read and validate manifest
        let manifest_path = extract_dir.join("plugin.json");
        let manifest = self.read_and_validate_manifest(&manifest_path)?;
        
        // Check permissions and compatibility
        self.validate_plugin_compatibility(&manifest)?;
        
        // Create plugin metadata
        let plugin_metadata = PluginMetadata {
            manifest,
            install_path: extract_dir.clone(),
            dll_path: extract_dir.join("plugin.dll"),
            installed_at: Utc::now(),
        };
        
        Ok(plugin_metadata)
    }
    
    /// Load a plugin DLL
    pub fn load_plugin_dll(&self, metadata: &PluginMetadata) -> Result<LoadedPlugin, PluginLoadError> {
        // Load the DLL
        let library = unsafe {
            Library::new(&metadata.dll_path).map_err(|e| {
                PluginLoadError::DllLoadFailed(format!("Failed to load DLL: {}", e))
            })?
        };
        
        // Check required exports
        unsafe {
            let _: Symbol<PluginInitFn> = library.get(b"plugin_init")
                .map_err(|e| PluginLoadError::MissingExport(format!("plugin_init: {}", e)))?;
            
            let _: Symbol<PluginTeardownFn> = library.get(b"plugin_teardown")
                .map_err(|e| PluginLoadError::MissingExport(format!("plugin_teardown: {}", e)))?;
        }
        
        Ok(LoadedPlugin {
            library,
            metadata: metadata.clone(),
        })
    }
    
    /// Extract a plugin package to a temporary directory
    fn extract_plugin_package(&self, package_path: &Path) -> Result<PathBuf, PluginLoadError> {
        // Create a unique directory for extraction
        let extract_dir = self.extract_base_dir.join(format!(
            "plugin_{}", 
            chrono::Utc::now().timestamp_millis()
        ));
        fs::create_dir_all(&extract_dir)?;
        
        // Open the ZIP file
        let file = File::open(package_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Extract all files
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => extract_dir.join(path),
                None => continue,
            };
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        
        Ok(extract_dir)
    }
    
    /// Read and validate the plugin manifest
    fn read_and_validate_manifest(&self, manifest_path: &Path) -> Result<PluginManifest, PluginLoadError> {
        // Read the manifest file
        let mut file = File::open(manifest_path)
            .map_err(|e| PluginLoadError::ManifestError(format!("Failed to open manifest: {}", e)))?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| PluginLoadError::ManifestError(format!("Failed to read manifest: {}", e)))?;
        
        // Parse the manifest
        let manifest: PluginManifest = serde_json::from_str(&contents)?;
        
        // Basic validation
        if manifest.name.is_empty() {
            return Err(PluginLoadError::ManifestError("Plugin name cannot be empty".into()));
        }
        
        if manifest.version.is_empty() {
            return Err(PluginLoadError::ManifestError("Plugin version cannot be empty".into()));
        }
        
        if manifest.entry.is_empty() {
            return Err(PluginLoadError::ManifestError("Plugin entry point cannot be empty".into()));
        }
        
        if manifest.api_version.is_empty() {
            return Err(PluginLoadError::ManifestError("API version cannot be empty".into()));
        }
        
        Ok(manifest)
    }
    
    /// Validate plugin compatibility
    fn validate_plugin_compatibility(&self, manifest: &PluginManifest) -> Result<(), PluginLoadError> {
        // Check API version compatibility
        // For now, we only support API version 1.0.0
        if manifest.api_version != "1.0.0" {
            return Err(PluginLoadError::Incompatible(
                format!("Unsupported API version: {}", manifest.api_version)
            ));
        }
        
        Ok(())
    }
}
