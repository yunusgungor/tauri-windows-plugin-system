// Tauri Windows Plugin System - Resource Usage Test Plugin Console App
//
// Bu uygulama, Resource Usage Plugin'i bağımsız olarak test eder.

use plugin_interface::{PluginInterface, ResourceMonitorPlugin, ResourceUsage};
use resource_usage_plugin::ResourceUsagePlugin;
use std::{thread, time::Duration};

fn main() {
    println!("Tauri Windows Plugin System - Resource Usage Test Plugin");
    println!("======================================================\n");
    
    // Plugin oluştur
    let mut plugin = ResourceUsagePlugin::new();
    
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
    
    // Kaynak kullanımı ölç
    println!("\nKaynak kullanımı ölçülüyor...");
    match plugin.measure_resource_usage() {
        Ok(usage) => print_resource_usage(&usage),
        Err(e) => eprintln!("Kaynak ölçüm hatası: {:?}", e),
    }
    
    // Kaynak sınırlarını ayarla
    println!("\nKaynak sınırları ayarlanıyor...");
    match plugin.set_resource_limits(Some(80.0), Some(500 * 1024 * 1024)) {
        Ok(_) => println!("Kaynak sınırları ayarlandı: CPU <= 80%, Bellek <= 500 MB"),
        Err(e) => eprintln!("Sınır ayarlama hatası: {:?}", e),
    }
    
    // İzlemeyi başlat
    println!("\nKaynak izleme başlatılıyor...");
    match plugin.start_monitoring() {
        Ok(_) => println!("Kaynak izleme başlatıldı."),
        Err(e) => {
            eprintln!("İzleme başlatma hatası: {:?}", e);
            return;
        }
    }
    
    // Yüksek CPU kullanımını simüle et
    println!("\nYüksek CPU kullanımı simüle ediliyor...");
    match plugin.execute_command("simulate_high_cpu", "") {
        Ok(_) => println!("Yüksek CPU simülasyonu başlatıldı."),
        Err(e) => eprintln!("Simülasyon hatası: {:?}", e),
    }
    
    // Bir süre bekle ve kaynak kullanımını izle
    println!("\nKaynak kullanımı izleniyor (5 saniye)...");
    for i in 0..5 {
        thread::sleep(Duration::from_secs(1));
        
        match plugin.measure_resource_usage() {
            Ok(usage) => {
                println!("\nÖlçüm #{}", i + 1);
                print_resource_usage(&usage);
            },
            Err(e) => eprintln!("Kaynak ölçüm hatası: {:?}", e),
        }
    }
    
    // Yüksek bellek kullanımını simüle et
    println!("\nYüksek bellek kullanımı simüle ediliyor...");
    match plugin.execute_command("simulate_high_memory", "200000000") { // 200 MB
        Ok(_) => println!("Yüksek bellek simülasyonu başlatıldı."),
        Err(e) => eprintln!("Simülasyon hatası: {:?}", e),
    }
    
    // Bir süre bekle ve kaynak kullanımını izle
    println!("\nKaynak kullanımı izleniyor (5 saniye)...");
    for i in 0..5 {
        thread::sleep(Duration::from_secs(1));
        
        match plugin.measure_resource_usage() {
            Ok(usage) => {
                println!("\nÖlçüm #{}", i + 1);
                print_resource_usage(&usage);
            },
            Err(e) => eprintln!("Kaynak ölçüm hatası: {:?}", e),
        }
    }
    
    // İzlemeyi durdur
    println!("\nKaynak izleme durduruluyor...");
    match plugin.stop_monitoring() {
        Ok(_) => println!("Kaynak izleme durduruldu."),
        Err(e) => eprintln!("İzleme durdurma hatası: {:?}", e),
    }
    
    // Plugin'i kapat
    println!("\nPlugin kapatılıyor...");
    match plugin.shutdown() {
        Ok(_) => println!("Plugin başarıyla kapatıldı."),
        Err(e) => eprintln!("Plugin kapatma hatası: {:?}", e),
    }
    
    println!("\nTest tamamlandı!");
}

// Kaynak kullanımını göster
fn print_resource_usage(usage: &ResourceUsage) {
    println!("CPU Kullanımı: {:.2}%", usage.cpu_percent);
    println!("Bellek Kullanımı: {:.2} MB", usage.memory_bytes as f64 / (1024.0 * 1024.0));
    println!("Disk I/O: {:.2} MB/s", usage.disk_bytes_per_sec as f64 / (1024.0 * 1024.0));
    println!("Ağ I/O: {:.2} KB/s", usage.network_bytes_per_sec as f64 / 1024.0);
}
