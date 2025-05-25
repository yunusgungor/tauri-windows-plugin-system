// Tauri Windows Plugin System - WASM Güvenlik Katmanı
//
// Bu modül, WASM modüllerinin kaynaklara erişimini kontrol eden ve
// güvenlik sınırlarını uygulayan güvenlik katmanını sağlar.

use crate::wasm_types::WasmModuleMetadata;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// WASM Güvenlik hatası
#[derive(Error, Debug)]
pub enum WasmSecurityError {
    #[error("İzin hatası: {0}")]
    PermissionDenied(String),

    #[error("Geçersiz izin: {0}")]
    InvalidPermission(String),

    #[error("Erişim hatası: {0}")]
    AccessError(String),

    #[error("Konfigürasyon hatası: {0}")]
    ConfigurationError(String),

    #[error("IO hatası: {0}")]
    IoError(#[from] std::io::Error),
}

/// WASM izin tipi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WasmPermissionType {
    /// Dosya sistemi erişimi
    Filesystem,
    /// Ağ erişimi
    Network,
    /// Çevre değişkenleri erişimi
    Environment,
    /// Host API erişimi
    HostApi,
    /// Process kontrolü
    Process,
    /// UI erişimi
    UserInterface,
    /// Diğer WASM modüllerine erişim
    ModuleAccess,
}

impl WasmPermissionType {
    /// İzin tipinin açıklamasını döndürür
    pub fn description(&self) -> &'static str {
        match self {
            Self::Filesystem => "Dosya sistemi erişimi",
            Self::Network => "Ağ erişimi",
            Self::Environment => "Çevre değişkenleri erişimi",
            Self::HostApi => "Host API erişimi",
            Self::Process => "Process kontrolü",
            Self::UserInterface => "UI erişimi",
            Self::ModuleAccess => "Diğer WASM modüllerine erişim",
        }
    }
}

/// WASM izin kapsamı
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WasmPermissionScope {
    /// Tam erişim
    Full,
    /// Salt okunur
    ReadOnly,
    /// Salt yazılır
    WriteOnly,
    /// Belirli bir dizine erişim
    Path(PathBuf),
    /// Belirli bir host'a erişim
    Host(String),
    /// Belirli bir port'a erişim
    Port(u16),
    /// Belirli bir API'ye erişim
    Api(String),
    /// Belirli bir modüle erişim
    Module(String),
}

/// WASM izni
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPermission {
    /// İzin tipi
    pub permission_type: WasmPermissionType,
    /// İzin kapsamı
    pub scope: WasmPermissionScope,
    /// İzin açıklaması
    pub description: String,
    /// İzin riski (0-100)
    pub risk_level: u8,
}

impl WasmPermission {
    /// Yeni bir izin oluşturur
    pub fn new(
        permission_type: WasmPermissionType,
        scope: WasmPermissionScope,
        description: &str,
        risk_level: u8,
    ) -> Self {
        Self {
            permission_type,
            scope,
            description: description.to_string(),
            risk_level,
        }
    }
    
    /// İzin anahtarı oluşturur
    pub fn key(&self) -> String {
        format!("{:?}:{:?}", self.permission_type, self.scope)
    }
}

/// WASM izin yöneticisi
pub struct WasmSecurityManager {
    /// İzin haritası (modül ID -> izinler)
    permissions: Arc<RwLock<HashMap<String, HashSet<WasmPermission>>>>,
    /// İzin politikası
    policy: WasmSecurityPolicy,
    /// İzin istekleri geçmişi
    permission_requests: Arc<RwLock<Vec<WasmPermissionRequest>>>,
    /// Güvenli dizinler
    safe_directories: Arc<RwLock<HashMap<String, PathBuf>>>,
    /// Güvenli host'lar
    safe_hosts: Arc<RwLock<HashSet<String>>>,
    /// Güvenli API'ler
    safe_apis: Arc<RwLock<HashSet<String>>>,
}

