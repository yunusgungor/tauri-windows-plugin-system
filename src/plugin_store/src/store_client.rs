// Tauri Windows Plugin System - Store Client
//
// Bu modül, plugin mağazasıyla iletişim kuran ana istemci sınıfını içerir.
// Plugin arama, indirme, kurma ve güncelleme işlemlerini koordine eder.

use crate::store_types::{
    PluginMetadata, PluginSearchFilter, PluginSearchResult,
    PluginDownloadInfo, PluginDownloadStatus, PluginInstallStatus,
    PluginUpdateStatus, PluginInstallation, PluginReview, PluginUpdate,
    StoreError,
};
use crate::auth_manager::AuthManager;
use crate::download_manager::DownloadManager;
use crate::install_manager::InstallManager;
use crate::local_registry::LocalPluginRegistry;
use crate::api::ApiClient;
use crate::Error;

use log::{debug, error, info, warn};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::api::path;
use thiserror::Error;
use tokio::sync::RwLock;

/// Store Client konfigürasyonu
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

/// Store Client
pub struct StoreClient {
    /// Konfigürasyon
    config: StoreClientConfig,
    /// API istemcisi
    api_client: Arc<ApiClient>,
    /// Kimlik doğrulama yöneticisi
    auth_manager: Arc<RwLock<AuthManager>>,
    /// İndirme yöneticisi
    download_manager: Arc<RwLock<DownloadManager>>,
    /// Kurulum yöneticisi
    install_manager: Arc<RwLock<InstallManager>>,
    /// Yerel plugin kayıt sistemi
    local_registry: Arc<RwLock<LocalPluginRegistry>>,
}

impl StoreClient {
    /// Yeni bir Store Client oluşturur
    pub fn new(config: StoreClientConfig) -> Result<Self, Error> {
        // Kurulum dizinini belirle
        let install_dir = if let Some(dir) = &config.install_directory {
            dir.clone()
        } else {
            // Varsayılan kurulum dizini
            let app_data_dir = path::app_data_dir(&tauri::Config::default())
                .ok_or_else(|| Error::Configuration("App data dizini bulunamadı".to_string()))?;
            app_data_dir.join("plugins")
        };
        
        // Dizinin var olduğundan emin ol
        if !install_dir.exists() {
            std::fs::create_dir_all(&install_dir)?;
        }
        
        // API istemcisi oluştur
        let api_client = ApiClient::new(&config.api_url)?;
        let api_client = Arc::new(api_client);
        
        // Kimlik doğrulama yöneticisi oluştur
        let auth_manager = AuthManager::new(
            api_client.clone(),
            config.api_key.clone(),
            config.user_token.clone(),
        )?;
        let auth_manager = Arc::new(RwLock::new(auth_manager));
        
        // İndirme yöneticisi oluştur
        let concurrent_downloads = config.concurrent_downloads.unwrap_or(3);
        let download_manager = DownloadManager::new(
            api_client.clone(),
            auth_manager.clone(),
            concurrent_downloads,
        )?;
        let download_manager = Arc::new(RwLock::new(download_manager));
        
        // Yerel plugin kayıt sistemi oluştur
        let local_registry = LocalPluginRegistry::new(install_dir.clone())?;
        let local_registry = Arc::new(RwLock::new(local_registry));
        
        // Kurulum yöneticisi oluştur
        let install_manager = InstallManager::new(
            install_dir,
            download_manager.clone(),
            local_registry.clone(),
        )?;
        let install_manager = Arc::new(RwLock::new(install_manager));
        
        Ok(Self {
            config,
            api_client,
            auth_manager,
            download_manager,
            install_manager,
            local_registry,
        })
    }
    
    /// Plugin ara
    pub async fn search_plugins(&self, filter: PluginSearchFilter) -> Result<PluginSearchResult, Error> {
        // API token'ını al
        let token = self.auth_manager.read().await.get_api_token().await?;
        
        // API isteği yap
        let result = self.api_client.search_plugins(&token, filter).await?;
        Ok(result)
    }
    
    /// Plugin detaylarını al
    pub async fn get_plugin_details(&self, plugin_id: &str) -> Result<PluginMetadata, Error> {
        // API token'ını al
        let token = self.auth_manager.read().await.get_api_token().await?;
        
        // API isteği yap
        let plugin = self.api_client.get_plugin_details(&token, plugin_id).await?;
        Ok(plugin)
    }
    
    /// Plugin indir
    pub async fn download_plugin(&self, plugin_id: &str, version: Option<&str>) -> Result<PluginDownloadStatus, Error> {
        // Plugin detaylarını al
        let plugin = self.get_plugin_details(plugin_id).await?;
        
        // İndirme bilgilerini al
        let download_info = self.get_download_info(plugin_id, version).await?;
        
        // İndirme işlemini başlat
        let download_manager = self.download_manager.write().await;
        let status = download_manager.download_plugin(download_info).await?;
        
        Ok(status)
    }
    
    /// Plugin indirme bilgilerini al
    async fn get_download_info(&self, plugin_id: &str, version: Option<&str>) -> Result<PluginDownloadInfo, Error> {
        // API token'ını al
        let token = self.auth_manager.read().await.get_api_token().await?;
        
        // İndirme bilgilerini al
        let download_info = self.api_client.get_download_info(&token, plugin_id, version).await?;
        Ok(download_info)
    }
    
