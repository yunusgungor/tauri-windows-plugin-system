// Tauri Windows Plugin System - Gelişmiş İzin Modülü
//
// Bu modül, Tauri plugin'leri için kapsamlı bir izin sistemi sağlar.

pub mod permission_types;
pub mod permission_manager;
pub mod permission_store;
pub mod prompt;

use permission_types::{
    PermissionCategory, PermissionDescriptor, PermissionRequest, PermissionResponse,
    PermissionSet, PermissionState, PermissionToken,
};
use permission_manager::{
    EnforcementLevel, PermissionError, PermissionManagerConfig, PermissionManager, PermissionPolicy,
};
use permission_store::{PermissionStore, PermissionStoreError, PluginInfo};
use prompt::{PermissionPrompt, PromptOptions, PromptResponse, PromptResult, PromptStyle};

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{plugin::Builder, AppHandle, Manager, Runtime, State};

// Dışa aktarılan türler
pub use permission_types::{
    CommandScope, FilesystemScope, HardwareScope, InterprocessScope,
    NetworkScope, SystemScope, UIScope,
};

// Tauri komut işleyicileri
#[tauri::command]
async fn request_permission(
    app: AppHandle,
    plugin_id: String,
    category: String,
    scope: u32,
    reason: String,
    permission_manager: State<'_, Arc<PermissionManager>>,
) -> Result<String, String> {
    // Kategori string'ini enum'a dönüştür
    let category = match category.as_str() {
        "filesystem" => PermissionCategory::Filesystem,
        "network" => PermissionCategory::Network,
        "system" => PermissionCategory::System,
        "ui" => PermissionCategory::UI,
        "hardware" => PermissionCategory::Hardware,
        "interprocess" => PermissionCategory::Interprocess,
        "command" => PermissionCategory::Command,
        _ => {
            return Err(format!("Geçersiz izin kategorisi: {}", category));
        }
    };

    // İzin tanımlayıcısı oluştur
    let descriptor = PermissionDescriptor {
        category,
        scope,
        reason,
    };

    // İzin iste
    match permission_manager
        .check_permissions(&plugin_id, vec![descriptor])
        .await
    {
        Ok(token) => Ok(format!(
            "İzin verildi. Belirteç ID: {}",
            token.id
        )),
        Err(e) => Err(format!("İzin hatası: {}", e)),
    }
}

#[tauri::command]
fn check_permission(
    plugin_id: String,
    category: String,
    scope: u32,
    permission_manager: State<'_, Arc<PermissionManager>>,
) -> Result<bool, String> {
    // Kategori string'ini enum'a dönüştür
    let category = match category.as_str() {
        "filesystem" => PermissionCategory::Filesystem,
        "network" => PermissionCategory::Network,
        "system" => PermissionCategory::System,
        "ui" => PermissionCategory::UI,
        "hardware" => PermissionCategory::Hardware,
        "interprocess" => PermissionCategory::Interprocess,
        "command" => PermissionCategory::Command,
        _ => {
            return Err(format!("Geçersiz izin kategorisi: {}", category));
        }
    };

    // İzni kontrol et
    permission_manager
        .has_permission(&plugin_id, category, scope)
        .map_err(|e| format!("İzin kontrolü hatası: {}", e))
}

#[tauri::command]
fn get_permissions(
    plugin_id: String,
    permission_manager: State<'_, Arc<PermissionManager>>,
) -> Result<String, String> {
    // Plugin izin belirtecini al
    match permission_manager.get_permission_token(&plugin_id) {
        Some(token) => {
            let json = serde_json::to_string_pretty(&token).unwrap_or_default();
            Ok(json)
        }
        None => Err(format!("Plugin için izin belirteci bulunamadı: {}", plugin_id)),
    }
}

#[tauri::command]
fn revoke_permission(
    plugin_id: String,
    permission_manager: State<'_, Arc<PermissionManager>>,
) -> Result<(), String> {
    // Plugin iznini iptal et
    permission_manager.remove_permission_token(&plugin_id);
    Ok(())
}

/// Tauri plugin'ini oluştur
pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    Builder::new("permissions")
        .invoke_handler(tauri::generate_handler![
            request_permission,
            check_permission,
            get_permissions,
            revoke_permission,
        ])
        .setup(|app| {
            // İzin deposu oluştur
            let app_dir = app.path_resolver().app_data_dir().unwrap_or_else(|| {
                PathBuf::from(".").join(".permissions")
            });
            let permissions_dir = app_dir.join(".permissions");
            
            let store = Arc::new(
                PermissionStore::new(permissions_dir)
                    .map_err(|e| tauri::Error::Setup(format!("İzin deposu hatası: {}", e)))?
            );
            
            // İzin prompt'u oluştur
            let prompt = Arc::new(PermissionPrompt::new(app.app_handle()));
            
            // Varsayılan konfigürasyon
            let config = PermissionManagerConfig::default();
            
            // İzin yöneticisi oluştur
            let permission_manager = Arc::new(PermissionManager::new(
                config,
                store,
                prompt,
            ));
            
            // İzin belirteçlerini yükle
            permission_manager
                .load_all_tokens()
                .map_err(|e| tauri::Error::Setup(format!("İzin belirteçleri yüklenemedi: {}", e)))?;
            
            // Uygulama durumuna ekle
            app.manage(permission_manager);
            
            Ok(())
        })
        .build()
}
