// Tauri Windows Plugin System - Plugin Store Client
//
// Bu modül, merkezi bir eklenti mağazasından eklentileri keşfetme, indirme,
// kurma ve güncelleme işlevlerini sağlayan bir istemci sunar.

pub mod store_types;
pub mod store_client;
pub mod auth_manager;
pub mod download_manager;
pub mod install_manager;
pub mod local_registry;
pub mod api;

pub use store_types::{
    PluginMetadata, PluginType, PluginCategory, PluginVendor, 
    PluginPermission, PluginDependency, PluginPricing, PluginRating,
    PluginInstallation, PluginSearchFilter, PluginSearchResult,
    PluginDownloadInfo, PluginDownloadStatus, PluginInstallStatus,
    PluginUpdateStatus, PricingModel, ReleaseChannel, StoreError,
};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::RwLock;

/// Plugin Store hatası
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Store hatası: {0}")]
    Store(#[from] store_types::StoreError),

    #[error("Serileştirme hatası: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tauri hatası: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Kimlik doğrulama hatası: {0}")]
    Authentication(String),

    #[error("İndirme hatası: {0}")]
    Download(String),

    #[error("Kurulum hatası: {0}")]
    Installation(String),

    #[error("Güncelleme hatası: {0}")]
    Update(String),

    #[error("Yapılandırma hatası: {0}")]
    Configuration(String),
}

type Result<T> = std::result::Result<T, Error>;

/// Store Client komutları
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Commands {
    /// Plugin ara
    SearchPlugins {
        /// Arama filtresi
        filter: store_types::PluginSearchFilter,
    },
    /// Plugin detaylarını al
    GetPluginDetails {
        /// Plugin ID'si
        plugin_id: String,
    },
    /// Plugin indir
    DownloadPlugin {
        /// Plugin ID'si
        plugin_id: String,
        /// Plugin sürümü (opsiyonel)
        version: Option<String>,
    },
    /// Plugin indir ve kur
    InstallPlugin {
        /// Plugin ID'si
        plugin_id: String,
        /// Plugin sürümü (opsiyonel)
        version: Option<String>,
    },
    /// Plugin güncelle
    UpdatePlugin {
        /// Plugin ID'si
        plugin_id: String,
    },
    /// Plugin kaldır
    UninstallPlugin {
        /// Plugin ID'si
        plugin_id: String,
    },
    /// Kurulu pluginleri listele
    GetInstalledPlugins {},
    /// Güncelleme kontrolü yap
    CheckForUpdates {},
    /// Plugin aktif/pasif yap
    SetPluginEnabled {
        /// Plugin ID'si
        plugin_id: String,
        /// Aktif mi?
        enabled: bool,
    },
    /// Plugin yorumlarını al
    GetPluginReviews {
        /// Plugin ID'si
        plugin_id: String,
    },
}

/// Store Client plugin state
pub struct StoreClientState {
    /// Store client
    store_client: Arc<RwLock<store_client::StoreClient>>,
    /// App handle
    app_handle: Option<AppHandle>,
}

/// Store Client plugin konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreClientConfig {
    /// API base URL
    pub api_url: String,
    /// API anahtarı
    pub api_key: Option<String>,
    /// Kullanıcı belirteci
    pub user_token: Option<String>,
    /// Plugin kurulum dizini
    pub install_directory: Option<PathBuf>,
    /// Eşzamanlı indirme sayısı
    pub concurrent_downloads: Option<usize>,
    /// Otomatik güncelleme kontrolü
    pub auto_check_updates: Option<bool>,
    /// Otomatik güncelleme kontrolü aralığı (saat)
    pub auto_check_interval_hours: Option<u32>,
    /// Güvenilen sertifika otoriteleri
    pub trusted_certificates: Option<Vec<String>>,
}

impl Default for StoreClientConfig {
    fn default() -> Self {
        Self {
            api_url: "https://plugins.tauri-windows-plugin-system.dev/api".to_string(),
            api_key: None,
            user_token: None,
            install_directory: None,
            concurrent_downloads: Some(3),
            auto_check_updates: Some(true),
            auto_check_interval_hours: Some(24),
            trusted_certificates: None,
        }
    }
}

/// Store Client plugin
pub struct StorePlugin<R: Runtime> {
    /// Tauri plugin
    plugin: TauriPlugin<R>,
}