/// WASM güvenlik politikası
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmSecurityPolicy {
    /// Her zaman sor
    AlwaysAsk,
    /// Bir kere sor
    AskOnce,
    /// Otomatik kabul et
    AutoAccept,
    /// Otomatik reddet
    AutoDeny,
    /// Güvenlik seviyesine göre
    RiskBased(u8), // 0-100 risk seviyesi eşiği
}

/// WASM izin isteği
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPermissionRequest {
    /// İstek ID'si
    pub id: String,
    /// Modül ID'si
    pub module_id: String,
    /// İstenen izin
    pub permission: WasmPermission,
    /// İstek zamanı
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// İstek durumu
    pub status: WasmPermissionRequestStatus,
    /// Yanıt
    pub response: Option<WasmPermissionResponse>,
}

/// WASM izin isteği durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmPermissionRequestStatus {
    /// Bekliyor
    Pending,
    /// Kabul edildi
    Granted,
    /// Reddedildi
    Denied,
    /// Zaman aşımı
    Timeout,
    /// İptal edildi
    Cancelled,
}

/// WASM izin yanıtı
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPermissionResponse {
    /// Yanıt zamanı
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Yanıt durumu
    pub status: WasmPermissionRequestStatus,
    /// Hatırla
    pub remember: bool,
    /// Süre (saniye)
    pub duration: Option<u64>,
}

