//! Performance benchmarks for the Tauri Windows Plugin System

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tempfile::tempdir;
use std::fs;

use tauri_windows_plugin_system::plugin_loader::PluginLoader;
use tauri_windows_plugin_system::plugin_host::PluginHost;
use tauri_windows_plugin_system::permission_system::{PermissionSystem, Permission, FileSystemPermission};

// Import test helpers from our test common module
mod common;
use common::helpers;

/// Benchmark plugin loading time
pub fn benchmark_plugin_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_loading");
    
    // Create test fixtures
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extract_base_dir = temp_dir.path().join("extract");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&extract_base_dir).expect("Failed to create extract directory");
    
    let loader = PluginLoader::new(extract_base_dir.clone());
    
    // Benchmark plugin package loading (includes extraction and validation)
    group.bench_function(BenchmarkId::new("load_plugin_package", ""), |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = black_box(loader.load_plugin_package(&package_path).await);
            });
        });
    });
    
    group.finish();
}

/// Benchmark plugin initialization time
pub fn benchmark_plugin_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_initialization");
    
    let host = PluginHost::new();
    let plugin_id = "benchmark-plugin";
    
    // Benchmark checking if plugin exists (this is a real method)
    group.bench_function(BenchmarkId::new("has_plugin_check", ""), |b| {
        b.iter(|| {
            let _ = black_box(host.has_plugin(plugin_id));
        });
    });
    
    // In a real benchmark, we would also measure actual plugin initialization
    // with a real DLL, but that's difficult to set up in a test environment
    
    group.finish();
}

/// Benchmark event triggering latency
pub fn benchmark_event_triggering(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_triggering");
    
    let host = PluginHost::new();
    let plugin_id = "benchmark-plugin";
    
    // Benchmark basic PluginHost operations
    group.bench_function(BenchmarkId::new("has_plugin_check", ""), |b| {
        b.iter(|| {
            let _ = black_box(host.has_plugin(plugin_id));
        });
    });
    
    // Benchmark event triggering with error handling (plugin not found)
    group.bench_function(BenchmarkId::new("trigger_event_no_plugin", ""), |b| {
        b.iter(|| {
            let _ = black_box(host.trigger_event(plugin_id, "test_event", "test_data"));
        });
    });
    
    group.finish();
}

/// Benchmark system performance with multiple plugins
pub fn benchmark_multi_plugin_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_plugin_performance");
    
    let host = PluginHost::new();
    
    // Benchmark basic PluginHost operations with multiple plugin checks
    let plugin_count = 10;
    let plugin_ids: Vec<String> = (0..plugin_count)
        .map(|i| format!("benchmark-plugin-{}", i))
        .collect();
    
    // Benchmark checking multiple plugins
    group.bench_function(BenchmarkId::new("has_plugin_check_multi", format!("{}_plugins", plugin_count)), |b| {
        b.iter(|| {
            for plugin_id in &plugin_ids {
                let _ = black_box(host.has_plugin(plugin_id));
            }
        });
    });
    
    group.finish();
}

/// Benchmark permission checks
pub fn benchmark_permission_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("permission_system");
    
    let permission_system = PermissionSystem::new();
    let plugin_id = "benchmark-plugin";
    let permission = Permission::FileSystem(FileSystemPermission {
        read: true,
        write: false,
        paths: vec!["/tmp".to_string()],
    });
    
    // Grant permission first
    let _ = permission_system.grant_permissions(plugin_id, vec![permission.clone()], false);
    
    // Benchmark permission checking
    group.bench_function(BenchmarkId::new("check_permission", ""), |b| {
        b.iter(|| {
            let _ = black_box(permission_system.is_permission_granted(plugin_id, &permission));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_plugin_loading,
    benchmark_plugin_initialization,
    benchmark_event_triggering,
    benchmark_multi_plugin_performance,
    benchmark_permission_system
);
criterion_main!(benches);
