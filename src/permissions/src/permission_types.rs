// Tauri Windows Plugin System - İzin Türleri
//
// Bu modül, plugin'lerin talep edebileceği izin türlerini tanımlar.

use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Temel izin kategorileri
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionCategory {
    /// Dosya sistemi erişimi
    Filesystem,
    /// Ağ erişimi
    Network,
    /// Sistem bilgisi erişimi
    System,
    /// Kullanıcı arayüzü erişimi
    UI,
    /// Donanım erişimi
    Hardware,
    /// Plugin'ler arası iletişim
    Interprocess,
    /// Tauri komutlarına erişim
    Command,
}

impl fmt::Display for PermissionCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Filesystem => write!(f, "filesystem"),
            Self::Network => write!(f, "network"),
            Self::System => write!(f, "system"),
            Self::UI => write!(f, "ui"),
            Self::Hardware => write!(f, "hardware"),
            Self::Interprocess => write!(f, "interprocess"),
            Self::Command => write!(f, "command"),
        }
    }
}

/// Dosya sistemi izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct FilesystemScope: u32 {
        /// Plugin veri dizinine okuma erişimi
        const READ_PLUGIN_DATA = 0x0001;
        /// Plugin veri dizinine yazma erişimi
        const WRITE_PLUGIN_DATA = 0x0002;
        /// Plugin dışındaki özel dizinlere okuma erişimi
        const READ_EXTERNAL = 0x0004;
        /// Plugin dışındaki özel dizinlere yazma erişimi
        const WRITE_EXTERNAL = 0x0008;
        /// Uygulama dizinine okuma erişimi
        const READ_APP_DIR = 0x0010;
        /// Uygulama dizinine yazma erişimi
        const WRITE_APP_DIR = 0x0020;
        /// Herhangi bir dosyaya okuma erişimi (yüksek risk)
        const READ_ANY = 0x0040;
        /// Herhangi bir dosyaya yazma erişimi (yüksek risk)
        const WRITE_ANY = 0x0080;
        
        /// Varsayılan izinler (sadece plugin kendi veri dizinine erişebilir)
        const DEFAULT = Self::READ_PLUGIN_DATA.bits | Self::WRITE_PLUGIN_DATA.bits;
    }
}

/// Ağ izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct NetworkScope: u32 {
        /// HTTP istekleri gönderme yetkisi
        const HTTP = 0x0001;
        /// HTTPS istekleri gönderme yetkisi
        const HTTPS = 0x0002;
        /// WebSocket bağlantıları oluşturma
        const WEBSOCKET = 0x0004;
        /// TCP soketleri oluşturma
        const TCP = 0x0008;
        /// UDP soketleri oluşturma
        const UDP = 0x0010;
        /// Sunucu soketleri dinleme (gelen bağlantılar)
        const LISTEN = 0x0020;
        /// Yerel ağa erişim
        const LOCAL_NETWORK = 0x0040;
        /// Herhangi bir adrese erişim
        const ANY_HOST = 0x0080;
        
        /// Varsayılan izinler (sadece HTTPS)
        const DEFAULT = Self::HTTPS.bits;
    }
}

/// Sistem izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct SystemScope: u32 {
        /// Sistem bilgisi okuma
        const READ_INFO = 0x0001;
        /// Process bilgisi okuma
        const READ_PROCESS = 0x0002;
        /// Environment değişkenlerini okuma
        const READ_ENV = 0x0004;
        /// Hafıza kullanımını okuma
        const READ_MEMORY = 0x0008;
        /// İşlemci kullanımını okuma
        const READ_CPU = 0x0010;
        /// Disk kullanımını okuma
        const READ_DISK = 0x0020;
        /// Ağ kullanımını okuma
        const READ_NETWORK = 0x0040;
        /// Shell komutları çalıştırma (yüksek risk)
        const EXECUTE_COMMAND = 0x0080;
        
        /// Varsayılan izinler (sadece temel sistem bilgisi)
        const DEFAULT = Self::READ_INFO.bits;
    }
}

/// Kullanıcı arayüzü izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct UIScope: u32 {
        /// Ana pencere UI'ye erişim
        const MAIN_WINDOW = 0x0001;
        /// Dialog pencereleri gösterme
        const DIALOG = 0x0002;
        /// Bildirim gösterme
        const NOTIFICATION = 0x0004;
        /// Menü öğeleri ekleme
        const MENU = 0x0008;
        /// Sistem tepsisine erişim
        const TRAY = 0x0010;
        /// Yeni pencereler oluşturma
        const NEW_WINDOW = 0x0020;
        /// Global kısayol tuşları oluşturma
        const GLOBAL_SHORTCUT = 0x0040;
        /// Clipboard erişimi
        const CLIPBOARD = 0x0080;
        
        /// Varsayılan izinler (bildirimlere ve dialoglara izin verilir)
        const DEFAULT = Self::NOTIFICATION.bits | Self::DIALOG.bits;
    }
}