    /// Plugin indir ve kur
    pub async fn install_plugin(&mut self, plugin_id: &str, version: Option<&str>) -> Result<PluginInstallStatus, Error> {
        // Plugin indirme bilgilerini al
        let download_info = self.get_download_info(plugin_id, version).await?;
        
        // İndirme işlemini başlat
        let download_status = {
            let mut download_manager = self.download_manager.write().await;
            download_manager.download_plugin(download_info.clone()).await?
        };
        
        // İndirme tamamlandı mı kontrol et
        if download_status.status != crate::store_types::DownloadStatus::Completed {
            return Err(Error::Download(format!(
                "Plugin indirme başarısız: {:?}", 
                download_status.error.unwrap_or_else(|| "Bilinmeyen hata".to_string())
            )));
        }
        
        // Kurulum işlemini başlat
        let install_path = download_status.file_path.ok_or_else(|| {
            Error::Download("İndirilen dosya yolu bulunamadı".to_string())
        })?;
        
        let mut install_manager = self.install_manager.write().await;
        let install_status = install_manager.install_plugin(plugin_id, &install_path).await?;
        
        Ok(install_status)
    }
    
    /// Plugin güncelle
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<PluginUpdateStatus, Error> {
        // Kurulu plugin'i kontrol et
        let local_registry = self.local_registry.read().await;
        let installed_plugin = local_registry.get_plugin(plugin_id).await?;
        
        // Plugin güncelleme bilgilerini al
        let update_info = self.check_plugin_update(plugin_id).await?;
        
        if let Some(update) = update_info {
            // Güncelleme varsa indir
            let download_info = self.get_download_info(plugin_id, Some(&update.version.to_string())).await?;
            
            // İndirme işlemini başlat
            let download_status = {
                let mut download_manager = self.download_manager.write().await;
                download_manager.download_plugin(download_info).await?
            };
            
            // İndirme tamamlandı mı kontrol et
            if download_status.status != crate::store_types::DownloadStatus::Completed {
                return Err(Error::Download(format!(
                    "Plugin güncelleme indirmesi başarısız: {:?}", 
                    download_status.error.unwrap_or_else(|| "Bilinmeyen hata".to_string())
                )));
            }
            
            // Kurulum işlemini başlat
            let install_path = download_status.file_path.ok_or_else(|| {
                Error::Download("İndirilen dosya yolu bulunamadı".to_string())
            })?;
            
            let mut install_manager = self.install_manager.write().await;
            let update_status = install_manager.update_plugin(plugin_id, &install_path, &installed_plugin, &update).await?;
            
            Ok(update_status)
        } else {
            // Güncelleme yoksa mevcut durumu döndür
            Ok(PluginUpdateStatus {
                id: plugin_id.to_string(),
                name: installed_plugin.name.clone(),
                current_version: installed_plugin.installed_version.clone(),
                new_version: installed_plugin.installed_version.clone(),
                stage: crate::store_types::UpdateStage::Completed,
                progress: 100.0,
                error: None,
                success: true,
            })
        }
    }
    
    /// Plugin'i kaldır
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<bool, Error> {
        let mut install_manager = self.install_manager.write().await;
        let success = install_manager.uninstall_plugin(plugin_id).await?;
        Ok(success)
    }
    
    /// Kurulu pluginleri listele
    pub async fn get_installed_plugins(&self) -> Result<Vec<PluginInstallation>, Error> {
        let local_registry = self.local_registry.read().await;
        let plugins = local_registry.get_all_plugins().await?;
        Ok(plugins)
    }
    
    /// Güncelleme kontrolü yap
    pub async fn check_for_updates(&self) -> Result<HashMap<String, PluginUpdate>, Error> {
        // Kurulu tüm pluginleri al
        let local_registry = self.local_registry.read().await;
        let installed_plugins = local_registry.get_all_plugins().await?;
        
        let mut updates = HashMap::new();
        
        // Her plugin için güncelleme kontrolü yap
        for plugin in installed_plugins {
            if let Ok(Some(update)) = self.check_plugin_update(&plugin.id).await {
                updates.insert(plugin.id.clone(), update);
            }
        }
        
        Ok(updates)
    }
    
    /// Belirli bir plugin için güncelleme kontrolü yap
    async fn check_plugin_update(&self, plugin_id: &str) -> Result<Option<PluginUpdate>, Error> {
        // API token'ını al
        let token = self.auth_manager.read().await.get_api_token().await?;
        
        // Kurulu plugin'i kontrol et
        let local_registry = self.local_registry.read().await;
        let installed_plugin = local_registry.get_plugin(plugin_id).await?;
        
        // API'den güncelleme bilgilerini al
        let updates = self.api_client.check_plugin_updates(
            &token,
            plugin_id,
            &installed_plugin.installed_version.to_string(),
        ).await?;
        
        if let Some(update) = updates {
            // Kurulu sürümden daha yeni mi kontrol et
            if update.version > installed_plugin.installed_version {
                return Ok(Some(update));
            }
        }
        
        Ok(None)
    }
    
    /// Plugin'i aktif/pasif yap
    pub async fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<bool, Error> {
        let mut local_registry = self.local_registry.write().await;
        let success = local_registry.set_plugin_enabled(plugin_id, enabled).await?;
        Ok(success)
    }
    
    /// Plugin yorumlarını al
    pub async fn get_plugin_reviews(&self, plugin_id: &str) -> Result<Vec<PluginReview>, Error> {
        // API token'ını al
        let token = self.auth_manager.read().await.get_api_token().await?;
        
        // API isteği yap
        let reviews = self.api_client.get_plugin_reviews(&token, plugin_id).await?;
        Ok(reviews)
    }
}
