// Tauri Windows Plugin System - İzin Yöneticisi
//
// Bu modül, plugin'lerin izin taleplerini yönetir, kullanıcı izinlerini saklar
// ve izin kontrolü yapar.

use crate::permission_types::{
    PermissionCategory, PermissionDescriptor, PermissionRequest, PermissionResponse,
    PermissionSet, PermissionState, PermissionToken,
};
use crate::permission_store::PermissionStore;
use crate::prompt::{PermissionPrompt, PromptOptions, PromptResult, PromptStyle};

use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use parking_lot::RwLock;

/// İzin yöneticisi hata türleri
#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("İzin reddedildi: {0}")]
    PermissionDenied(String),

    #[error("İzin süresi dolmuş: {0}")]
    PermissionExpired(String),

    #[error("Bilinmeyen plugin: {0}")]
    UnknownPlugin(String),

    #[error("Geçersiz izin talebi: {0}")]
    InvalidRequest(String),

    #[error("Depolama hatası: {0}")]
    StorageError(String),

    #[error("Kullanıcı etkileşimi başarısız: {0}")]
    UserInteractionFailed(String),

    #[error("İstek zaman aşımına uğradı")]
    RequestTimeout,

    #[error("Sistem hatası: {0}")]
    SystemError(String),
}

/// İzin politika türleri
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionPolicy {
    /// Her zaman kullanıcıya sor
    AlwaysAsk,
    /// İlk kullanımda sor, sonra hatırla
    AskOnce,
    /// Tüm izinleri otomatik olarak kabul et (test modu)
    AutoGrant,
    /// Tüm izinleri otomatik olarak reddet
    AutoDeny,
}

/// İzin denetim düzeyi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnforcementLevel {
    /// Sıkı denetim - tüm izinler açıkça verilmelidir
    Strict,
    /// Normal denetim - bazı düşük riskli izinler otomatik verilebilir
    Normal,
    /// Gevşek denetim - yüksek riskli izinler dışında çoğu izin otomatik verilir
    Relaxed,
    /// Denetim yok - tüm izinler otomatik olarak verilir (sadece geliştirme için)
    Disabled,
}

/// İzin sağlayıcı konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionManagerConfig {
    /// İzin politikası
    pub policy: PermissionPolicy,
    /// Denetim düzeyi
    pub enforcement_level: EnforcementLevel,
    /// İzin depolama dizini
    pub storage_dir: PathBuf,
    /// Varsayılan izin belirteç süresi (saniye)
    pub default_token_duration: Option<u64>,
    /// İzin isteği zaman aşımı (saniye)
    pub request_timeout: u64,
    /// Güvenilir geliştirici kimliklerinin listesi
    pub trusted_developers: Vec<String>,
    /// İzin isteği stil seçenekleri
    pub prompt_style: PromptStyle,
}

impl Default for PermissionManagerConfig {
    fn default() -> Self {
        Self {
            policy: PermissionPolicy::AskOnce,
            enforcement_level: EnforcementLevel::Normal,
            storage_dir: PathBuf::from(".permissions"),
            default_token_duration: Some(30 * 24 * 60 * 60), // 30 gün
            request_timeout: 60,                            // 60 saniye
            trusted_developers: Vec::new(),
            prompt_style: PromptStyle::Modal,
        }
    }
}

/// Kullanıcı yanıtı için geri çağırma türü
pub type UserResponseCallback = Box<dyn Fn(PermissionResponse) + Send + Sync>;

/// İzin yöneticisi
pub struct PermissionManager {
    /// Konfigürasyon
    config: RwLock<PermissionManagerConfig>,
    /// İzin deposu
    store: Arc<PermissionStore>,
    /// İzin isteme arayüzü
    prompt: Arc<PermissionPrompt>,
    /// Aktif izin istekleri
    active_requests: RwLock<HashMap<String, (PermissionRequest, UserResponseCallback)>>,
    /// Plugin izin belirteçleri
    tokens: RwLock<HashMap<String, PermissionToken>>,
}

