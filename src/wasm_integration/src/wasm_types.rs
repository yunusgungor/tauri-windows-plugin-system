// Tauri Windows Plugin System - WASM Tipleri
//
// Bu modül, WASM entegrasyonu için temel veri tiplerini ve yapıları tanımlar.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// WASM modül tipleri
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmModuleType {
    /// Standart WASM modülü (wasm-core)
    Standard,
    /// WASI destekli modül (wasi-preview1)
    Wasi,
    /// Component model modülü (component-model)
    Component,
    /// Özel modül (tauri-plugin-specific)
    Custom,
}

impl WasmModuleType {
    /// Modül tipinin açıklamasını döndürür
    pub fn description(&self) -> &'static str {
        match self {
            Self::Standard => "Standard WebAssembly Module",
            Self::Wasi => "WASI-enabled WebAssembly Module",
            Self::Component => "WebAssembly Component Model",
            Self::Custom => "Custom Tauri Plugin Module",
        }
    }

    /// Dosya uzantısına göre modül tipini tahmin eder
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wasm" => Some(Self::Standard),
            "wasmw" => Some(Self::Wasi),
            "wasmc" => Some(Self::Component),
            "wasmp" => Some(Self::Custom),
            _ => None,
        }
    }

    /// Modül içeriğine göre modül tipini tespit eder
    pub fn detect_from_bytes(bytes: &[u8]) -> Option<Self> {
        // Wasm magic number: 0x00, 0x61, 0x73, 0x6D
        if bytes.len() < 8 || bytes[0..4] != [0x00, 0x61, 0x73, 0x6D] {
            return None;
        }

        // İleride daha gelişmiş bir tür tespiti yapılabilir
        // Şimdilik sadece basic wasm kontrolü yapılıyor
        Some(Self::Standard)
    }
}

/// WASM modül metadatası
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleMetadata {
    /// Modül ID'si
    pub id: String,
    /// Modül adı
    pub name: String,
    /// Modül versiyonu
    pub version: String,
    /// Modül açıklaması
    pub description: Option<String>,
    /// Modül yazarı
    pub author: Option<String>,
    /// Modül lisansı
    pub license: Option<String>,
    /// Modül tipi
    pub module_type: WasmModuleType,
    /// Modül özellikleri
    pub features: Vec<String>,
    /// Modül izinleri
    pub permissions: Vec<String>,
    /// İçe aktarılan fonksiyonlar
    pub imports: Vec<String>,
    /// Dışa aktarılan fonksiyonlar
    pub exports: Vec<String>,
    /// Ek metadata
    pub extra: HashMap<String, String>,
}

impl WasmModuleMetadata {
    /// Yeni bir modül metadata'sı oluşturur
    pub fn new(id: &str, name: &str, version: &str, module_type: WasmModuleType) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            description: None,
            author: None,
            license: None,
            module_type,
            features: Vec::new(),
            permissions: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            extra: HashMap::new(),
        }
    }

    /// Metadata'ya bir özellik ekler
    pub fn add_feature(&mut self, feature: &str) -> &mut Self {
        self.features.push(feature.to_string());
        self
    }

    /// Metadata'ya bir izin ekler
    pub fn add_permission(&mut self, permission: &str) -> &mut Self {
        self.permissions.push(permission.to_string());
        self
    }

    /// Metadata'ya bir import ekler
    pub fn add_import(&mut self, import: &str) -> &mut Self {
        self.imports.push(import.to_string());
        self
    }

    /// Metadata'ya bir export ekler
    pub fn add_export(&mut self, export: &str) -> &mut Self {
        self.exports.push(export.to_string());
        self
    }

    /// Metadata'ya ekstra bir alan ekler
    pub fn add_extra(&mut self, key: &str, value: &str) -> &mut Self {
        self.extra.insert(key.to_string(), value.to_string());
        self
    }
}

/// WASM modül konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleConfig {
    /// Bellek limiti (byte)
    pub memory_limit: Option<u64>,
    /// Çalışma süresi limiti
    pub execution_time_limit: Option<Duration>,
    /// Tablo boyutu limiti
    pub table_size_limit: Option<u32>,
    /// İzin verilen izinler
    pub allowed_permissions: Vec<String>,
    /// WASI özellikleri
    pub wasi_features: Option<WasiFeatures>,
    /// Host fonksiyonlarına erişim
    pub host_functions: Vec<String>,
    /// Ön yükleme yapılacak modüller
    pub preload_modules: Vec<String>,
    /// JIT optimizasyon seviyesi
    pub optimization_level: OptimizationLevel,
    /// Debug bilgisi
    pub debug_info: bool,
    /// Fuel tüketim limiti
    pub fuel_limit: Option<u64>,
}

impl Default for WasmModuleConfig {
    fn default() -> Self {
        Self {
            memory_limit: Some(100 * 1024 * 1024), // 100 MB
            execution_time_limit: Some(Duration::from_secs(30)),
            table_size_limit: Some(10000),
            allowed_permissions: Vec::new(),
            wasi_features: Some(WasiFeatures::default()),
            host_functions: Vec::new(),
            preload_modules: Vec::new(),
            optimization_level: OptimizationLevel::Speed,
            debug_info: false,
            fuel_limit: Some(10_000_000), // 10M birim
        }
    }
}

/// WASM optimizasyon seviyesi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// Optimizasyon yok
    None,
    /// Hız optimizasyonu
    Speed,
    /// Boyut optimizasyonu
    Size,
    /// Hız ve boyut optimizasyonu
    SpeedAndSize,
}

