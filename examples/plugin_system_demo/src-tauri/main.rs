#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod plugin_manager;
mod test_integration;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    CustomMenuItem, Manager, RunEvent, State, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, Window,
};
use tauri_plugin_sandbox::SandboxOptions;
use tauri_plugin_signature::SignatureInfo;
use tauri_plugin_permissions::{Permission, PermissionRequest, PluginInfo, Decision};
use tauri_plugin_resource_monitor::ResourceLimits;
use tauri_plugin_store::{StoreClientConfig, PluginSearchFilter};
use tokio::sync::Mutex;
use tokio::sync::RwLock;

// Plugin durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginStatus {
    id: String,
    name: String,
    version: String,
    installed: bool,
    enabled: bool,
    running: bool,
    permissions: Vec<String>,
    sandbox_id: Option<String>,
    process_id: Option<u32>,
}

// Uygulama durumu
struct AppState {
    plugins: Arc<RwLock<HashMap<String, PluginStatus>>>,
}

#[tauri::command]
async fn list_plugins(state: State<'_, AppState>) -> Result<Vec<PluginStatus>, String> {
    let plugins = state.plugins.read().await;
    let plugin_list = plugins.values().cloned().collect();
    Ok(plugin_list)
}

#[tauri::command]
async fn search_plugins(
    query: String,
    store_state: State<'_, tauri_plugin_store::StoreClientState>,
) -> Result<Vec<tauri_plugin_store::store_types::PluginMetadata>, String> {
    let store_client = store_state.store_client.read().await;
    
    let filter = PluginSearchFilter {
        query: Some(query),
        ..Default::default()
    };
    
    match store_client.search_plugins(filter).await {
        Ok(result) => Ok(result.items),
        Err(e) => Err(format!("Plugin arama hatası: {:?}", e)),
    }
}

#[tauri::command]
async fn install_plugin(
    plugin_id: String,
    store_state: State<'_, tauri_plugin_store::StoreClientState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut store_client = store_state.store_client.write().await;
    
    match store_client.install_plugin(&plugin_id, None).await {
        Ok(status) => {
            if status.success {
                // Plugin durumunu güncelle
                let mut plugins = app_state.plugins.write().await;
                plugins.insert(plugin_id.clone(), PluginStatus {
                    id: plugin_id,
                    name: status.name,
                    version: "1.0.0".to_string(), // Gerçek uygulamada sürüm bilgisi alınmalı
                    installed: true,
                    enabled: true,
                    running: false,
                    permissions: vec![],
                    sandbox_id: None,
                    process_id: None,
                });
                
                Ok(true)
            } else {
                Err(format!("Kurulum başarısız: {:?}", status.error))
            }
        },
        Err(e) => Err(format!("Kurulum hatası: {:?}", e)),
    }
}

#[tauri::command]
async fn uninstall_plugin(
    plugin_id: String,
    store_state: State<'_, tauri_plugin_store::StoreClientState>,
    app_state: State<'_, AppState>,
) -> Result<bool, String> {
    let mut store_client = store_state.store_client.write().await;
    
    match store_client.uninstall_plugin(&plugin_id).await {
        Ok(success) => {
            if success {
                // Plugin durumunu güncelle
                let mut plugins = app_state.plugins.write().await;
                plugins.remove(&plugin_id);
                
                Ok(true)
            } else {
                Err("Kaldırma başarısız".to_string())
            }
        },
        Err(e) => Err(format!("Kaldırma hatası: {:?}", e)),
    }
}

#[tauri::command]
async fn run_plugin(
    plugin_id: String,
    app_state: State<'_, AppState>,
    sandbox_state: State<'_, tauri_plugin_sandbox::SandboxState>,
    permissions_state: State<'_, tauri_plugin_permissions::PermissionState>,
    resource_state: State<'_, tauri_plugin_resource_monitor::ResourceMonitorState>,
) -> Result<bool, String> {
    // Plugin durumunu kontrol et
    let plugins = app_state.plugins.read().await;
    let plugin = plugins.get(&plugin_id).ok_or("Plugin bulunamadı")?;
    
    if plugin.running {
        return Err("Plugin zaten çalışıyor".to_string());
    }
    
    drop(plugins); // Lock'ı bırak
    
    // İzinleri kontrol et
    let mut permission_manager = permissions_state.permission_manager.write().await;
    
    let plugin_info = PluginInfo {
        id: plugin_id.clone(),
        name: plugin.name.clone(),
        version: plugin.version.clone(),
        vendor: "Demo Vendor".to_string(),
    };
    
    let permission_request = PermissionRequest {
        plugin: plugin_info,
        permissions: vec![
            Permission::FileSystem {
                scope: "read".to_string(),
                paths: vec!["/temp".to_string()],
            },
            Permission::Network {
                hosts: vec!["api.example.com".to_string()],
            },
        ],
        reason: "Demo plugin çalıştırma".to_string(),
    };
    
    let permission_response = permission_manager.validate_permissions(&permission_request)
        .map_err(|e| format!("İzin hatası: {:?}", e))?;
    
    if permission_response.decision != Decision::Granted {
        return Err("İzinler reddedildi".to_string());
    }
    
    // Sandbox oluştur
    let mut sandbox_manager = sandbox_state.sandbox_manager.write().await;
    
    let sandbox_options = SandboxOptions {
        memory_limit: Some(100 * 1024 * 1024), // 100 MB
        cpu_rate_limit: Some(50.0),           // %50 CPU
        network_limit: Some(true),
        file_system_limit: Some(true),
        ..Default::default()
    };
    
    let sandbox_id = sandbox_manager.create_sandbox(&plugin_id, sandbox_options)
        .map_err(|e| format!("Sandbox oluşturma hatası: {:?}", e))?;
    
    // Örnek plugin yolu (gerçek uygulamada kurulu plugin dizininden alınır)
    let plugin_path = PathBuf::from("/path/to/plugin/executable");
    
    // Plugin'i sandbox içinde çalıştır
    let process_id = sandbox_manager.run_process(
        &sandbox_id, 
        &plugin_path, 
        &[],
        None
    ).map_err(|e| format!("Süreç başlatma hatası: {:?}", e))?;
    
    // Kaynak izleme başlat
    let mut resource_monitor = resource_state.resource_monitor.write().await;
    resource_monitor.start_monitoring(&plugin_id, process_id)
        .await
        .map_err(|e| format!("Kaynak izleme hatası: {:?}", e))?;
    
    // Plugin durumunu güncelle
    let mut plugins = app_state.plugins.write().await;
    if let Some(plugin) = plugins.get_mut(&plugin_id) {
        plugin.running = true;
        plugin.sandbox_id = Some(sandbox_id);
        plugin.process_id = Some(process_id);
    }
    
    Ok(true)
}

