# Tauri Windows Plugin System - API Reference

## Overview

This API reference documents the interfaces, types, and functions provided by the Tauri Windows Plugin System. It is intended for both plugin developers and application developers integrating the plugin system into their Tauri applications.

## Core Interfaces

### Plugin API

#### Plugin Initialization and Teardown

```rust
// Required plugin export functions (implemented by plugins)

/// Initialize the plugin
/// 
/// # Parameters
/// * `context` - Pointer to the plugin context
/// 
/// # Returns
/// * `0` on success
/// * Negative values on error
#[no_mangle]
extern "C" fn plugin_init(context: *mut PluginContext) -> i32;

/// Clean up and tear down the plugin
/// 
/// # Parameters
/// * `context` - Pointer to the plugin context
/// 
/// # Returns
/// * `0` on success
/// * Negative values on error
#[no_mangle]
extern "C" fn plugin_teardown(context: *mut PluginContext) -> i32;
```

#### Plugin Context Structure

```rust
/// Plugin context structure for communication between host and plugin
#[repr(C)]
pub struct PluginContext {
    /// API version for compatibility checking
    pub api_version: u32,
    
    /// Pointer to host-specific data (opaque to the plugin)
    pub host_data: *mut c_void,
    
    /// Pointer to plugin-specific data (opaque to the host)
    pub plugin_data: *mut c_void,
    
    /// Function to register callbacks for events
    pub register_callback: Option<
        unsafe extern "C" fn(
            context: *mut PluginContext,
            event_name: *const c_char,
            callback: Option<CallbackFn>,
        ) -> c_int,
    >,
    
    /// Function to log messages to the host application
    pub log: Option<
        unsafe extern "C" fn(context: *mut PluginContext, level: u32, message: *const c_char),
    >,
}
```

#### Callback Function Type

```rust
/// Callback function type for event handling
pub type CallbackFn = unsafe extern "C" fn(
    context: *mut PluginContext,
    event_data: *const c_char,
    data_len: u32,
) -> c_int;
```

#### Log Levels

```rust
/// Debug level logging
pub const LOG_DEBUG: u32 = 0;
/// Info level logging
pub const LOG_INFO: u32 = 1;
/// Warning level logging
pub const LOG_WARN: u32 = 2;
/// Error level logging
pub const LOG_ERROR: u32 = 3;
```

### Plugin Manager API

#### Plugin Installation

```rust
/// Install a plugin from a package source
/// 
/// # Parameters
/// * `source` - The source of the plugin package (file path, URL, etc.)
/// 
/// # Returns
/// * `Result<PluginInfo, PluginInstallError>` - Plugin information on success, error on failure
pub async fn install_plugin(source: PluginSource) -> Result<PluginInfo, PluginInstallError>;
```

#### Plugin Lifecycle Management

```rust
/// Enable a plugin by its ID
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// 
/// # Returns
/// * `Result<(), PluginError>` - Success or error
pub async fn enable_plugin(plugin_id: &str) -> Result<(), PluginError>;

/// Disable a plugin by its ID
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// 
/// # Returns
/// * `Result<(), PluginError>` - Success or error
pub async fn disable_plugin(plugin_id: &str) -> Result<(), PluginError>;

/// Uninstall a plugin by its ID
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// 
/// # Returns
/// * `Result<(), PluginError>` - Success or error
pub async fn uninstall_plugin(plugin_id: &str) -> Result<(), PluginError>;

/// Update a plugin to a newer version
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// * `source` - Optional source for the update, if different from the original source
/// 
/// # Returns
/// * `Result<PluginInfo, PluginUpdateError>` - Updated plugin information on success, error on failure
pub async fn update_plugin(plugin_id: &str, source: Option<PluginSource>) -> Result<PluginInfo, PluginUpdateError>;
```

#### Plugin Query

```rust
/// Get information about all installed plugins
/// 
/// # Returns
/// * `Vec<PluginInfo>` - List of plugin information
pub fn get_all_plugins() -> Vec<PluginInfo>;

/// Get information about a specific plugin by its ID
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// 
/// # Returns
/// * `Option<PluginInfo>` - Plugin information if found, None otherwise
pub fn get_plugin(plugin_id: &str) -> Option<PluginInfo>;

/// Get all enabled plugins
/// 
/// # Returns
/// * `Vec<PluginInfo>` - List of enabled plugin information
pub fn get_enabled_plugins() -> Vec<PluginInfo>;

/// Get all disabled plugins
/// 
/// # Returns
/// * `Vec<PluginInfo>` - List of disabled plugin information
pub fn get_disabled_plugins() -> Vec<PluginInfo>;
```

### Permission System API

#### Permission Validation

