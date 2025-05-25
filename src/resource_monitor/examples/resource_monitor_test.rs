// Tauri Windows Plugin System - Kaynak İzleyici Test
//
// Bu örnek, plugin'lerin kaynak kullanımını izlemek için Resource Monitor
// modülünün nasıl kullanılacağını gösterir.

use std::path::Path;
use std::time::Duration;
use tauri_plugin_resource_monitor::{
    LimitAction, ResourceLimit, ResourceMonitorConfig, ResourceType, ResourceUnit,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logları etkinleştir
    env_logger::init();
    println!("Tauri Windows Plugin System - Kaynak İzleyici Test");

    // Kaynak izleme konfigürasyonu
    let config = ResourceMonitorConfig {
        monitoring_interval_ms: 1000, // 1 saniye
        resources_to_monitor: vec![
            ResourceType::CpuUsage,
            ResourceType::MemoryUsage,
            ResourceType::ThreadCount,
            ResourceType::HandleCount,
        ],
        resource_limits: vec![
            ResourceLimit {
                resource_type: ResourceType::CpuUsage,
                soft_limit: 70.0,  // %70 CPU kullanımı yumuşak limit
                hard_limit: 90.0,  // %90 CPU kullanımı sert limit
                measurement_period: 5, // 5 saniye ölçüm periyodu
                action: LimitAction::Throttle, // Sınırlama uygula
            },
            ResourceLimit {
                resource_type: ResourceType::MemoryUsage,
                soft_limit: 200.0, // 200 MB bellek kullanımı yumuşak limit
                hard_limit: 500.0, // 500 MB bellek kullanımı sert limit
                measurement_period: 5, // 5 saniye ölçüm periyodu
                action: LimitAction::Terminate, // Sonlandır
            },
        ],
        auto_monitoring: true,
        history_retention_days: 7,
        notify_on_limit_breach: true,
        gather_statistics: true,
    };

    // Resource Monitor'ü başlat
    let monitor = tauri_plugin_resource_monitor::monitor::ResourceMonitor::new(config).await?;
    println!("Kaynak izleyici başlatıldı.");

    // Test plugin'i olarak Notepad'i başlat
    let notepad_path = Path::new("C:\\Windows\\System32\\notepad.exe");
    
    println!("Test uygulaması başlatılıyor: {:?}", notepad_path);
    let mut child = tokio::process::Command::new(notepad_path)
        .spawn()
        .expect("Notepad başlatılamadı");
    
    let process_id = child.id().expect("Process ID alınamadı") as u32;
    println!("Notepad başlatıldı. PID: {}", process_id);

    // Plugin'i izlemeye başla
    monitor.start_monitoring("test-plugin", process_id).await?;
    println!("Notepad izlemeye alındı.");

    // Birkaç ölçüm yap
    println!("\n-- Kaynak Kullanımı İzleniyor --");
    for i in 0..10 {
        // 1 saniye bekle
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Manuel olarak CPU ve bellek kullanımını ölç
        monitor.measure("test-plugin", ResourceType::CpuUsage).await?;
        monitor.measure("test-plugin", ResourceType::MemoryUsage).await?;
        
        // Kullanım profilini al
        if let Some(profile) = monitor.get_usage_profile("test-plugin") {
            println!("Ölçüm #{}", i + 1);
            
            if let Some(cpu) = profile.current_usage(ResourceType::CpuUsage) {
                println!("  CPU Kullanımı: {:.2}%", cpu);
            }
            
            if let Some(memory) = profile.current_usage(ResourceType::MemoryUsage) {
                println!("  Bellek Kullanımı: {:.2} MB", memory);
            }
            
            if let Some(threads) = profile.current_usage(ResourceType::ThreadCount) {
                println!("  Thread Sayısı: {}", threads as u32);
            }
            
            if let Some(handles) = profile.current_usage(ResourceType::HandleCount) {
                println!("  Handle Sayısı: {}", handles as u32);
            }
        }
        
        // Limit aşım olaylarını kontrol et
        let events = monitor.get_limit_events("test-plugin");
        if !events.is_empty() {
            println!("\n-- Limit Aşım Olayları --");
            for event in &events {
                println!(
                    "  {} - {}: {:.2} > {:.2} ({:.2}% aşım)",
                    event.timestamp.format("%H:%M:%S"),
                    event.resource_type.description(),
                    event.actual_value,
                    event.limit,
                    event.overage_percent
                );
                println!("  Uygulanan Eylem: {:?}", event.action_taken);
            }
        }
        
        println!("");
    }

    // İzlemeyi durdur
    monitor.stop_monitoring("test-plugin").await?;
    println!("Notepad izleme durduruldu.");

    // Notepad'i kapat
    child.kill().await?;
    println!("Notepad kapatıldı.");

    // Kaynak izleyiciyi kapat
    monitor.shutdown().await?;
    println!("Kaynak izleyici kapatıldı.");

    Ok(())
}