#[tauri::command]
async fn stop_plugin(
    plugin_id: String,
    app_state: State<'_, AppState>,
    sandbox_state: State<'_, tauri_plugin_sandbox::SandboxState>,
    resource_state: State<'_, tauri_plugin_resource_monitor::ResourceMonitorState>,
) -> Result<bool, String> {
    // Plugin durumunu kontrol et
    let plugins = app_state.plugins.read().await;
    let plugin = plugins.get(&plugin_id).ok_or("Plugin bulunamadı")?;
    
    if !plugin.running {
        return Err("Plugin çalışmıyor".to_string());
    }
    
    let sandbox_id = plugin.sandbox_id.clone().ok_or("Sandbox ID bulunamadı")?;
    let process_id = plugin.process_id.ok_or("Process ID bulunamadı")?;
    
    drop(plugins); // Lock'ı bırak
    
    // Kaynak izlemeyi durdur
    let mut resource_monitor = resource_state.resource_monitor.write().await;
    resource_monitor.stop_monitoring(&plugin_id)
        .await
        .map_err(|e| format!("Kaynak izleme durdurma hatası: {:?}", e))?;
    
    // Süreci sonlandır
    let mut sandbox_manager = sandbox_state.sandbox_manager.write().await;
    sandbox_manager.terminate_process(&sandbox_id, process_id)
        .map_err(|e| format!("Süreç sonlandırma hatası: {:?}", e))?;
    
    // Sandbox'ı kaldır
    sandbox_manager.destroy_sandbox(&sandbox_id)
        .map_err(|e| format!("Sandbox kaldırma hatası: {:?}", e))?;
    
    // Plugin durumunu güncelle
    let mut plugins = app_state.plugins.write().await;
    if let Some(plugin) = plugins.get_mut(&plugin_id) {
        plugin.running = false;
        plugin.sandbox_id = None;
        plugin.process_id = None;
    }
    
    Ok(true)
}

#[tauri::command]
async fn get_resource_usage(
    plugin_id: String,
    resource_state: State<'_, tauri_plugin_resource_monitor::ResourceMonitorState>,
) -> Result<tauri_plugin_resource_monitor::ResourceUsage, String> {
    let resource_monitor = resource_state.resource_monitor.read().await;
    
    resource_monitor.get_resource_usage(&plugin_id)
        .map_err(|e| format!("Kaynak kullanımı alınamadı: {:?}", e))
}

#[tauri::command]
async fn check_for_updates(
    store_state: State<'_, tauri_plugin_store::StoreClientState>,
) -> Result<HashMap<String, tauri_plugin_store::store_types::PluginUpdate>, String> {
    let store_client = store_state.store_client.read().await;
    
    match store_client.check_for_updates().await {
        Ok(updates) => Ok(updates),
        Err(e) => Err(format!("Güncelleme kontrolü hatası: {:?}", e)),
    }
}

fn main() {
    env_logger::init();
    info!("Plugin System Demo başlatılıyor...");

    let system_tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show".to_string(), "Göster"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Çıkış"));

    let system_tray = SystemTray::new().with_menu(system_tray_menu);

    let app_state = AppState {
        plugins: Arc::new(RwLock::new(HashMap::new())),
    };

    let plugin_manager = Arc::new(Mutex::new(plugin_manager::PluginManager::new()));

    // Plugin Store yapılandırması
    let store_config = StoreClientConfig {
        api_url: "http://localhost:8080/api/v1".to_string(), // Mock Store Server
        plugins_dir: None, // Varsayılan dizini kullan
        auto_update_check: true,
        update_check_interval: Some(60 * 60), // 1 saat
    };

    // Tauri uygulamasını oluştur
    tauri::Builder::default()
        .plugin(tauri_plugin_store::init(store_config))
        .plugin(tauri_plugin_sandbox::init())
        .plugin(tauri_plugin_signature::init())
        .plugin(tauri_plugin_permissions::init())
        .plugin(tauri_plugin_resource_monitor::init())
        .manage(app_state)
        .manage(plugin_manager)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            list_plugins,
            search_plugins,
            install_plugin,
            uninstall_plugin,
            run_plugin,
            stop_plugin,
            get_resource_usage,
            check_for_updates,
            // Test entegrasyon komutları
            test_integration::start_plugin_test,
            test_integration::get_test_status,
        ])
        .build(tauri::generate_context!())
        .expect("Tauri uygulaması oluşturulamadı")
        .run(|_app_handle, event| match event {
            RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
        });
}
