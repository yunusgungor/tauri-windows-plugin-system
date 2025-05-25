// Tauri Windows Plugin System - Download Manager
//
// Bu modül, plugin indirme işlemlerini yönetir.
// Eşzamanlı indirme, ilerleme izleme ve doğrulama işlemlerini gerçekleştirir.

use crate::api::ApiClient;
use crate::auth_manager::AuthManager;
use crate::store_types::{PluginDownloadInfo, PluginDownloadStatus, DownloadStatus, StoreError};
use crate::Error;

use log::{debug, error, info, warn};
use reqwest::Client;
use semver::Version;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::api::path;
use tokio::sync::{Mutex, RwLock, Semaphore};
use uuid::Uuid;

/// İndirme işlemi
#[derive(Debug)]
struct DownloadTask {
    /// İndirme ID'si
    id: String,
    /// Plugin ID'si
    plugin_id: String,
    /// Plugin versiyonu
    version: Version,
    /// İndirme URL'si
    url: String,
    /// Beklenen dosya hash'i
    expected_hash: Option<String>,
    /// İndirme durumu
    status: DownloadStatus,
    /// İndirme ilerlemesi (0-100)
    progress: f32,
    /// İndirilen dosya yolu
    file_path: Option<PathBuf>,
    /// Hata mesajı
    error: Option<String>,
}

/// İndirme yöneticisi
pub struct DownloadManager {
    /// API istemcisi
    api_client: Arc<ApiClient>,
    /// Kimlik doğrulama yöneticisi
    auth_manager: Arc<RwLock<AuthManager>>,
    /// HTTP istemcisi
    http_client: Client,
    /// Eşzamanlı indirme sayısı için semaphore
    download_semaphore: Arc<Semaphore>,
    /// Aktif indirme işlemleri
    active_downloads: Arc<Mutex<HashMap<String, DownloadTask>>>,
    /// Geçici indirme dizini
    temp_dir: PathBuf,
}

impl DownloadManager {
    /// Yeni bir indirme yöneticisi oluştur
    pub fn new(
        api_client: Arc<ApiClient>,
        auth_manager: Arc<RwLock<AuthManager>>,
        concurrent_downloads: usize,
    ) -> Result<Self, Error> {
        // HTTP istemcisi oluştur
        let http_client = Client::new();
        
        // Geçici dizin oluştur
        let app_cache_dir = path::cache_dir()
            .ok_or_else(|| Error::Configuration("Cache dizini bulunamadı".to_string()))?
            .join("tauri-plugin-store")
            .join("downloads");
        
        // Dizinin var olduğundan emin ol
        if !app_cache_dir.exists() {
            fs::create_dir_all(&app_cache_dir)?;
        }
        
        Ok(Self {
            api_client,
            auth_manager,
            http_client,
            download_semaphore: Arc::new(Semaphore::new(concurrent_downloads)),
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            temp_dir: app_cache_dir,
        })
    }
    
    /// Plugin indir
    pub async fn download_plugin(&self, download_info: PluginDownloadInfo) -> Result<PluginDownloadStatus, Error> {
        // İndirme işlemi oluştur
        let download_id = Uuid::new_v4().to_string();
        let task = DownloadTask {
            id: download_id.clone(),
            plugin_id: download_info.plugin_id.clone(),
            version: download_info.version.clone(),
            url: download_info.download_url.clone(),
            expected_hash: download_info.sha256_hash.clone(),
            status: DownloadStatus::Pending,
            progress: 0.0,
            file_path: None,
            error: None,
        };
        
        // Aktif indirmeler listesine ekle
        {
            let mut active_downloads = self.active_downloads.lock().await;
            active_downloads.insert(download_id.clone(), task);
        }
        
        // İndirme işlemini başlat
        self.start_download(download_id.clone()).await?;
        
        // İndirme durumunu bekle
        let download_status = self.wait_for_download(download_id.clone()).await?;
        
        Ok(download_status)
    }
    
