// Tauri Windows Plugin System - API Client
//
// Bu modül, plugin mağazası API'si ile iletişim kuran fonksiyonları içerir.

use crate::store_types::{
    PluginMetadata, PluginSearchFilter, PluginSearchResult,
    PluginDownloadInfo, PluginReview, PluginUpdate, StoreError,
};
use log::{debug, error, info, warn};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

/// API istemcisi
pub struct ApiClient {
    /// API base URL
    api_url: String,
    /// HTTP istemcisi
    client: Client,
}

impl ApiClient {
    /// Yeni bir API istemcisi oluştur
    pub fn new(api_url: &str) -> Result<Self, crate::Error> {
        // HTTP istemcisi oluştur
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| crate::Error::Configuration(format!("HTTP istemcisi oluşturulamadı: {}", e)))?;
        
        Ok(Self {
            api_url: api_url.to_string(),
            client,
        })
    }
    
    /// API URL'sini al
    pub fn api_url(&self) -> &str {
        &self.api_url
    }
    
    /// Başlıkları oluştur
    fn create_headers(&self, token: &str) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))
                .unwrap_or_else(|_| header::HeaderValue::from_static(""))
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json")
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json")
        );
        headers.insert(
            "X-Client-ID",
            header::HeaderValue::from_str(&Uuid::new_v4().to_string())
                .unwrap_or_else(|_| header::HeaderValue::from_static(""))
        );
        
        headers
    }
    
    /// Plugin ara
    pub async fn search_plugins(&self, token: &str, filter: PluginSearchFilter) -> Result<PluginSearchResult, StoreError> {
        // Endpoint
        let url = format!("{}/plugins/search", self.api_url);
        
        // API isteği yap
        let response = self.client
            .post(&url)
            .headers(self.create_headers(token))
            .json(&filter)
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "API hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        let result: PluginSearchResult = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(result)
    }
    
    /// Plugin detaylarını al
    pub async fn get_plugin_details(&self, token: &str, plugin_id: &str) -> Result<PluginMetadata, StoreError> {
        // Endpoint
        let url = format!("{}/plugins/{}", self.api_url, plugin_id);
        
        // API isteği yap
        let response = self.client
            .get(&url)
            .headers(self.create_headers(token))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "API hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        let plugin: PluginMetadata = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(plugin)
    }
    
    /// Plugin indirme bilgilerini al
    pub async fn get_download_info(&self, token: &str, plugin_id: &str, version: Option<&str>) -> Result<PluginDownloadInfo, StoreError> {
        // Endpoint
        let url = if let Some(v) = version {
            format!("{}/plugins/{}/download?version={}", self.api_url, plugin_id, v)
        } else {
            format!("{}/plugins/{}/download", self.api_url, plugin_id)
        };
        
        // API isteği yap
        let response = self.client
            .get(&url)
            .headers(self.create_headers(token))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "API hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        let info: PluginDownloadInfo = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(info)
    }
    
    /// Plugin güncelleme kontrolü
    pub async fn check_plugin_updates(&self, token: &str, plugin_id: &str, current_version: &str) -> Result<Option<PluginUpdate>, StoreError> {
        // Endpoint
        let url = format!("{}/plugins/{}/updates?current_version={}", self.api_url, plugin_id, current_version);
        
        // API isteği yap
        let response = self.client
            .get(&url)
            .headers(self.create_headers(token))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if response.status().is_not_found() {
            // Güncelleme yok, hata değil
            return Ok(None);
        }
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "API hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        let update: PluginUpdate = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(Some(update))
    }
    
    /// Plugin yorumlarını al
    pub async fn get_plugin_reviews(&self, token: &str, plugin_id: &str) -> Result<Vec<PluginReview>, StoreError> {
        // Endpoint
        let url = format!("{}/plugins/{}/reviews", self.api_url, plugin_id);
        
        // API isteği yap
        let response = self.client
            .get(&url)
            .headers(self.create_headers(token))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "API hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        let reviews: Vec<PluginReview> = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(reviews)
    }
    
    /// API'ye kimlik doğrulama yap
    pub async fn authenticate(&self, username: &str, password: &str) -> Result<String, StoreError> {
        // Endpoint
        let url = format!("{}/auth/login", self.api_url);
        
        // Kimlik bilgileri
        #[derive(Serialize)]
        struct Credentials {
            username: String,
            password: String,
        }
        
        let credentials = Credentials {
            username: username.to_string(),
            password: password.to_string(),
        };
        
        // API isteği yap
        let response = self.client
            .post(&url)
            .json(&credentials)
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("Kimlik doğrulama isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            
            if status.as_u16() == 401 {
                return Err(StoreError::AuthenticationError("Geçersiz kullanıcı adı veya şifre".to_string()));
            }
            
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "Kimlik doğrulama hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        #[derive(Deserialize)]
        struct AuthResponse {
            token: String,
        }
        
        let auth: AuthResponse = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(auth.token)
    }
    
    /// API anahtarını doğrula
    pub async fn verify_api_key(&self, api_key: &str) -> Result<bool, StoreError> {
        // Endpoint
        let url = format!("{}/auth/verify", self.api_url);
        
        // API isteği yap
        let response = self.client
            .get(&url)
            .header(header::AUTHORIZATION, format!("ApiKey {}", api_key))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("API anahtarı doğrulama isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        Ok(response.status().is_success())
    }
    
    /// Kullanıcı token'ını yenile
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<String, StoreError> {
        // Endpoint
        let url = format!("{}/auth/refresh", self.api_url);
        
        // API isteği yap
        let response = self.client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", refresh_token))
            .send()
            .await
            .map_err(|e| StoreError::ApiError(format!("Token yenileme isteği başarısız: {}", e)))?;
        
        // Yanıtı kontrol et
        if !response.status().is_success() {
            let status = response.status();
            
            if status.as_u16() == 401 {
                return Err(StoreError::AuthenticationError("Yenileme token'ı geçersiz veya süresi dolmuş".to_string()));
            }
            
            let error_text = response.text().await
                .unwrap_or_else(|_| "Hata detayları alınamadı".to_string());
            
            return Err(StoreError::ApiError(format!(
                "Token yenileme hatası ({}): {}", 
                status, 
                error_text
            )));
        }
        
        // Yanıtı JSON olarak ayrıştır
        #[derive(Deserialize)]
        struct RefreshResponse {
            token: String,
        }
        
        let refresh: RefreshResponse = response.json().await
            .map_err(|e| StoreError::DeserializationError(format!("Yanıt ayrıştırılamadı: {}", e)))?;
        
        Ok(refresh.token)
    }
}