```rust
/// Validate a set of permissions against allowed permissions
/// 
/// # Parameters
/// * `permissions` - The permissions to validate
/// 
/// # Returns
/// * `Result<(), PermissionValidationError>` - Success or error
pub fn validate_permissions(permissions: &[Permission]) -> Result<(), PermissionValidationError>;

/// Check if a specific permission is granted
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// * `permission` - The permission to check
/// 
/// # Returns
/// * `bool` - True if the permission is granted, false otherwise
pub fn is_permission_granted(plugin_id: &str, permission: &Permission) -> bool;
```

#### Permission Prompting

```rust
/// Prompt the user to grant permissions
/// 
/// # Parameters
/// * `plugin_id` - The unique identifier of the plugin
/// * `plugin_name` - The name of the plugin
/// * `permissions` - The permissions to request
/// 
/// # Returns
/// * `Result<Vec<Permission>, PermissionPromptError>` - Granted permissions on success, error on failure
pub async fn prompt_for_permissions(
    plugin_id: &str,
    plugin_name: &str,
    permissions: &[Permission],
) -> Result<Vec<Permission>, PermissionPromptError>;
```

### UI Integration API

#### Tauri Commands

```rust
/// Register Tauri commands for plugin management
/// 
/// # Parameters
/// * `app` - The Tauri application handle
/// 
/// # Returns
/// * `Result<(), TauriError>` - Success or error
pub fn register_plugin_commands(app: &mut App) -> Result<(), TauriError>;
```

## Data Types

### Plugin Types

#### PluginInfo

```rust
/// Information about a plugin
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
```

#### PluginSource

```rust
/// Source of a plugin package
pub enum PluginSource {
    /// Local file path
    File(PathBuf),
    
    /// Remote URL
    Url(String),
    
    /// Plugin store identifier
    Store(String),
}
```

#### PluginStatus

```rust
/// Status of a plugin
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
```

#### PluginMetadata

```rust
/// Internal metadata about a plugin
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
```

#### PluginManifest

```rust
/// Plugin manifest information
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
    pub homepage: Option<String>,
}
```

### Permission Types

#### Permission

```rust
/// Permission definition
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
```

#### FileSystemPermission

```rust
/// File system access permission
pub struct FileSystemPermission {
    /// Whether read access is granted
    pub read: bool,
    
    /// Whether write access is granted
    pub write: bool,
    
    /// Paths that can be accessed
    pub paths: Vec<String>,
}
```

#### NetworkPermission

```rust
/// Network access permission
pub struct NetworkPermission {
    /// Hosts that can be accessed
    pub allowed_hosts: Vec<String>,
}
```

#### UIPermission

```rust
/// UI permission
pub struct UIPermission {
    /// Whether the plugin can show notifications
    pub show_notifications: bool,
    
    /// Whether the plugin can create windows
    pub create_windows: bool,
}
```

#### SystemPermission

```rust
/// System access permission
pub struct SystemPermission {
    /// Whether the plugin can read the clipboard
    pub read_clipboard: bool,
    
    /// Whether the plugin can write to the clipboard
    pub write_clipboard: bool,
    
    /// Whether the plugin can read system information
    pub read_system_info: bool,
}
```

### Error Types

#### PluginError

```rust
/// Generic plugin error
pub enum PluginError {
    /// Plugin not found
    NotFound(String),
    
    /// Plugin already exists
    AlreadyExists(String),
    
    /// Plugin is in an invalid state
    InvalidState(String),
    
    /// Permission error
    Permission(PermissionError),
    
    /// I/O error
    IO(std::io::Error),
    
    /// Other error with message
    Other(String),
}
```

#### PluginLoadError

```rust
/// Error during plugin loading
pub enum PluginLoadError {
    /// Failed to extract plugin package
    ExtractFailed(std::io::Error),
    
    /// Failed to read or parse plugin manifest
    ManifestError(String),
    
    /// Plugin is not compatible
    Incompatible(String),
    
    /// Failed to load plugin DLL
    DllLoadFailed(String),
    
    /// Required export function missing
    MissingExport(String),
    
    /// Plugin initialization failed
    InitializationFailed(i32),
}
```

#### PluginInstallError

```rust
/// Error during plugin installation
pub enum PluginInstallError {
    /// Failed to download plugin
    DownloadFailed(String),
    
    /// Failed to load plugin
    LoadFailed(PluginLoadError),
    
    /// Permission validation failed
    PermissionFailed(PermissionValidationError),
    
    /// Failed to install plugin files
    InstallFailed(std::io::Error),
    
    /// Failed to update registry
    RegistryFailed(String),
}
```

#### PermissionError

```rust
/// Permission-related error
pub enum PermissionError {
    /// Permission denied
    Denied(String),
    
    /// Failed to save permission settings
    SaveFailed(std::io::Error),
    
    /// Failed to prompt for permissions
    PromptFailed(String),
}
```

#### PermissionValidationError