    /// İndirme işlemini başlat
    async fn start_download(&self, download_id: String) -> Result<(), Error> {
        // İndirme işlemini al
        let task = {
            let active_downloads = self.active_downloads.lock().await;
            active_downloads.get(&download_id).cloned().ok_or_else(|| {
                Error::Download(format!("İndirme bulunamadı: {}", download_id))
            })?
        };
        
        // İndirme durumunu güncelle
        self.update_download_status(
            &download_id, 
            DownloadStatus::InProgress, 
            0.0, 
            None, 
            None
        ).await?;
        
        // İndirme işlemini asenkron olarak başlat
        let semaphore = self.download_semaphore.clone();
        let active_downloads = self.active_downloads.clone();
        let http_client = self.http_client.clone();
        let temp_dir = self.temp_dir.clone();
        let expected_hash = task.expected_hash.clone();
        
        tokio::spawn(async move {
            // Semaphore'dan izin al
            let _permit = semaphore.acquire().await.unwrap();
            
            // Geçici dosya oluştur
            let file_name = format!("{}_{}.plugin", task.plugin_id, task.version);
            let file_path = temp_dir.join(file_name);
            
            // İndirme işlemini başlat
            match Self::download_file(
                &http_client, 
                &task.url, 
                &file_path, 
                expected_hash.as_deref(),
                |progress| {
                    // İlerleme güncellemesi
                    let _ = tokio::spawn(async move {
                        let mut downloads = active_downloads.lock().await;
                        if let Some(task) = downloads.get_mut(&download_id) {
                            task.progress = progress;
                        }
                    });
                }
            ).await {
                Ok(_) => {
                    // İndirme başarılı
                    let mut downloads = active_downloads.lock().await;
                    if let Some(task) = downloads.get_mut(&download_id) {
                        task.status = DownloadStatus::Completed;
                        task.progress = 100.0;
                        task.file_path = Some(file_path);
                    }
                }
                Err(e) => {
                    // İndirme başarısız
                    let mut downloads = active_downloads.lock().await;
                    if let Some(task) = downloads.get_mut(&download_id) {
                        task.status = DownloadStatus::Failed;
                        task.error = Some(e.to_string());
                    }
                }
            }
            
            // Semaphore izni otomatik olarak bırakılır
        });
        
        Ok(())
    }
    
