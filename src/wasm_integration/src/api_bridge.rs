// Tauri Windows Plugin System - WASM API Bridge
//
// Bu modül, WASM modülleri ile host uygulama arasında API köprüsü sağlar.
// İki yönlü fonksiyon çağrıları, veri dönüşümleri ve hata işleme mekanizmaları içerir.

use crate::wasm_runtime::{WasmRuntimeManager, WasmRuntimeManagerError};
use crate::wasm_types::{WasmLoadOptions, WasmModuleConfig, WasmModuleSummary, WasmValueType};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use wasmtime::Val;

/// API Köprüsü Hatası
#[derive(Error, Debug)]
pub enum ApiBridgeError {
    #[error("Çalışma zamanı hatası: {0}")]
    Runtime(#[from] WasmRuntimeManagerError),

    #[error("Serileştirme hatası: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Parametre dönüşüm hatası: {0}")]
    ParameterConversion(String),

    #[error("Dönüş değeri dönüşüm hatası: {0}")]
    ResultConversion(String),

    #[error("API fonksiyonu bulunamadı: {0}")]
    ApiFunctionNotFound(String),

    #[error("API hatası: {0}")]
    ApiError(String),

    #[error("Geçersiz modül: {0}")]
    InvalidModule(String),
}

/// API köprüsü
pub struct ApiBridge {
    /// WASM çalışma zamanı yöneticisi
    runtime: Arc<RwLock<WasmRuntimeManager>>,
    /// API fonksiyonları
    api_functions: HashMap<String, Arc<dyn Fn(JsonValue) -> Result<JsonValue, ApiBridgeError> + Send + Sync>>,
    /// Varsayılan modül konfigürasyonu
    default_config: WasmModuleConfig,
}

impl ApiBridge {
    /// Yeni bir API köprüsü oluşturur
    pub fn new(
        runtime: Arc<RwLock<WasmRuntimeManager>>,
        default_config: Option<WasmModuleConfig>,
    ) -> Self {
        Self {
            runtime,
            api_functions: HashMap::new(),
            default_config: default_config.unwrap_or_default(),
        }
    }
    
    /// Bir API fonksiyonu ekler
    pub fn add_api_function<F>(
        &mut self,
        name: &str,
        function: F,
    ) where
        F: Fn(JsonValue) -> Result<JsonValue, ApiBridgeError> + Send + Sync + 'static,
    {
        self.api_functions.insert(name.to_string(), Arc::new(function));
    }
    
    /// API fonksiyonunu çağırır
    pub async fn call_api_function(
        &self,
        function_name: &str,
        params: JsonValue,
    ) -> Result<JsonValue, ApiBridgeError> {
        // API fonksiyonunu bul
        let func = self.api_functions.get(function_name).ok_or_else(|| {
            ApiBridgeError::ApiFunctionNotFound(function_name.to_string())
        })?;
        
        // Fonksiyonu çağır
        func(params)
    }
    
    /// Dosyadan bir WASM modülü yükler
    pub async fn load_module_from_file(
        &self,
        path: impl AsRef<Path>,
        options: Option<WasmLoadOptions>,
    ) -> Result<String, ApiBridgeError> {
        let mut runtime = self.runtime.write().await;
        let module_id = runtime.load_module_from_file(path, options).await?;
        Ok(module_id)
    }
    
    /// Byte dizisinden bir WASM modülü yükler
    pub async fn load_module_from_bytes(
        &self,
        bytes: &[u8],
        options: Option<WasmLoadOptions>,
    ) -> Result<String, ApiBridgeError> {
        let mut runtime = self.runtime.write().await;
        let module_id = runtime.load_module_from_bytes(bytes, options).await?;
        Ok(module_id)
    }
    
    /// Modülü başlatır
    pub async fn instantiate_module(
        &self,
        module_id: &str,
    ) -> Result<(), ApiBridgeError> {
        let mut runtime = self.runtime.write().await;
        runtime.instantiate_module(module_id).await?;
        Ok(())
    }
    
    /// WASM modülünde bir fonksiyonu çağırır
    pub async fn call_wasm_function(
        &self,
        module_id: &str,
        function_name: &str,
        params: JsonValue,
    ) -> Result<JsonValue, ApiBridgeError> {
        // JSON parametreleri WASM değerlerine dönüştür
        let args = self.json_to_wasm_values(params)?;
        
        // Fonksiyonu çağır
        let runtime = self.runtime.read().await;
        let result = runtime.call_function(module_id, function_name, args).await?;
        
        // Sonucu JSON'a dönüştür
        let json_result = self.wasm_values_to_json(&result)?;
        
        Ok(json_result)
    }
    
    /// JSON değerini WASM değerlerine dönüştürür
    fn json_to_wasm_values(&self, json: JsonValue) -> Result<Vec<Val>, ApiBridgeError> {
        let mut values = Vec::new();
        
        match json {
            JsonValue::Array(array) => {
                for value in array {
                    values.push(self.json_to_wasm_value(value)?);
                }
            }
            _ => {
                values.push(self.json_to_wasm_value(json)?);
            }
        }
        
        Ok(values)
    }
    
