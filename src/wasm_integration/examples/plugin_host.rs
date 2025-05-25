// Tauri Windows Plugin System - WASM Plugin Host Örneği
//
// Bu örnek, WASM plugin'lerini yükleyen ve çalıştıran bir host uygulamasını gösterir.

use std::path::Path;
use tauri_plugin_wasm::{
    WasmLoadOptions, WasmModuleConfig, WasmModuleType, OptimizationLevel,
    WasmRuntimeManager, WasmRuntimeManagerError, WasmSecurityManager, WasmSecurityPolicy,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logları etkinleştir
    env_logger::init();
    println!("Tauri Windows Plugin System - WASM Plugin Host Örneği");

    // Varsayılan modül konfigürasyonu
    let config = WasmModuleConfig::default();

    // WASM runtime yöneticisini oluştur
    let mut runtime = WasmRuntimeManager::new(Some(config))?;
    
    // Güvenlik yöneticisini oluştur
    let security = WasmSecurityManager::new(WasmSecurityPolicy::AskOnce);

    // Örnek WASM modülü
    let wasm_path = "./examples/sample_plugin.wasm";
    println!("WASM modülü yükleniyor: {}", wasm_path);

    // WASM dosyası var mı kontrol et
    if !Path::new(wasm_path).exists() {
        println!("UYARI: {} dosyası bulunamadı. Örnek veri kullanılacak.", wasm_path);
        
        // Eğer WASM dosyası yoksa, basit bir wat kaynak kodu kullanarak
        // örnek bir modül oluştur (Örnek olarak toplama işlemi yapan modül)
        let wat_source = r#"
            (module
                (func $add (export "add") (param $a i32) (param $b i32) (result i32)
                    local.get $a
                    local.get $b
                    i32.add
                )
                
                (memory (export "memory") 1)
                
                (func $hello (export "hello") (result i32)
                    ;; "Hello, World!" mesajını belleğe yaz
                    (i32.const 0) ;; Bellek offseti
                    (i32.const 72)  ;; H
                    (i32.store8)
                    
                    (i32.const 1)
                    (i32.const 101) ;; e
                    (i32.store8)
                    
                    (i32.const 2)
                    (i32.const 108) ;; l
                    (i32.store8)
                    
                    (i32.const 3)
                    (i32.const 108) ;; l
                    (i32.store8)
                    
                    (i32.const 4)
                    (i32.const 111) ;; o
                    (i32.store8)
                    
                    (i32.const 5)
                    (i32.const 44)  ;; ,
                    (i32.store8)
                    
                    (i32.const 6)
                    (i32.const 32)  ;; space
                    (i32.store8)
                    
                    (i32.const 7)
                    (i32.const 87)  ;; W
                    (i32.store8)
                    
                    (i32.const 8)
                    (i32.const 111) ;; o
                    (i32.store8)
                    
                    (i32.const 9)
                    (i32.const 114) ;; r
                    (i32.store8)
                    
                    (i32.const 10)
                    (i32.const 108) ;; l
                    (i32.store8)
                    
                    (i32.const 11)
                    (i32.const 100) ;; d
                    (i32.store8)
                    
                    (i32.const 12)
                    (i32.const 33)  ;; !
                    (i32.store8)
                    
                    (i32.const 13)
                    (i32.const 0)   ;; null terminator
                    (i32.store8)
                    
                    (i32.const 0) ;; Bellek offset dönüş değeri
                )
            )
        "#;
        
        // WAT'ı WASM'e dönüştür
        let wasm_bytes = wat::parse_str(wat_source)?;
        
        // Yükleme seçenekleri
        let options = WasmLoadOptions {
            name: Some("sample_plugin".to_string()),
            id: Some("sample_plugin_1".to_string()),
            auto_start: true,
            ..Default::default()
        };
        
        // Modülü yükle
        let module_id = runtime.load_module_from_bytes(&wasm_bytes, Some(options)).await?;
        println!("WASM modülü yüklendi: {}", module_id);
        
        // add fonksiyonunu çağır
        let args = vec![wasmtime::Val::I32(5), wasmtime::Val::I32(7)];
        let result = runtime.call_function(&module_id, "add", args).await?;
        
        println!("add(5, 7) = {:?}", result[0]);
        
        // hello fonksiyonunu çağır
        let result = runtime.call_function(&module_id, "hello", vec![]).await?;
        
        // Bellek pointer'ını al
        if let wasmtime::Val::I32(ptr) = result[0] {
            // Belleği oku
            let memory_data = runtime.read_memory(&module_id, ptr as usize, 100).await?;
            
            // String'e dönüştür (null terminator'a kadar)
            let message = memory_data
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();
            
            println!("hello() = {}", message);
        }
    } else {
        // Gerçek bir WASM dosyası kullan
        let options = WasmLoadOptions {
            name: Some("real_plugin".to_string()),
            auto_start: true,
            ..Default::default()
        };
        
        // Modülü yükle
        let module_id = runtime.load_module_from_file(wasm_path, Some(options)).await?;
        println!("WASM modülü yüklendi: {}", module_id);
        
        // Modül fonksiyonlarını çağır
        // Not: Gerçek modülün API'sine göre uyarlanmalıdır
    }

    // Modülleri listele
    let module_ids = runtime.list_modules().await;
    println!("Yüklü modüller:");
    for id in &module_ids {
        let summary = runtime.get_module_summary(id).await?;
        println!("  {} ({}): {:?}", summary.name, id, summary.state);
        println!("  Dışa aktarılan fonksiyonlar: {:?}", summary.exports);
    }

    // İlk modülü durdur
    if !module_ids.is_empty() {
        let id = &module_ids[0];
        println!("Modül durduruluyor: {}", id);
        runtime.stop_module(id).await?;
        
        // Modül durumunu kontrol et
        let summary = runtime.get_module_summary(id).await?;
        println!("Modül durumu: {:?}", summary.state);
    }

    Ok(())
}
