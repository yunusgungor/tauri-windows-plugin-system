// Tauri Windows Plugin System - Plugin Interface
//
// Bu modül, test plugin'leri için standart bir arayüz tanımlar.
// Tüm plugin'ler bu arayüzü uygulamalıdır.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Plugin hata tipi
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Genel plugin hatası: {0}")]
    General(String),
    
    #[error("İzin hatası: {0}")]
    Permission(String),
    
    #[error("Kaynak hatası: {0}")]
    Resource(String),
    
    #[error("API hatası: {0}")]
    Api(String),
    
    #[error("Serileştirme hatası: {0}")]
    Serialization(String),
}

/// Plugin tipi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginType {
    /// Native Windows plugin
    Native,
    /// WebAssembly plugin
    Wasm,
    /// .NET plugin
    DotNet,
    /// Hybrid plugin
    Hybrid,
}

impl fmt::Display for PluginType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginType::Native => write!(f, "Native"),
            PluginType::Wasm => write!(f, "WebAssembly"),
            PluginType::DotNet => write!(f, ".NET"),
            PluginType::Hybrid => write!(f, "Hybrid"),
        }
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Plugin versiyonu
    pub version: String,
    /// Plugin açıklaması
    pub description: String,
    /// Plugin tipi
    pub plugin_type: PluginType,
    /// Plugin sağlayıcısı
    pub vendor: String,
    /// Plugin sağlayıcı web sitesi
    pub vendor_url: Option<String>,
    /// Plugin izinleri
    pub permissions: Vec<String>,
    /// Minimum host versiyonu
    pub min_host_version: Option<String>,
}

/// Plugin arayüzü
///
/// Bu trait, tüm plugin'lerin uygulaması gereken arayüzü tanımlar.
/// Her plugin, benzersiz bir ID, metadata ve temel işlevler sağlamalıdır.
pub trait PluginInterface {
    /// Plugin ID'sini döndürür
    fn get_id(&self) -> &str;
    
    /// Plugin metadata'sını döndürür
    fn get_metadata(&self) -> PluginMetadata;
    
    /// Plugin'i başlatır
    fn initialize(&mut self) -> Result<(), PluginError>;
    
    /// Plugin'i durdurur
    fn shutdown(&mut self) -> Result<(), PluginError>;
    
    /// Plugin'e bir komut gönderir ve yanıtı alır
    fn execute_command(&mut self, command: &str, args: &str) -> Result<String, PluginError>;
}

/// Resource kullanımı durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU kullanımı yüzdesi (0-100)
    pub cpu_percent: f64,
    /// Bellek kullanımı (bytes)
    pub memory_bytes: u64,
    /// Disk I/O (bytes/sec)
    pub disk_bytes_per_sec: u64,
    /// Ağ I/O (bytes/sec)
    pub network_bytes_per_sec: u64,
}

/// Resource kullanımı ölçen plugin arayüzü
///
/// Bu trait, kaynak kullanımı izleme özelliği sağlayan plugin'ler tarafından uygulanır.
pub trait ResourceMonitorPlugin: PluginInterface {
    /// Mevcut kaynak kullanımını ölçer
    fn measure_resource_usage(&self) -> Result<ResourceUsage, PluginError>;
    
    /// Kaynak sınırlarını ayarlar
    fn set_resource_limits(&mut self, cpu_percent: Option<f64>, memory_bytes: Option<u64>) -> Result<(), PluginError>;
    
    /// Kaynak izlemeyi başlatır
    fn start_monitoring(&mut self) -> Result<(), PluginError>;
    
    /// Kaynak izlemeyi durdurur
    fn stop_monitoring(&mut self) -> Result<(), PluginError>;
}

/// Plugin konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Genel ayarlar
    pub general: HashMap<String, String>,
    /// Özel ayarlar
    pub custom: serde_json::Value,
}

/// Plugin süreci
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginProcess {
    /// Süreç ID'si
    pub process_id: u32,
    /// Süreç başlangıç zamanı
    pub start_time: u64,
    /// Kaynak kullanımı
    pub resource_usage: Option<ResourceUsage>,
}

/// Plugin durum bilgisi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Başlatılıyor
    Starting,
    /// Çalışıyor
    Running,
    /// Duraklatıldı
    Paused,
    /// Durduruldu
    Stopped,
    /// Hata oluştu
    Error(String),
}

/// Plugin durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    /// Plugin ID'si
    pub id: String,
    /// Mevcut durum
    pub status: PluginStatus,
    /// Çalışma süresi (saniye)
    pub uptime: u64,
    /// Süreç bilgisi
    pub process: Option<PluginProcess>,
    /// Son hata
    pub last_error: Option<String>,
}

// Gerekli import'ları ekliyoruz
use std::collections::HashMap;
