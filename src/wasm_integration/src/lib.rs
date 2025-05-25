// Tauri Windows Plugin System - WASM Entegrasyonu
//
// Bu modül, Tauri uygulamalarında WebAssembly (WASM) plugin'leri çalıştırmak için
// gerekli altyapıyı sağlar. WASM modüllerini yüklemek, çalıştırmak ve izole etmek
// için Wasmtime kullanır ve güvenli bir sandbox ortamı sunar.

pub mod wasm_types;
pub mod wasm_runtime;
pub mod api_bridge;
pub mod security;

pub use wasm_types::{
    OptimizationLevel, WasmFunctionSignature, WasmLoadError, WasmLoadOptions,
    WasmModuleConfig, WasmModuleMetadata, WasmModuleState, WasmModuleStats,
    WasmModuleSummary, WasmModuleType, WasmRuntimeError, WasmValueType, WasiFeatures,
};
pub use wasm_runtime::{WasmRuntimeManager, WasmRuntimeManagerError};
pub use api_bridge::{ApiBridge, ApiBridgeError};
pub use security::{
    WasmPermission, WasmPermissionRequest, WasmPermissionResponse,
    WasmPermissionRequestStatus, WasmPermissionScope, WasmPermissionType,
    WasmSecurityError, WasmSecurityManager, WasmSecurityPolicy,
};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{
    command,
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};
use tokio::sync::RwLock;

/// WASM plugin hatası
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Çalışma zamanı hatası: {0}")]
    Runtime(#[from] WasmRuntimeManagerError),

    #[error("API köprüsü hatası: {0}")]
    ApiBridge(#[from] ApiBridgeError),

    #[error("Güvenlik hatası: {0}")]
    Security(#[from] WasmSecurityError),

    #[error("JSON serileştirme hatası: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tauri hatası: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("Yapılandırma hatası: {0}")]
    Configuration(String),
}

type Result<T> = std::result::Result<T, Error>;

/// WASM plugin komutları
#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Commands {
    /// Dosyadan bir WASM modülü yükle
    LoadModuleFromFile {
        /// Modül dosya yolu
        path: String,
        /// Yükleme seçenekleri
        options: Option<WasmLoadOptions>,
    },
    /// Byte dizisinden bir WASM modülü yükle
    LoadModuleFromBytes {
        /// Modül verileri (base64)
        data: String,
        /// Yükleme seçenekleri
        options: Option<WasmLoadOptions>,
    },
    /// Modülü başlat
    InstantiateModule {
        /// Modül ID'si
        module_id: String,
    },
    /// WASM fonksiyonu çağır
    CallWasmFunction {
        /// Modül ID'si
        module_id: String,
        /// Fonksiyon adı
        function_name: String,
        /// Parametreler
        params: serde_json::Value,
    },
    /// Tüm modülleri listele
    ListModules {},
    /// Modül özetini al
    GetModuleSummary {
        /// Modül ID'si
        module_id: String,
    },
    /// Modülü durdur
    StopModule {
        /// Modül ID'si
        module_id: String,
    },
    /// Modülü kaldır
    UnloadModule {
        /// Modül ID'si
        module_id: String,
    },
    /// İzin iste
    RequestPermission {
        /// Modül ID'si
        module_id: String,
        /// İzin tipi
        permission_type: WasmPermissionType,
        /// İzin kapsamı
        scope: WasmPermissionScope,
        /// İzin açıklaması
        description: String,
        /// Risk seviyesi
        risk_level: u8,
    },
    /// İzinleri al
    GetPermissions {
        /// Modül ID'si
        module_id: String,
    },
}

/// WASM plugin durumu
pub struct WasmPluginState {
    /// WASM çalışma zamanı yöneticisi
    runtime_manager: Arc<RwLock<WasmRuntimeManager>>,
    /// API köprüsü
    api_bridge: Arc<RwLock<ApiBridge>>,
    /// Güvenlik yöneticisi
    security_manager: Arc<RwLock<WasmSecurityManager>>,
    /// Tauri uygulama handle'ı
    app_handle: Option<AppHandle>,
}

/// WASM plugin konfigürasyonu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmPluginConfig {
    /// Varsayılan modül konfigürasyonu
    pub default_module_config: Option<WasmModuleConfig>,
    /// Güvenlik politikası
    pub security_policy: Option<WasmSecurityPolicy>,
    /// Otomatik izinler
    pub auto_permissions: Option<Vec<WasmPermission>>,
    /// Güvenli dizinler
    pub safe_directories: Option<HashMap<String, PathBuf>>,
    /// Güvenli host'lar
    pub safe_hosts: Option<Vec<String>>,
    /// Güvenli API'ler
    pub safe_apis: Option<Vec<String>>,
}

