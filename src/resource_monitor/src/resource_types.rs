// Tauri Windows Plugin System - Kaynak Tipleri
//
// Bu modül, plugin kaynak kullanımını izlemek için kullanılan veri tiplerini tanımlar.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Kaynak ölçüm birimi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceUnit {
    /// Byte
    Bytes,
    /// Kilobyte
    KB,
    /// Megabyte
    MB,
    /// Gigabyte
    GB,
    /// Yüzde (0-100)
    Percent,
    /// İşlem sayısı
    Count,
    /// Saniyede işlem sayısı
    CountPerSecond,
    /// Saniyede byte
    BytesPerSecond,
    /// Saniyede kilobyte
    KBPerSecond,
    /// Saniyede megabyte
    MBPerSecond,
    /// Milisaniye
    Milliseconds,
}

/// Kaynak tipi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// CPU kullanımı
    CpuUsage,
    /// Bellek kullanımı
    MemoryUsage,
    /// Aktif process sayısı
    ProcessCount,
    /// Disk okuma
    DiskRead,
    /// Disk yazma
    DiskWrite,
    /// Ağ indirme
    NetworkDownload,
    /// Ağ yükleme
    NetworkUpload,
    /// Disk alanı kullanımı
    DiskSpace,
    /// Thread sayısı
    ThreadCount,
    /// Handle sayısı
    HandleCount,
    /// GDI obje sayısı
    GdiObjectCount,
    /// Sayfa hatası
    PageFaults,
    /// Sistem çağrısı sayısı
    SystemCalls,
}

impl ResourceType {
    /// Kaynak tipi için ölçüm birimini al
    pub fn unit(&self) -> ResourceUnit {
        match self {
            Self::CpuUsage => ResourceUnit::Percent,
            Self::MemoryUsage => ResourceUnit::MB,
            Self::ProcessCount => ResourceUnit::Count,
            Self::DiskRead => ResourceUnit::KBPerSecond,
            Self::DiskWrite => ResourceUnit::KBPerSecond,
            Self::NetworkDownload => ResourceUnit::KBPerSecond,
            Self::NetworkUpload => ResourceUnit::KBPerSecond,
            Self::DiskSpace => ResourceUnit::MB,
            Self::ThreadCount => ResourceUnit::Count,
            Self::HandleCount => ResourceUnit::Count,
            Self::GdiObjectCount => ResourceUnit::Count,
            Self::PageFaults => ResourceUnit::CountPerSecond,
            Self::SystemCalls => ResourceUnit::CountPerSecond,
        }
    }
    
    /// Kaynak tipi için açıklama
    pub fn description(&self) -> &'static str {
        match self {
            Self::CpuUsage => "CPU Kullanımı",
            Self::MemoryUsage => "Bellek Kullanımı",
            Self::ProcessCount => "Process Sayısı",
            Self::DiskRead => "Disk Okuma Hızı",
            Self::DiskWrite => "Disk Yazma Hızı",
            Self::NetworkDownload => "Ağ İndirme Hızı",
            Self::NetworkUpload => "Ağ Yükleme Hızı",
            Self::DiskSpace => "Disk Alanı Kullanımı",
            Self::ThreadCount => "Thread Sayısı",
            Self::HandleCount => "Handle Sayısı",
            Self::GdiObjectCount => "GDI Nesne Sayısı",
            Self::PageFaults => "Sayfa Hatası Oranı",
            Self::SystemCalls => "Sistem Çağrısı Oranı",
        }
    }
}

/// Kaynak limiti konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimit {
    /// Kaynak tipi
    pub resource_type: ResourceType,
    /// Yumuşak limit
    pub soft_limit: f64,
    /// Sert limit
    pub hard_limit: f64,
    /// Ölçüm periyodu (saniye)
    pub measurement_period: u32,
    /// Limit aşıldığında eylem
    pub action: LimitAction,
}

/// Limit aşıldığında yapılacak eylem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LimitAction {
    /// Uyarı gönder
    Warn,
    /// Kaynağı kısıtla
    Throttle,
    /// Plugin'i durdur
    Suspend,
    /// Plugin'i sonlandır
    Terminate,
}

/// Kaynak ölçümü
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMeasurement {
    /// Ölçüm ID'si
    pub id: String,
    /// Plugin ID'si
    pub plugin_id: String,
    /// Process ID'si
    pub process_id: u32,
    /// Kaynak tipi
    pub resource_type: ResourceType,
    /// Ölçüm değeri
    pub value: f64,
    /// Ölçüm zamanı
    pub timestamp: DateTime<Utc>,
}

