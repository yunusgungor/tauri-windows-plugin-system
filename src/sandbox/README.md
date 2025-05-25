# Tauri Windows Plugin Sandbox

Bu modül, Tauri Windows Plugin System için güvenli bir sandbox izolasyon mekanizması sağlar. Windows Job Objects kullanarak eklentileri izole eder ve kaynak kullanımını sınırlar.

## Özellikler

- **Process İzolasyonu**: Windows Job Objects kullanarak process düzeyinde izolasyon
- **Kaynak Sınırlama**: Bellek, CPU, process sayısı ve working set limitleri
- **İzin Kontrolü**: Ayrıntılı izin sistemi ile güvenlik denetimi
- **Tauri Entegrasyonu**: Tauri uygulamalarına doğrudan entegrasyon

## Gereksinimler

- Windows işletim sistemi (Windows 10 veya üzeri)
- Rust 1.64 veya üzeri
- Tauri 1.4 veya üzeri

## Kullanım

### Cargo.toml'a bağımlılığı ekleme

```toml
[dependencies]
tauri-plugin-sandbox = { path = "../path/to/sandbox" }
```

### Tauri uygulamasına entegre etme

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sandbox::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Sandbox Manager kullanımı

```rust
use tauri_plugin_sandbox::{PermissionLevel, ResourceLimits, SandboxManager};

// Sandbox Manager oluştur
let sandbox_manager = SandboxManager::new();

// Process'i sandbox'la
sandbox_manager.sandbox_plugin(
    "my-plugin",           // Plugin ID
    process_id,            // Process ID
    Some(ResourceLimits {  // Kaynak sınırlamaları
        max_memory_mb: 256,
        max_cpu_percentage: 30,
        max_process_count: 5,
        max_working_set_mb: 128,
    }),
    vec![                  // İzinler
        PermissionLevel::Core,
        PermissionLevel::Filesystem,
    ],
)?;

// İzin kontrolü
let has_permission = sandbox_manager.check_permission(
    "my-plugin", 
    PermissionLevel::Filesystem
)?;

// Sandbox durumunu kontrol et
if let Some((pid, limits, permissions)) = sandbox_manager.get_plugin_sandbox_status("my-plugin") {
    println!("Plugin sandbox'lanmış. PID: {}", pid);
}

// Sandbox'ı kaldır
sandbox_manager.remove_sandbox("my-plugin")?;
```

## Tauri JS API

Bu modül, sandbox durumunu kontrol etmek için bir JavaScript API'si de sağlar:

```javascript
// Sandbox durumunu kontrol et
const status = await window.__TAURI__.invoke('plugin:sandbox|check_sandbox_status', {
  pluginId: 'my-plugin'
});
console.log(status);
```

## Mimari

Sandbox modülü, aşağıdaki ana bileşenlerden oluşur:

1. **SandboxManager**: Tüm sandbox'ları yöneten ana sınıf
2. **SandboxedPlugin**: Sandbox'lanmış bir plugin'i temsil eden yapı
3. **ResourceLimits**: Kaynak sınırlamalarını tanımlayan yapı
4. **PermissionLevel**: İzin düzeylerini tanımlayan enum

Windows Job Objects, process gruplarını izole etmek ve sınırlandırmak için kullanılır. Her plugin için ayrı bir Job Object oluşturulur ve process bu Job Object'e atanır.

## Güvenlik Sınırlamaları

- Bu modül yalnızca Windows işletim sisteminde çalışır
- İzolasyon, Windows Job Objects'in sağladığı güvenlik sınırlarına bağlıdır
- CPU sınırlaması gerçek zamanlı olarak uygulanmaz, periyodik izleme gerektirir
- Bazı sistem çağrıları sandbox içinden de yapılabilir

## Test

Test etmek için örnek uygulamayı çalıştırın:

```
cargo run --example sandbox_test
```

## Gelecek Geliştirmeler

- Gelişmiş dosya sistemi izolasyonu
- Network erişim kontrolü
- Registry erişim kontrolü
- Windows App Container entegrasyonu
- Gerçek zamanlı CPU sınırlaması
- Tehdit tespiti ve davranış analizi
