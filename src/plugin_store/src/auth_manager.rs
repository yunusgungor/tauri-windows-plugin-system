// Tauri Windows Plugin System - Auth Manager
//
// Bu modül, plugin mağazası ile kimlik doğrulama işlemlerini yönetir.
// API token'larını saklar, yeniler ve doğrular.

use crate::api::ApiClient;
use crate::store_types::StoreError;
use crate::Error;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// Token bilgisi
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenInfo {
    /// API token'ı
    token: String,
    /// Yenileme token'ı
    refresh_token: Option<String>,
    /// Geçerlilik süresi (saniye)
    expires_in: Option<u64>,
    /// Oluşturulma zamanı (Unix timestamp)
    created_at: u64,
}

/// Kimlik doğrulama yöneticisi
pub struct AuthManager {
    /// API istemcisi
    api_client: Arc<ApiClient>,
    /// API anahtarı
    api_key: Option<String>,
    /// Token bilgisi
    token_info: Arc<Mutex<Option<TokenInfo>>>,
    /// Kullanıcı belirteci
    user_token: Option<String>,
}

impl AuthManager {
    /// Yeni bir kimlik doğrulama yöneticisi oluştur
    pub fn new(
        api_client: Arc<ApiClient>,
        api_key: Option<String>,
        user_token: Option<String>,
    ) -> Result<Self, Error> {
        let token_info = if let Some(token) = &user_token {
            Some(TokenInfo {
                token: token.clone(),
                refresh_token: None,
                expires_in: None, // Süresi bilinmiyor, ilk kullanımda yenilenecek
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            })
        } else {
            None
        };

        Ok(Self {
            api_client,
            api_key,
            token_info: Arc::new(Mutex::new(token_info)),
            user_token,
        })
    }

    /// API token'ını al
    pub async fn get_api_token(&self) -> Result<String, Error> {
        // Token bilgisini al
        let mut token_info = self.token_info.lock().await;

        // Token var mı kontrol et
        if let Some(info) = token_info.as_ref() {
            // Token'ın süresi dolmuş mu kontrol et
            if let Some(expires_in) = info.expires_in {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let elapsed = now - info.created_at;

                // Token süresi dolmuşsa yenile
                if elapsed >= expires_in {
                    // Yenileme token'ı varsa yenile
                    if let Some(refresh_token) = &info.refresh_token {
                        debug!("Token süresi dolmuş, yenileniyor...");
                        match self.api_client.refresh_token(refresh_token).await {
                            Ok(new_token) => {
                                // Token'ı güncelle
                                *token_info = Some(TokenInfo {
                                    token: new_token,
                                    refresh_token: Some(refresh_token.clone()),
                                    expires_in: Some(3600), // Varsayılan 1 saat
                                    created_at: SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .map(|d| d.as_secs())
                                        .unwrap_or(0),
                                });
                                debug!("Token başarıyla yenilendi.");
                            }
                            Err(e) => {
                                warn!("Token yenileme başarısız: {:?}", e);
                                // Yenileme başarısız, token'ı sıfırla
                                *token_info = None;
                            }
                        }
                    } else {
                        // Yenileme token'ı yok, token'ı sıfırla
                        debug!("Token süresi dolmuş ve yenileme token'ı yok, token sıfırlanıyor...");
                        *token_info = None;
                    }
                }
            }
        }

        // Token hala yok mu kontrol et
        if token_info.is_none() {
            // API anahtarı kullan
            if let Some(api_key) = &self.api_key {
                debug!("API anahtarı kullanılıyor...");
                // API anahtarını doğrula
                match self.api_client.verify_api_key(api_key).await {
                    Ok(true) => {
                        debug!("API anahtarı doğrulandı.");
                        // API anahtarını token olarak kullan
                        *token_info = Some(TokenInfo {
                            token: api_key.clone(),
                            refresh_token: None,
                            expires_in: None, // API anahtarının süresi yok
                            created_at: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0),
                        });
                    }
                    Ok(false) => {
                        return Err(Error::Authentication("API anahtarı geçersiz".to_string()));
                    }
                    Err(e) => {
                        return Err(Error::Authentication(format!(
                            "API anahtarı doğrulanamadı: {:?}",
                            e
                        )));
                    }
                }
            } else {
                // Kimlik doğrulama gerekiyor
                return Err(Error::Authentication(
                    "Kimlik doğrulama gerekiyor. Lütfen giriş yapın veya API anahtarı sağlayın."
                        .to_string(),
                ));
            }
        }

        // Token'ı döndür
        if let Some(info) = token_info.as_ref() {
            Ok(info.token.clone())
        } else {
            Err(Error::Authentication(
                "Kimlik doğrulama bilgisi alınamadı".to_string(),
            ))
        }
    }

    /// Kullanıcı girişi yap
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        // Kimlik doğrulama yap
        match self.api_client.authenticate(username, password).await {
            Ok(token) => {
                // Token'ı kaydet
                let mut token_info = self.token_info.lock().await;
                *token_info = Some(TokenInfo {
                    token: token.clone(),
                    refresh_token: Some(token.clone()), // Aynı token'ı kullan
                    expires_in: Some(3600),             // Varsayılan 1 saat
                    created_at: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                });
                
                // Kullanıcı token'ını güncelle
                self.user_token = Some(token);
                
                Ok(())
            }
            Err(e) => Err(Error::Authentication(format!("Giriş başarısız: {:?}", e))),
        }
    }

    /// Oturumu kapat
    pub async fn logout(&mut self) -> Result<(), Error> {
        // Token'ı sıfırla
        let mut token_info = self.token_info.lock().await;
        *token_info = None;
        
        // Kullanıcı token'ını sıfırla
        self.user_token = None;
        
        Ok(())
    }

    /// API anahtarını ayarla
    pub fn set_api_key(&mut self, api_key: Option<String>) {
        self.api_key = api_key;
    }

    /// Kullanıcı token'ını al
    pub fn get_user_token(&self) -> Option<String> {
        self.user_token.clone()
    }

    /// Oturum açık mı kontrol et
    pub async fn is_logged_in(&self) -> bool {
        let token_info = self.token_info.lock().await;
        token_info.is_some()
    }
}
