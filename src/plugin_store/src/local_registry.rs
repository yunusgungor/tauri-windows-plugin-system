// Tauri Windows Plugin System - Local Plugin Registry
//
// Bu modül, yerel olarak kurulu pluginlerin kaydını tutar.
// Plugin metadata'sını depolar ve güncelleme kontrolü sağlar.

use crate::store_types::{PluginInstallation, StoreError};
use crate::Error;

use log::{debug, error, info, warn};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::api::path;
use tokio::sync::RwLock;

/// Yerel plugin kaydı
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginRegistryData {
    /// Plugin listesi
    plugins: HashMap<String, PluginRegistryEntry>,
    /// Son güncelleme zamanı (Unix timestamp)
    last_updated: u64,
}

/// Plugin kayıt girişi
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginRegistryEntry {
    /// Plugin ID'si
    id: String,
    /// Plugin adı
    name: String,
    /// Kurulu versiyon
    version: String,
    /// Açıklama
    description: String,
    /// Geliştirici
    vendor: String,
    /// Aktif mi?
    enabled: bool,
    /// Kurulum zamanı (Unix timestamp)
    installed_at: u64,
    /// Son güncelleme zamanı (Unix timestamp)
    last_updated: u64,
}

/// Yerel plugin kayıt sistemi
pub struct LocalPluginRegistry {
    /// Veri dosyası yolu
    registry_file: PathBuf,
    /// Plugin kurulum dizini
    install_dir: PathBuf,
    /// Plugin kayıt verileri
    data: RwLock<PluginRegistryData>,
}

