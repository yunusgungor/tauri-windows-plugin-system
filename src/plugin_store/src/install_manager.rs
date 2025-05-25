// Tauri Windows Plugin System - Install Manager
//
// Bu modül, plugin kurulum ve güncelleme işlemlerini yönetir.
// Plugin paketlerini çıkarma, bağımlılıkları kurma ve kurulum sonrası
// yapılandırma işlemlerini gerçekleştirir.

use crate::download_manager::DownloadManager;
use crate::local_registry::LocalPluginRegistry;
use crate::store_types::{
    PluginInstallStatus, PluginUpdateStatus, InstallStage, UpdateStage, 
    PluginInstallation, PluginMetadata, PluginUpdate, StoreError
};
use crate::Error;

use log::{debug, error, info, warn};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::api::path;
use tokio::sync::RwLock;
use uuid::Uuid;
use zip::ZipArchive;

/// Kurulum yöneticisi
pub struct InstallManager {
    /// Plugin kurulum dizini
    install_dir: PathBuf,
    /// İndirme yöneticisi
    download_manager: Arc<RwLock<DownloadManager>>,
    /// Yerel plugin kayıt sistemi
    local_registry: Arc<RwLock<LocalPluginRegistry>>,
}

impl InstallManager {
    /// Yeni bir kurulum yöneticisi oluştur
    pub fn new(
        install_dir: PathBuf,
        download_manager: Arc<RwLock<DownloadManager>>,
        local_registry: Arc<RwLock<LocalPluginRegistry>>,
    ) -> Result<Self, Error> {
        // Dizinin var olduğundan emin ol
        if !install_dir.exists() {
            fs::create_dir_all(&install_dir)?;
        }
        
        Ok(Self {
            install_dir,
            download_manager,
            local_registry,
        })
    }
    
    /// Plugin kur
    pub async fn install_plugin(&mut self, plugin_id: &str, package_path: &Path) -> Result<PluginInstallStatus, Error> {
        // Kurulum durumu oluştur
        let mut install_status = PluginInstallStatus {
            id: plugin_id.to_string(),
            name: plugin_id.to_string(), // Başlangıçta ID kullan
            stage: InstallStage::Validating,
            progress: 0.0,
            error: None,
            success: false,
        };
        
        // Kurulum dizini oluştur
        let plugin_dir = self.install_dir.join(plugin_id);
        if plugin_dir.exists() {
            // Mevcut kurulumu kontrol et
            let local_registry = self.local_registry.read().await;
            if local_registry.is_plugin_installed(plugin_id).await? {
                // Plugin zaten kurulu, önce kaldır
                drop(local_registry); // Kilidi bırak
                
                self.uninstall_plugin(plugin_id).await?;
            }
        }
        
        // Kurulum dizini oluştur
        fs::create_dir_all(&plugin_dir)?;
        
        // Paket dosyasını doğrula
        install_status.stage = InstallStage::Validating;
        install_status.progress = 10.0;
        
        if !package_path.exists() {
            return Err(Error::Installation(format!(
                "Paket dosyası bulunamadı: {:?}", 
                package_path
            )));
        }
        
        // Paket içeriğini çıkar
        install_status.stage = InstallStage::Extracting;
        install_status.progress = 20.0;
        
        let plugin_metadata = self.extract_package(package_path, &plugin_dir)?;
        
        // İsim güncelle
        install_status.name = plugin_metadata.name.clone();
        
        // Bağımlılıkları kur
        install_status.stage = InstallStage::InstallingDependencies;
        install_status.progress = 60.0;
        
        self.install_dependencies(&plugin_metadata).await?;
        
        // Kurulum sonrası yapılandırma
        install_status.stage = InstallStage::Configuring;
        install_status.progress = 80.0;
        
        self.configure_plugin(plugin_id, &plugin_dir, &plugin_metadata).await?;
        
        // Yerel kayıt güncelle
        let mut local_registry = self.local_registry.write().await;
        local_registry.add_plugin(
            plugin_id,
            &plugin_metadata.name,
            &plugin_metadata.version.to_string(),
            &plugin_metadata.description,
            &plugin_metadata.vendor.name,
            true, // Varsayılan olarak aktif
        ).await?;
        
        // Kurulum tamamlandı
        install_status.stage = InstallStage::Completed;
        install_status.progress = 100.0;
        install_status.success = true;
        
        Ok(install_status)
    }
    
