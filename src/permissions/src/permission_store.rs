// Tauri Windows Plugin System - İzin Deposu
//
// Bu modül, plugin izinlerini ve kullanıcı kararlarını kalıcı olarak saklar.

use crate::permission_types::{PermissionDescriptor, PermissionToken};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// İzin deposu hatası
#[derive(Error, Debug)]
pub enum PermissionStoreError {
    #[error("I/O hatası: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serileştirme hatası: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Geçersiz veri: {0}")]
    InvalidData(String),

    #[error("Plugin bulunamadı: {0}")]
    PluginNotFound(String),
}

/// Plugin bilgisi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin kimliği
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Plugin sürümü
    pub version: String,
    /// Geliştirici bilgisi
    pub developer: String,
    /// İkon URL'i (opsiyonel)
    pub icon_url: Option<String>,
}

/// İzin kararı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    /// İzin tanımlayıcısı
    pub descriptor: PermissionDescriptor,
    /// İzin verildi mi?
    pub granted: bool,
    /// Karar zamanı
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// İzin deposu
pub struct PermissionStore {
    /// Depolama dizini
    storage_dir: PathBuf,
    /// Önbellek: Plugin kimliği -> Plugin bilgisi
    plugin_cache: HashMap<String, PluginInfo>,
}

impl PermissionStore {
    /// Yeni bir izin deposu oluştur
    pub fn new(storage_dir: PathBuf) -> Result<Self, PermissionStoreError> {
        // Depolama dizininin var olduğundan emin ol
        fs::create_dir_all(&storage_dir)?;
        
        // Plugin alt dizinini de oluştur
        let plugin_dir = storage_dir.join("plugins");
        fs::create_dir_all(&plugin_dir)?;
        
        // İzinler alt dizinini oluştur
        let permissions_dir = storage_dir.join("permissions");
        fs::create_dir_all(&permissions_dir)?;
        
        Ok(Self {
            storage_dir,
            plugin_cache: HashMap::new(),
        })
    }
    
    /// Plugin bilgisini kaydet
    pub fn save_plugin_info(&mut self, info: &PluginInfo) -> Result<(), PermissionStoreError> {
        let plugin_dir = self.storage_dir.join("plugins");
        let file_path = plugin_dir.join(format!("{}.json", info.id));
        
        let json = serde_json::to_string_pretty(info)?;
        fs::write(file_path, json)?;
        
        // Önbelleğe ekle
        self.plugin_cache.insert(info.id.clone(), info.clone());
        
        Ok(())
    }
    
    /// Plugin bilgisini al
    pub fn get_plugin_info(&self, plugin_id: &str) -> Result<PluginInfo, PermissionStoreError> {
        // Önce önbellekte kontrol et
        if let Some(info) = self.plugin_cache.get(plugin_id) {
            return Ok(info.clone());
        }
        
        // Dosyadan oku
        let plugin_dir = self.storage_dir.join("plugins");
        let file_path = plugin_dir.join(format!("{}.json", plugin_id));
        
        if !file_path.exists() {
            return Err(PermissionStoreError::PluginNotFound(plugin_id.to_string()));
        }
        
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let info: PluginInfo = serde_json::from_str(&contents)?;
        
        Ok(info)
    }
    
    /// İzin kararını kaydet
    pub fn save_permission_decision(
        &self,
        plugin_id: &str,
        descriptors: &[PermissionDescriptor],
        granted: bool,
    ) -> Result<(), PermissionStoreError> {
        let plugin_permissions_dir = self.storage_dir.join("permissions").join(plugin_id);
        fs::create_dir_all(&plugin_permissions_dir)?;
        
        let now = chrono::Utc::now();
        
        for descriptor in descriptors {
            let decision = PermissionDecision {
                descriptor: descriptor.clone(),
                granted,
                timestamp: now,
            };
            
            // Dosya adı için izin kategorisi ve kapsamını kullan
            let file_name = format!("{}_{:x}.json", descriptor.category, descriptor.scope);
            let file_path = plugin_permissions_dir.join(file_name);
            
            let json = serde_json::to_string_pretty(&decision)?;
            fs::write(file_path, json)?;
        }
        
        Ok(())
    }
    
    /// İzin kararını al
    pub fn get_permission_decision(
        &self,
        plugin_id: &str,
        descriptor: &PermissionDescriptor,
    ) -> Result<Option<PermissionDecision>, PermissionStoreError> {
        let plugin_permissions_dir = self.storage_dir.join("permissions").join(plugin_id);
        
        if !plugin_permissions_dir.exists() {
            return Ok(None);
        }
        
        // Dosya adını oluştur
        let file_name = format!("{}_{:x}.json", descriptor.category, descriptor.scope);
        let file_path = plugin_permissions_dir.join(file_name);
        
        if !file_path.exists() {
            return Ok(None);
        }
        
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        let decision: PermissionDecision = serde_json::from_str(&contents)?;
        
        Ok(Some(decision))
    }
    
    /// Tüm plugin izin belirteçlerini kaydet
    pub fn save_all_permissions(
        &self,
        tokens: &HashMap<String, PermissionToken>,
    ) -> Result<(), PermissionStoreError> {
        let tokens_dir = self.storage_dir.join("tokens");
        fs::create_dir_all(&tokens_dir)?;
        
        // Tüm mevcut belirteç dosyalarını temizle
        for entry in fs::read_dir(&tokens_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                fs::remove_file(entry.path())?;
            }
        }
        
        // Yeni belirteçleri kaydet
        for (plugin_id, token) in tokens {
            let file_path = tokens_dir.join(format!("{}.json", plugin_id));
            let json = serde_json::to_string_pretty(token)?;
            fs::write(file_path, json)?;
        }
        
        Ok(())
    }
    
    /// Tüm plugin izin belirteçlerini yükle
    pub fn load_all_permissions(&self) -> Result<HashMap<String, PermissionToken>, PermissionStoreError> {
        let tokens_dir = self.storage_dir.join("tokens");
        
        if !tokens_dir.exists() {
            fs::create_dir_all(&tokens_dir)?;
            return Ok(HashMap::new());
        }
        
        let mut tokens = HashMap::new();
        
        for entry in fs::read_dir(tokens_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension != "json" {
                    continue;
                }
            } else {
                continue;
            }
            
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            let token: PermissionToken = serde_json::from_str(&contents)?;
            tokens.insert(token.plugin_id.clone(), token);
        }
        
        Ok(tokens)
    }
    
    /// Belirli bir plugin'in tüm izin kararlarını al
    pub fn get_all_decisions_for_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<PermissionDecision>, PermissionStoreError> {
        let plugin_permissions_dir = self.storage_dir.join("permissions").join(plugin_id);
        
        if !plugin_permissions_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut decisions = Vec::new();
        
        for entry in fs::read_dir(plugin_permissions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension != "json" {
                    continue;
                }
            } else {
                continue;
            }
            
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            let decision: PermissionDecision = serde_json::from_str(&contents)?;
            decisions.push(decision);
        }
        
        Ok(decisions)
    }
    
    /// Tüm plugin bilgilerini listele
    pub fn list_all_plugins(&self) -> Result<Vec<PluginInfo>, PermissionStoreError> {
        let plugin_dir = self.storage_dir.join("plugins");
        
        if !plugin_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut plugins = Vec::new();
        
        for entry in fs::read_dir(plugin_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension != "json" {
                    continue;
                }
            } else {
                continue;
            }
            
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            
            let info: PluginInfo = serde_json::from_str(&contents)?;
            plugins.push(info);
        }
        
        Ok(plugins)
    }
}
