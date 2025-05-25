// Tauri Windows Plugin System - Data Processor WASM Plugin
//
// Bu plugin, WASM entegrasyonunu test etmek için veri işleme işlevleri sağlar.
// Cross-platform çalışabilirlik gösterilmektedir.

use plugin_interface::{
    PluginError, PluginInterface, PluginMetadata, PluginType,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

// WASM bellek allocator
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Panik hook
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_setup() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

// WASM modülü wrapper
#[wasm_bindgen]
pub struct WasmPluginWrapper {
    plugin: Mutex<DataProcessorPlugin>,
}

#[wasm_bindgen]
impl WasmPluginWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            plugin: Mutex::new(DataProcessorPlugin::new()),
        }
    }
    
    #[wasm_bindgen]
    pub fn get_id(&self) -> String {
        if let Ok(plugin) = self.plugin.lock() {
            return plugin.get_id().to_string();
        }
        "unknown".to_string()
    }
    
    #[wasm_bindgen]
    pub fn get_metadata(&self) -> JsValue {
        if let Ok(plugin) = self.plugin.lock() {
            let metadata = plugin.get_metadata();
            return serde_wasm_bindgen::to_value(&metadata).unwrap_or(JsValue::NULL);
        }
        JsValue::NULL
    }
    
    #[wasm_bindgen]
    pub fn initialize(&self) -> Result<(), JsValue> {
        if let Ok(mut plugin) = self.plugin.lock() {
            plugin.initialize().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(())
        } else {
            Err(JsValue::from_str("Mutex lock failed"))
        }
    }
    
    #[wasm_bindgen]
    pub fn shutdown(&self) -> Result<(), JsValue> {
        if let Ok(mut plugin) = self.plugin.lock() {
            plugin.shutdown().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(())
        } else {
            Err(JsValue::from_str("Mutex lock failed"))
        }
    }
    
    #[wasm_bindgen]
    pub fn execute_command(&self, command: String, args: String) -> Result<String, JsValue> {
        if let Ok(mut plugin) = self.plugin.lock() {
            let result = plugin.execute_command(&command, &args)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(result)
        } else {
            Err(JsValue::from_str("Mutex lock failed"))
        }
    }
    
    // Özel veri işleme fonksiyonları
    #[wasm_bindgen]
    pub fn process_text(&self, text: String) -> Result<String, JsValue> {
        if let Ok(plugin) = self.plugin.lock() {
            let result = plugin.process_text(&text)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(result)
        } else {
            Err(JsValue::from_str("Mutex lock failed"))
        }
    }
    
    #[wasm_bindgen]
    pub fn analyze_data(&self, data_json: String) -> Result<String, JsValue> {
        if let Ok(plugin) = self.plugin.lock() {
            let result = plugin.analyze_data(&data_json)
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            Ok(result)
        } else {
            Err(JsValue::from_str("Mutex lock failed"))
        }
    }
}

// İşlenmiş metin sonucu
#[derive(Serialize, Deserialize)]
struct ProcessedText {
    original: String,
    word_count: usize,
    char_count: usize,
    uppercase: String,
    lowercase: String,
    reversed: String,
}

// Veri analiz sonucu
#[derive(Serialize, Deserialize)]
struct DataAnalysisResult {
    item_count: usize,
    numeric_items: usize,
    text_items: usize,
    min_value: Option<f64>,
    max_value: Option<f64>,
    avg_value: Option<f64>,
    summary: String,
}

// Veri öğesi
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum DataItem {
    Number(f64),
    Text(String),
    Boolean(bool),
}

// Veri analizörü
#[derive(Serialize, Deserialize)]
struct DataAnalyzer {
    items: Vec<DataItem>,
}

// Plugin durumu
#[derive(Default)]
struct PluginState {
    text_processed_count: usize,
    data_analyzed_count: usize,
    last_operation_time: Option<f64>,
}

/// Data Processor Plugin
pub struct DataProcessorPlugin {
    /// Plugin ID'si
    id: String,
    /// Plugin metadatası
    metadata: PluginMetadata,
    /// Plugin başlatıldı mı?
    initialized: bool,
    /// Plugin durumu
    state: PluginState,
}