/// Plugin için kaynak kullanım profili
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageProfile {
    /// Plugin ID'si
    pub plugin_id: String,
    /// Process ID'si
    pub process_id: u32,
    /// Son ölçümler (ResourceType -> Ölçüm)
    pub current_measurements: HashMap<ResourceType, ResourceMeasurement>,
    /// Maksimum ölçümler (ResourceType -> Ölçüm)
    pub peak_measurements: HashMap<ResourceType, ResourceMeasurement>,
    /// Toplam kaynak kullanımı (ResourceType -> Toplam değer)
    pub total_usage: HashMap<ResourceType, f64>,
    /// İzleme başlangıç zamanı
    pub monitoring_start: DateTime<Utc>,
    /// Son güncelleme zamanı
    pub last_updated: DateTime<Utc>,
}

impl ResourceUsageProfile {
    /// Yeni bir kaynak kullanım profili oluştur
    pub fn new(plugin_id: String, process_id: u32) -> Self {
        let now = Utc::now();
        Self {
            plugin_id,
            process_id,
            current_measurements: HashMap::new(),
            peak_measurements: HashMap::new(),
            total_usage: HashMap::new(),
            monitoring_start: now,
            last_updated: now,
        }
    }
    
    /// Yeni ölçüm ekle
    pub fn add_measurement(&mut self, measurement: ResourceMeasurement) {
        let resource_type = measurement.resource_type;
        
        // Mevcut ölçüm olarak ekle
        self.current_measurements.insert(resource_type, measurement.clone());
        
        // Zirve ölçümü güncelle
        if let Some(peak) = self.peak_measurements.get(&resource_type) {
            if measurement.value > peak.value {
                self.peak_measurements.insert(resource_type, measurement.clone());
            }
        } else {
            self.peak_measurements.insert(resource_type, measurement.clone());
        }
        
        // Toplam kullanımı güncelle
        let total = self.total_usage.entry(resource_type).or_insert(0.0);
        *total += measurement.value;
        
        // Son güncelleme zamanını ayarla
        self.last_updated = measurement.timestamp;
    }
    
    /// Belirli bir kaynak için mevcut kullanımı al
    pub fn current_usage(&self, resource_type: ResourceType) -> Option<f64> {
        self.current_measurements.get(&resource_type).map(|m| m.value)
    }
    
    /// Belirli bir kaynak için zirve kullanımı al
    pub fn peak_usage(&self, resource_type: ResourceType) -> Option<f64> {
        self.peak_measurements.get(&resource_type).map(|m| m.value)
    }
    
    /// Belirli bir kaynak için ortalama kullanımı hesapla
    pub fn average_usage(&self, resource_type: ResourceType) -> Option<f64> {
        if let Some(total) = self.total_usage.get(&resource_type) {
            let duration = (self.last_updated - self.monitoring_start).num_seconds();
            if duration > 0 {
                Some(*total / duration as f64)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Kaynak limit aşımı olayı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitEvent {
    /// Olay ID'si
    pub id: String,
    /// Plugin ID'si
    pub plugin_id: String,
    /// Process ID'si
    pub process_id: u32,
    /// Aşılan limit tipi
    pub resource_type: ResourceType,
    /// Limit değeri
    pub limit: f64,
    /// Gerçek değer
    pub actual_value: f64,
    /// Aşım yüzdesi
    pub overage_percent: f64,
    /// Uygulanan eylem
    pub action_taken: LimitAction,
    /// Olay zamanı
    pub timestamp: DateTime<Utc>,
}

/// Kaynak izleme konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMonitorConfig {
    /// İzleme aralığı (milisaniye)
    pub monitoring_interval_ms: u32,
    /// İzlenecek kaynaklar
    pub resources_to_monitor: Vec<ResourceType>,
    /// Kaynak limitleri
    pub resource_limits: Vec<ResourceLimit>,
    /// Otomatik izleme etkin mi?
    pub auto_monitoring: bool,
    /// Tarihsel veri saklama süresi (gün)
    pub history_retention_days: u32,
    /// Limit aşımında bildirim gönderme
    pub notify_on_limit_breach: bool,
    /// İstatistik toplama etkin mi?
    pub gather_statistics: bool,
}
