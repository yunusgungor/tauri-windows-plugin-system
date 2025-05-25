// Tauri Windows Plugin System Demo - Plugin Manager
//
// Bu modül, pluginlerin yaşam döngüsü yönetimini ve izlenmesini sağlar.

use crate::{AppState, PluginStatus};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime, State, Window};
use tauri_plugin_sandbox::{SandboxError, SandboxManager, SandboxOptions};
use tauri_plugin_permissions::{Decision, Permission, PermissionManager, PermissionRequest, PluginInfo};
use tauri_plugin_resource_monitor::{ResourceLimits, ResourceMonitor};
use tauri_plugin_signature::{SignatureError, SignatureInfo, SignatureManager, VerificationResult};
use tauri_plugin_store::{PluginInstallStatus, PluginUpdateStatus, StoreClientState};
use tokio::sync::RwLock;

/// Plugin yöneticisi
pub struct PluginManager {
    app_handle: AppHandle,
    app_state: Arc<RwLock<AppState>>,
}

/// Plugin kurulum sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    success: bool,
    plugin_id: String,
    name: String,
    version: String,
    error: Option<String>,
}

/// Plugin çalıştırma sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    success: bool,
    plugin_id: String,
    process_id: Option<u32>,
    sandbox_id: Option<String>,
    error: Option<String>,
}

impl PluginManager {
    /// Yeni bir plugin yöneticisi oluştur
    pub fn new(app_handle: AppHandle) -> Self {
        let app_state = app_handle.state::<AppState>();
        
        Self {
            app_handle,
            app_state: app_state.plugins.clone(),
        }
    }
    
    /// Kurulu plugin listesini güncelle
    pub async fn refresh_plugin_list(&self) -> Result<Vec<PluginStatus>, String> {
        let store_state = self.app_handle.state::<StoreClientState>();
        let store_client = store_state.store_client.read().await;
        
        match store_client.get_installed_plugins().await {
            Ok(plugins) => {
                let mut plugin_map = HashMap::new();
                
                for plugin in plugins {
                    let status = PluginStatus {
                        id: plugin.id.clone(),
                        name: plugin.name.clone(),
                        version: plugin.installed_version.to_string(),
                        installed: true,
                        enabled: plugin.enabled,
                        running: false,
                        permissions: vec![], // Gerçek uygulamada izinler alınmalı
                        sandbox_id: None,
                        process_id: None,
                    };
                    
                    plugin_map.insert(plugin.id.clone(), status);
                }
                
                // Uygulama durumunu güncelle
                let mut app_state = self.app_state.write().await;
                *app_state = plugin_map;
                
                let plugin_list = app_state.values().cloned().collect();
                Ok(plugin_list)
            },
            Err(e) => Err(format!("Plugin listesi alınamadı: {:?}", e)),
        }
    }
    
    /// Plugin kur
    pub async fn install_plugin(&self, plugin_id: &str) -> Result<InstallResult, String> {
        let store_state = self.app_handle.state::<StoreClientState>();
        let mut store_client = store_state.store_client.write().await;
        
        // UI'a bildirim gönder
        self.send_event("plugin_install_started", plugin_id);
        
        // Plugin indir ve kur
        match store_client.install_plugin(plugin_id, None).await {
            Ok(status) => {
                if status.success {
                    // Plugin durumunu güncelle
                    let mut app_state = self.app_state.write().await;
                    app_state.insert(plugin_id.to_string(), PluginStatus {
                        id: plugin_id.to_string(),
                        name: status.name.clone(),
                        version: "1.0.0".to_string(), // Gerçek uygulamada sürüm bilgisi alınmalı
                        installed: true,
                        enabled: true,
                        running: false,
                        permissions: vec![],
                        sandbox_id: None,
                        process_id: None,
                    });
                    
                    // UI'a bildirim gönder
                    self.send_event("plugin_installed", plugin_id);
                    
                    Ok(InstallResult {
                        success: true,
                        plugin_id: plugin_id.to_string(),
                        name: status.name,
                        version: "1.0.0".to_string(), // Gerçek uygulamada sürüm bilgisi alınmalı
                        error: None,
                    })
                } else {
                    // UI'a bildirim gönder
                    self.send_event("plugin_install_failed", plugin_id);
                    
                    Err(format!("Kurulum başarısız: {:?}", status.error))
                }
            },
            Err(e) => {
                // UI'a bildirim gönder
                self.send_event("plugin_install_failed", plugin_id);
                
                Err(format!("Kurulum hatası: {:?}", e))
            },
        }
    }
    
