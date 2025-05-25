// Tauri Windows Plugin System - İzin İsteme Arayüzü
//
// Bu modül, kullanıcıya izin isteklerini göstermek ve yanıtları almak için
// kullanılan arayüz bileşenlerini içerir.

use crate::permission_types::{PermissionRequest, PermissionResponse, PermissionState};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

use tauri::{AppHandle, Manager, Window};

/// İzin isteme hatası
#[derive(Error, Debug)]
pub enum PromptError {
    #[error("İletişim hatası: {0}")]
    CommunicationError(String),

    #[error("Zaman aşımı")]
    Timeout,

    #[error("Kullanıcı arayüzü hatası: {0}")]
    UIError(String),

    #[error("İstek iptal edildi")]
    Cancelled,
}

/// İzin isteme stili
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptStyle {
    /// Modal dialog (tam blokaj)
    Modal,
    /// Sayfa içi dialog (yarı blokaj)
    InlineDialog,
    /// Banner (blokaj yok)
    Banner,
    /// Bildirim (minimal blokaj)
    Notification,
}

/// İzin isteme seçenekleri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptOptions {
    /// İsteme stili
    pub style: PromptStyle,
    /// Zaman aşımı (saniye)
    pub timeout: Option<u32>,
    /// İzin kararını hatırlama seçeneği
    pub can_remember: bool,
}

/// İzin yanıtı sonucu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptResult {
    /// İzin ver
    Allow,
    /// İzin verme
    Deny,
    /// Zaman aşımı
    Timeout,
}

/// İzin yanıtı (prompt'tan)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    /// İstek ID'si
    pub request_id: String,
    /// Sonuç
    pub result: PromptResult,
    /// Kararı hatırla
    pub remember: bool,
}

/// İzin isteme arayüzü
pub struct PermissionPrompt {
    /// Tauri app handle
    app: AppHandle,
}

impl PermissionPrompt {
    /// Yeni bir izin isteme arayüzü oluştur
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// İzin istek prompt'unu göster
    pub async fn show_permission_prompt(
        &self,
        request: &PermissionRequest,
        options: &PromptOptions,
    ) -> Result<PromptResponse, PromptError> {
        // Stil türüne göre uygun prompt'u göster
        match options.style {
            PromptStyle::Modal => self.show_modal_prompt(request, options).await,
            PromptStyle::InlineDialog => self.show_inline_dialog(request, options).await,
            PromptStyle::Banner => self.show_banner(request, options).await,
            PromptStyle::Notification => self.show_notification(request, options).await,
        }
    }

    /// Modal dialog prompt göster
    async fn show_modal_prompt(
        &self,
        request: &PermissionRequest,
        options: &PromptOptions,
    ) -> Result<PromptResponse, PromptError> {
        // İzin isteği bilgilerini JS tarafına gönder
        let request_data = serde_json::to_value(request)
            .map_err(|e| PromptError::CommunicationError(format!("Serileştirme hatası: {}", e)))?;

        let options_data = serde_json::to_value(options)
            .map_err(|e| PromptError::CommunicationError(format!("Serileştirme hatası: {}", e)))?;

        // Ana pencereyi bul
        let window = self
            .app
            .get_window("main")
            .ok_or_else(|| PromptError::UIError("Ana pencere bulunamadı".to_string()))?;

        // İstek event'i gönder
        window
            .emit("permission:request", (request_data, options_data))
            .map_err(|e| PromptError::CommunicationError(format!("Event emit hatası: {}", e)))?;

        // Yanıt için bekle
        let (tx, rx) = tokio::sync::oneshot::channel();

        // Event listener ayarla
        let request_id = request.id.clone();
        let event_name = format!("permission:response:{}", request_id);

        let window_clone = window.clone();
        let listener_id = window.listen(event_name, move |event| {
            if let Some(payload) = event.payload() {
                match serde_json::from_str::<PromptResponse>(payload) {
                    Ok(response) => {
                        let _ = tx.send(response);
                        // Listener'ı temizle
                        window_clone.unlisten(listener_id);
                    }
                    Err(e) => {
                        eprintln!("İzin yanıtı ayrıştırma hatası: {}", e);
                    }
                }
            }
        });

        // Zaman aşımı kontrolü
        if let Some(timeout_seconds) = options.timeout {
            tokio::select! {
                response = rx => {
                    response.map_err(|_| PromptError::CommunicationError("Yanıt kanalı kapandı".to_string()))
                }
                _ = tokio::time::sleep(Duration::from_secs(timeout_seconds.into())) => {
                    // Timeout olduğunda listener'ı temizle
                    window.unlisten(listener_id);
                    
                    // Timeout yanıtı oluştur
                    Ok(PromptResponse {
                        request_id: request.id.clone(),
                        result: PromptResult::Timeout,
                        remember: false,
                    })
                }
            }
        } else {
            // Zaman aşımı olmadan sadece yanıtı bekle
            rx.await
                .map_err(|_| PromptError::CommunicationError("Yanıt kanalı kapandı".to_string()))
        }
    }

    /// Inline dialog prompt göster
    async fn show_inline_dialog(
        &self,
        request: &PermissionRequest,
        options: &PromptOptions,
    ) -> Result<PromptResponse, PromptError> {
        // Bu implementasyon Modal ile aynı, fakat frontend farklı gösterecek
        self.show_modal_prompt(request, options).await
    }

    /// Banner prompt göster
    async fn show_banner(
        &self,
        request: &PermissionRequest,
        options: &PromptOptions,
    ) -> Result<PromptResponse, PromptError> {
        // Bu implementasyon Modal ile aynı, fakat frontend farklı gösterecek
        self.show_modal_prompt(request, options).await
    }

    /// Bildirim prompt göster
    async fn show_notification(
        &self,
        request: &PermissionRequest,
        options: &PromptOptions,
    ) -> Result<PromptResponse, PromptError> {
        // Bu implementasyon Modal ile aynı, fakat frontend farklı gösterecek
        self.show_modal_prompt(request, options).await
    }

    /// İzin yanıtını PermissionResponse'a dönüştür
    pub fn convert_to_permission_response(&self, prompt_response: PromptResponse) -> PermissionResponse {
        PermissionResponse {
            request_id: prompt_response.request_id,
            state: match prompt_response.result {
                PromptResult::Allow => PermissionState::Granted,
                PromptResult::Deny => PermissionState::Denied,
                PromptResult::Timeout => PermissionState::NotRequested,
            },
            expires_in: if prompt_response.remember {
                Some(30 * 24 * 60 * 60) // 30 gün
            } else {
                Some(60 * 60) // 1 saat
            },
            timestamp: Utc::now(),
        }
    }
}