    /// JSON değerini bir WASM değerine dönüştürür
    fn json_to_wasm_value(&self, json: JsonValue) -> Result<Val, ApiBridgeError> {
        match json {
            JsonValue::Null => {
                // Wasmtime'da null için i32(0) kullan
                Ok(Val::I32(0))
            }
            JsonValue::Bool(b) => {
                // Wasmtime'da bool için i32(0/1) kullan
                Ok(Val::I32(if b { 1 } else { 0 }))
            }
            JsonValue::Number(n) => {
                if n.is_i64() {
                    if let Some(i) = n.as_i64() {
                        if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                            Ok(Val::I32(i as i32))
                        } else {
                            Ok(Val::I64(i))
                        }
                    } else {
                        Err(ApiBridgeError::ParameterConversion(format!(
                            "Geçersiz sayı değeri: {:?}",
                            n
                        )))
                    }
                } else if n.is_f64() {
                    if let Some(f) = n.as_f64() {
                        Ok(Val::F64(f))
                    } else {
                        Err(ApiBridgeError::ParameterConversion(format!(
                            "Geçersiz float değeri: {:?}",
                            n
                        )))
                    }
                } else {
                    Err(ApiBridgeError::ParameterConversion(format!(
                        "Desteklenmeyen sayı tipi: {:?}",
                        n
                    )))
                }
            }
            JsonValue::String(s) => {
                // Wasmtime'da string doğrudan temsil edilemez
                // Bu, belleğe yazılarak işlenir
                // Bu örnekte, string değeri bir i32 pointer olarak temsil edilecek
                // Gerçek implementasyonda, belleğe yazılıp pointer döndürülecek
                Err(ApiBridgeError::ParameterConversion(
                    "String değerleri için özel bellek yönetimi gerekli".to_string(),
                ))
            }
            JsonValue::Array(_) | JsonValue::Object(_) => {
                // Karmaşık türler için özel bellek yönetimi gerekli
                Err(ApiBridgeError::ParameterConversion(
                    "Karmaşık türler için özel bellek yönetimi gerekli".to_string(),
                ))
            }
        }
    }
    
    /// WASM değerlerini JSON'a dönüştürür
    fn wasm_values_to_json(&self, values: &[Val]) -> Result<JsonValue, ApiBridgeError> {
        if values.is_empty() {
            return Ok(JsonValue::Null);
        }
        
        if values.len() == 1 {
            return self.wasm_value_to_json(&values[0]);
        }
        
        let mut array = Vec::with_capacity(values.len());
        for value in values {
            array.push(self.wasm_value_to_json(value)?);
        }
        
        Ok(JsonValue::Array(array))
    }
    
    /// WASM değerini JSON'a dönüştürür
    fn wasm_value_to_json(&self, value: &Val) -> Result<JsonValue, ApiBridgeError> {
        match value {
            Val::I32(i) => Ok(JsonValue::Number((*i).into())),
            Val::I64(i) => Ok(JsonValue::Number((*i).into())),
            Val::F32(f) => Ok(JsonValue::Number((*f as f64).into())),
            Val::F64(f) => Ok(JsonValue::Number((*f).into())),
            _ => Err(ApiBridgeError::ResultConversion(format!(
                "Desteklenmeyen değer tipi: {:?}",
                value
            ))),
        }
    }
    
    /// Tüm modülleri listeler
    pub async fn list_modules(&self) -> Result<Vec<WasmModuleSummary>, ApiBridgeError> {
        let runtime = self.runtime.read().await;
        let module_ids = runtime.list_modules().await;
        
        let mut summaries = Vec::with_capacity(module_ids.len());
        for id in module_ids {
            match runtime.get_module_summary(&id).await {
                Ok(summary) => summaries.push(summary),
                Err(e) => {
                    warn!("Modül özeti alınamadı {}: {}", id, e);
                }
            }
        }
        
        Ok(summaries)
    }
    
    /// Modülü durdurur
    pub async fn stop_module(&self, module_id: &str) -> Result<(), ApiBridgeError> {
        let mut runtime = self.runtime.write().await;
        runtime.stop_module(module_id).await?;
        Ok(())
    }
    
    /// Modülü kaldırır
    pub async fn unload_module(&self, module_id: &str) -> Result<(), ApiBridgeError> {
        let mut runtime = self.runtime.write().await;
        runtime.unload_module(module_id).await?;
        Ok(())
    }
}

/// WASM tipine göre JSON değerini WASM değerine dönüştürür
pub fn json_to_wasm_typed(
    json: &JsonValue,
    wasm_type: WasmValueType,
) -> Result<Val, ApiBridgeError> {
    match (json, wasm_type) {
        (JsonValue::Number(n), WasmValueType::I32) => {
            if let Some(i) = n.as_i64() {
                if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                    Ok(Val::I32(i as i32))
                } else {
                    Err(ApiBridgeError::ParameterConversion(format!(
                        "Sayı i32 aralığı dışında: {}",
                        i
                    )))
                }
            } else {
                Err(ApiBridgeError::ParameterConversion(
                    "Geçersiz i32 değeri".to_string(),
                ))
            }
        }
        (JsonValue::Number(n), WasmValueType::I64) => {
            if let Some(i) = n.as_i64() {
                Ok(Val::I64(i))
            } else {
                Err(ApiBridgeError::ParameterConversion(
                    "Geçersiz i64 değeri".to_string(),
                ))
            }
        }
        (JsonValue::Number(n), WasmValueType::F32) => {
            if let Some(f) = n.as_f64() {
                Ok(Val::F32(f as f32))
            } else {
                Err(ApiBridgeError::ParameterConversion(
                    "Geçersiz f32 değeri".to_string(),
                ))
            }
        }
        (JsonValue::Number(n), WasmValueType::F64) => {
            if let Some(f) = n.as_f64() {
                Ok(Val::F64(f))
            } else {
                Err(ApiBridgeError::ParameterConversion(
                    "Geçersiz f64 değeri".to_string(),
                ))
            }
        }
        (JsonValue::Bool(b), WasmValueType::I32) => {
            Ok(Val::I32(if *b { 1 } else { 0 }))
        }
        _ => Err(ApiBridgeError::ParameterConversion(format!(
            "Uyumsuz türler: {:?} -> {:?}",
            json, wasm_type
        ))),
    }
}
