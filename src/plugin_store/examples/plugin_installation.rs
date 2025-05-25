// Tauri Windows Plugin System - Plugin Installation Example
//
// Bu örnek, plugin indirme, kurma, güncelleme ve kaldırma işlemlerini gösterir.
// Ayrıca diğer modüllerle (Sandbox, Signature, Permission, Resource Monitor) entegrasyonu gösterir.

use tauri_plugin_store::{
    store_types::{PluginDownloadStatus, PluginInstallStatus, DownloadStatus, InstallStage},
    StoreClientConfig,
};
use std::error::Error;
use std::path::PathBuf;
use tauri::Manager;

// Diğer modüller ile entegrasyon
use tauri_plugin_signature::SignatureManager;
use tauri_plugin_permissions::PermissionManager;
use tauri_plugin_sandbox::SandboxManager;
use tauri_plugin_resource_monitor::ResourceMonitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Log sistemi başlat
    env_logger::init();
    
    println!("Plugin Installation Example");
    println!("==========================\n");
    
    // Store client konfigürasyonu
    let config = StoreClientConfig {
        api_url: "https://plugins.tauri-windows-plugin-system.dev/api".to_string(),
        api_key: Some("demo-api-key".to_string()),
        user_token: None,
        install_directory: None, // Varsayılanı kullan
        concurrent_downloads: Some(1),
        auto_check_updates: Some(true),
        auto_check_interval_hours: Some(24),
        trusted_certificates: Some(vec![
            include_str!("../../signature/examples/test_ca.pem").to_string(),
        ]),
    };
    
    // Tauri plugin'leri ile bir uygulama oluştur
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_store::init(config))
        .plugin(tauri_plugin_signature::init())
        .plugin(tauri_plugin_permissions::init())
        .plugin(tauri_plugin_sandbox::init())
        .plugin(tauri_plugin_resource_monitor::init())
        .build(tauri::generate_context!())
        .expect("Tauri uygulaması oluşturulamadı");
    
    // State'lerden bileşenleri al
    let store_state = app.state::<tauri_plugin_store::StoreClientState>();
    let mut store_client = store_state.store_client.write().await;
    
    let signature_state = app.state::<tauri_plugin_signature::SignatureState>();
    let signature_manager = signature_state.signature_manager.read().await;
    
    let permissions_state = app.state::<tauri_plugin_permissions::PermissionState>();
    let permission_manager = permissions_state.permission_manager.write().await;
    
    let sandbox_state = app.state::<tauri_plugin_sandbox::SandboxState>();
    let sandbox_manager = sandbox_state.sandbox_manager.write().await;
    
    let resource_state = app.state::<tauri_plugin_resource_monitor::ResourceMonitorState>();
    let resource_monitor = resource_state.resource_monitor.write().await;
    
    // Test için plugin ID
    let plugin_id = "com.example.security-scanner";
    
    // Plugin indir
    println!("Plugin indiriliyor: {}", plugin_id);
    
    match store_client.download_plugin(plugin_id, None).await {
        Ok(status) => {
            println!("İndirme başladı: {}", status.id);
            println!("Plugin: {} v{}", status.plugin_id, status.version);
            
            if status.status != DownloadStatus::Completed {
                println!("İndirme devam ediyor, bekleniyor...");
                // Gerçek uygulamada bu işlem event sistemi ile yapılabilir
                
                // İndirme tamamlandı mı kontrol et
                if let Some(path) = status.file_path {
                    println!("İndirme tamamlandı: {:?}", path);
                    
                    // İmza doğrulama
                    println!("\nPlugin imzası doğrulanıyor...");
                    
                    // Örnek imza bilgileri (gerçek uygulamada indirme bilgisinden alınır)
                    let signature_info = tauri_plugin_signature::SignatureInfo {
                        signature: "example-signature".to_string(),
                        algorithm: tauri_plugin_signature::SignatureAlgorithm::RsaPkcs1v15Sha256,
                        certificate_fingerprint: "example-fingerprint".to_string(),
                        signature_date: chrono::Utc::now(),
                    };
                    
                    // İmza doğrulama (örnek, gerçek uygulamada indirme bilgisinden alınır)
                    let cert_pem = include_str!("../../signature/examples/test_cert.pem");
                    match signature_manager.verify_signature(
                        &path, 
                        &signature_info, 
                        cert_pem,
                        Some(tauri_plugin_signature::TrustLevel::Trusted)
                    ) {
                        Ok(result) => {
                            if result.is_valid {
                                println!("İmza doğrulandı: {}", result.subject_name);
                                println!("İmzalayan: {}", result.issuer_name);
                                println!("Güven düzeyi: {:?}", result.trust_level);
                                
                                // Plugin kur
                                println!("\nPlugin kuruluyor...");
                                match store_client.install_plugin(plugin_id, None).await {
                                    Ok(install_status) => {
                                        print_install_progress(&install_status);
                                        
                                        if install_status.success {
                                            println!("\nPlugin başarıyla kuruldu: {}", install_status.name);
                                            
                                            // İzin kontrolü
                                            println!("\nPlugin izinleri kontrol ediliyor...");
                                            
                                            // Örnek plugin izin isteği
                                            let plugin_info = tauri_plugin_permissions::PluginInfo {
                                                id: plugin_id.to_string(),
                                                name: install_status.name.clone(),
                                                version: "1.0.0".to_string(),
                                                vendor: "Example Vendor".to_string(),
                                            };
                                            
                                            let permission_request = tauri_plugin_permissions::PermissionRequest {
                                                plugin: plugin_info.clone(),
                                                permissions: vec![
                                                    tauri_plugin_permissions::Permission::FileSystem {
                                                        scope: "read".to_string(),
                                                        paths: vec!["/temp".to_string()],
                                                    },
                                                    tauri_plugin_permissions::Permission::Network {
                                                        hosts: vec!["api.example.com".to_string()],
                                                    },
                                                ],
                                                reason: "Güvenlik taraması için gerekli".to_string(),
                                            };
                                            
                                            // İzin doğrulama
                                            match permission_manager.validate_permissions(&permission_request) {
                                                Ok(response) => {
                                                    println!("İzin yanıtı: {:?}", response.decision);
                                                    
                                                    if response.decision == tauri_plugin_permissions::Decision::Granted {
                                                        // Plugin çalıştır
                                                        println!("\nPlugin çalıştırılıyor...");
                                                        
                                                        // Sandbox oluştur
                                                        let sandbox_options = tauri_plugin_sandbox::SandboxOptions {
                                                            memory_limit: Some(100 * 1024 * 1024), // 100 MB
                                                            cpu_rate_limit: Some(50.0),           // %50 CPU
                                                            network_limit: Some(true),
                                                            file_system_limit: Some(true),
                                                            ..Default::default()
                                                        };
                                                        
                                                        match sandbox_manager.create_sandbox(plugin_id, sandbox_options) {
                                                            Ok(sandbox_id) => {
                                                                println!("Sandbox oluşturuldu: {}", sandbox_id);
                                                                
                                                                // Örnek plugin yolu (kurulu plugin dizininden alınır)
                                                                let plugin_path = PathBuf::from("/path/to/plugin/executable");
                                                                
                                                                // Plugin'i sandbox içinde çalıştır
                                                                match sandbox_manager.run_process(
                                                                    &sandbox_id, 
                                                                    &plugin_path, 
                                                                    &[],
                                                                    None
                                                                ) {
                                                                    Ok(process_id) => {
                                                                        println!("Plugin süreci başlatıldı: {}", process_id);
                                                                        
                                                                        // Kaynak izleme başlat
                                                                        match resource_monitor.start_monitoring(plugin_id, process_id) {
                                                                            Ok(_) => {
                                                                                println!("Kaynak izleme başlatıldı");
                                                                                
                                                                                // Örnek için bir süre bekle
                                                                                std::thread::sleep(std::time::Duration::from_secs(2));
                                                                                
                                                                                // Kaynak kullanımını al
                                                                                if let Ok(usage) = resource_monitor.get_resource_usage(plugin_id) {
                                                                                    println!("\nKaynak Kullanımı:");
                                                                                    println!("CPU: {:.2}%", usage.cpu_usage);
                                                                                    println!("Bellek: {:.2} MB", usage.memory_usage_mb);
                                                                                    println!("Ağ: {:.2} KB/s", usage.network_usage_kbps);
                                                                                }
                                                                                
                                                                                // İzlemeyi durdur
                                                                                let _ = resource_monitor.stop_monitoring(plugin_id);
                                                                                
                                                                                // Süreci sonlandır
                                                                                let _ = sandbox_manager.terminate_process(&sandbox_id, process_id);
                                                                                
                                                                                // Sandbox'ı kaldır
                                                                                let _ = sandbox_manager.destroy_sandbox(&sandbox_id);
                                                                            },
                                                                            Err(e) => {
                                                                                println!("Kaynak izleme başlatılamadı: {:?}", e);
                                                                            }
                                                                        }
                                                                    },
                                                                    Err(e) => {
                                                                        println!("Plugin süreci başlatılamadı: {:?}", e);
                                                                    }
                                                                }
                                                            },
                                                            Err(e) => {
                                                                println!("Sandbox oluşturulamadı: {:?}", e);
                                                            }
                                                        }
                                                    } else {
                                                        println!("Plugin için gerekli izinler verilmedi");
                                                    }
                                                },
                                                Err(e) => {
                                                    println!("İzin doğrulama hatası: {:?}", e);
                                                }
                                            }
                                            
                                            // Plugin güncelleme kontrolü
                                            println!("\nGüncelleme kontrolü yapılıyor...");
                                            match store_client.check_for_updates().await {
                                                Ok(updates) => {
                                                    if let Some(update) = updates.get(plugin_id) {
                                                        println!("Güncelleme bulundu: v{}", update.version);
                                                        println!("Güncelleme türü: {:?}", update.update_type);
                                                        println!("Sürüm notları: {}", update.release_notes);
                                                        
                                                        // Güncelleme kur
                                                        println!("\nPlugin güncelleniyor...");
                                                        match store_client.update_plugin(plugin_id).await {
                                                            Ok(update_status) => {
                                                                println!("Güncelleme durumu: {:?}", update_status.stage);
                                                                println!("Eski sürüm: {}", update_status.current_version);
                                                                println!("Yeni sürüm: {}", update_status.new_version);
                                                                
                                                                if update_status.success {
                                                                    println!("Plugin başarıyla güncellendi");
                                                                } else if let Some(error) = update_status.error {
                                                                    println!("Güncelleme hatası: {}", error);
                                                                }
                                                            },
                                                            Err(e) => {
                                                                println!("Güncelleme hatası: {:?}", e);
                                                            }
                                                        }
                                                    } else {
                                                        println!("Plugin güncel");
                                                    }
                                                },
                                                Err(e) => {
                                                    println!("Güncelleme kontrolü hatası: {:?}", e);
                                                }
                                            }
                                            
                                            // Plugin kaldır
                                            println!("\nPlugin kaldırılıyor...");
                                            match store_client.uninstall_plugin(plugin_id).await {
                                                Ok(success) => {
                                                    if success {
                                                        println!("Plugin başarıyla kaldırıldı");
                                                    } else {
                                                        println!("Plugin kaldırılamadı");
                                                    }
                                                },
                                                Err(e) => {
                                                    println!("Plugin kaldırma hatası: {:?}", e);
                                                }
                                            }
                                        } else if let Some(error) = install_status.error {
                                            println!("Kurulum hatası: {}", error);
                                        }
                                    },
                                    Err(e) => {
                                        println!("Kurulum hatası: {:?}", e);
                                    }
                                }
                            } else {
                                println!("İmza doğrulanamadı: {:?}", result.error);
                            }
                        },
                        Err(e) => {
                            println!("İmza doğrulama hatası: {:?}", e);
                        }
                    }
                } else {
                    println!("İndirme tamamlanamadı");
                    if let Some(error) = status.error {
                        println!("Hata: {}", error);
                    }
                }
            }
        },
        Err(e) => {
            println!("İndirme hatası: {:?}", e);
        }
    }
    
    println!("\nÖrnek tamamlandı!");
    
    Ok(())
}

// Kurulum ilerlemesini göster
fn print_install_progress(status: &PluginInstallStatus) {
    println!("Kurulum aşaması: {:?} ({:.1}%)", status.stage, status.progress);
    
    match status.stage {
        InstallStage::Validating => println!("Plugin paketi doğrulanıyor..."),
        InstallStage::Extracting => println!("Paket içeriği çıkarılıyor..."),
        InstallStage::InstallingDependencies => println!("Bağımlılıklar kuruluyor..."),
        InstallStage::Configuring => println!("Plugin yapılandırılıyor..."),
        InstallStage::Completed => println!("Kurulum tamamlandı"),
        InstallStage::Failed => println!("Kurulum başarısız: {:?}", status.error),
    }
}
