// Tauri Windows Plugin System - Store Types
//
// Bu modül, plugin store için gerekli veri tiplerini ve yapılarını tanımlar.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;
use uuid::Uuid;

/// Plugin türü
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// Native Windows plugin (C/C++/Rust)
    Native,
    /// WASM tabanlı plugin
    Wasm,
    /// .NET tabanlı plugin
    DotNet,
    /// Python tabanlı plugin
    Python,
    /// Node.js tabanlı plugin
    NodeJs,
    /// Karışık bileşenli plugin
    Hybrid,
}

impl PluginType {
    /// Plugin türünün açıklamasını döndürür
    pub fn description(&self) -> &'static str {
        match self {
            Self::Native => "Native Windows Plugin",
            Self::Wasm => "WebAssembly Plugin",
            Self::DotNet => ".NET Plugin",
            Self::Python => "Python Plugin",
            Self::NodeJs => "Node.js Plugin",
            Self::Hybrid => "Hybrid Plugin",
        }
    }
}

/// Plugin satın alma modeli
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PricingModel {
    /// Ücretsiz
    Free,
    /// Tek seferlik ödeme
    PaidOneTime,
    /// Abonelik
    Subscription,
    /// Freemium (bazı özellikleri ücretsiz, bazıları ücretli)
    Freemium,
    /// Deneme (belirli süre ücretsiz, sonra ücretli)
    Trial,
    /// Bağış tabanlı
    DonationWare,
}

/// Plugin kategorisi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginCategory {
    /// Araçlar
    Utility,
    /// Üretkenlik
    Productivity,
    /// Geliştirme araçları
    Development,
    /// Güvenlik
    Security,
    /// Medya
    Media,
    /// İletişim
    Communication,
    /// Oyun
    Gaming,
    /// Eğitim
    Education,
    /// Finans
    Finance,
    /// Diğer
    Other,
}

/// Plugin sürüm kanalı
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReleaseChannel {
    /// Kararlı sürüm
    Stable,
    /// Beta sürüm
    Beta,
    /// Alfa sürüm
    Alpha,
    /// Nightly build
    Nightly,
    /// Preview sürüm
    Preview,
}

/// Plugin satıcı bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVendor {
    /// Satıcı ID'si
    pub id: String,
    /// Satıcı adı
    pub name: String,
    /// Satıcı web sitesi
    pub website: Option<String>,
    /// Satıcı e-posta adresi
    pub email: Option<String>,
    /// Satıcı sosyal medya bağlantıları
    pub social_media: Option<HashMap<String, String>>,
    /// Satıcı doğrulanmış mı?
    pub verified: bool,
}

/// Plugin ikonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIcon {
    /// İkon boyutu
    pub size: PluginIconSize,
    /// İkon URL'si
    pub url: String,
    /// İkon MIME tipi
    pub mime_type: String,
}

/// Plugin ikon boyutu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginIconSize {
    /// Küçük (16x16)
    Small,
    /// Orta (32x32)
    Medium,
    /// Büyük (64x64)
    Large,
    /// Çok büyük (128x128)
    XLarge,
}

/// Plugin ekran görüntüsü
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginScreenshot {
    /// Ekran görüntüsü URL'si
    pub url: String,
    /// Ekran görüntüsü başlığı
    pub caption: Option<String>,
    /// Ekran görüntüsü sırası
    pub order: u32,
    /// Thumbnail URL'si
    pub thumbnail_url: Option<String>,
}

/// Plugin izni
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    /// İzin tipi
    pub permission_type: String,
    /// İzin kapsamı
    pub scope: String,
    /// İzin açıklaması
    pub reason: String,
    /// İzin riski (0-100)
    pub risk_level: u8,
    /// İzin zorunlu mu?
    pub required: bool,
}

/// Plugin bağımlılığı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// Bağımlılık ID'si
    pub id: String,
    /// Bağımlılık sürüm gereksinimleri
    pub version_requirement: String,
    /// Bağımlılık zorunlu mu?
    pub required: bool,
    /// Bağımlılık açıklaması
    pub description: Option<String>,
}