```rust
/// Error during permission validation
pub enum PermissionValidationError {
    /// Unsupported permission
    UnsupportedPermission(String),
    
    /// Permission scope too broad
    ScopeTooLarge(String),
    
    /// Other validation error
    ValidationFailed(String),
}
```

## Events

### Plugin Events

```rust
/// Plugin status changed event
pub struct PluginStatusChangedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// New status of the plugin
    pub status: PluginStatus,
}

/// Plugin installed event
pub struct PluginInstalledEvent {
    /// Information about the installed plugin
    pub plugin: PluginInfo,
}

/// Plugin uninstalled event
pub struct PluginUninstalledEvent {
    /// ID of the uninstalled plugin
    pub plugin_id: String,
}

/// Plugin updated event
pub struct PluginUpdatedEvent {
    /// Information about the updated plugin
    pub plugin: PluginInfo,
    
    /// Previous version of the plugin
    pub previous_version: String,
}

/// Permission granted event
pub struct PermissionGrantedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// Permissions that were granted
    pub permissions: Vec<Permission>,
}

/// Permission denied event
pub struct PermissionDeniedEvent {
    /// ID of the plugin
    pub plugin_id: String,
    
    /// Permissions that were denied
    pub permissions: Vec<Permission>,
}
```

## UI Components

### React Components

```typescript
// Plugin list component props
interface PluginListProps {
  plugins: PluginInfo[];
  onEnable: (pluginId: string) => Promise<void>;
  onDisable: (pluginId: string) => Promise<void>;
  onUninstall: (pluginId: string) => Promise<void>;
  onSettings: (pluginId: string) => void;
}

// Plugin details component props
interface PluginDetailsProps {
  plugin: PluginInfo;
  onEnable: (pluginId: string) => Promise<void>;
  onDisable: (pluginId: string) => Promise<void>;
  onUninstall: (pluginId: string) => Promise<void>;
  onUpdate: (pluginId: string) => Promise<void>;
  onSettings: (pluginId: string) => void;
}

// Permission prompt component props
interface PermissionPromptProps {
  pluginName: string;
  permissions: Permission[];
  onAllow: () => void;
  onDeny: () => void;
  onRemember: (remember: boolean) => void;
}

// Plugin installation component props
interface PluginInstallProps {
  onInstallFromFile: () => Promise<void>;
  onInstallFromUrl: (url: string) => Promise<void>;
  onInstallFromStore: () => void;
}

// Plugin settings component props
interface PluginSettingsProps {
  plugin: PluginInfo;
  settings: Record<string, any>;
  onSave: (settings: Record<string, any>) => Promise<void>;
  onCancel: () => void;
}
```

## JavaScript / TypeScript API

### Plugin Management

```typescript
// Install a plugin from a file
async function installPluginFromFile(): Promise<PluginInfo>;

// Install a plugin from a URL
async function installPluginFromUrl(url: string): Promise<PluginInfo>;

// Get all installed plugins
async function getAllPlugins(): Promise<PluginInfo[]>;

// Get a specific plugin by ID
async function getPlugin(pluginId: string): Promise<PluginInfo | null>;

// Enable a plugin
async function enablePlugin(pluginId: string): Promise<void>;

// Disable a plugin
async function disablePlugin(pluginId: string): Promise<void>;

// Uninstall a plugin
async function uninstallPlugin(pluginId: string): Promise<void>;

// Update a plugin
async function updatePlugin(pluginId: string): Promise<PluginInfo>;

// Get plugin settings
async function getPluginSettings(pluginId: string): Promise<Record<string, any>>;

// Save plugin settings
async function savePluginSettings(pluginId: string, settings: Record<string, any>): Promise<void>;
```

### Event Listening

```typescript
// Listen for plugin status changes
function onPluginStatusChanged(callback: (event: PluginStatusChangedEvent) => void): () => void;

// Listen for plugin installation
function onPluginInstalled(callback: (event: PluginInstalledEvent) => void): () => void;

// Listen for plugin uninstallation
function onPluginUninstalled(callback: (event: PluginUninstalledEvent) => void): () => void;

// Listen for plugin updates
function onPluginUpdated(callback: (event: PluginUpdatedEvent) => void): () => void;

// Listen for permission grants
function onPermissionGranted(callback: (event: PermissionGrantedEvent) => void): () => void;

// Listen for permission denials
function onPermissionDenied(callback: (event: PermissionDeniedEvent) => void): () => void;
```

## Configuration

### Plugin System Configuration