impl PermissionManager {
    /// Yeni bir izin yöneticisi oluştur
    pub fn new(
        config: PermissionManagerConfig,
        store: Arc<PermissionStore>,
        prompt: Arc<PermissionPrompt>,
    ) -> Self {
        Self {
            config: RwLock::new(config),
            store,
            prompt,
            active_requests: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Konfigürasyonu güncelle
    pub fn update_config(&self, config: PermissionManagerConfig) {
        let mut current_config = self.config.write();
        *current_config = config;
    }

    /// İzinleri kontrol et ve gerekirse iste
    pub async fn check_permissions(
        &self,
        plugin_id: &str,
        descriptors: Vec<PermissionDescriptor>,
    ) -> Result<PermissionToken, PermissionError> {
        // Önce mevcut belirteci kontrol et
        if let Some(token) = self.get_permission_token(plugin_id) {
            // Belirteç süresi dolmuş mu kontrol et
            if let Some(expires_at) = token.expires_at {
                if expires_at < Utc::now() {
                    debug!("Plugin {} için izin belirteci süresi dolmuş", plugin_id);
                    self.remove_permission_token(plugin_id);
                } else if self.has_all_permissions(plugin_id, &descriptors)? {
                    // İstenen tüm izinler mevcut
                    return Ok(token);
                }
            } else if self.has_all_permissions(plugin_id, &descriptors)? {
                // Süresiz belirteç ve tüm izinler mevcut
                return Ok(token);
            }
        }

        // Mevcut bir belirteç yok veya eksik izinler var
        // Politikaya göre karar ver
        let config = self.config.read();
        match config.policy {
            PermissionPolicy::AutoGrant => {
                // Otomatik izin ver
                self.grant_permissions(plugin_id, descriptors, config.default_token_duration).await
            }
            PermissionPolicy::AutoDeny => {
                // Otomatik reddet
                Err(PermissionError::PermissionDenied(format!(
                    "Politika gereği otomatik reddedildi: {}",
                    plugin_id
                )))
            }
            PermissionPolicy::AlwaysAsk | PermissionPolicy::AskOnce => {
                // Kullanıcıya sor
                self.request_permissions(plugin_id, descriptors).await
            }
        }
    }

    /// İzin talebinde bulun
    pub async fn request_permissions(
        &self,
        plugin_id: &str,
        descriptors: Vec<PermissionDescriptor>,
    ) -> Result<PermissionToken, PermissionError> {
        // Plugin bilgilerini al
        let plugin_info = self.store.get_plugin_info(plugin_id).map_err(|e| {
            PermissionError::StorageError(format!("Plugin bilgisi alınamadı: {}", e))
        })?;

        // İstek oluştur
        let request = PermissionRequest {
            id: Uuid::new_v4().to_string(),
            plugin_id: plugin_id.to_string(),
            descriptors: descriptors.clone(),
            title: format!("{} İzin İsteği", plugin_info.name),
            description: format!(
                "{} eklentisi aşağıdaki izinleri istiyor. Bu eklentiye güveniyorsanız izin verin.",
                plugin_info.name
            ),
            icon_url: plugin_info.icon_url,
            timestamp: Utc::now(),
        };

        // Prompt gösterme seçeneklerini hazırla
        let prompt_options = PromptOptions {
            style: self.config.read().prompt_style,
            timeout: Some(self.config.read().request_timeout as u32),
            can_remember: self.config.read().policy == PermissionPolicy::AskOnce,
        };

        // Kullanıcıya istek gönder
        let response = self
            .prompt
            .show_permission_prompt(&request, &prompt_options)
            .await
            .map_err(|e| {
                PermissionError::UserInteractionFailed(format!("İzin isteği gösterilemedi: {}", e))
            })?;

        // Yanıta göre işlem yap
        match response.result {
            PromptResult::Allow => {
                // İzin ver
                let duration = if response.remember {
                    self.config.read().default_token_duration
                } else {
                    // Hatırlanmasını istemiyorsa tek seferlik izin ver
                    Some(60 * 60) // 1 saat
                };

                self.grant_permissions(plugin_id, descriptors, duration).await
            }
            PromptResult::Deny => {
                // Reddet
                if response.remember {
                    // Kalıcı reddetme
                    self.store
                        .save_permission_decision(plugin_id, &descriptors, false)
                        .map_err(|e| {
                            PermissionError::StorageError(format!(
                                "İzin kararı kaydedilemedi: {}",
                                e
                            ))
                        })?;
                }

                Err(PermissionError::PermissionDenied(format!(
                    "Kullanıcı izni reddetti: {}",
                    plugin_id
                )))
            }
            PromptResult::Timeout => Err(PermissionError::RequestTimeout),
        }
    }

    /// İzin ver
    pub async fn grant_permissions(
        &self,
        plugin_id: &str,
        descriptors: Vec<PermissionDescriptor>,
        duration: Option<u64>,
    ) -> Result<PermissionToken, PermissionError> {
        // Yeni bir izin kümesi oluştur
        let mut permission_set = PermissionSet::default();

        // Tanımlayıcılardan izin kümesini güncelle
        for descriptor in &descriptors {
            match descriptor.category {
                PermissionCategory::Filesystem => {
                    permission_set.filesystem = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.filesystem.bits())
                    }
                }
                PermissionCategory::Network => {
                    permission_set.network = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.network.bits())
                    }
                }
                PermissionCategory::System => {
                    permission_set.system = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.system.bits())
                    }
                }
                PermissionCategory::UI => {
                    permission_set.ui =
                        unsafe { std::mem::transmute(descriptor.scope | permission_set.ui.bits()) }
                }
                PermissionCategory::Hardware => {
                    permission_set.hardware = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.hardware.bits())
                    }
                }
                PermissionCategory::Interprocess => {
                    permission_set.interprocess = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.interprocess.bits())
                    }
                }
                PermissionCategory::Command => {
                    permission_set.command = unsafe {
                        std::mem::transmute(descriptor.scope | permission_set.command.bits())
                    }
                }
            }
        }

        // İzinleri depoya kaydet
        self.store
            .save_permission_decision(plugin_id, &descriptors, true)
            .map_err(|e| {
                PermissionError::StorageError(format!("İzin kararı kaydedilemedi: {}", e))
            })?;

        // Sona erme zamanını hesapla
        let expires_at = duration.map(|secs| Utc::now() + Duration::seconds(secs as i64));

        // Belirteç oluştur
        let token = PermissionToken {
            id: Uuid::new_v4().to_string(),
            plugin_id: plugin_id.to_string(),
            permissions: permission_set,
            created_at: Utc::now(),
            expires_at,
        };

        // Belirteci sakla
        {
            let mut tokens = self.tokens.write();
            tokens.insert(plugin_id.to_string(), token.clone());
        }

        // Belirteci döndür
        Ok(token)
    }

    /// Belirli izinlerin var olup olmadığını kontrol et
    pub fn has_permission(
        &self,
        plugin_id: &str,
        category: PermissionCategory,
        scope: u32,
    ) -> Result<bool, PermissionError> {
        let tokens = self.tokens.read();
        
        if let Some(token) = tokens.get(plugin_id) {
            // Belirteç süresi dolmuş mu kontrol et
            if let Some(expires_at) = token.expires_at {
                if expires_at < Utc::now() {
                    return Err(PermissionError::PermissionExpired(format!(
                        "İzin belirteci süresi dolmuş: {}",
                        plugin_id
                    )));
                }
            }
            
            // İlgili kategorideki izinleri kontrol et
            let permission_bits = match category {
                PermissionCategory::Filesystem => token.permissions.filesystem.bits(),
                PermissionCategory::Network => token.permissions.network.bits(),
                PermissionCategory::System => token.permissions.system.bits(),
                PermissionCategory::UI => token.permissions.ui.bits(),
                PermissionCategory::Hardware => token.permissions.hardware.bits(),
                PermissionCategory::Interprocess => token.permissions.interprocess.bits(),
                PermissionCategory::Command => token.permissions.command.bits(),
            };
            
            // Tüm istenen bitlerin var olup olmadığını kontrol et
            Ok((permission_bits & scope) == scope)
        } else {
            Err(PermissionError::UnknownPlugin(format!(
                "Plugin için izin belirteci bulunamadı: {}",
                plugin_id
            )))
        }
    }

    /// Birden fazla izin tanımlayıcısını kontrol et
    pub fn has_all_permissions(
        &self,
        plugin_id: &str,
        descriptors: &[PermissionDescriptor],
    ) -> Result<bool, PermissionError> {
        for descriptor in descriptors {
            if !self.has_permission(plugin_id, descriptor.category, descriptor.scope)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Belirli bir plugin için izin belirtecini al
    pub fn get_permission_token(&self, plugin_id: &str) -> Option<PermissionToken> {
        let tokens = self.tokens.read();
        tokens.get(plugin_id).cloned()
    }

    /// Belirli bir plugin için izin belirtecini kaldır
    pub fn remove_permission_token(&self, plugin_id: &str) {
        let mut tokens = self.tokens.write();
        tokens.remove(plugin_id);
    }

    /// Tüm plugin'lerin izin belirteçlerini yükle
    pub fn load_all_tokens(&self) -> Result<(), PermissionError> {
        let plugin_tokens = self.store.load_all_permissions().map_err(|e| {
            PermissionError::StorageError(format!("İzin belirteçleri yüklenemedi: {}", e))
        })?;
        
        let mut tokens = self.tokens.write();
        *tokens = plugin_tokens;
        
        // Süresi dolmuş belirteçleri temizle
        tokens.retain(|_, token| {
            if let Some(expires_at) = token.expires_at {
                expires_at > Utc::now()
            } else {
                true
            }
        });
        
        Ok(())
    }

    /// Tüm plugin'lerin izin belirteçlerini kaydet
    pub fn save_all_tokens(&self) -> Result<(), PermissionError> {
        let tokens = self.tokens.read();
        self.store.save_all_permissions(&tokens).map_err(|e| {
            PermissionError::StorageError(format!("İzin belirteçleri kaydedilemedi: {}", e))
        })
    }

    /// Denetim düzeyine göre otomatik izin verilip verilmeyeceğini kontrol et
    pub fn should_auto_grant(
        &self,
        category: PermissionCategory,
        scope: u32,
    ) -> Result<bool, PermissionError> {
        let config = self.config.read();
        
        match config.enforcement_level {
            EnforcementLevel::Disabled => {
                // Denetim devre dışı, tüm izinlere otomatik izin ver
                Ok(true)
            }
            EnforcementLevel::Relaxed => {
                // Gevşek denetim, yüksek riskli izinler hariç otomatik izin ver
                match category {
                    PermissionCategory::Filesystem => {
                        // READ_ANY ve WRITE_ANY izinleri yüksek riskli
                        Ok((scope & 0x00C0) == 0)
                    }
                    PermissionCategory::Network => {
                        // ANY_HOST izni yüksek riskli
                        Ok((scope & 0x0080) == 0)
                    }
                    PermissionCategory::System => {
                        // EXECUTE_COMMAND izni yüksek riskli
                        Ok((scope & 0x0080) == 0)
                    }
                    PermissionCategory::Hardware => {
                        // Kamera, mikrofon ve ekran yakalama yüksek riskli
                        Ok((scope & 0x0043) == 0)
                    }
                    // Diğer kategoriler için normal izinler ver
                    _ => Ok(true),
                }
            }
            EnforcementLevel::Normal => {
                // Normal denetim, sadece düşük riskli izinlere otomatik izin ver
                match category {
                    PermissionCategory::UI => {
                        // Bildirim ve dialog izinleri düşük riskli
                        Ok((scope & 0x0006) == scope)
                    }
                    PermissionCategory::Network => {
                        // Sadece HTTPS izni düşük riskli
                        Ok((scope & 0x0002) == scope)
                    }
                    // Diğer kategoriler için izin istenmeli
                    _ => Ok(false),
                }
            }
            EnforcementLevel::Strict => {
                // Sıkı denetim, hiçbir izne otomatik izin verme
                Ok(false)
            }
        }
    }
}
