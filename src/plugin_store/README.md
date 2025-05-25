# Tauri Windows Plugin System - Plugin Store Client

Bu modül, Tauri Windows Plugin System için merkezi bir eklenti mağazası istemcisi sağlar. Eklentileri keşfetme, indirme, kurma, güncelleme ve yönetme işlevlerini içerir.

## Özellikler

- **Eklenti Keşfi**: Merkezi bir mağazadan eklentileri arama ve filtreleme
- **Güvenli İndirme**: Doğrulama ve imza kontrolü ile güvenli eklenti indirme
- **Kurulum Yönetimi**: Eklenti kurulumu, güncelleme ve kaldırma işlemleri
- **Güvenlik Entegrasyonu**: Diğer güvenlik modülleriyle (Sandbox, Signature, Permissions) entegrasyon
- **Kaynak İzleme**: Resource Monitor ile eklenti kaynak kullanımını izleme
- **Oturum Yönetimi**: Kimlik doğrulama ve kullanıcı yönetimi
- **Yerel Kayıt**: Kurulu eklentilerin yerel veritabanı
- **Otomatik Güncellemeler**: Kurulu eklentiler için güncelleme kontrolü ve uygulama

## Mimari

Plugin Store Client modülü, aşağıdaki bileşenlerden oluşan modüler bir mimariye sahiptir:

1. **Store Client**: Ana istemci sınıfı, diğer bileşenlerle iletişim kurarak eklenti yönetimini koordine eder.
2. **API Client**: Eklenti mağazası API'si ile iletişim kuran fonksiyonları içerir, HTTP isteklerini yönetir.
3. **Auth Manager**: Kimlik doğrulama ve token yönetimini sağlar, API anahtarlarını ve kullanıcı oturumlarını yönetir.
4. **Download Manager**: Eşzamanlı indirme, ilerleme izleme ve hash doğrulama işlemlerini gerçekleştirir.
5. **Install Manager**: İndirilen paketleri çıkarma, kurma, güncelleme ve kaldırma işlemlerini yönetir.
6. **Local Registry**: Yerel olarak kurulu eklentilerin kaydını tutar, versiyon ve durum bilgilerini depolar.

## Kurulum

```toml
# Cargo.toml
[dependencies]
tauri-plugin-store = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "main" }
```

## Kullanım

```rust
use tauri_plugin_store::{StoreClientConfig, init};

fn main() {
    // Store client konfigürasyonu
    let config = StoreClientConfig {
        api_url: "https://plugins.tauri-windows-plugin-system.dev/api".to_string(),
        api_key: Some("your-api-key".to_string()),
        user_token: None,
        install_directory: None, // Varsayılanı kullan
        concurrent_downloads: Some(3),
        auto_check_updates: Some(true),
        auto_check_interval_hours: Some(24),
        trusted_certificates: None,
    };
    
    // Tauri uygulamasını oluştur
    let app = tauri::Builder::default()
        .plugin(init(config))
        .build(tauri::generate_context!())
        .expect("Tauri uygulaması oluşturulamadı");
        
    app.run(|_, _| {});
}
```

### Eklenti Arama

```rust
use tauri_plugin_store::store_types::{PluginCategory, PluginSearchFilter, PluginSortType};

// Store Client'ı al
let state = app.state::<tauri_plugin_store::StoreClientState>();
let store_client = state.store_client.read().await;

// Arama filtresi oluştur
let search_filter = PluginSearchFilter {
    query: Some("security".to_string()),
    categories: Some(vec![PluginCategory::Security]),
    plugin_type: None,
    vendor_id: None,
    free_only: Some(true),
    sort_by: Some(PluginSortType::Rating),
    page: Some(1),
    page_size: Some(10),
};

// Eklenti ara
match store_client.search_plugins(search_filter).await {
    Ok(result) => {
        println!("Toplam {} eklenti bulundu", result.total_count);
        for plugin in result.items {
            println!("- {} (v{})", plugin.name, plugin.version);
        }
    },
    Err(e) => println!("Arama hatası: {:?}", e),
}
```

### Eklenti İndirme ve Kurma

```rust
// Store Client'ı al
let state = app.state::<tauri_plugin_store::StoreClientState>();
let mut store_client = state.store_client.write().await;

// Eklenti ID'si
let plugin_id = "com.example.security-scanner";

// Eklenti indir
match store_client.download_plugin(plugin_id, None).await {
    Ok(status) => {
        if status.status == DownloadStatus::Completed {
            println!("İndirme tamamlandı: {:?}", status.file_path);
            
            // Eklenti kur
            match store_client.install_plugin(plugin_id, None).await {
                Ok(install_status) => {
                    if install_status.success {
                        println!("Eklenti başarıyla kuruldu: {}", install_status.name);
                    }
                },
                Err(e) => println!("Kurulum hatası: {:?}", e),
            }
        }
    },
    Err(e) => println!("İndirme hatası: {:?}", e),
}
```