/// Plugin fiyatlandırma bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPricing {
    /// Fiyatlandırma modeli
    pub model: PricingModel,
    /// Fiyat (model Free değilse)
    pub price: Option<f64>,
    /// Para birimi
    pub currency: Option<String>,
    /// Deneme süresi (gün)
    pub trial_days: Option<u32>,
    /// Abonelik periyodu (ay)
    pub subscription_period: Option<u32>,
}

/// Plugin değerlendirmesi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRating {
    /// Ortalama puan (1-5)
    pub average: f32,
    /// Toplam değerlendirme sayısı
    pub count: u32,
    /// Puan dağılımı (1-5 arası puanların sayısı)
    pub distribution: HashMap<u8, u32>,
}

/// Plugin incelemesi
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    /// İnceleme ID'si
    pub id: String,
    /// Kullanıcı ID'si
    pub user_id: String,
    /// Kullanıcı adı
    pub user_name: String,
    /// Puan (1-5)
    pub rating: u8,
    /// Başlık
    pub title: Option<String>,
    /// İçerik
    pub content: String,
    /// Tarih
    pub date: DateTime<Utc>,
    /// Plugin sürümü
    pub plugin_version: Option<String>,
    /// Yararlı oy sayısı
    pub helpful_votes: u32,
    /// Geliştiricinin yanıtı
    pub developer_response: Option<PluginReviewResponse>,
}

/// Plugin incelemesine geliştirici yanıtı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReviewResponse {
    /// Yanıt içeriği
    pub content: String,
    /// Yanıt tarihi
    pub date: DateTime<Utc>,
}

/// Plugin güncelleme bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdate {
    /// Yeni sürüm
    pub version: Version,
    /// Sürüm notları
    pub release_notes: String,
    /// Yayın tarihi
    pub release_date: DateTime<Utc>,
    /// İndirme URL'si
    pub download_url: String,
    /// Dosya boyutu (byte)
    pub file_size: u64,
    /// Sürüm kanalı
    pub channel: ReleaseChannel,
    /// Otomatik güncelleme mümkün mü?
    pub auto_update_compatible: bool,
    /// Minimum uygulama sürümü
    pub min_app_version: Option<Version>,
    /// SHA-256 hash
    pub sha256: String,
    /// İmza
    pub signature: Option<String>,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin ID'si
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Plugin sürümü
    pub version: Version,
    /// Plugin satıcısı
    pub vendor: PluginVendor,
    /// Kısa açıklama
    pub short_description: String,
    /// Uzun açıklama
    pub full_description: Option<String>,
    /// Plugin kategorileri
    pub categories: Vec<PluginCategory>,
    /// Plugin etiketleri
    pub tags: Vec<String>,
    /// Plugin ikonları
    pub icons: Vec<PluginIcon>,
    /// Plugin ekran görüntüleri
    pub screenshots: Vec<PluginScreenshot>,
    /// Plugin izinleri
    pub permissions: Vec<PluginPermission>,
    /// Plugin bağımlılıkları
    pub dependencies: Vec<PluginDependency>,
    /// Plugin fiyatlandırması
    pub pricing: PluginPricing,
    /// Sürüm notları
    pub release_notes: Option<String>,
    /// Plugin türü
    pub plugin_type: PluginType,
    /// Minimum uygulama sürümü
    pub min_app_version: Version,
    /// Yayın tarihi
    pub release_date: DateTime<Utc>,
    /// Son güncelleme tarihi
    pub last_updated: DateTime<Utc>,
    /// İndirme sayısı
    pub download_count: u64,
    /// Değerlendirmeler
    pub rating: Option<PluginRating>,
    /// Plugin lisansı
    pub license: String,
    /// İndirme URL'si
    pub download_url: Option<String>,
    /// Plugin dosya boyutu (byte)
    pub file_size: Option<u64>,
    /// Demo sürümü URL'si
    pub demo_url: Option<String>,
    /// Destek URL'si
    pub support_url: Option<String>,
    /// Doküman URL'si
    pub documentation_url: Option<String>,
    /// Kaynak kodu URL'si
    pub source_code_url: Option<String>,
    /// Sürüm kanalı
    pub channel: ReleaseChannel,
    /// Öne çıkan mı?
    pub featured: bool,
    /// Doğrulanmış mı?
    pub verified: bool,
    /// SHA-256 hash
    pub sha256: Option<String>,
    /// İmza
    pub signature: Option<String>,
    /// Ek metadata
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