    /// Plugin kaldır
    pub async fn uninstall_plugin(&self, plugin_id: &str) -> Result<bool, String> {
        let store_state = self.app_handle.state::<StoreClientState>();
        let mut store_client = store_state.store_client.write().await;
        
        // UI'a bildirim gönder
        self.send_event("plugin_uninstall_started", plugin_id);
        
        // Plugin'i kaldır
        match store_client.uninstall_plugin(plugin_id).await {
            Ok(success) => {
                if success {
                    // Plugin durumunu güncelle
                    let mut app_state = self.app_state.write().await;
                    app_state.remove(plugin_id);
                    
                    // UI'a bildirim gönder
                    self.send_event("plugin_uninstalled", plugin_id);
                    
                    Ok(true)
                } else {
                    // UI'a bildirim gönder
                    self.send_event("plugin_uninstall_failed", plugin_id);
                    
                    Err("Kaldırma başarısız".to_string())
                }
            },
            Err(e) => {
                // UI'a bildirim gönder
                self.send_event("plugin_uninstall_failed", plugin_id);
                
                Err(format!("Kaldırma hatası: {:?}", e))
            },
        }
    }
    
    /// Plugin çalıştır
    pub async fn run_plugin(&self, plugin_id: &str) -> Result<RunResult, String> {
        // Plugin durumunu kontrol et
        let app_state = self.app_state.read().await;
        let plugin = app_state.get(plugin_id).ok_or("Plugin bulunamadı")?;
        
        if plugin.running {
            return Err("Plugin zaten çalışıyor".to_string());
        }
        
        drop(app_state); // Lock'ı bırak
        
        // UI'a bildirim gönder
        self.send_event("plugin_start_requested", plugin_id);
        
        // İzinleri kontrol et
        let permissions_state = self.app_handle.state::<tauri_plugin_permissions::PermissionState>();
        let mut permission_manager = permissions_state.permission_manager.write().await;
        
        let plugin_info = PluginInfo {
            id: plugin_id.to_string(),
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            vendor: "Demo Vendor".to_string(),
        };
        
        // Gerçek uygulamada plugin'den izinler alınmalı
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
        
        // UI'a bildirim gönder
        self.send_event("plugin_permissions_requested", plugin_id);
        
        let permission_response = permission_manager.validate_permissions(&permission_request)
            .map_err(|e| format!("İzin hatası: {:?}", e))?;
        
        if permission_response.decision != Decision::Granted {
            // UI'a bildirim gönder
            self.send_event("plugin_permissions_denied", plugin_id);
            
            return Err("İzinler reddedildi".to_string());
        }
        
        // UI'a bildirim gönder
        self.send_event("plugin_permissions_granted", plugin_id);
        
        // Sandbox oluştur
        let sandbox_state = self.app_handle.state::<tauri_plugin_sandbox::SandboxState>();
        let mut sandbox_manager = sandbox_state.sandbox_manager.write().await;
        
        let sandbox_options = SandboxOptions {
            memory_limit: Some(100 * 1024 * 1024), // 100 MB
            cpu_rate_limit: Some(50.0),           // %50 CPU
            network_limit: Some(true),
            file_system_limit: Some(true),
            ..Default::default()
        };
        
        // UI'a bildirim gönder
        self.send_event("plugin_sandbox_creating", plugin_id);
        
        let sandbox_id = sandbox_manager.create_sandbox(plugin_id, sandbox_options)
            .map_err(|e| format!("Sandbox oluşturma hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_sandbox_created", &sandbox_id);
        
        // Örnek plugin yolu (gerçek uygulamada kurulu plugin dizininden alınır)
        let plugin_path = PathBuf::from("/path/to/plugin/executable");
        
        // Plugin'i sandbox içinde çalıştır
        let process_id = sandbox_manager.run_process(
            &sandbox_id, 
            &plugin_path, 
            &[],
            None
        ).map_err(|e| format!("Süreç başlatma hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_process_started", &process_id.to_string());
        
        // Kaynak izleme başlat
        let resource_state = self.app_handle.state::<tauri_plugin_resource_monitor::ResourceMonitorState>();
        let mut resource_monitor = resource_state.resource_monitor.write().await;
        
        resource_monitor.start_monitoring(plugin_id, process_id)
            .await
            .map_err(|e| format!("Kaynak izleme hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_monitoring_started", plugin_id);
        
        // Plugin durumunu güncelle
        let mut app_state = self.app_state.write().await;
        if let Some(plugin) = app_state.get_mut(plugin_id) {
            plugin.running = true;
            plugin.sandbox_id = Some(sandbox_id.clone());
            plugin.process_id = Some(process_id);
        }
        
        // UI'a bildirim gönder
        self.send_event("plugin_running", plugin_id);
        
        Ok(RunResult {
            success: true,
            plugin_id: plugin_id.to_string(),
            process_id: Some(process_id),
            sandbox_id: Some(sandbox_id),
            error: None,
        })
    }
    
    /// Plugin durdur
    pub async fn stop_plugin(&self, plugin_id: &str) -> Result<bool, String> {
        // Plugin durumunu kontrol et
        let app_state = self.app_state.read().await;
        let plugin = app_state.get(plugin_id).ok_or("Plugin bulunamadı")?;
        
        if !plugin.running {
            return Err("Plugin çalışmıyor".to_string());
        }
        
        let sandbox_id = plugin.sandbox_id.clone().ok_or("Sandbox ID bulunamadı")?;
        let process_id = plugin.process_id.ok_or("Process ID bulunamadı")?;
        
        drop(app_state); // Lock'ı bırak
        
        // UI'a bildirim gönder
        self.send_event("plugin_stop_requested", plugin_id);
        
        // Kaynak izlemeyi durdur
        let resource_state = self.app_handle.state::<tauri_plugin_resource_monitor::ResourceMonitorState>();
        let mut resource_monitor = resource_state.resource_monitor.write().await;
        
        resource_monitor.stop_monitoring(plugin_id)
            .await
            .map_err(|e| format!("Kaynak izleme durdurma hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_monitoring_stopped", plugin_id);
        
        // Süreci sonlandır
        let sandbox_state = self.app_handle.state::<tauri_plugin_sandbox::SandboxState>();
        let mut sandbox_manager = sandbox_state.sandbox_manager.write().await;
        
        sandbox_manager.terminate_process(&sandbox_id, process_id)
            .map_err(|e| format!("Süreç sonlandırma hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_process_terminated", &process_id.to_string());
        
        // Sandbox'ı kaldır
        sandbox_manager.destroy_sandbox(&sandbox_id)
            .map_err(|e| format!("Sandbox kaldırma hatası: {:?}", e))?;
        
        // UI'a bildirim gönder
        self.send_event("plugin_sandbox_destroyed", &sandbox_id);
        
        // Plugin durumunu güncelle
        let mut app_state = self.app_state.write().await;
        if let Some(plugin) = app_state.get_mut(plugin_id) {
            plugin.running = false;
            plugin.sandbox_id = None;
            plugin.process_id = None;
        }
        
        // UI'a bildirim gönder
        self.send_event("plugin_stopped", plugin_id);
        
        Ok(true)
    }
    
    /// Plugin güncelleme kontrolü
    pub async fn check_for_updates(&self) -> Result<HashMap<String, tauri_plugin_store::store_types::PluginUpdate>, String> {
        let store_state = self.app_handle.state::<StoreClientState>();
        let store_client = store_state.store_client.read().await;
        
        // UI'a bildirim gönder
        self.send_event("update_check_started", "");
        
        match store_client.check_for_updates().await {
            Ok(updates) => {
                // UI'a bildirim gönder
                if updates.is_empty() {
                    self.send_event("no_updates_available", "");
                } else {
                    self.send_event("updates_available", &updates.len().to_string());
                }
                
                Ok(updates)
            },
            Err(e) => {
                // UI'a bildirim gönder
                self.send_event("update_check_failed", "");
                
                Err(format!("Güncelleme kontrolü hatası: {:?}", e))
            },
        }
    }
    
    /// Plugin güncelle
    pub async fn update_plugin(&self, plugin_id: &str) -> Result<PluginUpdateStatus, String> {
        let store_state = self.app_handle.state::<StoreClientState>();
        let mut store_client = store_state.store_client.write().await;
        
        // UI'a bildirim gönder
        self.send_event("plugin_update_started", plugin_id);
        
        match store_client.update_plugin(plugin_id).await {
            Ok(status) => {
                if status.success {
                    // UI'a bildirim gönder
                    self.send_event("plugin_updated", plugin_id);
                    
                    // Plugin durumunu güncelle
                    let mut app_state = self.app_state.write().await;
                    if let Some(plugin) = app_state.get_mut(plugin_id) {
                        plugin.version = status.new_version.clone();
                    }
                    
                    Ok(status)
                } else {
                    // UI'a bildirim gönder
                    self.send_event("plugin_update_failed", plugin_id);
                    
                    Err(format!("Güncelleme başarısız: {:?}", status.error))
                }
            },
            Err(e) => {
                // UI'a bildirim gönder
                self.send_event("plugin_update_failed", plugin_id);
                
                Err(format!("Güncelleme hatası: {:?}", e))
            },
        }
    }
    
    /// Kaynak kullanımı al
    pub async fn get_resource_usage(&self, plugin_id: &str) -> Result<tauri_plugin_resource_monitor::ResourceUsage, String> {
        let resource_state = self.app_handle.state::<tauri_plugin_resource_monitor::ResourceMonitorState>();
        let resource_monitor = resource_state.resource_monitor.read().await;
        
        resource_monitor.get_resource_usage(plugin_id)
            .map_err(|e| format!("Kaynak kullanımı alınamadı: {:?}", e))
    }
    
    /// UI'a event gönder
    fn send_event(&self, event: &str, payload: &str) {
        if let Some(window) = self.app_handle.get_window("main") {
            let _ = window.emit(event, payload);
        }
    }
}