impl DataProcessorPlugin {
    /// Yeni bir veri işleme plugin'i oluşturur
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: "com.tauri.plugins.data-processor".to_string(),
            name: "Data Processor Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "WASM tabanlı veri işleme plugin'i".to_string(),
            plugin_type: PluginType::Wasm,
            vendor: "Tauri Windows Plugin System Team".to_string(),
            vendor_url: Some("https://tauri.app".to_string()),
            permissions: vec![
                "data.read".to_string(),
                "data.write".to_string(),
            ],
            min_host_version: Some("0.1.0".to_string()),
        };
        
        Self {
            id: metadata.id.clone(),
            metadata,
            initialized: false,
            state: PluginState::default(),
        }
    }
    
    /// Metin işleme
    pub fn process_text(&self, text: &str) -> Result<String, PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        // Metin istatistiklerini hesapla
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        let uppercase = text.to_uppercase();
        let lowercase = text.to_lowercase();
        let reversed: String = text.chars().rev().collect();
        
        // Sonucu oluştur
        let result = ProcessedText {
            original: text.to_string(),
            word_count,
            char_count,
            uppercase,
            lowercase,
            reversed,
        };
        
        // JSON olarak dönüştür
        serde_json::to_string(&result).map_err(|e| 
            PluginError::Serialization(format!("Serileştirme hatası: {}", e))
        )
    }
    
    /// Veri analizi
    pub fn analyze_data(&self, data_json: &str) -> Result<String, PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        // JSON'ı ayrıştır
        let analyzer: DataAnalyzer = serde_json::from_str(data_json).map_err(|e| 
            PluginError::Serialization(format!("Veri ayrıştırma hatası: {}", e))
        )?;
        
        // İstatistikleri hesapla
        let item_count = analyzer.items.len();
        let mut numeric_items = 0;
        let mut text_items = 0;
        let mut numeric_values = Vec::new();
        
        for item in &analyzer.items {
            match item {
                DataItem::Number(n) => {
                    numeric_items += 1;
                    numeric_values.push(*n);
                },
                DataItem::Text(_) => text_items += 1,
                DataItem::Boolean(_) => {},
            }
        }
        
        // İstatistiksel değerleri hesapla
        let min_value = numeric_values.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let max_value = numeric_values.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let avg_value = if !numeric_values.is_empty() {
            Some(numeric_values.iter().sum::<f64>() / numeric_values.len() as f64)
        } else {
            None
        };
        
        // Özet metni oluştur
        let summary = format!(
            "Toplam {} öğe analiz edildi: {} sayısal, {} metin.", 
            item_count, 
            numeric_items, 
            text_items
        );
        
        // Sonucu oluştur
        let result = DataAnalysisResult {
            item_count,
            numeric_items,
            text_items,
            min_value,
            max_value,
            avg_value,
            summary,
        };
        
        // JSON olarak dönüştür
        serde_json::to_string(&result).map_err(|e| 
            PluginError::Serialization(format!("Serileştirme hatası: {}", e))
        )
    }
    
    // Mevcut zamanı al (saniye cinsinden)
    fn get_current_time() -> f64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() / 1000.0
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64()
        }
    }
}

impl PluginInterface for DataProcessorPlugin {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Err(PluginError::General("Plugin zaten başlatılmış".to_string()));
        }
        
        self.initialized = true;
        self.state.last_operation_time = Some(Self::get_current_time());
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        self.initialized = false;
        self.state.last_operation_time = None;
        
        Ok(())
    }
    
    fn execute_command(&mut self, command: &str, args: &str) -> Result<String, PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        self.state.last_operation_time = Some(Self::get_current_time());
        
        match command {
            "process_text" => {
                let result = self.process_text(args)?;
                self.state.text_processed_count += 1;
                Ok(result)
            },
            "analyze_data" => {
                let result = self.analyze_data(args)?;
                self.state.data_analyzed_count += 1;
                Ok(result)
            },
            "get_stats" => {
                // Plugin istatistiklerini döndür
                #[derive(Serialize)]
                struct PluginStats {
                    text_processed_count: usize,
                    data_analyzed_count: usize,
                    uptime: Option<f64>,
                }
                
                let uptime = self.state.last_operation_time.map(|start_time| {
                    Self::get_current_time() - start_time
                });
                
                let stats = PluginStats {
                    text_processed_count: self.state.text_processed_count,
                    data_analyzed_count: self.state.data_analyzed_count,
                    uptime,
                };
                
                serde_json::to_string(&stats).map_err(|e| 
                    PluginError::Serialization(format!("Serileştirme hatası: {}", e))
                )
            },
            _ => Err(PluginError::Api(format!("Bilinmeyen komut: {}", command))),
        }
    }
}

// C-style API (WASM için)
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn PluginInterface {
    Box::into_raw(Box::new(DataProcessorPlugin::new()))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn PluginInterface) {
    if !plugin.is_null() {
        unsafe {
            drop(Box::from_raw(plugin));
        }
    }
}