/// Plugin kurulum bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallation {
    /// Plugin ID'si
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Kurulu sürüm
    pub installed_version: Version,
    /// Kurulum tarihi
    pub install_date: DateTime<Utc>,
    /// Son kullanım tarihi
    pub last_used: Option<DateTime<Utc>>,
    /// Kurulum dizini
    pub install_directory: PathBuf,
    /// Güncelleme mevcut mu?
    pub update_available: bool,
    /// Güncelleme bilgileri
    pub available_update: Option<PluginUpdate>,
    /// Aktif mi?
    pub enabled: bool,
    /// Otomatik güncelleme yapılsın mı?
    pub auto_update: bool,
    /// Kullanıcı izinleri
    pub user_permissions: Vec<PluginPermission>,
    /// Kullanıcı ayarları
    pub user_settings: Option<serde_json::Value>,
    /// Lisans anahtarı
    pub license_key: Option<String>,
    /// Lisans sona erme tarihi
    pub license_expiry: Option<DateTime<Utc>>,
}

/// Plugin arama filtresi
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginSearchFilter {
    /// Arama terimi
    pub query: Option<String>,
    /// Kategoriler
    pub categories: Option<Vec<PluginCategory>>,
    /// Etiketler
    pub tags: Option<Vec<String>>,
    /// Minimum puan
    pub min_rating: Option<f32>,
    /// Fiyatlandırma modeli
    pub pricing: Option<Vec<PricingModel>>,
    /// Plugin türü
    pub plugin_type: Option<Vec<PluginType>>,
    /// Sadece doğrulanmış pluginler
    pub verified_only: Option<bool>,
    /// Sıralama tipi
    pub sort_by: Option<PluginSortType>,
    /// Sıralama yönü
    pub sort_direction: Option<SortDirection>,
    /// Sayfa numarası
    pub page: Option<u32>,
    /// Sayfa başına sonuç sayısı
    pub page_size: Option<u32>,
}

/// Plugin sıralama tipi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginSortType {
    /// Alakalılık
    Relevance,
    /// Popülerlik
    Popularity,
    /// İndirme sayısı
    Downloads,
    /// Değerlendirme
    Rating,
    /// Tarih (yeniden eskiye)
    Date,
    /// Fiyat (düşükten yükseğe)
    Price,
    /// İsim (A-Z)
    Name,
}

/// Sıralama yönü
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortDirection {
    /// Artan
    Ascending,
    /// Azalan
    Descending,
}

/// Plugin arama sonucu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchResult {
    /// Sonuçlar
    pub items: Vec<PluginMetadata>,
    /// Toplam sonuç sayısı
    pub total_count: u64,
    /// Sayfa numarası
    pub page: u32,
    /// Sayfa başına sonuç sayısı
    pub page_size: u32,
    /// Toplam sayfa sayısı
    pub total_pages: u32,
}

/// Plugin indirme bilgileri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDownloadInfo {
    /// Plugin ID'si
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Plugin sürümü
    pub version: Version,
    /// İndirme URL'si
    pub download_url: String,
    /// Dosya boyutu (byte)
    pub file_size: u64,
    /// SHA-256 hash
    pub sha256: String,
    /// İmza
    pub signature: Option<String>,
    /// MIME tipi
    pub mime_type: String,
    /// İndirme token'ı
    pub download_token: Option<String>,
    /// Token sona erme tarihi
    pub token_expires: Option<DateTime<Utc>>,
}