impl LocalPluginRegistry {
    /// Yeni bir yerel plugin kayıt sistemi oluştur
    pub fn new(install_dir: PathBuf) -> Result<Self, Error> {
        // Veri dosyası yolu
        let app_data_dir = path::app_data_dir(&tauri::Config::default())
            .ok_or_else(|| Error::Configuration("App data dizini bulunamadı".to_string()))?;
        let registry_dir = app_data_dir.join("plugin_registry");
        let registry_file = registry_dir.join("registry.json");
        
        // Dizinin var olduğundan emin ol
        if !registry_dir.exists() {
            fs::create_dir_all(&registry_dir)?;
        }
        
        // Veri dosyasını oku veya oluştur
        let data = if registry_file.exists() {
            let mut file = File::open(&registry_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            // JSON'u ayrıştır
            serde_json::from_str(&contents).unwrap_or_else(|_| PluginRegistryData {
                plugins: HashMap::new(),
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            })
        } else {
            // Yeni veri oluştur
            PluginRegistryData {
                plugins: HashMap::new(),
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            }
        };
        
        Ok(Self {
            registry_file,
            install_dir,
            data: RwLock::new(data),
        })
    }
    
    /// Kayıt dosyasını kaydet
    async fn save_registry(&self) -> Result<(), Error> {
        // Veriyi al
        let data = self.data.read().await;
        
        // JSON'a çevir
        let json = serde_json::to_string_pretty(&*data)?;
        
        // Dosyaya yaz
        let mut file = File::create(&self.registry_file)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    /// Plugin ekle
    pub async fn add_plugin(
        &mut self,
        plugin_id: &str,
        name: &str,
        version: &str,
        description: &str,
        vendor: &str,
        enabled: bool,
    ) -> Result<(), Error> {
        // Versiyon doğrula
        let _version = Version::parse(version).map_err(|e| {
            Error::Configuration(format!("Geçersiz versiyon formatı: {}", e))
        })?;
        
        // Plugini ekle
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        let entry = PluginRegistryEntry {
            id: plugin_id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            vendor: vendor.to_string(),
            enabled,
            installed_at: now,
            last_updated: now,
        };
        
        // Veriyi güncelle
        {
            let mut data = self.data.write().await;
            data.plugins.insert(plugin_id.to_string(), entry);
            data.last_updated = now;
        }
        
        // Kaydet
        self.save_registry().await?;
        
        Ok(())
    }
    
    /// Plugin sil
    pub async fn remove_plugin(&mut self, plugin_id: &str) -> Result<(), Error> {
        // Plugini sil
        {
            let mut data = self.data.write().await;
            data.plugins.remove(plugin_id);
            data.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
        }
        
        // Kaydet
        self.save_registry().await?;
        
        Ok(())
    }
    
    /// Plugin versiyonunu güncelle
    pub async fn update_plugin_version(&mut self, plugin_id: &str, version: &str) -> Result<(), Error> {
        // Versiyon doğrula
        let _version = Version::parse(version).map_err(|e| {
            Error::Configuration(format!("Geçersiz versiyon formatı: {}", e))
        })?;
        
        // Plugini güncelle
        {
            let mut data = self.data.write().await;
            if let Some(entry) = data.plugins.get_mut(plugin_id) {
                entry.version = version.to_string();
                entry.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                
                data.last_updated = entry.last_updated;
            } else {
                return Err(Error::Configuration(format!(
                    "Plugin bulunamadı: {}", 
                    plugin_id
                )));
            }
        }
        
        // Kaydet
        self.save_registry().await?;
        
        Ok(())
    }
    
    /// Plugin aktif/pasif yap
    pub async fn set_plugin_enabled(&mut self, plugin_id: &str, enabled: bool) -> Result<bool, Error> {
        // Plugini güncelle
        {
            let mut data = self.data.write().await;
            if let Some(entry) = data.plugins.get_mut(plugin_id) {
                entry.enabled = enabled;
                entry.last_updated = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                
                data.last_updated = entry.last_updated;
            } else {
                return Err(Error::Configuration(format!(
                    "Plugin bulunamadı: {}", 
                    plugin_id
                )));
            }
        }
        
        // Kaydet
        self.save_registry().await?;
        
        Ok(enabled)
    }
    
    /// Plugin bilgisini al
    pub async fn get_plugin(&self, plugin_id: &str) -> Result<PluginInstallation, Error> {
        // Plugini al
        let data = self.data.read().await;
        if let Some(entry) = data.plugins.get(plugin_id) {
            let version = Version::parse(&entry.version).map_err(|e| {
                Error::Configuration(format!("Geçersiz versiyon formatı: {}", e))
            })?;
            
            Ok(PluginInstallation {
                id: entry.id.clone(),
                name: entry.name.clone(),
                installed_version: version,
                description: entry.description.clone(),
                vendor: entry.vendor.clone(),
                enabled: entry.enabled,
                install_path: self.install_dir.join(plugin_id),
                installed_at: entry.installed_at,
                last_updated: entry.last_updated,
            })
        } else {
            Err(Error::Configuration(format!(
                "Plugin bulunamadı: {}", 
                plugin_id
            )))
        }
    }
    
    /// Tüm pluginleri al
    pub async fn get_all_plugins(&self) -> Result<Vec<PluginInstallation>, Error> {
        // Tüm pluginleri al
        let data = self.data.read().await;
        let mut plugins = Vec::new();
        
        for entry in data.plugins.values() {
            let version = Version::parse(&entry.version).map_err(|e| {
                Error::Configuration(format!("Geçersiz versiyon formatı: {}", e))
            })?;
            
            plugins.push(PluginInstallation {
                id: entry.id.clone(),
                name: entry.name.clone(),
                installed_version: version,
                description: entry.description.clone(),
                vendor: entry.vendor.clone(),
                enabled: entry.enabled,
                install_path: self.install_dir.join(&entry.id),
                installed_at: entry.installed_at,
                last_updated: entry.last_updated,
            });
        }
        
        Ok(plugins)
    }
    
    /// Plugin kurulu mu kontrol et
    pub async fn is_plugin_installed(&self, plugin_id: &str) -> Result<bool, Error> {
        // Plugini kontrol et
        let data = self.data.read().await;
        Ok(data.plugins.contains_key(plugin_id))
    }
    
    /// Plugin aktif mi kontrol et
    pub async fn is_plugin_enabled(&self, plugin_id: &str) -> Result<bool, Error> {
        // Plugini kontrol et
        let data = self.data.read().await;
        if let Some(entry) = data.plugins.get(plugin_id) {
            Ok(entry.enabled)
        } else {
            Err(Error::Configuration(format!(
                "Plugin bulunamadı: {}", 
                plugin_id
            )))
        }
    }
    
    /// Son güncelleme zamanını al
    pub async fn get_last_update_time(&self) -> u64 {
        let data = self.data.read().await;
        data.last_updated
    }
    
    /// Plugin sayısını al
    pub async fn get_plugin_count(&self) -> usize {
        let data = self.data.read().await;
        data.plugins.len()
    }
    
    /// Aktif plugin sayısını al
    pub async fn get_enabled_plugin_count(&self) -> usize {
        let data = self.data.read().await;
        data.plugins.values().filter(|p| p.enabled).count()
    }
}
