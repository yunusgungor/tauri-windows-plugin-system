// Tauri Windows Plugin System - Resource Monitor
//
// Bu modül, plugin'lerin kaynak kullanımını izler ve yönetir.
// Windows API'lerini kullanarak CPU, bellek, disk ve ağ kullanımını
// gerçek zamanlı olarak izler ve kaynak limitlerini uygular.

mod resource_types;
mod monitor;

pub use resource_types::{
    LimitAction, ResourceLimit, ResourceLimitEvent, ResourceMeasurement,
    ResourceMonitorConfig, ResourceType, ResourceUnit, ResourceUsageProfile,
};

use log::{debug, error, info, warn};
use monitor::ResourceMonitor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Kaynak izleyici state
pub struct ResourceMonitorState {
    /// Kaynak izleyici
    monitor: Arc<Mutex<ResourceMonitor>>,
    /// Tauri uygulama handle'ı
    app_handle: Option<AppHandle>,
}

/// Kaynak izleyici hatası
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Kaynak izleyici hatası: {0}")]
    Monitor(#[from] monitor::ResourceMonitorError),

    #[error("JSON serileştirme hatası: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tauri hatası: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Plugin bulunamadı: {0}")]
    PluginNotFound(String),

    #[error("Geçersiz yapılandırma: {0}")]
    InvalidConfig(String),
}

type Result<T> = std::result::Result<T, Error>;

/// Kaynak izleme komutları
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Commands {
    /// Plugin'i izlemeye başla
    StartMonitoring {
        /// Plugin ID'si
        plugin_id: String,
        /// Process ID'si
        process_id: u32,
    },
    /// Plugin'i izlemeyi durdur
    StopMonitoring {
        /// Plugin ID'si
        plugin_id: String,
    },
    /// Plugin kaynak kullanımını al
    GetResourceUsage {
        /// Plugin ID'si
        plugin_id: String,
    },
    /// Kaynak limitlerini al
    GetResourceLimits {},
    /// Kaynak limitlerini güncelle
    UpdateResourceLimits {
        /// Yeni limitler
        limits: Vec<ResourceLimit>,
    },
    /// Tüm izlenen plugin'leri listele
    ListMonitoredPlugins {},
    /// Limit aşım olaylarını al
    GetLimitEvents {
        /// Plugin ID'si
        plugin_id: String,
    },
}

/// Kaynak izleyici Tauri eklentisi
pub struct ResourceMonitorPlugin<R: Runtime> {
    /// Tauri eklentisi
    plugin: TauriPlugin<R>,
}

impl<R: Runtime> ResourceMonitorPlugin<R> {
    /// Yeni bir kaynak izleyici eklentisi oluştur
    pub fn new(config: ResourceMonitorConfig) -> Result<Self> {
        let monitor = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async {
                ResourceMonitor::new(config).await
            })?;

        // Plugin state
        let state = ResourceMonitorState {
            monitor: Arc::new(Mutex::new(monitor)),
            app_handle: None,
        };

        // Tauri plugin builder
        let plugin = Builder::new("resource_monitor")
            .setup(move |app| {
                let state: State<'_, ResourceMonitorState> = app.state();
                let mut state = state.inner().lock().blocking_unwrap();
                state.app_handle = Some(app.app_handle());
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                start_monitoring,
                stop_monitoring,
                get_resource_usage,
                get_resource_limits,
                update_resource_limits,
                list_monitored_plugins,
                get_limit_events,
            ])
            .build();

        Ok(Self { plugin })
    }

    /// Tauri plugin'ini döndür
    pub fn plugin(&self) -> TauriPlugin<R> {
        self.plugin.clone()
    }
}

/// Tauri plugin init
pub fn init<R: Runtime>(config: ResourceMonitorConfig) -> TauriPlugin<R> {
    match ResourceMonitorPlugin::new(config) {
        Ok(plugin) => plugin.plugin(),
        Err(e) => {
            error!("Kaynak izleyici başlatılamadı: {}", e);
            Builder::new("resource_monitor").build()
        }
    }
}

/// Plugin'i izlemeye başla
#[tauri::command]
async fn start_monitoring(
    plugin_id: String,
    process_id: u32,
    state: State<'_, ResourceMonitorState>,
) -> Result<()> {
    info!("Plugin izlemeye başlanıyor: {} (PID: {})", plugin_id, process_id);
    let monitor = state.monitor.lock().await;
    monitor.start_monitoring(&plugin_id, process_id).await?;
    Ok(())
}

/// Plugin'i izlemeyi durdur
#[tauri::command]
async fn stop_monitoring(
    plugin_id: String,
    state: State<'_, ResourceMonitorState>,
) -> Result<()> {
    info!("Plugin izleme durduruluyor: {}", plugin_id);
    let monitor = state.monitor.lock().await;
    monitor.stop_monitoring(&plugin_id).await?;
    Ok(())
}

/// Plugin kaynak kullanımını al
#[tauri::command]
async fn get_resource_usage(
    plugin_id: String,
    state: State<'_, ResourceMonitorState>,
) -> Result<String> {
    let monitor = state.monitor.lock().await;
    let profile = monitor
        .get_usage_profile(&plugin_id)
        .ok_or_else(|| Error::PluginNotFound(plugin_id.clone()))?;
    
    let json = serde_json::to_string(&profile)?;
    Ok(json)
}

/// Kaynak limitlerini al
#[tauri::command]
async fn get_resource_limits(
    state: State<'_, ResourceMonitorState>,
) -> Result<String> {
    let monitor = state.monitor.lock().await;
    let config = monitor.get_config();
    let json = serde_json::to_string(&config.resource_limits)?;
    Ok(json)
}

/// Kaynak limitlerini güncelle
#[tauri::command]
async fn update_resource_limits(
    limits: Vec<ResourceLimit>,
    state: State<'_, ResourceMonitorState>,
) -> Result<()> {
    let mut monitor = state.monitor.lock().await;
    let mut config = monitor.get_config();
    config.resource_limits = limits;
    monitor.update_config(config).await?;
    Ok(())
}

/// Tüm izlenen plugin'leri listele
#[tauri::command]
async fn list_monitored_plugins(
    state: State<'_, ResourceMonitorState>,
) -> Result<String> {
    let monitor = state.monitor.lock().await;
    let plugins = monitor.list_monitored_plugins();
    let json = serde_json::to_string(&plugins)?;
    Ok(json)
}

/// Limit aşım olaylarını al
#[tauri::command]
async fn get_limit_events(
    plugin_id: String,
    state: State<'_, ResourceMonitorState>,
) -> Result<String> {
    let monitor = state.monitor.lock().await;
    let events = monitor.get_limit_events(&plugin_id);
    let json = serde_json::to_string(&events)?;
    Ok(json)
}