impl Default for WasmPluginConfig {
    fn default() -> Self {
        Self {
            default_module_config: None,
            security_policy: Some(WasmSecurityPolicy::AskOnce),
            auto_permissions: None,
            safe_directories: None,
            safe_hosts: None,
            safe_apis: None,
        }
    }
}

/// WASM plugin
pub struct WasmPlugin<R: Runtime> {
    /// Tauri plugin
    plugin: TauriPlugin<R>,
}

impl<R: Runtime> WasmPlugin<R> {
    /// Yeni bir WASM plugin oluşturur
    pub fn new(config: WasmPluginConfig) -> Result<Self> {
        // WASM çalışma zamanı yöneticisini oluştur
        let runtime_manager = WasmRuntimeManager::new(config.default_module_config.clone())?;
        let runtime_manager = Arc::new(RwLock::new(runtime_manager));
        
        // Güvenlik yöneticisini oluştur
        let security_manager = WasmSecurityManager::new(
            config.security_policy.unwrap_or(WasmSecurityPolicy::AskOnce),
        );
        let security_manager = Arc::new(RwLock::new(security_manager));
        
        // API köprüsünü oluştur
        let api_bridge = ApiBridge::new(
            runtime_manager.clone(),
            config.default_module_config,
        );
        let api_bridge = Arc::new(RwLock::new(api_bridge));
        
        // Plugin durumu
        let state = WasmPluginState {
            runtime_manager,
            api_bridge,
            security_manager,
            app_handle: None,
        };
        
        // Tauri plugin
        let plugin = Builder::new("wasm")
            .setup(move |app| {
                let state: State<'_, WasmPluginState> = app.state();
                let mut state = state.inner().lock().blocking_unwrap();
                state.app_handle = Some(app.app_handle());
                
                // Güvenlik konfigürasyonunu uygula
                let security_manager = state.security_manager.clone();
                let config = config.clone();
                
                tokio::spawn(async move {
                    // Güvenli dizinleri ekle
                    if let Some(safe_dirs) = config.safe_directories {
                        for (name, path) in safe_dirs {
                            if let Err(e) = security_manager.write().await.add_safe_directory(&name, path).await {
                                error!("Güvenli dizin eklenemedi: {}", e);
                            }
                        }
                    }
                    
                    // Güvenli host'ları ekle
                    if let Some(safe_hosts) = config.safe_hosts {
                        for host in safe_hosts {
                            if let Err(e) = security_manager.write().await.add_safe_host(&host).await {
                                error!("Güvenli host eklenemedi: {}", e);
                            }
                        }
                    }
                    
                    // Güvenli API'leri ekle
                    if let Some(safe_apis) = config.safe_apis {
                        for api in safe_apis {
                            if let Err(e) = security_manager.write().await.add_safe_api(&api).await {
                                error!("Güvenli API eklenemedi: {}", e);
                            }
                        }
                    }
                    
                    // Otomatik izinleri ekle
                    if let Some(auto_permissions) = config.auto_permissions {
                        for permission in auto_permissions {
                            if let Err(e) = security_manager.write().await.add_permission("*", permission).await {
                                error!("Otomatik izin eklenemedi: {}", e);
                            }
                        }
                    }
                });
                
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                load_module_from_file,
                load_module_from_bytes,
                instantiate_module,
                call_wasm_function,
                list_modules,
                get_module_summary,
                stop_module,
                unload_module,
                request_permission,
                get_permissions,
            ])
            .build();
        
        Ok(Self { plugin })
    }
    
