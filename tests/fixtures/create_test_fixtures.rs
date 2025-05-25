//! Script to create test fixtures for the Tauri Windows Plugin System

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use zip::write::{FileOptions, ZipWriter};

/// Path to the fixtures directory
pub fn fixtures_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path
}

/// Create a valid plugin manifest JSON
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

/// Create an invalid plugin manifest JSON (missing required fields)
pub fn create_invalid_manifest_json() -> String {
    r#"{
        "name": "test-plugin",
        "version": "1.0.0"
    }"#.to_string()
}

/// Create a test plugin package with valid manifest
pub fn create_valid_plugin_package(output_path: &Path) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
        
    // Add manifest
    zip.start_file("plugin.json", options)?;
    zip.write_all(create_valid_manifest_json().as_bytes())?;
    
    // Add a dummy DLL file
    zip.start_file("plugin.dll", options)?;
    zip.write_all(b"This is not a real DLL but simulates one for testing")?;
    
    // Add some resource files
    zip.start_file("resources/test.txt", options)?;
    zip.write_all(b"Test resource file")?;
    
    zip.finish()?;
    Ok(())
}

/// Create a test plugin package with invalid manifest
pub fn create_invalid_manifest_plugin(output_path: &Path) -> std::io::Result<()> {
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
        
    // Add invalid manifest
    zip.start_file("plugin.json", options)?;
    zip.write_all(create_invalid_manifest_json().as_bytes())?;
    
    // Add a dummy DLL file
    zip.start_file("plugin.dll", options)?;
    zip.write_all(b"This is not a real DLL but simulates one for testing")?;
    
    zip.finish()?;
    Ok(())
}

/// Create a corrupted ZIP file
pub fn create_corrupted_plugin_package(output_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    // Write some random bytes that don't constitute a valid ZIP file
    file.write_all(b"This is not a valid ZIP file")?;
    Ok(())
}

/// Create a mock DLL with valid exports
pub fn create_valid_plugin_dll(output_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    // In a real implementation, this would be an actual DLL
    // For testing purposes, we just create a file with a recognizable signature
    file.write_all(b"MOCK_DLL_VALID_EXPORTS")?;
    Ok(())
}

/// Create a mock DLL with missing exports
pub fn create_missing_exports_dll(output_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    // In a real implementation, this would be an actual DLL with missing exports
    // For testing purposes, we just create a file with a recognizable signature
    file.write_all(b"MOCK_DLL_MISSING_EXPORTS")?;
    Ok(())
}

/// Main function to create all test fixtures
pub fn create_all_fixtures() -> std::io::Result<()> {
    let fixtures_path = fixtures_dir();
    fs::create_dir_all(&fixtures_path)?;
    
    // Create plugin packages
    create_valid_plugin_package(&fixtures_path.join("valid_plugin_package.zip"))?;
    create_invalid_manifest_plugin(&fixtures_path.join("invalid_manifest_plugin.zip"))?;
    create_corrupted_plugin_package(&fixtures_path.join("corrupted_plugin_package.zip"))?;
    
    // Create mock DLLs
    create_valid_plugin_dll(&fixtures_path.join("valid_plugin.dll"))?;
    create_missing_exports_dll(&fixtures_path.join("missing_exports.dll"))?;
    
    // Create individual manifest files for testing
    let mut valid_manifest_file = File::create(fixtures_path.join("valid_manifest.json"))?;
    valid_manifest_file.write_all(create_valid_manifest_json().as_bytes())?;
    
    let mut invalid_manifest_file = File::create(fixtures_path.join("invalid_manifest.json"))?;
    invalid_manifest_file.write_all(create_invalid_manifest_json().as_bytes())?;
    
    println!("All test fixtures created successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_fixtures() {
        // This test actually creates the fixtures
        let result = create_all_fixtures();
        assert!(result.is_ok());
    }
}
