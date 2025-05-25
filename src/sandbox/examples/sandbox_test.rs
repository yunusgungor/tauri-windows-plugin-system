// Tauri Windows Plugin System - Sandbox Test Uygulaması
//
// Bu örnek uygulama, Windows Job Objects kullanarak plugin sandbox izolasyonunu test eder.
// Varsayılan bir Windows uygulamasını (Notepad) başlatır ve sandbox içine alır.

use std::process::Command;
use std::thread;
use std::time::Duration;
use tauri_plugin_sandbox::{PermissionLevel, ResourceLimits, SandboxManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tauri Windows Plugin Sandbox Test Uygulaması");
    println!("=============================================");
    
    // Sandbox Manager oluştur
    let sandbox_manager = SandboxManager::new();
    
    // Test için bir uygulama başlat (Windows'ta notepad.exe)
    println!("Test uygulaması başlatılıyor...");
    let child = match Command::new("notepad.exe").spawn() {
        Ok(child) => child,
        Err(e) => {
            println!("Test uygulaması başlatılamadı: {}", e);
            return Err(Box::new(e));
        }
    };
    
    let pid = child.id();
    println!("Test uygulaması başlatıldı. PID: {}", pid);
    
    // Sandbox kaynak sınırlamaları
    let resource_limits = ResourceLimits {
        max_memory_mb: 128,          // 128 MB bellek limiti
        max_cpu_percentage: 20,      // %20 CPU kullanımı
        max_process_count: 2,        // Maksimum 2 process
        max_working_set_mb: 64,      // 64 MB working set
    };
    
    // İzinler
    let permissions = vec![
        PermissionLevel::Core,
        PermissionLevel::UI,
        // PermissionLevel::Filesystem, // Dosya sistemi erişimi yok
        // PermissionLevel::Network,    // Ağ erişimi yok
    ];
    
    // Uygulamayı sandbox'la
    println!("Uygulama sandbox'lanıyor...");
    match sandbox_manager.sandbox_plugin("test-notepad", pid, Some(resource_limits), permissions) {
        Ok(_) => println!("Uygulama başarıyla sandbox'landı"),
        Err(e) => {
            println!("Sandbox oluşturma hatası: {:?}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Sandbox hatası: {:?}", e),
            )));
        }
    }
    
    // Sandbox durumunu göster
    println!("\nSandbox Durumu:");
    println!("---------------");
    
    // Aktif sandbox'ları listele
    let plugins = sandbox_manager.list_sandboxed_plugins();
    println!("Aktif sandbox'lar: {:?}", plugins);
    
    // Test-notepad sandbox durumunu kontrol et
    if let Some((process_id, limits, perms)) = sandbox_manager.get_plugin_sandbox_status("test-notepad") {
        println!("Plugin: test-notepad");
        println!("  Process ID: {}", process_id);
        println!("  Bellek Limiti: {} MB", limits.max_memory_mb);
        println!("  Working Set Limiti: {} MB", limits.max_working_set_mb);
        println!("  Process Sayısı Limiti: {}", limits.max_process_count);
        println!("  CPU Limiti: %{}", limits.max_cpu_percentage);
        println!("  İzinler: {:?}", perms);
    } else {
        println!("test-notepad sandbox durumu alınamadı");
    }
    
    // İzin kontrolü
    println!("\nİzin Kontrolleri:");
    println!("---------------");
    
    for perm in [
        PermissionLevel::Core,
        PermissionLevel::Filesystem,
        PermissionLevel::Network,
        PermissionLevel::UI,
        PermissionLevel::System,
        PermissionLevel::Interprocess,
    ] {
        match sandbox_manager.check_permission("test-notepad", perm.clone()) {
            Ok(true) => println!("  {:?}: İzin VAR", perm),
            Ok(false) => println!("  {:?}: İzin YOK", perm),
            Err(e) => println!("  {:?}: Kontrol hatası - {:?}", perm, e),
        }
    }
    
    // Kullanıcı işlem yapması için bekle
    println!("\nTest uygulaması 30 saniye boyunca çalışacak...");
    println!("(Not: Bu süre içinde Notepad'in dosya oluşturma veya internet erişimi gibi işlemleri");
    println!("deneyerek sandbox sınırlamalarını test edebilirsiniz)");
    
    // 30 saniye bekle
    thread::sleep(Duration::from_secs(30));
    
    // Sandbox'ı kaldır
    println!("\nSandbox kaldırılıyor...");
    match sandbox_manager.remove_sandbox("test-notepad") {
        Ok(_) => println!("Sandbox başarıyla kaldırıldı"),
        Err(e) => println!("Sandbox kaldırma hatası: {:?}", e),
    }
    
    // Process'i sonlandır
    println!("Test uygulaması sonlandırılıyor...");
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .output();
    
    println!("Test tamamlandı.");
    Ok(())
}