impl<R: Runtime> StorePlugin<R> {
    /// Yeni bir Store Client plugin oluştur
    pub fn new(config: StoreClientConfig) -> Result<Self> {
        // Store client oluştur
        let store_client = store_client::StoreClient::new(config)?;
        let store_client = Arc::new(RwLock::new(store_client));
        
        // Plugin state
        let state = StoreClientState {
            store_client,
            app_handle: None,
        };
        
        // Tauri plugin builder
        let plugin = Builder::new("store")
            .setup(move |app| {
                let state: State<'_, StoreClientState> = app.state();
                let mut state = state.inner().lock().blocking_unwrap();
                state.app_handle = Some(app.app_handle());
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                search_plugins,
                get_plugin_details,
                download_plugin,
                install_plugin,
                update_plugin,
                uninstall_plugin,
                get_installed_plugins,
                check_for_updates,
                set_plugin_enabled,
                get_plugin_reviews,
            ])
            .build();
        
        Ok(Self { plugin })
    }
    
    /// Tauri plugin'i al
    pub fn plugin(&self) -> TauriPlugin<R> {
        self.plugin.clone()
    }
}

/// Tauri plugin init
pub fn init<R: Runtime>(config: StoreClientConfig) -> TauriPlugin<R> {
    match StorePlugin::new(config) {
        Ok(plugin) => plugin.plugin(),
        Err(e) => {
            error!("Store Client başlatılamadı: {}", e);
            Builder::new("store").build()
        }
    }
}

/// Plugin ara
#[command]
async fn search_plugins(
    filter: store_types::PluginSearchFilter,
    state: State<'_, StoreClientState>,
) -> Result<store_types::PluginSearchResult> {
    let store_client = state.store_client.read().await;
    let result = store_client.search_plugins(filter).await?;
    Ok(result)
}

/// Plugin detaylarını al
#[command]
async fn get_plugin_details(
    plugin_id: String,
    state: State<'_, StoreClientState>,
) -> Result<store_types::PluginMetadata> {
    let store_client = state.store_client.read().await;
    let plugin = store_client.get_plugin_details(&plugin_id).await?;
    Ok(plugin)
}

/// Plugin indir
#[command]
async fn download_plugin(
    plugin_id: String,
    version: Option<String>,
    state: State<'_, StoreClientState>,
) -> Result<store_types::PluginDownloadStatus> {
    let store_client = state.store_client.read().await;
    let status = store_client.download_plugin(&plugin_id, version.as_deref()).await?;
    Ok(status)
}

/// Plugin indir ve kur
#[command]
async fn install_plugin(
    plugin_id: String,
    version: Option<String>,
    state: State<'_, StoreClientState>,
) -> Result<store_types::PluginInstallStatus> {
    let mut store_client = state.store_client.write().await;
    let status = store_client.install_plugin(&plugin_id, version.as_deref()).await?;
    Ok(status)
}

/// Plugin güncelle
#[command]
async fn update_plugin(
    plugin_id: String,
    state: State<'_, StoreClientState>,
) -> Result<store_types::PluginUpdateStatus> {
    let mut store_client = state.store_client.write().await;
    let status = store_client.update_plugin(&plugin_id).await?;
    Ok(status)
}

/// Plugin kaldır
#[command]
async fn uninstall_plugin(
    plugin_id: String,
    state: State<'_, StoreClientState>,
) -> Result<bool> {
    let mut store_client = state.store_client.write().await;
    let success = store_client.uninstall_plugin(&plugin_id).await?;
    Ok(success)
}

/// Kurulu pluginleri listele
#[command]
async fn get_installed_plugins(
    state: State<'_, StoreClientState>,
) -> Result<Vec<store_types::PluginInstallation>> {
    let store_client = state.store_client.read().await;
    let plugins = store_client.get_installed_plugins().await?;
    Ok(plugins)
}

/// Güncelleme kontrolü yap
#[command]
async fn check_for_updates(
    state: State<'_, StoreClientState>,
) -> Result<HashMap<String, store_types::PluginUpdate>> {
    let store_client = state.store_client.read().await;
    let updates = store_client.check_for_updates().await?;
    Ok(updates)
}

/// Plugin aktif/pasif yap
#[command]
async fn set_plugin_enabled(
    plugin_id: String,
    enabled: bool,
    state: State<'_, StoreClientState>,
) -> Result<bool> {
    let mut store_client = state.store_client.write().await;
    let success = store_client.set_plugin_enabled(&plugin_id, enabled).await?;
    Ok(success)
}

/// Plugin yorumlarını al
#[command]
async fn get_plugin_reviews(
    plugin_id: String,
    state: State<'_, StoreClientState>,
) -> Result<Vec<store_types::PluginReview>> {
    let store_client = state.store_client.read().await;
    let reviews = store_client.get_plugin_reviews(&plugin_id).await?;
    Ok(reviews)
}