/// Donanım izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct HardwareScope: u32 {
        /// Kamera erişimi
        const CAMERA = 0x0001;
        /// Mikrofon erişimi
        const MICROPHONE = 0x0002;
        /// Bluetooth erişimi
        const BLUETOOTH = 0x0004;
        /// USB cihazlara erişim
        const USB = 0x0008;
        /// Seri porta erişim
        const SERIAL = 0x0010;
        /// HID cihazlara erişim
        const HID = 0x0020;
        /// Ekran yakalama
        const SCREEN_CAPTURE = 0x0040;
        /// Ses çıkışı kontrolü
        const AUDIO_OUTPUT = 0x0080;
        
        /// Varsayılan izinler (hiçbir donanım erişimi yok)
        const DEFAULT = 0;
    }
}

/// Procesler arası iletişim izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct InterprocessScope: u32 {
        /// Diğer plugin'leri keşfetme
        const DISCOVER = 0x0001;
        /// Belirli plugin'lere mesaj gönderme
        const SEND_MESSAGE = 0x0002;
        /// Herhangi bir plugin'e mesaj gönderme
        const SEND_ANY = 0x0004;
        /// Paylaşılan veri erişimi
        const SHARED_DATA = 0x0008;
        /// Event'leri dinleme
        const LISTEN_EVENTS = 0x0010;
        /// Event'leri gönderme
        const EMIT_EVENTS = 0x0020;
        /// Plugin API'lerine erişim
        const CALL_API = 0x0040;
        /// Plugin'leri kontrol etme (yükleme/kaldırma)
        const CONTROL = 0x0080;
        
        /// Varsayılan izinler (sadece keşif ve event dinleme)
        const DEFAULT = Self::DISCOVER.bits | Self::LISTEN_EVENTS.bits;
    }
}

/// Tauri komut izin kapsamları
bitflags! {
    #[derive(Serialize, Deserialize)]
    pub struct CommandScope: u32 {
        /// Dosya sistemi komutları
        const FS_COMMANDS = 0x0001;
        /// Shell komutları
        const SHELL_COMMANDS = 0x0002;
        /// HTTP komutları
        const HTTP_COMMANDS = 0x0004;
        /// Dialog komutları
        const DIALOG_COMMANDS = 0x0008;
        /// Bildirim komutları
        const NOTIFICATION_COMMANDS = 0x0010;
        /// Global kısayol komutları
        const SHORTCUT_COMMANDS = 0x0020;
        /// Clipboard komutları
        const CLIPBOARD_COMMANDS = 0x0040;
        /// Uygulama pencere komutları
        const WINDOW_COMMANDS = 0x0080;
        
        /// Varsayılan izinler (dialog ve bildirim)
        const DEFAULT = Self::DIALOG_COMMANDS.bits | Self::NOTIFICATION_COMMANDS.bits;
    }
}

/// İzin türlerinin birleşik yapısı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    /// Dosya sistemi izinleri
    pub filesystem: FilesystemScope,
    /// Ağ izinleri
    pub network: NetworkScope,
    /// Sistem izinleri
    pub system: SystemScope,
    /// Kullanıcı arayüzü izinleri
    pub ui: UIScope,
    /// Donanım izinleri
    pub hardware: HardwareScope,
    /// Procesler arası iletişim izinleri
    pub interprocess: InterprocessScope,
    /// Tauri komut izinleri
    pub command: CommandScope,
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self {
            filesystem: FilesystemScope::DEFAULT,
            network: NetworkScope::DEFAULT,
            system: SystemScope::DEFAULT,
            ui: UIScope::DEFAULT,
            hardware: HardwareScope::DEFAULT,
            interprocess: InterprocessScope::DEFAULT,
            command: CommandScope::DEFAULT,
        }
    }
}

/// Belirli bir izin talebinin açıklaması
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDescriptor {
    /// İzin kategorisi
    pub category: PermissionCategory,
    /// İzin kapsamı (bitflags olarak)
    pub scope: u32,
    /// İznin neden gerektiğine dair açıklama
    pub reason: String,
}

/// İzin durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionState {
    /// İzin verildi
    Granted,
    /// İzin reddedildi
    Denied,
    /// İzin henüz sorulmadı
    NotRequested,
    /// İzin için kullanıcı sorgulanıyor
    Requesting,
}

/// İzin isteği
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    /// İstek ID'si
    pub id: String,
    /// Plugin ID'si
    pub plugin_id: String,
    /// İstenen izinler
    pub descriptors: Vec<PermissionDescriptor>,
    /// İsteğin kullanıcıya gösterilecek başlığı
    pub title: String,
    /// İstek açıklaması
    pub description: String,
    /// Opsiyonel olarak isteğin grafik gösterimi için URL
    pub icon_url: Option<String>,
    /// İsteğin oluşturulma zamanı
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// İzin yanıtı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionResponse {
    /// İstek ID'si
    pub request_id: String,
    /// Yanıt durumu
    pub state: PermissionState,
    /// Geçerlilik süresi (saniye), None ise süresiz
    pub expires_in: Option<u64>,
    /// Yanıt zamanı
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// İzin belirteci (plugin için)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionToken {
    /// Belirteç ID'si
    pub id: String,
    /// Plugin ID'si
    pub plugin_id: String,
    /// İzin kümesi
    pub permissions: PermissionSet,
    /// Oluşturma zamanı
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Sona erme zamanı (None ise süresiz)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