impl WasmSecurityManager {
    /// Yeni bir güvenlik yöneticisi oluşturur
    pub fn new(policy: WasmSecurityPolicy) -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
            policy,
            permission_requests: Arc::new(RwLock::new(Vec::new())),
            safe_directories: Arc::new(RwLock::new(HashMap::new())),
            safe_hosts: Arc::new(RwLock::new(HashSet::new())),
            safe_apis: Arc::new(RwLock::new(HashSet::new())),
        }
    }
    
    /// Varsayılan güvenlik yöneticisi oluşturur
    pub fn default() -> Self {
        Self::new(WasmSecurityPolicy::AskOnce)
    }
    
    /// Güvenli bir dizin ekler
    pub async fn add_safe_directory(&self, name: &str, path: impl AsRef<Path>) -> Result<(), WasmSecurityError> {
        let path = path.as_ref().to_path_buf();
        
        // Dizinin var olup olmadığını kontrol et
        if !path.exists() || !path.is_dir() {
            return Err(WasmSecurityError::ConfigurationError(format!(
                "Geçersiz dizin: {:?}",
                path
            )));
        }
        
        let mut safe_dirs = self.safe_directories.write().await;
        safe_dirs.insert(name.to_string(), path);
        
        Ok(())
    }
    
    /// Güvenli bir host ekler
    pub async fn add_safe_host(&self, host: &str) -> Result<(), WasmSecurityError> {
        let mut safe_hosts = self.safe_hosts.write().await;
        safe_hosts.insert(host.to_string());
        
        Ok(())
    }
    
    /// Güvenli bir API ekler
    pub async fn add_safe_api(&self, api: &str) -> Result<(), WasmSecurityError> {
        let mut safe_apis = self.safe_apis.write().await;
        safe_apis.insert(api.to_string());
        
        Ok(())
    }
    
    /// Modüle izin ekler
    pub async fn add_permission(
        &self,
        module_id: &str,
        permission: WasmPermission,
    ) -> Result<(), WasmSecurityError> {
        let mut permissions = self.permissions.write().await;
        
        let module_permissions = permissions
            .entry(module_id.to_string())
            .or_insert_with(HashSet::new);
        
        module_permissions.insert(permission);
        
        Ok(())
    }
    
    /// Modülün iznini kontrol eder
    pub async fn check_permission(
        &self,
        module_id: &str,
        permission_type: WasmPermissionType,
        scope: WasmPermissionScope,
    ) -> Result<bool, WasmSecurityError> {
        let permissions = self.permissions.read().await;
        
        // Modül izinlerini al
        if let Some(module_permissions) = permissions.get(module_id) {
            // İzin anahtarını oluştur
            let perm = WasmPermission::new(permission_type.clone(), scope.clone(), "", 0);
            let key = perm.key();
            
            // İzni ara
            for p in module_permissions {
                if p.key() == key {
                    return Ok(true);
                }
                
                // Full erişim kontrolü
                if p.permission_type == permission_type && p.scope == WasmPermissionScope::Full {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// İzin ister
    pub async fn request_permission(
        &self,
        module_id: &str,
        permission: WasmPermission,
    ) -> Result<bool, WasmSecurityError> {
        // Politikayı kontrol et
        match &self.policy {
            WasmSecurityPolicy::AlwaysAsk => {
                // Her zaman sor
                self.prompt_permission(module_id, permission).await
            }
            WasmSecurityPolicy::AskOnce => {
                // Daha önce soruldu mu kontrol et
                let requests = self.permission_requests.read().await;
                let existing = requests.iter().find(|r| {
                    r.module_id == module_id && r.permission.key() == permission.key()
                });
                
                if let Some(request) = existing {
                    // Daha önce sorulmuş
                    match request.status {
                        WasmPermissionRequestStatus::Granted => Ok(true),
                        WasmPermissionRequestStatus::Denied => Ok(false),
                        _ => self.prompt_permission(module_id, permission).await,
                    }
                } else {
                    // İlk kez soruluyor
                    self.prompt_permission(module_id, permission).await
                }
            }
            WasmSecurityPolicy::AutoAccept => {
                // Otomatik kabul et
                self.grant_permission(module_id, permission).await?;
                Ok(true)
            }
            WasmSecurityPolicy::AutoDeny => {
                // Otomatik reddet
                self.deny_permission(module_id, permission).await?;
                Ok(false)
            }
            WasmSecurityPolicy::RiskBased(threshold) => {
                // Risk seviyesine göre
                if permission.risk_level <= *threshold {
                    // Düşük risk, otomatik kabul
                    self.grant_permission(module_id, permission).await?;
                    Ok(true)
                } else {
                    // Yüksek risk, sor
                    self.prompt_permission(module_id, permission).await
                }
            }
        }
    }
    
    /// İzin için kullanıcıya sor
    async fn prompt_permission(
        &self,
        module_id: &str,
        permission: WasmPermission,
    ) -> Result<bool, WasmSecurityError> {
        // İstek oluştur
        let request = WasmPermissionRequest {
            id: uuid::Uuid::new_v4().to_string(),
            module_id: module_id.to_string(),
            permission: permission.clone(),
            timestamp: chrono::Utc::now(),
            status: WasmPermissionRequestStatus::Pending,
            response: None,
        };
        
        // İsteği kaydet
        {
            let mut requests = self.permission_requests.write().await;
            requests.push(request.clone());
        }
        
        // TODO: Gerçek implementasyonda, burada kullanıcıya bir dialog gösterilecek
        // Şimdilik otomatik kabul ediyoruz
        info!(
            "İzin isteniyor: {} - {} ({})",
            module_id,
            permission.permission_type.description(),
            permission.description
        );
        
        // Otomatik kabul et
        self.grant_permission(module_id, permission).await?;
        
        // İstek durumunu güncelle
        {
            let mut requests = self.permission_requests.write().await;
            if let Some(req) = requests.iter_mut().find(|r| r.id == request.id) {
                req.status = WasmPermissionRequestStatus::Granted;
                req.response = Some(WasmPermissionResponse {
                    timestamp: chrono::Utc::now(),
                    status: WasmPermissionRequestStatus::Granted,
                    remember: true,
                    duration: None,
                });
            }
        }
        
        Ok(true)
    }
    
    /// İzni kabul et
    async fn grant_permission(
        &self,
        module_id: &str,
        permission: WasmPermission,
    ) -> Result<(), WasmSecurityError> {
        self.add_permission(module_id, permission).await
    }
    
    /// İzni reddet
    async fn deny_permission(
        &self,
        module_id: &str,
        permission: WasmPermission,
    ) -> Result<(), WasmSecurityError> {
        // Burada reddetme işlemi yapılabilir, şimdilik boş bırakıyoruz
        Ok(())
    }
    
    /// Modülün izinlerini alır
    pub async fn get_permissions(
        &self,
        module_id: &str,
    ) -> Result<Vec<WasmPermission>, WasmSecurityError> {
        let permissions = self.permissions.read().await;
        
        if let Some(module_permissions) = permissions.get(module_id) {
            Ok(module_permissions.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Modülün izin isteklerini alır
    pub async fn get_permission_requests(
        &self,
        module_id: &str,
    ) -> Result<Vec<WasmPermissionRequest>, WasmSecurityError> {
        let requests = self.permission_requests.read().await;
        
        let module_requests = requests
            .iter()
            .filter(|r| r.module_id == module_id)
            .cloned()
            .collect();
        
        Ok(module_requests)
    }
    
    /// Bir dosya yolunun güvenli olup olmadığını kontrol eder
    pub async fn is_safe_path(
        &self,
        module_id: &str,
        path: impl AsRef<Path>,
    ) -> Result<bool, WasmSecurityError> {
        let path = path.as_ref();
        
        // Modülün dosya sistemi izni var mı?
        let has_permission = self.check_permission(
            module_id,
            WasmPermissionType::Filesystem,
            WasmPermissionScope::Full,
        ).await?;
        
        if has_permission {
            return Ok(true);
        }
        
        // Güvenli dizinleri kontrol et
        let safe_dirs = self.safe_directories.read().await;
        
        for (_, safe_path) in safe_dirs.iter() {
            if path.starts_with(safe_path) {
                // Bu dizine özel izin var mı?
                let has_permission = self.check_permission(
                    module_id,
                    WasmPermissionType::Filesystem,
                    WasmPermissionScope::Path(safe_path.clone()),
                ).await?;
                
                if has_permission {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }
    
    /// Bir host'un güvenli olup olmadığını kontrol eder
    pub async fn is_safe_host(
        &self,
        module_id: &str,
        host: &str,
    ) -> Result<bool, WasmSecurityError> {
        // Modülün ağ izni var mı?
        let has_permission = self.check_permission(
            module_id,
            WasmPermissionType::Network,
            WasmPermissionScope::Full,
        ).await?;
        
        if has_permission {
            return Ok(true);
        }
        
        // Bu host'a özel izin var mı?
        let has_permission = self.check_permission(
            module_id,
            WasmPermissionType::Network,
            WasmPermissionScope::Host(host.to_string()),
        ).await?;
        
        if has_permission {
            return Ok(true);
        }
        
        // Güvenli host'ları kontrol et
        let safe_hosts = self.safe_hosts.read().await;
        
        if safe_hosts.contains(host) {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Bir API'nin güvenli olup olmadığını kontrol eder
    pub async fn is_safe_api(
        &self,
        module_id: &str,
        api: &str,
    ) -> Result<bool, WasmSecurityError> {
        // Modülün API izni var mı?
        let has_permission = self.check_permission(
            module_id,
            WasmPermissionType::HostApi,
            WasmPermissionScope::Full,
        ).await?;
        
        if has_permission {
            return Ok(true);
        }
        
        // Bu API'ye özel izin var mı?
        let has_permission = self.check_permission(
            module_id,
            WasmPermissionType::HostApi,
            WasmPermissionScope::Api(api.to_string()),
        ).await?;
        
        if has_permission {
            return Ok(true);
        }
        
        // Güvenli API'leri kontrol et
        let safe_apis = self.safe_apis.read().await;
        
        if safe_apis.contains(api) {
            return Ok(true);
        }
        
        Ok(false)
    }
}
