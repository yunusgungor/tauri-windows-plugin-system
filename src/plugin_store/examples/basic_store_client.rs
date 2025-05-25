// Tauri Windows Plugin System - Plugin Store Client Example
//
// Bu örnek, plugin mağazasında arama yapma ve plugin detaylarını görüntüleme
// işlevlerini gösterir.

use tauri_plugin_store::{
    store_types::{PluginCategory, PluginSearchFilter, PluginSortType},
    StoreClientConfig,
};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Log sistemi başlat
    env_logger::init();
    
    println!("Plugin Store Client Example");
    println!("==========================\n");
    
    // Store client konfigürasyonu
    let config = StoreClientConfig {
        api_url: "https://plugins.tauri-windows-plugin-system.dev/api".to_string(),
        api_key: Some("demo-api-key".to_string()),
        user_token: None,
        install_directory: None, // Varsayılanı kullan
        concurrent_downloads: Some(2),
        auto_check_updates: Some(true),
        auto_check_interval_hours: Some(12),
        trusted_certificates: None,
    };
    
    // Tauri plugin oluştur
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_store::init(config))
        .build(tauri::generate_context!())
        .expect("Tauri uygulaması oluşturulamadı");
    
    // State'ten Store Client'ı al
    let state = app.state::<tauri_plugin_store::StoreClientState>();
    let store_client = state.store_client.read().await;
    
    // Plugin arama
    println!("Mağazada plugin aranıyor...");
    
    let search_filter = PluginSearchFilter {
        query: Some("security".to_string()),
        categories: Some(vec![PluginCategory::Security]),
        plugin_type: None,
        vendor_id: None,
        free_only: Some(true),
        sort_by: Some(PluginSortType::Rating),
        page: Some(1),
        page_size: Some(5),
    };
    
    match store_client.search_plugins(search_filter).await {
        Ok(result) => {
            println!("\nToplam {} plugin bulundu:", result.total_count);
            println!("Sayfa {}/{}\n", result.current_page, result.total_pages);
            
            for plugin in result.items {
                println!("- {} (v{})", plugin.name, plugin.version);
                println!("  ID: {}", plugin.id);
                println!("  Açıklama: {}", plugin.description);
                println!("  Geliştirici: {}", plugin.vendor.name);
                println!("  Puan: {:.1} ({} değerlendirme)", 
                    plugin.rating.average_rating, 
                    plugin.rating.total_ratings);
                println!();
            }
            
            // İlk plugin'in detaylarını göster
            if !result.items.is_empty() {
                let first_plugin = &result.items[0];
                println!("\nİlk plugin detayları alınıyor: {}", first_plugin.name);
                
                match store_client.get_plugin_details(&first_plugin.id).await {
                    Ok(details) => {
                        println!("\nPlugin Detayları: {}", details.name);
                        println!("===================={}", "=".repeat(details.name.len()));
                        println!("Sürüm: {}", details.version);
                        println!("Tür: {:?}", details.plugin_type);
                        println!("Kategori: {:?}", details.category);
                        println!("Açıklama: {}", details.description);
                        println!("Geliştirici: {} {}", 
                            details.vendor.name,
                            if details.vendor.verified { "(Doğrulanmış)" } else { "" });
                        
                        if let Some(website) = &details.vendor.website {
                            println!("Web sitesi: {}", website);
                        }
                        
                        println!("\nFiyatlandırma: {:?}", details.pricing.model);
                        if let Some(price) = details.pricing.price {
                            println!("Fiyat: {} {}", 
                                price, 
                                details.pricing.currency.as_deref().unwrap_or("USD"));
                        }
                        
                        println!("\nİzinler:");
                        for perm in details.permissions {
                            println!("- {} ({}): {}", 
                                perm.permission_type,
                                if perm.required { "Zorunlu" } else { "Opsiyonel" },
                                perm.reason);
                        }
                        
                        println!("\nBağımlılıklar:");
                        if details.dependencies.is_empty() {
                            println!("- Yok");
                        } else {
                            for dep in details.dependencies {
                                println!("- {} ({}): {}", 
                                    dep.id,
                                    dep.version_requirement,
                                    dep.description.unwrap_or_default());
                            }
                        }
                    },
                    Err(e) => {
                        println!("Plugin detayları alınamadı: {:?}", e);
                    }
                }
            }
        },
        Err(e) => {
            println!("Plugin arama hatası: {:?}", e);
        }
    }
    
    // Kurulu pluginleri göster
    match store_client.get_installed_plugins().await {
        Ok(plugins) => {
            println!("\nKurulu Pluginler:");
            if plugins.is_empty() {
                println!("- Kurulu plugin yok");
            } else {
                for plugin in plugins {
                    println!("- {} (v{}): {}", 
                        plugin.name, 
                        plugin.installed_version,
                        if plugin.enabled { "Aktif" } else { "Pasif" });
                }
            }
        },
        Err(e) => {
            println!("Kurulu pluginler alınamadı: {:?}", e);
        }
    }
    
    println!("\nÖrnek tamamlandı!");
    
    Ok(())
}
