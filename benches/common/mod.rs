//! Common utilities for benchmarks

use std::path::Path;
use std::fs::File;
use std::io::Write;
use zip::write::FileOptions;

pub mod helpers {
    use super::*;

    /// Create a test plugin package for benchmarking
    pub fn create_test_plugin_package(path: &Path, _valid: bool) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        // Add plugin.json manifest
        let manifest = r#"{
    "name": "benchmark-plugin",
    "version": "1.0.0",
    "description": "A benchmark plugin",
    "author": "Benchmark",
    "permissions": [
        "read_file",
        "write_file"
    ],
    "entry_point": "benchmark_plugin.dll"
}"#;
        
        zip.start_file("plugin.json", FileOptions::default())?;
        zip.write_all(manifest.as_bytes())?;
        
        // Add a mock DLL file (just empty data for benchmarking)
        zip.start_file("benchmark_plugin.dll", FileOptions::default())?;
        zip.write_all(b"mock dll content for benchmarking")?;
        
        zip.finish()?;
        Ok(())
    }
}
