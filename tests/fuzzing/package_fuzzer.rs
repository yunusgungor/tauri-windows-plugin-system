//! Fuzzing tests for plugin package validation

use crate::plugin_loader::package::{PluginPackage, PackageError};
use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use zip::write::{FileOptions, ZipWriter};

#[derive(Debug, Arbitrary)]
struct FuzzPackageInput {
    // Control what's in the package
    include_manifest: bool,
    include_dll: bool,
    include_resources: bool,
    
    // Control content validity
    valid_manifest: bool,
    valid_dll: bool,
    
    // Control ZIP structure
    corrupt_zip_structure: bool,
    use_invalid_paths: bool,
    
    // Random content
    manifest_content: Vec<u8>,
    dll_content: Vec<u8>,
    resource_files: Vec<(String, Vec<u8>)>, // (path, content) pairs
    
    // Random extra files
    extra_files: Vec<(String, Vec<u8>)>, // (path, content) pairs
}

/// Creates a ZIP package based on fuzzing input
fn create_fuzzed_package(input: &FuzzPackageInput, output_path: &Path) -> io::Result<()> {
    if input.corrupt_zip_structure {
        // Create an invalid ZIP file
        let mut file = File::create(output_path)?;
        file.write_all(b"This is not a valid ZIP file")?;
        file.write_all(&input.dll_content[..std::cmp::min(input.dll_content.len(), 100)])?;
        return Ok(());
    }
    
    // Create a valid ZIP structure
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    
    // Add manifest if needed
    if input.include_manifest {
        let manifest_path = if input.use_invalid_paths {
            // Use a potentially problematic path
            "../plugin.json".to_string()
        } else {
            "plugin.json".to_string()
        };
        
        zip.start_file(manifest_path, options)?;
        
        if input.valid_manifest {
            // Write a valid manifest
            zip.write_all(b"{\n")?;
            zip.write_all(b"  \"name\": \"fuzz-plugin\",\n")?;
            zip.write_all(b"  \"version\": \"1.0.0\",\n")?;
            zip.write_all(b"  \"description\": \"A fuzzing test plugin\",\n")?;
            zip.write_all(b"  \"author\": \"Fuzzer\",\n")?;
            zip.write_all(b"  \"permissions\": [\"read_file\"],\n")?;
            zip.write_all(b"  \"min_host_version\": \"1.0.0\",\n")?;
            zip.write_all(b"  \"entry_point\": \"plugin.dll\"\n")?;
            zip.write_all(b"}")?;
        } else {
            // Write invalid manifest content
            zip.write_all(&input.manifest_content[..std::cmp::min(input.manifest_content.len(), 200)])?;
        }
    }
    
    // Add DLL if needed
    if input.include_dll {
        let dll_path = if input.use_invalid_paths {
            "../plugin.dll".to_string()
        } else {
            "plugin.dll".to_string()
        };
        
        zip.start_file(dll_path, options)?;
        
        if input.valid_dll {
            // Write a dummy DLL content
            zip.write_all(b"This is not a real DLL but simulates one for testing")?;
        } else {
            // Write potentially problematic DLL content
            zip.write_all(&input.dll_content[..std::cmp::min(input.dll_content.len(), 200)])?;
        }
    }
    
    // Add resources if needed
    if input.include_resources {
        for (i, (path, content)) in input.resource_files.iter().enumerate().take(5) {
            let resource_path = if input.use_invalid_paths {
                format!("../resources/{}", path)
            } else {
                format!("resources/{}", path)
            };
            
            zip.start_file(&resource_path, options)?;
            zip.write_all(&content[..std::cmp::min(content.len(), 100)])?;
        }
    }
    
    // Add extra random files
    for (i, (path, content)) in input.extra_files.iter().enumerate().take(10) {
        let file_path = if input.use_invalid_paths {
            format!("../extra/{}", path)
        } else {
            format!("extra/{}", path)
        };
        
        zip.start_file(&file_path, options)?;
        zip.write_all(&content[..std::cmp::min(content.len(), 100)])?;
    }
    
    zip.finish()?;
    Ok(())
}

/// Fuzz target for plugin package validation
fuzz_target!(|input: FuzzPackageInput| {
    // Create temporary directories
    let temp_dir = match tempdir() {
        Ok(dir) => dir,
        Err(_) => return, // Skip this input if we can't create temp dir
    };
    
    let package_path = temp_dir.path().join("fuzz_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    // Create the fuzzed package
    if let Err(_) = create_fuzzed_package(&input, &package_path) {
        return; // Skip this input if we can't create the package
    }
    
    // Create extraction directory
    if let Err(_) = fs::create_dir_all(&extraction_dir) {
        return; // Skip this input if we can't create extraction dir
    }
    
    // Test package extraction
    let extract_result = PluginPackage::extract_package(&package_path, &extraction_dir);
    
    // If extraction succeeded, test manifest reading
    if extract_result.is_ok() {
        let manifest_path = extraction_dir.join("plugin.json");
        if manifest_path.exists() {
            let _manifest_result = PluginPackage::read_manifest_from_file(&manifest_path);
            // We don't assert anything, just check it doesn't panic
        }
        
        // Test package structure validation
        let _structure_result = PluginPackage::validate_package_structure(&extraction_dir);
    }
});