```typescript
interface PluginSystemConfig {
  // Directory to store plugins
  pluginDir: string;
  
  // Whether to enable automatic updates
  autoUpdate: boolean;
  
  // Update check interval in hours (0 to disable)
  updateCheckInterval: number;
  
  // Default permission settings
  defaultPermissions: {
    // Whether to allow file system access by default
    allowFileSystem: boolean;
    
    // Whether to allow network access by default
    allowNetwork: boolean;
    
    // Whether to allow UI operations by default
    allowUI: boolean;
    
    // Whether to allow system access by default
    allowSystem: boolean;
  };
  
  // Plugin sources
  sources: {
    // URL to the plugin store
    storeUrl: string;
    
    // Trusted sources for direct installation
    trustedUrls: string[];
  };
  
  // Security settings
  security: {
    // Whether to verify plugin signatures
    verifySignatures: boolean;
    
    // Whether to allow installation from untrusted sources
    allowUntrustedSources: boolean;
    
    // Whether to isolate plugins from each other
    isolatePlugins: boolean;
  };
}
```

## Implementation Examples

### Installing a Plugin

```rust
// Rust example
async fn install_plugin_example() {
    let plugin_manager = PluginManager::new();
    let source = PluginSource::File(PathBuf::from("path/to/plugin.zip"));
    
    match plugin_manager.install_plugin(source).await {
        Ok(plugin_info) => {
            println!("Installed plugin: {}", plugin_info.name);
            // Enable the plugin
            if let Err(e) = plugin_manager.enable_plugin(&plugin_info.id).await {
                eprintln!("Failed to enable plugin: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Failed to install plugin: {}", e);
        }
    }
}
```

```typescript
// TypeScript example
async function installPluginExample() {
  try {
    // Install from file
    const pluginInfo = await installPluginFromFile();
    console.log(`Installed plugin: ${pluginInfo.name}`);
    
    // Enable the plugin
    await enablePlugin(pluginInfo.id);
    console.log(`Enabled plugin: ${pluginInfo.name}`);
  } catch (error) {
    console.error(`Failed to install or enable plugin: ${error}`);
  }
}
```

### Handling Plugin Events

```typescript
// TypeScript example
function setupPluginEventListeners() {
  // Listen for plugin installation
  const unsubscribeInstall = onPluginInstalled((event) => {
    console.log(`Plugin installed: ${event.plugin.name}`);
  });
  
  // Listen for plugin status changes
  const unsubscribeStatus = onPluginStatusChanged((event) => {
    console.log(`Plugin ${event.plugin_id} status changed to: ${event.status}`);
  });
  
  // Listen for permission grants
  const unsubscribePermission = onPermissionGranted((event) => {
    console.log(`Permissions granted to plugin ${event.plugin_id}:`, event.permissions);
  });
  
  // Return a cleanup function
  return () => {
    unsubscribeInstall();
    unsubscribeStatus();
    unsubscribePermission();
  };
}
```

### Creating a Simple Plugin

```rust
// Rust example
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

#[repr(C)]
pub struct PluginContext {
    pub api_version: u32,
    pub host_data: *mut c_void,
    pub plugin_data: *mut c_void,
    pub register_callback: Option<
        unsafe extern "C" fn(
            context: *mut PluginContext,
            event_name: *const c_char,
            callback: Option<CallbackFn>,
        ) -> c_int,
    >,
    pub log: Option<
        unsafe extern "C" fn(context: *mut PluginContext, level: u32, message: *const c_char),
    >,
}

pub type CallbackFn = unsafe extern "C" fn(
    context: *mut PluginContext,
    event_data: *const c_char,
    data_len: u32,
) -> c_int;

pub const LOG_INFO: u32 = 1;

pub struct PluginData {
    message: String,
}

unsafe fn log(context: *mut PluginContext, level: u32, message: &str) {
    if !context.is_null() {
        let context_ref = &*context;
        if let Some(log_fn) = context_ref.log {
            let c_message = CString::new(message).unwrap_or_default();
            log_fn(context, level, c_message.as_ptr());
        }
    }
}

#[no_mangle]
pub extern "C" fn plugin_init(context: *mut PluginContext) -> c_int {
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        let context_ref = &mut *context;
        
        // Create plugin data
        let plugin_data = Box::new(PluginData {
            message: "Hello from plugin!".to_string(),
        });
        
        // Store plugin data
        context_ref.plugin_data = Box::into_raw(plugin_data) as *mut c_void;
        
        log(context, LOG_INFO, "Plugin initialized successfully");
        
        0 // Success
    }
}

#[no_mangle]
pub extern "C" fn plugin_teardown(context: *mut PluginContext) -> c_int {
    if context.is_null() {
        return -1;
    }
    
    unsafe {
        let context_ref = &mut *context;
        
        // Clean up plugin data
        if !context_ref.plugin_data.is_null() {
            let _ = Box::from_raw(context_ref.plugin_data as *mut PluginData);
            context_ref.plugin_data = ptr::null_mut();
        }
        
        log(context, LOG_INFO, "Plugin teardown completed");
        
        0 // Success
    }
}
```