/// WASI özellikleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasiFeatures {
    /// Dosya sistemi erişimi
    pub filesystem: bool,
    /// Ağ erişimi
    pub network: bool,
    /// Çevre değişkenleri erişimi
    pub env_vars: bool,
    /// Saat erişimi
    pub clock: bool,
    /// Rastgele sayı üreteci erişimi
    pub random: bool,
    /// Process manipülasyonu
    pub process: bool,
    /// Argüman erişimi
    pub args: bool,
}

impl Default for WasiFeatures {
    fn default() -> Self {
        Self {
            filesystem: false,
            network: false,
            env_vars: false,
            clock: true,
            random: true,
            process: false,
            args: false,
        }
    }
}

/// WASM modül durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmModuleState {
    /// Yükleniyor
    Loading,
    /// Derleniyor
    Compiling,
    /// Hazır
    Ready,
    /// Çalışıyor
    Running,
    /// Duraklatıldı
    Paused,
    /// Sonlandırıldı
    Terminated,
    /// Hata oluştu
    Error,
}

/// WASM modül yükleme hatası
#[derive(Debug, thiserror::Error)]
pub enum WasmLoadError {
    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Geçersiz WASM modülü: {0}")]
    InvalidModule(String),

    #[error("Desteklenmeyen WASM modül tipi: {0}")]
    UnsupportedModuleType(String),

    #[error("WASM derleme hatası: {0}")]
    CompilationError(String),

    #[error("WASM bağlama hatası: {0}")]
    LinkError(String),

    #[error("Eksik içe aktarma: {0}")]
    MissingImport(String),

    #[error("WASM bellek hatası: {0}")]
    MemoryError(String),

    #[error("İzin hatası: {0}")]
    PermissionDenied(String),

    #[error("Sınırlama hatası: {0}")]
    LimitExceeded(String),
}

/// WASM çalıştırma hatası
#[derive(Debug, thiserror::Error)]
pub enum WasmRuntimeError {
    #[error("WASM hata: {0}")]
    Trap(String),

    #[error("Bellek erişim hatası: {0}")]
    MemoryAccessError(String),

    #[error("Yürütme zaman aşımı")]
    ExecutionTimeout,

    #[error("Bellek limiti aşıldı")]
    MemoryLimitExceeded,

    #[error("Fuel tüketimi aşıldı")]
    FuelExhausted,

    #[error("Host fonksiyon hatası: {0}")]
    HostFunctionError(String),

    #[error("İzin hatası: {0}")]
    PermissionDenied(String),

    #[error("İç hata: {0}")]
    InternalError(String),
}

/// WASM modül istatistikleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleStats {
    /// Başlangıç zamanı
    pub start_time: std::time::SystemTime,
    /// Toplam çalışma süresi
    pub total_execution_time: Duration,
    /// Toplam bellek kullanımı
    pub peak_memory_usage: u64,
    /// Toplam tablo boyutu
    pub table_size: u32,
    /// Toplam çağrı sayısı
    pub call_count: u64,
    /// Tüketilen fuel
    pub fuel_consumed: u64,
    /// Başarılı çağrı sayısı
    pub successful_calls: u64,
    /// Başarısız çağrı sayısı
    pub failed_calls: u64,
}

impl WasmModuleStats {
    /// Yeni modül istatistikleri oluşturur
    pub fn new() -> Self {
        Self {
            start_time: std::time::SystemTime::now(),
            total_execution_time: Duration::from_secs(0),
            peak_memory_usage: 0,
            table_size: 0,
            call_count: 0,
            fuel_consumed: 0,
            successful_calls: 0,
            failed_calls: 0,
        }
    }
}

/// WASM fonksiyon imzası
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmFunctionSignature {
    /// Fonksiyon adı
    pub name: String,
    /// Parametre tipleri
    pub parameters: Vec<WasmValueType>,
    /// Dönüş tipleri
    pub results: Vec<WasmValueType>,
}

/// WASM değer tipi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmValueType {
    /// 32-bit tamsayı
    I32,
    /// 64-bit tamsayı
    I64,
    /// 32-bit kayan nokta
    F32,
    /// 64-bit kayan nokta
    F64,
    /// 128-bit vektör
    V128,
    /// Fonksiyon referansı
    FuncRef,
    /// Dış referans
    ExternRef,
}

/// WASM modül özeti
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmModuleSummary {
    /// Modül ID'si
    pub id: String,
    /// Modül adı
    pub name: String,
    /// Modül versiyonu
    pub version: String,
    /// Modül tipi
    pub module_type: WasmModuleType,
    /// Modül durumu
    pub state: WasmModuleState,
    /// Dışa aktarılan fonksiyonlar
    pub exports: Vec<String>,
    /// Bellek kullanımı
    pub memory_usage: u64,
    /// Çalışma süresi
    pub execution_time: Duration,
}

/// WASM modül yükleme seçenekleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmLoadOptions {
    /// Modül adı
    pub name: Option<String>,
    /// Modül ID'si
    pub id: Option<String>,
    /// Modül konfigürasyonu
    pub config: Option<WasmModuleConfig>,
    /// Metadata yükleme
    pub load_metadata: bool,
    /// Otomatik başlatma
    pub auto_start: bool,
    /// Debug modu
    pub debug_mode: bool,
}

impl Default for WasmLoadOptions {
    fn default() -> Self {
        Self {
            name: None,
            id: None,
            config: None,
            load_metadata: true,
            auto_start: false,
            debug_mode: false,
        }
    }
}
