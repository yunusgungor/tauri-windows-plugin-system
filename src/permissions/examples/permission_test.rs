// Tauri Windows Plugin System - İzin Test Uygulaması
//
// Bu örnek uygulama, gelişmiş izin sisteminin nasıl kullanılacağını gösterir.
// Farklı izin seviyelerini ve kullanıcı etkileşimini test eder.

use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_permissions::{
    FilesystemScope, NetworkScope, PermissionCategory, PermissionDescriptor,
    PermissionManager, PermissionManagerConfig, PermissionPolicy, PermissionPrompt,
    PermissionStore, PromptStyle, EnforcementLevel,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tauri Windows Plugin İzin Test Uygulaması");
    println!("=========================================");
    
    // Test ortamı oluştur
    let temp_dir = std::env::temp_dir().join("tauri_permissions_test");
    std::fs::create_dir_all(&temp_dir)?;
    println!("Test dizini oluşturuldu: {:?}", temp_dir);
    
    // Tauri test uygulamasını başlat
    println!("\nTest Tauri uygulaması başlatılıyor...");
    let builder = tauri::Builder::default()
        .setup(|app| {
            println!("Tauri uygulaması başlatıldı.");
            Ok(())
        });
    
    // Manuel olarak test ortamı oluştur
    let app = builder.build(tauri::generate_context!()).unwrap();
    let app_handle = app.app_handle();
    
    // İzin alt sistemini oluştur
    let store = Arc::new(PermissionStore::new(temp_dir.clone())?);
    let prompt = Arc::new(PermissionPrompt::new(app_handle.clone()));
    
    // Test yapılandırması
    let config = PermissionManagerConfig {
        policy: PermissionPolicy::AskOnce,
        enforcement_level: EnforcementLevel::Normal,
        storage_dir: temp_dir.clone(),
        default_token_duration: Some(3600),  // 1 saat
        request_timeout: 60,                 // 60 saniye
        trusted_developers: vec!["test-developer".to_string()],
        prompt_style: PromptStyle::Modal,
    };
    
    let permission_manager = Arc::new(PermissionManager::new(
        config,
        store.clone(),
        prompt.clone(),
    ));
    
    // Test plugin bilgilerini kaydet
    let mut plugin_store = store.clone();
    plugin_store.save_plugin_info(&tauri_plugin_permissions::PluginInfo {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        developer: "Test Developer".to_string(),
        icon_url: None,
    })?;
    
    println!("\nTest plugin kaydedildi.");
    
    // Birkaç izin tanımlayıcısı oluştur
    let filesystem_descriptor = PermissionDescriptor {
        category: PermissionCategory::Filesystem,
        scope: FilesystemScope::READ_PLUGIN_DATA.bits() | FilesystemScope::WRITE_PLUGIN_DATA.bits(),
        reason: "Plugin verilerini okumak ve yazmak için".to_string(),
    };
    
    let network_descriptor = PermissionDescriptor {
        category: PermissionCategory::Network,
        scope: NetworkScope::HTTPS.bits(),
        reason: "Güvenli API istekleri göndermek için".to_string(),
    };
    
    // Farklı politikalarla izin talepleri simülasyonu
    println!("\nOtomatik İzin Verme Testi:");
    println!("-------------------------");
    {
        let mut config = permission_manager.config.write();
        config.policy = PermissionPolicy::AutoGrant;
    }
    
    match permission_manager.check_permissions("test-plugin", vec![filesystem_descriptor.clone()]).await {
        Ok(token) => {
            println!("AutoGrant politikası başarılı - İzin otomatik verildi");
            println!("  Belirteç ID: {}", token.id);
            println!("  Plugin ID: {}", token.plugin_id);
            println!("  Oluşturma Zamanı: {}", token.created_at);
            if let Some(expires) = token.expires_at {
                println!("  Sona Erme Zamanı: {}", expires);
            }
        },
        Err(e) => {
            println!("AutoGrant politikası hatası: {:?}", e);
        }
    }
    
    println!("\nOtomatik İzin Reddetme Testi:");
    println!("---------------------------");
    {
        let mut config = permission_manager.config.write();
        config.policy = PermissionPolicy::AutoDeny;
        permission_manager.remove_permission_token("test-plugin");
    }
    
    match permission_manager.check_permissions("test-plugin", vec![filesystem_descriptor.clone()]).await {
        Ok(_) => {
            println!("Beklenmeyen başarı - İzin verilmemeliydi");
        },
        Err(e) => {
            println!("AutoDeny politikası başarılı - İzin otomatik reddedildi");
            println!("  Hata: {:?}", e);
        }
    }
    
    // İzin kontrolü testi
    println!("\nİzin Kontrolü Testi:");
    println!("------------------");
    {
        let mut config = permission_manager.config.write();
        config.policy = PermissionPolicy::AutoGrant;
    }
    
    // İzin ver
    let _ = permission_manager.check_permissions("test-plugin", vec![filesystem_descriptor.clone()]).await?;
    
    // İzin kontrolü
    match permission_manager.has_permission(
        "test-plugin",
        PermissionCategory::Filesystem,
        FilesystemScope::READ_PLUGIN_DATA.bits(),
    ) {
        Ok(true) => println!("İzin kontrolü başarılı - READ_PLUGIN_DATA izni var"),
        Ok(false) => println!("İzin kontrolü başarısız - READ_PLUGIN_DATA izni olmalıydı"),
        Err(e) => println!("İzin kontrolü hatası: {:?}", e),
    }
    
    match permission_manager.has_permission(
        "test-plugin",
        PermissionCategory::Network,
        NetworkScope::HTTPS.bits(),
    ) {
        Ok(true) => println!("Beklenmeyen başarı - HTTPS izni olmamalıydı"),
        Ok(false) => println!("İzin kontrolü başarılı - HTTPS izni yok (beklenen)"),
        Err(e) => println!("İzin kontrolü hatası: {:?}", e),
    }
    
    // İzin iptal etme testi
    println!("\nİzin İptal Testi:");
    println!("---------------");
    
    permission_manager.remove_permission_token("test-plugin");
    println!("Test plugin izni iptal edildi");
    
    match permission_manager.has_permission(
        "test-plugin",
        PermissionCategory::Filesystem,
        FilesystemScope::READ_PLUGIN_DATA.bits(),
    ) {
        Ok(_) => println!("Beklenmeyen başarı - İzin iptal edilmişti"),
        Err(e) => println!("İzin iptal testi başarılı: {:?}", e),
    }
    
    // Temizlik
    println!("\nTest temizleniyor...");
    std::fs::remove_dir_all(temp_dir)?;
    
    println!("\nTest tamamlandı!");
    Ok(())
}
