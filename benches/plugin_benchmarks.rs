//! Performance benchmarks for the Tauri Windows Plugin System

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::path::Path;
use tempfile::tempdir;
use std::fs;

use tauri_windows_plugins::plugin_loader::{PluginLoader, PluginPackage};
use tauri_windows_plugins::plugin_host::PluginHost;
use tauri_windows_plugins::permission_system::PermissionSystem;
use tauri_windows_plugins::plugin_manager::PluginManager;
use tauri_windows_plugins::tests::common::helpers;

/// Benchmark plugin loading time
pub fn benchmark_plugin_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_loading");
    
    // Create test fixtures
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let package_path = temp_dir.path().join("valid_plugin_package.zip");
    let extraction_dir = temp_dir.path().join("extracted");
    
    helpers::create_test_plugin_package(&package_path, true)
        .expect("Failed to create test plugin package");
    
    fs::create_dir_all(&extraction_dir).expect("Failed to create extraction directory");
    
    let loader = PluginLoader::new();
    
    // Benchmark package extraction
    group.bench_function(BenchmarkId::new("extract_package", ""), |b| {
        b.iter(|| {
            let _ = black_box(PluginPackage::extract_package(
                &package_path, 
                &extraction_dir
            ));
        });
    });
    
    // Create extracted package for other benchmarks
    let _ = PluginPackage::extract_package(&package_path, &extraction_dir);
    
    // Benchmark manifest validation
    let manifest_path = extraction_dir.join("plugin.json");
    group.bench_function(BenchmarkId::new("validate_manifest", ""), |b| {
        b.iter(|| {
            let _ = black_box(loader.read_and_validate_manifest(&manifest_path));
        });
    });
    
    group.finish();
}

/// Benchmark plugin initialization time
pub fn benchmark_plugin_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("plugin_initialization");
    
    let host = PluginHost::new();
    let plugin_id = "benchmark-plugin";
    
    // Benchmark context creation
    group.bench_function(BenchmarkId::new("create_context", ""), |b| {
        b.iter(|| {
            let _ = black_box(host.create_plugin_context(plugin_id));
        });
    });
    
    // In a real benchmark, we would also measure actual plugin initialization
    // with a real DLL, but that's difficult to set up in a test environment
    
    group.finish();
}

/// Benchmark event triggering latency
pub fn benchmark_event_triggering(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_triggering");
    
    let mut host = PluginHost::new();
    let plugin_id = "benchmark-plugin";
    let context = host.create_plugin_context(plugin_id);
    
    // Initialize a test plugin
    let _ = host.initialize_plugin_for_testing(plugin_id, &context);
    
    // Register a test callback
    let _ = host.register_callback(plugin_id, "test_event", |_data| {
        // Do minimal work to measure overhead
        Ok(())
    });
    
    // Benchmark event triggering
    group.bench_function(BenchmarkId::new("trigger_event", ""), |b| {
        b.iter(|| {
            let _ = black_box(host.trigger_event("test_event", "test data"));
        });
    });
    
    group.finish();
}

/// Benchmark system performance with multiple plugins
pub fn benchmark_multi_plugin_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_plugin_performance");
    
    let mut host = PluginHost::new();
    
    // Initialize multiple test plugins
    let plugin_count = 10;
    for i in 0..plugin_count {
        let plugin_id = format!("benchmark-plugin-{}", i);
        let context = host.create_plugin_context(&plugin_id);
        let _ = host.initialize_plugin_for_testing(&plugin_id, &context);
        
        // Register the same callback for each plugin
        let _ = host.register_callback(&plugin_id, "test_event", move |_data| {
            // Do minimal work
            Ok(())
        });
    }
    
    // Benchmark event triggering with multiple plugins
    group.bench_function(BenchmarkId::new("trigger_event_multi", format!("{}_plugins", plugin_count)), |b| {
        b.iter(|| {
            let _ = black_box(host.trigger_event("test_event", "test data"));
        });
    });
    
    group.finish();
}

/// Benchmark permission checks
pub fn benchmark_permission_system(c: &mut Criterion) {
    let mut group = c.benchmark_group("permission_system");
    
    let mut permission_system = PermissionSystem::new();
    let plugin_id = "benchmark-plugin";
    let permission = "read_file";
    
    // Grant permission first
    let _ = permission_system.grant_permission(plugin_id, permission);
    
    // Benchmark permission checking
    group.bench_function(BenchmarkId::new("check_permission", ""), |b| {
        b.iter(|| {
            let _ = black_box(permission_system.is_permission_granted(plugin_id, permission));
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
