# Tauri Windows Plugin System

A modular, secure plugin system for Tauri applications running on Windows. This system allows dynamic loading and unloading of plugins with a comprehensive permission system, providing a robust foundation for extending your Tauri applications.

## Features

- **Dynamic Loading**: Load and unload plugins at runtime
- **Security-First Design**: Comprehensive permission system for controlled plugin access
- **ZIP Package Format**: Simple plugin distribution format with manifest validation
- **C ABI Interface**: Compatible interface for plugins written in various languages
- **UI Integration**: Seamless integration with Tauri's UI through commands and events
- **Cross-Language Support**: Primary support for Rust plugins with potential for other languages

## Architecture

The plugin system consists of five main components:

1. **Plugin Loader**: Handles loading plugin packages, extracting ZIP files, and validating manifests
2. **Plugin Host**: Defines interfaces for plugin execution and lifecycle management
3. **Permission System**: Manages plugin permissions with validation and user prompting
4. **Plugin Manager**: Coordinates plugin lifecycle operations (install, load, enable, disable, uninstall)
5. **UI Integration**: Integrates with Tauri UI via commands and events

## Getting Started

### Installation

Add the plugin system to your Tauri application's dependencies:

```toml
[dependencies]
tauri-windows-plugin-system = "0.1.0"
```

### Basic Usage

#### Setting Up the Plugin System

```rust
use std::sync::Arc;
use std::path::PathBuf;
use tauri::{App, Manager};
use tauri_windows_plugin_system::
    permission_system::PermissionSystem,
    plugin_manager::PluginManager,
    ui_integration;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Setup paths
            let app_dir = app.path_resolver().app_data_dir().unwrap();
            let plugins_dir = app_dir.join("plugins");
            let registry_path = app_dir.join("plugin_registry.json");
            let permissions_path = app_dir.join("plugin_permissions.json");
            
            // Create permission system
            let permission_system = Arc::new(PermissionSystem::new());
            permission_system.load_permissions(&permissions_path)
                .unwrap_or_else(|e| eprintln!("Failed to load permissions: {}", e));
            
            // Create plugin manager
            let plugin_manager = Arc::new(
                PluginManager::new(plugins_dir, registry_path, permission_system.clone())
                    .expect("Failed to create plugin manager")
            );
            
            // Setup UI integration
            ui_integration::setup(app, plugin_manager, permission_system)?;
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ui_integration::install_plugin_from_file,
            ui_integration::install_plugin_from_url,
            ui_integration::get_all_plugins,
            ui_integration::get_plugin,
            ui_integration::enable_plugin,
            ui_integration::disable_plugin,
            ui_integration::uninstall_plugin,
            ui_integration::update_plugin,
            ui_integration::trigger_plugin_event,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Using the Plugin System from JavaScript/TypeScript

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// Install a plugin
async function installPlugin(path: string) {
  try {
    const pluginInfo = await invoke('install_plugin_from_file', { path });
    console.log(`Installed plugin: ${pluginInfo.name}`);
    return pluginInfo;
  } catch (error) {
    console.error(`Failed to install plugin: ${error}`);
    throw error;
  }
}

// Enable a plugin
async function enablePlugin(pluginId: string) {
  try {
    await invoke('enable_plugin', { pluginId });
    console.log(`Enabled plugin: ${pluginId}`);
  } catch (error) {
    console.error(`Failed to enable plugin: ${error}`);
    throw error;
  }
}

// Listen for plugin events
function setupPluginEventListeners() {
  // Plugin installed
  listen('plugin-installed', (event) => {
    console.log(`Plugin installed: ${event.payload.plugin.name}`);
  });
  
  // Plugin status changed
  listen('plugin-status-changed', (event) => {
    console.log(`Plugin ${event.payload.plugin_id} status changed to: ${event.payload.status}`);
  });
  
  // Plugin uninstalled
  listen('plugin-uninstalled', (event) => {
    console.log(`Plugin uninstalled: ${event.payload.plugin_id}`);
  });
}
```

## Creating Plugins

See the [Developer Guide](./docs/guides/developer_guide.md) for detailed instructions on creating plugins for the Tauri Windows Plugin System.

## Plugin Structure

A plugin package is a ZIP file with the following structure:

```
plugin.zip
├── plugin.json    # Plugin manifest
├── plugin.dll     # Plugin binary
└── resources/     # Optional plugin resources
```

### Plugin Manifest

The `plugin.json` file contains metadata about the plugin:

```json
{
  "name": "my_plugin",
  "version": "0.1.0",
  "entry": "plugin.dll",
  "api_version": "1.0.0",
  "permissions": [
    {
      "type": "file_system",
      "read": true,
      "write": false,
      "paths": ["data/"]
    }
  ],
  "description": "My first Tauri plugin",
  "author": "Your Name",
  "homepage": "https://example.com"
}
```

## Permission System

The permission system ensures that plugins can only access resources they explicitly request and that the user approves. Permissions are categorized into four types:

1. **File System**: Access to read/write files in specific directories
2. **Network**: Access to connect to specific web addresses
3. **UI**: Ability to show notifications or create windows
4. **System**: Access to system information or resources like clipboard

## Documentation

- [API Reference](./docs/api/api_reference.md): Detailed API documentation
- [Developer Guide](./docs/guides/developer_guide.md): Guide for plugin developers
- [User Guide](./docs/guides/user_guide.md): Guide for end users

## Examples

Check out the examples in the `examples/` directory:

- `demo-app`: A sample Tauri application that uses the plugin system
- `sample-plugin`: A simple plugin that demonstrates the basic features

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