    /// Tauri plugin'ini döndürür
    pub fn plugin(&self) -> TauriPlugin<R> {
        self.plugin.clone()
    }
}

/// Tauri plugin init
pub fn init<R: Runtime>(config: WasmPluginConfig) -> TauriPlugin<R> {
    match WasmPlugin::new(config) {
        Ok(plugin) => plugin.plugin(),
        Err(e) => {
            error!("WASM plugin başlatılamadı: {}", e);
            Builder::new("wasm").build()
        }
    }
}

/// Dosyadan bir WASM modülü yükler
#[command]
async fn load_module_from_file(
    path: String,
    options: Option<WasmLoadOptions>,
    state: State<'_, WasmPluginState>,
) -> Result<String> {
    let api_bridge = state.api_bridge.read().await;
    let module_id = api_bridge.load_module_from_file(path, options).await?;
    Ok(module_id)
}

/// Byte dizisinden bir WASM modülü yükler
#[command]
async fn load_module_from_bytes(
    data: String,
    options: Option<WasmLoadOptions>,
    state: State<'_, WasmPluginState>,
) -> Result<String> {
    // Base64 decode
    let bytes = base64::decode(data).map_err(|e| {
        Error::Configuration(format!("Base64 decode hatası: {}", e))
    })?;
    
    let api_bridge = state.api_bridge.read().await;
    let module_id = api_bridge.load_module_from_bytes(&bytes, options).await?;
    Ok(module_id)
}

/// Modülü başlatır
#[command]
async fn instantiate_module(
    module_id: String,
    state: State<'_, WasmPluginState>,
) -> Result<()> {
    let api_bridge = state.api_bridge.read().await;
    api_bridge.instantiate_module(&module_id).await?;
    Ok(())
}

/// WASM fonksiyonu çağırır
#[command]
async fn call_wasm_function(
    module_id: String,
    function_name: String,
    params: serde_json::Value,
    state: State<'_, WasmPluginState>,
) -> Result<serde_json::Value> {
    let api_bridge = state.api_bridge.read().await;
    let result = api_bridge.call_wasm_function(&module_id, &function_name, params).await?;
    Ok(result)
}

/// Tüm modülleri listeler
#[command]
async fn list_modules(
    state: State<'_, WasmPluginState>,
) -> Result<Vec<WasmModuleSummary>> {
    let api_bridge = state.api_bridge.read().await;
    let modules = api_bridge.list_modules().await?;
    Ok(modules)
}

/// Modül özetini alır
#[command]
async fn get_module_summary(
    module_id: String,
    state: State<'_, WasmPluginState>,
) -> Result<WasmModuleSummary> {
    let runtime_manager = state.runtime_manager.read().await;
    let summary = runtime_manager.get_module_summary(&module_id).await?;
    Ok(summary)
}

/// Modülü durdurur
#[command]
async fn stop_module(
    module_id: String,
    state: State<'_, WasmPluginState>,
) -> Result<()> {
    let api_bridge = state.api_bridge.read().await;
    api_bridge.stop_module(&module_id).await?;
    Ok(())
}

/// Modülü kaldırır
#[command]
async fn unload_module(
    module_id: String,
    state: State<'_, WasmPluginState>,
) -> Result<()> {
    let api_bridge = state.api_bridge.read().await;
    api_bridge.unload_module(&module_id).await?;
    Ok(())
}

/// İzin ister
#[command]
async fn request_permission(
    module_id: String,
    permission_type: WasmPermissionType,
    scope: WasmPermissionScope,
    description: String,
    risk_level: u8,
    state: State<'_, WasmPluginState>,
) -> Result<bool> {
    let security_manager = state.security_manager.read().await;
    
    let permission = WasmPermission::new(
        permission_type,
        scope,
        &description,
        risk_level,
    );
    
    let result = security_manager.request_permission(&module_id, permission).await?;
    Ok(result)
}

/// İzinleri alır
#[command]
async fn get_permissions(
    module_id: String,
    state: State<'_, WasmPluginState>,
) -> Result<Vec<WasmPermission>> {
    let security_manager = state.security_manager.read().await;
    let permissions = security_manager.get_permissions(&module_id).await?;
    Ok(permissions)
}