    /// Plugin güncelle
    pub async fn update_plugin(
        &mut self, 
        plugin_id: &str, 
        package_path: &Path,
        installed_plugin: &PluginInstallation,
        update_info: &PluginUpdate,
    ) -> Result<PluginUpdateStatus, Error> {
        // Güncelleme durumu oluştur
        let mut update_status = PluginUpdateStatus {
            id: plugin_id.to_string(),
            name: installed_plugin.name.clone(),
            current_version: installed_plugin.installed_version.clone(),
            new_version: update_info.version.to_string(),
            stage: UpdateStage::Validating,
            progress: 0.0,
            error: None,
            success: false,
        };
        
        // Yedekleme dizini oluştur
        let backup_dir = self.install_dir.join(format!("{}_backup", plugin_id));
        let plugin_dir = self.install_dir.join(plugin_id);
        
        // Mevcut kurulumu yedekle
        update_status.stage = UpdateStage::Backing;
        update_status.progress = 10.0;
        
        if plugin_dir.exists() {
            // Önceki yedekleri temizle
            if backup_dir.exists() {
                fs::remove_dir_all(&backup_dir)?;
            }
            
            // Dizini yedekle
            fs::create_dir_all(&backup_dir)?;
            self.copy_dir_all(&plugin_dir, &backup_dir)?;
        } else {
            return Err(Error::Update(format!(
                "Plugin kurulum dizini bulunamadı: {:?}", 
                plugin_dir
            )));
        }
        
        // Mevcut kurulumu kaldır
        update_status.stage = UpdateStage::Removing;
        update_status.progress = 20.0;
        
        // Mevcut kurulumu sil, ancak yerel kayıttan silme
        fs::remove_dir_all(&plugin_dir)?;
        fs::create_dir_all(&plugin_dir)?;
        
        // Paket içeriğini çıkar
        update_status.stage = UpdateStage::Extracting;
        update_status.progress = 30.0;
        
        let plugin_metadata = self.extract_package(package_path, &plugin_dir)?;
        
        // Bağımlılıkları kur
        update_status.stage = UpdateStage::InstallingDependencies;
        update_status.progress = 60.0;
        
        self.install_dependencies(&plugin_metadata).await?;
        
        // Kurulum sonrası yapılandırma
        update_status.stage = UpdateStage::Configuring;
        update_status.progress = 80.0;
        
        self.configure_plugin(plugin_id, &plugin_dir, &plugin_metadata).await?;
        
        // Yerel kayıt güncelle
        let mut local_registry = self.local_registry.write().await;
        local_registry.update_plugin_version(
            plugin_id,
            &plugin_metadata.version.to_string(),
        ).await?;
        
        // Yedekleme dizinini temizle
        if backup_dir.exists() {
            let _ = fs::remove_dir_all(&backup_dir);
        }
        
        // Güncelleme tamamlandı
        update_status.stage = UpdateStage::Completed;
        update_status.progress = 100.0;
        update_status.success = true;
        
        Ok(update_status)
    }
    
    /// Plugin kaldır
    pub async fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<bool, Error> {
        // Plugin kurulum dizini
        let plugin_dir = self.install_dir.join(plugin_id);
        
        // Plugin kurulu mu kontrol et
        let local_registry = self.local_registry.read().await;
        if !local_registry.is_plugin_installed(plugin_id).await? {
            return Err(Error::Installation(format!(
                "Plugin kurulu değil: {}", 
                plugin_id
            )));
        }
        
        // Yerel kayıttan kaldır
        drop(local_registry); // Kilidi bırak
        let mut local_registry = self.local_registry.write().await;
        local_registry.remove_plugin(plugin_id).await?;
        
        // Kurulum dizinini temizle
        if plugin_dir.exists() {
            fs::remove_dir_all(&plugin_dir)?;
        }
        
        Ok(true)
    }
    
    /// Paket içeriğini çıkar ve plugin metadata'sını döndür
    fn extract_package(&self, package_path: &Path, extract_dir: &Path) -> Result<PluginMetadata, Error> {
        // ZIP arşivini aç
        let file = File::open(package_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Arşiv içeriğini çıkar
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => extract_dir.join(path),
                None => continue,
            };
            
            if file.name().ends_with('/') {
                // Dizin oluştur
                fs::create_dir_all(&outpath)?;
            } else {
                // Üst dizini oluştur
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                
                // Dosyayı çıkar
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        
        // Plugin metadata dosyasını oku
        let metadata_path = extract_dir.join("plugin.json");
        if !metadata_path.exists() {
            return Err(Error::Installation(
                "Plugin metadata dosyası bulunamadı: plugin.json".to_string()
            ));
        }
        
        // Metadata'yı oku
        let mut file = File::open(metadata_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // JSON'u ayrıştır
        let metadata: PluginMetadata = serde_json::from_str(&contents)
            .map_err(|e| Error::Installation(format!(
                "Plugin metadata ayrıştırılamadı: {}", 
                e
            )))?;
        
        Ok(metadata)
    }
    
    /// Plugin bağımlılıklarını kur
    async fn install_dependencies(&self, plugin_metadata: &PluginMetadata) -> Result<(), Error> {
        // Şu an için boş, gelecekte bağımlılık yönetimi eklenebilir
        Ok(())
    }
    
    /// Plugin yapılandırma
    async fn configure_plugin(&self, plugin_id: &str, plugin_dir: &Path, plugin_metadata: &PluginMetadata) -> Result<(), Error> {
        // Şu an için boş, gelecekte yapılandırma işlemleri eklenebilir
        Ok(())
    }
    
    /// Dizin kopyala
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<(), Error> {
        // Kaynak dizin var mı kontrol et
        if !src.exists() {
            return Err(Error::Installation(format!(
                "Kaynak dizin bulunamadı: {:?}", 
                src
            )));
        }
        
        // Hedef dizin oluştur
        if !dst.exists() {
            fs::create_dir_all(dst)?;
        }
        
        // Dizin içeriğini kopyala
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            
            if ty.is_dir() {
                // Alt dizinleri kopyala
                self.copy_dir_all(&src_path, &dst_path)?;
            } else {
                // Dosyaları kopyala
                fs::copy(&src_path, &dst_path)?;
            }
        }
        
        Ok(())
    }
}