/// Plugin indirme durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDownloadStatus {
    /// Plugin ID'si
    pub id: String,
    /// İndirilen boyut (byte)
    pub downloaded_bytes: u64,
    /// Toplam boyut (byte)
    pub total_bytes: u64,
    /// İlerleme yüzdesi (0-100)
    pub progress: f32,
    /// İndirme hızı (byte/saniye)
    pub speed: Option<u64>,
    /// Tahmini kalan süre (saniye)
    pub estimated_time_remaining: Option<u64>,
    /// İndirme durumu
    pub status: DownloadStatus,
    /// Hata mesajı (varsa)
    pub error: Option<String>,
    /// İndirilen dosya yolu
    pub file_path: Option<PathBuf>,
}

/// İndirme durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DownloadStatus {
    /// Bekliyor
    Pending,
    /// İndiriliyor
    Downloading,
    /// Doğrulanıyor
    Verifying,
    /// Tamamlandı
    Completed,
    /// Durduruldu
    Paused,
    /// İptal edildi
    Cancelled,
    /// Başarısız
    Failed,
}

/// Plugin kurulum durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInstallStatus {
    /// Plugin ID'si
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Kurulum aşaması
    pub stage: InstallStage,
    /// İlerleme yüzdesi (0-100)
    pub progress: f32,
    /// Kurulum dizini
    pub install_directory: Option<PathBuf>,
    /// Hata mesajı (varsa)
    pub error: Option<String>,
    /// Başarılı mı?
    pub success: bool,
}

/// Kurulum aşaması
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstallStage {
    /// İndiriliyor
    Downloading,
    /// Doğrulanıyor
    Verifying,
    /// Çıkartılıyor
    Extracting,
    /// Bağımlılıklar kuruluyor
    InstallingDependencies,
    /// Yapılandırılıyor
    Configuring,
    /// Kurulum tamamlanıyor
    Finalizing,
    /// Tamamlandı
    Completed,
    /// Başarısız
    Failed,
}

/// Plugin güncelleme durumu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdateStatus {
    /// Plugin ID'si
    pub id: String,
    /// Plugin adı
    pub name: String,
    /// Mevcut sürüm
    pub current_version: Version,
    /// Yeni sürüm
    pub new_version: Version,
    /// Güncelleme aşaması
    pub stage: UpdateStage,
    /// İlerleme yüzdesi (0-100)
    pub progress: f32,
    /// Hata mesajı (varsa)
    pub error: Option<String>,
    /// Başarılı mı?
    pub success: bool,
}

/// Güncelleme aşaması
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateStage {
    /// İndiriliyor
    Downloading,
    /// Doğrulanıyor
    Verifying,
    /// Eski sürüm yedekleniyor
    BackingUp,
    /// Plugin durduruluyor
    Stopping,
    /// Çıkartılıyor
    Extracting,
    /// Güncelleniyor
    Updating,
    /// Plugin başlatılıyor
    Starting,
    /// Tamamlandı
    Completed,
    /// Geri alınıyor
    RollingBack,
    /// Başarısız
    Failed,
}

/// Plugin store hatası
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("API hatası: {0}")]
    ApiError(String),

    #[error("Ağ hatası: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serileştirme hatası: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Plugin bulunamadı: {0}")]
    PluginNotFound(String),

    #[error("İndirme hatası: {0}")]
    DownloadError(String),

    #[error("Doğrulama hatası: {0}")]
    VerificationError(String),

    #[error("Kurulum hatası: {0}")]
    InstallationError(String),

    #[error("Güncelleme hatası: {0}")]
    UpdateError(String),

    #[error("İzin hatası: {0}")]
    PermissionError(String),

    #[error("Kimlik doğrulama hatası: {0}")]
    AuthenticationError(String),

    #[error("Lisans hatası: {0}")]
    LicensingError(String),

    #[error("IO hatası: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Yapılandırma hatası: {0}")]
    ConfigurationError(String),

    #[error("İmza hatası: {0}")]
    SignatureError(String),

    #[error("Bağımlılık hatası: {0}")]
    DependencyError(String),
}