    /// İndirme işlemini bekle
    async fn wait_for_download(&self, download_id: String) -> Result<PluginDownloadStatus, Error> {
        // İndirme tamamlanana kadar bekle
        loop {
            // İndirme durumunu al
            let task = {
                let active_downloads = self.active_downloads.lock().await;
                active_downloads.get(&download_id).cloned().ok_or_else(|| {
                    Error::Download(format!("İndirme bulunamadı: {}", download_id))
                })?
            };
            
            // İndirme tamamlandı mı kontrol et
            match task.status {
                DownloadStatus::Completed | DownloadStatus::Failed => {
                    // İndirme durumunu oluştur
                    let status = PluginDownloadStatus {
                        id: download_id.clone(),
                        plugin_id: task.plugin_id,
                        version: task.version.to_string(),
                        status: task.status,
                        progress: task.progress,
                        file_path: task.file_path,
                        error: task.error,
                    };
                    
                    // Aktif indirmeler listesinden kaldır
                    let mut active_downloads = self.active_downloads.lock().await;
                    active_downloads.remove(&download_id);
                    
                    return Ok(status);
                }
                _ => {
                    // Biraz bekle
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// İndirme durumunu güncelle
    async fn update_download_status(
        &self,
        download_id: &str,
        status: DownloadStatus,
        progress: f32,
        file_path: Option<PathBuf>,
        error: Option<String>,
    ) -> Result<(), Error> {
        // İndirme durumunu güncelle
        let mut active_downloads = self.active_downloads.lock().await;
        if let Some(task) = active_downloads.get_mut(download_id) {
            task.status = status;
            task.progress = progress;
            if let Some(path) = file_path {
                task.file_path = Some(path);
            }
            if let Some(err) = error {
                task.error = Some(err);
            }
            Ok(())
        } else {
            Err(Error::Download(format!("İndirme bulunamadı: {}", download_id)))
        }
    }
    
    /// İndirme durumunu al
    pub async fn get_download_status(&self, download_id: &str) -> Result<PluginDownloadStatus, Error> {
        // İndirme durumunu al
        let active_downloads = self.active_downloads.lock().await;
        if let Some(task) = active_downloads.get(download_id) {
            Ok(PluginDownloadStatus {
                id: task.id.clone(),
                plugin_id: task.plugin_id.clone(),
                version: task.version.to_string(),
                status: task.status.clone(),
                progress: task.progress,
                file_path: task.file_path.clone(),
                error: task.error.clone(),
            })
        } else {
            Err(Error::Download(format!("İndirme bulunamadı: {}", download_id)))
        }
    }
    
    /// Aktif indirmeleri al
    pub async fn get_active_downloads(&self) -> Vec<PluginDownloadStatus> {
        // Aktif indirmeleri al
        let active_downloads = self.active_downloads.lock().await;
        active_downloads.values()
            .map(|task| PluginDownloadStatus {
                id: task.id.clone(),
                plugin_id: task.plugin_id.clone(),
                version: task.version.to_string(),
                status: task.status.clone(),
                progress: task.progress,
                file_path: task.file_path.clone(),
                error: task.error.clone(),
            })
            .collect()
    }
    
    /// İndirme işlemini iptal et
    pub async fn cancel_download(&self, download_id: &str) -> Result<(), Error> {
        // İndirme durumunu güncelle
        self.update_download_status(
            download_id, 
            DownloadStatus::Cancelled, 
            0.0, 
            None, 
            Some("İndirme iptal edildi".to_string())
        ).await?;
        
        // Aktif indirmeler listesinden kaldır
        let mut active_downloads = self.active_downloads.lock().await;
        active_downloads.remove(download_id);
        
        Ok(())
    }
    
    /// Dosya indir
    async fn download_file<F>(
        client: &Client,
        url: &str,
        file_path: &Path,
        expected_hash: Option<&str>,
        progress_callback: F,
    ) -> Result<(), Error>
    where
        F: Fn(f32) + Send + 'static,
    {
        // Dosya oluştur
        let mut file = File::create(file_path).map_err(|e| {
            Error::Download(format!("Geçici dosya oluşturulamadı: {}", e))
        })?;
        
        // İndirme isteği yap
        let response = client.get(url).send().await.map_err(|e| {
            Error::Download(format!("İndirme isteği başarısız: {}", e))
        })?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            return Err(Error::Download(format!(
                "İndirme hatası: HTTP {}", 
                response.status()
            )));
        }
        
        // Toplam boyutu al
        let total_size = response.content_length().unwrap_or(0);
        let mut downloaded_size = 0;
        let mut hasher = Sha256::new();
        
        // Yanıtı akış olarak al
        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = tokio::stream::StreamExt::next(&mut stream).await {
            let chunk = chunk_result.map_err(|e| {
                Error::Download(format!("Veri alınamadı: {}", e))
            })?;
            
            // Chunk'ı dosyaya yaz
            file.write_all(&chunk).map_err(|e| {
                Error::Download(format!("Dosya yazma hatası: {}", e))
            })?;
            
            // Hash hesapla
            hasher.update(&chunk);
            
            // İndirilen boyutu güncelle
            downloaded_size += chunk.len() as u64;
            
            // İlerleme yüzdesini hesapla
            if total_size > 0 {
                let progress = (downloaded_size as f32 / total_size as f32) * 100.0;
                progress_callback(progress);
            }
        }
        
        // Dosyayı kapat
        drop(file);
        
        // Hash kontrolü
        if let Some(expected) = expected_hash {
            let hash = format!("{:x}", hasher.finalize());
            if hash != expected {
                // Dosyayı sil
                let _ = fs::remove_file(file_path);
                
                return Err(Error::Download(format!(
                    "Hash doğrulama başarısız. Beklenen: {}, Alınan: {}", 
                    expected, 
                    hash
                )));
            }
        }
        
        Ok(())
    }
}
