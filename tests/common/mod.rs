//! Common test utilities and helpers for the Tauri Windows Plugin System.

use std::path::PathBuf;
use std::sync::Once;

// Ensure test initialization happens only once
static INIT: Once = Once::new();

/// Initialize test environment
pub fn setup() {
    INIT.call_once(|| {
        // Set up logging for tests
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .init();
    });
}

/// Test fixture paths
pub struct TestFixtures;

impl TestFixtures {
    /// Get the path to the test fixtures directory
    pub fn fixtures_dir() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path
    }
    
    /// Get path to a valid plugin package
    pub fn valid_plugin_package() -> PathBuf {
        let mut path = Self::fixtures_dir();
        path.push("valid_plugin_package.zip");
        path
    }
    
    /// Get path to an invalid plugin package (corrupted)
    pub fn corrupted_plugin_package() -> PathBuf {
        let mut path = Self::fixtures_dir();
        path.push("corrupted_plugin_package.zip");
        path
    }
    
    /// Get path to a plugin with invalid manifest
    pub fn invalid_manifest_plugin() -> PathBuf {
        let mut path = Self::fixtures_dir();
        path.push("invalid_manifest_plugin.zip");
        path
    }
    
    /// Get path to a valid plugin DLL
    pub fn valid_plugin_dll() -> PathBuf {
        let mut path = Self::fixtures_dir();
        path.push("valid_plugin.dll");
        path
    }
    
    /// Get path to a plugin DLL with missing exports
    pub fn missing_exports_dll() -> PathBuf {
        let mut path = Self::fixtures_dir();
        path.push("missing_exports.dll");
        path
    }
}

/// Mock implementations for testing
pub mod mocks {
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    
    /// Mock plugin loader for testing
    #[derive(Default)]
    pub struct MockPluginLoader {
        pub load_plugin_package_calls: Arc<Mutex<Vec<String>>>,
        pub load_plugin_dll_calls: Arc<Mutex<Vec<String>>>,
        pub extract_plugin_package_calls: Arc<Mutex<Vec<String>>>,
        pub read_and_validate_manifest_calls: Arc<Mutex<Vec<String>>>,
        pub validate_plugin_compatibility_calls: Arc<Mutex<Vec<String>>>,
        pub should_fail: bool,
    }
    
    impl MockPluginLoader {
        pub fn new() -> Self {
            Self {
                load_plugin_package_calls: Arc::new(Mutex::new(Vec::new())),
                load_plugin_dll_calls: Arc::new(Mutex::new(Vec::new())),
                extract_plugin_package_calls: Arc::new(Mutex::new(Vec::new())),
                read_and_validate_manifest_calls: Arc::new(Mutex::new(Vec::new())),
                validate_plugin_compatibility_calls: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }
        
        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }
    
    /// Mock permission system for testing
    #[derive(Default)]
    pub struct MockPermissionSystem {
        pub validate_permissions_calls: Arc<Mutex<Vec<String>>>,
        pub prompt_for_permissions_calls: Arc<Mutex<Vec<String>>>,
        pub is_permission_granted_calls: Arc<Mutex<Vec<String>>>,
        pub grant_permissions_calls: Arc<Mutex<Vec<String>>>,
        pub revoke_permissions_calls: Arc<Mutex<Vec<String>>>,
        pub granted_permissions: Arc<Mutex<HashMap<String, bool>>>,
        pub should_fail: bool,
    }
    
    impl MockPermissionSystem {
        pub fn new() -> Self {
            Self {
                validate_permissions_calls: Arc::new(Mutex::new(Vec::new())),
                prompt_for_permissions_calls: Arc::new(Mutex::new(Vec::new())),
                is_permission_granted_calls: Arc::new(Mutex::new(Vec::new())),
                grant_permissions_calls: Arc::new(Mutex::new(Vec::new())),
                revoke_permissions_calls: Arc::new(Mutex::new(Vec::new())),
                granted_permissions: Arc::new(Mutex::new(HashMap::new())),
                should_fail: false,
            }
        }
        
        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
        
        pub fn with_granted_permission(self, permission: &str) -> Self {
            self.granted_permissions.lock().unwrap().insert(permission.to_string(), true);
            self
        }
    }
    
    /// Mock plugin host for testing
    #[derive(Default)]
    pub struct MockPluginHost {
        pub init_plugin_calls: Arc<Mutex<Vec<String>>>,
        pub teardown_plugin_calls: Arc<Mutex<Vec<String>>>,
        pub trigger_event_calls: Arc<Mutex<Vec<String>>>,
        pub has_plugin_calls: Arc<Mutex<Vec<String>>>,
        pub loaded_plugins: Arc<Mutex<Vec<String>>>,
        pub should_fail: bool,
    }
    
    impl MockPluginHost {
        pub fn new() -> Self {
            Self {
                init_plugin_calls: Arc::new(Mutex::new(Vec::new())),
                teardown_plugin_calls: Arc::new(Mutex::new(Vec::new())),
                trigger_event_calls: Arc::new(Mutex::new(Vec::new())),
                has_plugin_calls: Arc::new(Mutex::new(Vec::new())),
                loaded_plugins: Arc::new(Mutex::new(Vec::new())),
                should_fail: false,
            }
        }
        
        pub fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
        
        pub fn with_loaded_plugin(self, plugin_id: &str) -> Self {
            self.loaded_plugins.lock().unwrap().push(plugin_id.to_string());
            self
        }
    }
}

/// Test helpers for creating and validating test data
pub mod helpers {
    use std::path::Path;
    use std::fs::File;
    use std::io::Write;
    use zip::write::FileOptions;
    
    /// Create a valid plugin manifest JSON string
    pub fn create_valid_manifest_json() -> String {
        r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin for unit testing",
            "author": "Test Author",
            "permissions": ["read_file", "write_file"],
            "min_host_version": "1.0.0",
            "entry_point": "plugin.dll"
        }"#.to_string()
    }
    
    /// Create an invalid plugin manifest JSON string (missing required fields)
    pub fn create_invalid_manifest_json() -> String {
        r#"{
            "name": "test-plugin",
            "version": "1.0.0"
        }"#.to_string()
    }
    
    /// Create a test plugin ZIP package
    pub fn create_test_plugin_package(path: &Path, valid: bool) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
            
        // Add manifest
        let manifest_content = if valid {
            create_valid_manifest_json()
        } else {
            create_invalid_manifest_json()
        };
        
        zip.start_file("plugin.json", options)?;
        zip.write_all(manifest_content.as_bytes())?;
        
        // Add a dummy DLL file
        zip.start_file("plugin.dll", options)?;
        zip.write_all(b"This is not a real DLL but simulates one for testing")?;
        
        // Add some resource files
        zip.start_file("resources/test.txt", options)?;
        zip.write_all(b"Test resource file")?;
        
        zip.finish()?;
        Ok(())
    }
    
    /// Create a corrupted ZIP file
    pub fn create_corrupted_zip(path: &Path) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        // Write some random bytes that don't constitute a valid ZIP file
        file.write_all(b"This is not a valid ZIP file")?;
        Ok(())
    }
}
