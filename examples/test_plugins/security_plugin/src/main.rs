// Tauri Windows Plugin System - Security Scanner Test Plugin Console App
//
// Bu uygulama, Security Scanner Plugin'i bağımsız olarak test eder.

use plugin_interface::PluginInterface;
use security_scanner_plugin::SecurityScannerPlugin;
use serde_json::Value;
use std::{env, path::Path, thread, time::Duration};

fn main() {
    println!("Tauri Windows Plugin System - Security Scanner Test Plugin");
    println!("======================================================\n");
    
    // Plugin oluştur
    let mut plugin = SecurityScannerPlugin::new();
    
    // Metadata göster
    let metadata = plugin.get_metadata();
    println!("Plugin: {} ({})", metadata.name, metadata.id);
    println!("Versiyon: {}", metadata.version);
    println!("Açıklama: {}", metadata.description);
    println!("Tip: {}", metadata.plugin_type);
    println!("Geliştirici: {}", metadata.vendor);
    
    if let Some(url) = metadata.vendor_url {
        println!("Geliştirici URL: {}", url);
    }
    
    println!("\nİzinler:");
    for permission in metadata.permissions {
        println!("- {}", permission);
    }
    
    // Plugin'i başlat
    println!("\nPlugin başlatılıyor...");
    match plugin.initialize() {
        Ok(_) => println!("Plugin başarıyla başlatıldı."),
        Err(e) => {
            eprintln!("Plugin başlatma hatası: {:?}", e);
            return;
        }
    }
    
    // İzin durumunu al
    println!("\nİzin durumları alınıyor...");
    match plugin.execute_command("get_permissions", "") {
        Ok(json) => {
            let permissions: Value = serde_json::from_str(&json).unwrap();
            println!("İzin durumları:");
            for (id, permission) in permissions.as_object().unwrap() {
                println!("- {}: {}", id, if permission["approved"].as_bool().unwrap_or(false) {
                    "Onaylandı"
                } else {
                    "Onaylanmadı"
                });
            }
        },
        Err(e) => eprintln!("İzin durumu alma hatası: {:?}", e),
    }
    
    // İzinleri onayla
    println!("\nİzinler onaylanıyor...");
    for permission_id in &["fs.read", "process.query", "network.check", "registry.read"] {
        match plugin.execute_command("approve_permission", permission_id) {
            Ok(_) => println!("- {}: Onaylandı", permission_id),
            Err(e) => eprintln!("- {}: Onaylama hatası: {:?}", permission_id, e),
        }
    }
    
    // İmza doğrulama testi
    println!("\nİmza doğrulama testi yapılıyor...");
    
    // Geçerli imza testi
    let valid_signature_args = r#"{
        "signature": "3081890281810089E368EF09C84B8CEA598C2092015F92F546BBDCE8A94A337B52F22C96AB8A157F62D843",
        "data": "Test veri",
        "key_id": "test-app-v1.0.0"
    }"#;
    
    match plugin.execute_command("verify_signature", valid_signature_args) {
        Ok(json) => {
            let result: Value = serde_json::from_str(&json).unwrap();
            println!("Geçerli imza doğrulama sonucu:");
            println!("- Geçerli: {}", result["is_valid"]);
            println!("- İmzalayan: {}", result["signer"]);
            println!("- Mesaj: {}", result["message"]);
        },
        Err(e) => eprintln!("İmza doğrulama hatası: {:?}", e),
    }
    
    // Geçersiz imza testi
    let invalid_signature_args = r#"{
        "signature": "INVALID_SIGNATURE_DATA_HERE",
        "data": "Test veri",
        "key_id": "test-app-v1.0.0"
    }"#;
    
    match plugin.execute_command("verify_signature", invalid_signature_args) {
        Ok(json) => {
            let result: Value = serde_json::from_str(&json).unwrap();
            println!("\nGeçersiz imza doğrulama sonucu:");
            println!("- Geçerli: {}", result["is_valid"]);
            if !result["is_valid"].as_bool().unwrap_or(false) {
                println!("- Mesaj: {}", result["message"]);
            }
        },
        Err(e) => eprintln!("İmza doğrulama hatası: {:?}", e),
    }
    
    // Güvenlik taraması testi
    println!("\nGüvenlik taraması başlatılıyor...");
    
    // Taranacak dizini belirle
    let scan_dir = env::args().nth(1).unwrap_or_else(|| {
        // Varsayılan olarak geçerli dizini tara
        env::current_dir().unwrap().to_string_lossy().to_string()
    });
    
    println!("Taranacak dizin: {}", scan_dir);
    
    // Taramayı başlat
    match plugin.execute_command("start_scan", &scan_dir) {
        Ok(scan_id) => {
            println!("Tarama başlatıldı. Tarama ID: {}", scan_id);
            
            // Tarama tamamlanana kadar bekle
            println!("\nTarama sonuçları bekleniyor...");
            let mut completed = false;
            let mut attempt = 0;
            
            while !completed && attempt < 10 {
                thread::sleep(Duration::from_secs(1));
                attempt += 1;
                
                match plugin.execute_command("get_scan_result", &scan_id) {
                    Ok(json) => {
                        let result: Value = serde_json::from_str(&json).unwrap();
                        completed = result["completed"].as_bool().unwrap_or(false);
                        
                        if completed {
                            println!("\nTarama tamamlandı!");
                            println!("Güvenlik puanı: {}", result["security_score"]);
                            println!("Tarama süresi: {} ms", result["duration_ms"]);
                            
                            if let Some(issues) = result["issues"].as_array() {
                                println!("\nTespit edilen güvenlik sorunları: {}", issues.len());
                                
                                if !issues.is_empty() {
                                    for (i, issue) in issues.iter().enumerate() {
                                        println!("\nSorun #{}:", i + 1);
                                        println!("- Tip: {}", issue["issue_type"]);
                                        println!("- Açıklama: {}", issue["description"]);
                                        println!("- Şiddet: {}", issue["severity"]);
                                        if let Some(location) = issue["location"].as_str() {
                                            println!("- Konum: {}", location);
                                        }
                                        println!("- Öneri: {}", issue["recommendation"]);
                                        if let Some(cvss) = issue["cvss_score"].as_f64() {
                                            println!("- CVSS Puanı: {:.1}", cvss);
                                        }
                                    }
                                } else {
                                    println!("Güvenlik sorunu tespit edilmedi!");
                                }
                            }
                            
                            break;
                        } else {
                            print!(".");
                        }
                    },
                    Err(e) => {
                        eprintln!("\nTarama sonucu alma hatası: {:?}", e);
                        break;
                    },
                }
            }
            
            if !completed {
                println!("\nTarama zaman aşımına uğradı.");
            }
        },
        Err(e) => eprintln!("Tarama başlatma hatası: {:?}", e),
    }
    
    // Plugin'i kapat
    println!("\nPlugin kapatılıyor...");
    match plugin.shutdown() {
        Ok(_) => println!("Plugin başarıyla kapatıldı."),
        Err(e) => eprintln!("Plugin kapatma hatası: {:?}", e),
    }
    
    println!("\nTest tamamlandı!");
}