### Güncelleme Kontrolü

```rust
// Store Client'ı al
let state = app.state::<tauri_plugin_store::StoreClientState>();
let store_client = state.store_client.read().await;

// Tüm eklentiler için güncelleme kontrolü
match store_client.check_for_updates().await {
    Ok(updates) => {
        if updates.is_empty() {
            println!("Tüm eklentiler güncel");
        } else {
            println!("{} eklenti için güncelleme mevcut", updates.len());
            for (plugin_id, update) in updates {
                println!("- {}: v{} -> v{}", 
                    plugin_id, 
                    update.current_version,
                    update.version);
            }
        }
    },
    Err(e) => println!("Güncelleme kontrolü hatası: {:?}", e),
}
```

## Diğer Modüllerle Entegrasyon

### Signature Verification

```rust
use tauri_plugin_signature::SignatureManager;

// Signature Manager'ı al
let signature_state = app.state::<tauri_plugin_signature::SignatureState>();
let signature_manager = signature_state.signature_manager.read().await;

// İmza doğrula
let signature_info = /* ... */;
let cert_pem = /* ... */;

match signature_manager.verify_signature(
    &plugin_path, 
    &signature_info, 
    cert_pem,
    Some(tauri_plugin_signature::TrustLevel::Trusted)
) {
    Ok(result) => {
        if result.is_valid {
            println!("İmza doğrulandı: {}", result.subject_name);
        } else {
            println!("İmza doğrulanamadı: {:?}", result.error);
        }
    },
    Err(e) => println!("İmza doğrulama hatası: {:?}", e),
}
```

### Permission System

```rust
use tauri_plugin_permissions::PermissionManager;

// Permission Manager'ı al
let permissions_state = app.state::<tauri_plugin_permissions::PermissionState>();
let permission_manager = permissions_state.permission_manager.write().await;

// İzin isteği oluştur
let plugin_info = /* ... */;
let permission_request = /* ... */;

// İzin doğrula
match permission_manager.validate_permissions(&permission_request) {
    Ok(response) => {
        if response.decision == tauri_plugin_permissions::Decision::Granted {
            println!("İzinler verildi");
        } else {
            println!("İzinler reddedildi");
        }
    },
    Err(e) => println!("İzin doğrulama hatası: {:?}", e),
}
```

### Sandbox ve Resource Monitor

```rust
use tauri_plugin_sandbox::SandboxManager;
use tauri_plugin_resource_monitor::ResourceMonitor;

// Bileşenleri al
let sandbox_state = app.state::<tauri_plugin_sandbox::SandboxState>();
let sandbox_manager = sandbox_state.sandbox_manager.write().await;

let resource_state = app.state::<tauri_plugin_resource_monitor::ResourceMonitorState>();
let resource_monitor = resource_state.resource_monitor.write().await;

// Sandbox oluştur
let sandbox_options = /* ... */;
match sandbox_manager.create_sandbox(plugin_id, sandbox_options) {
    Ok(sandbox_id) => {
        // Eklentiyi çalıştır
        match sandbox_manager.run_process(&sandbox_id, &plugin_path, &[], None) {
            Ok(process_id) => {
                // Kaynak izleme başlat
                match resource_monitor.start_monitoring(plugin_id, process_id) {
                    Ok(_) => {
                        // Kaynak kullanımını al
                        if let Ok(usage) = resource_monitor.get_resource_usage(plugin_id) {
                            println!("CPU: {:.2}%", usage.cpu_usage);
                            println!("Bellek: {:.2} MB", usage.memory_usage_mb);
                        }
                    },
                    Err(e) => println!("Kaynak izleme hatası: {:?}", e),
                }
            },
            Err(e) => println!("Süreç başlatma hatası: {:?}", e),
        }
    },
    Err(e) => println!("Sandbox oluşturma hatası: {:?}", e),
}
```

## Örnekler

1. **Basic Store Client**: Eklenti arama ve detay görüntüleme örneği
   ```
   cargo run --example basic_store_client
   ```

2. **Plugin Installation**: Eklenti indirme, kurma ve entegrasyon örneği
   ```
   cargo run --example plugin_installation
   ```

## Lisans

MIT Lisansı altında dağıtılmaktadır.
