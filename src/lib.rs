//! Tauri Windows Plugin System
//!
//! A modular, secure plugin system for Tauri applications running on Windows.
//! Allows dynamic loading and unloading of plugins with a comprehensive permission system.

// Re-export main modules
pub mod plugin_loader;
pub mod plugin_host;
pub mod permission_system;
pub mod plugin_manager;
pub mod ui_integration;

// Re-export common types
pub use plugin_loader::PluginLoadError;
pub use plugin_host::PluginContext;
pub use permission_system::{Permission, PermissionError, PermissionValidationError};
pub use plugin_manager::{PluginManager, PluginInfo, PluginStatus, PluginError};
