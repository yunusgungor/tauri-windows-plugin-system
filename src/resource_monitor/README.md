# Tauri Windows Plugin Kaynak İzleme Sistemi

Bu modül, Tauri Windows Plugin System için kapsamlı bir kaynak izleme ve yönetim altyapısı sağlar. Plugin'lerin CPU, bellek, disk ve ağ kullanımını gerçek zamanlı olarak izler, kaynak sınırlamalarını uygular ve aşım durumlarında çeşitli eylemler gerçekleştirir.

## Özellikler

- **Gerçek Zamanlı Kaynak İzleme**: CPU, bellek, disk, ağ, thread sayısı ve diğer sistem kaynaklarını sürekli izleme
- **Granüler Kaynak Limitleri**: Her kaynak tipi için yumuşak ve sert limitler tanımlama
- **Otomatik Kaynak Yönetimi**: Limit aşımlarında otomatik eylemler (uyarı, kısıtlama, askıya alma, sonlandırma)
- **Tarihsel Veri Analizi**: Kaynak kullanım trendlerini ve zirve değerlerini izleme ve raporlama
- **Entegre Alarm Sistemi**: Kritik limit aşımlarında bildirim ve uyarı mekanizması
- **Tauri Entegrasyonu**: JavaScript API üzerinden kolay erişim
- **Windows Job Objects Entegrasyonu**: Sandbox modülü ile birlikte çalışarak kaynak sınırlamalarını uygulama

## Kurulum

### Cargo.toml'a bağımlılığı ekleme

```toml
[dependencies]
tauri-plugin-resource-monitor = { path = "../path/to/resource_monitor" }
```

### Tauri uygulamasına entegre etme

```rust
fn main() {
    // Kaynak izleme konfigürasyonu
    let config = tauri_plugin_resource_monitor::ResourceMonitorConfig {
        monitoring_interval_ms: 1000,
        resources_to_monitor: vec![
            ResourceType::CpuUsage,
            ResourceType::MemoryUsage,
            ResourceType::ThreadCount,
            ResourceType::HandleCount,
        ],
        resource_limits: vec![
            ResourceLimit {
                resource_type: ResourceType::CpuUsage,
                soft_limit: 70.0,
                hard_limit: 90.0,
                measurement_period: 5,
                action: LimitAction::Throttle,
            },
            ResourceLimit {
                resource_type: ResourceType::MemoryUsage,
                soft_limit: 200.0,
                hard_limit: 500.0,
                measurement_period: 5,
                action: LimitAction::Terminate,
            },
        ],
        auto_monitoring: true,
        history_retention_days: 7,
        notify_on_limit_breach: true,
        gather_statistics: true,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_resource_monitor::init(config))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Kullanım

### Rust API

```rust
use tauri_plugin_resource_monitor::{
    ResourceType, ResourceLimit, LimitAction, ResourceMonitorConfig,
};
use monitor::ResourceMonitor;

// Kaynak izleyici oluştur
let config = ResourceMonitorConfig {
    monitoring_interval_ms: 1000,
    resources_to_monitor: vec![ResourceType::CpuUsage, ResourceType::MemoryUsage],
    resource_limits: vec![
        ResourceLimit {
            resource_type: ResourceType::CpuUsage,
            soft_limit: 70.0,
            hard_limit: 90.0,
            measurement_period: 5,
            action: LimitAction::Throttle,
        },
    ],
    auto_monitoring: true,
    history_retention_days: 7,
    notify_on_limit_breach: true,
    gather_statistics: true,
};

let monitor = ResourceMonitor::new(config).await?;

// Plugin'i izlemeye başla
monitor.start_monitoring("my-plugin", 12345).await?;

// Anlık kaynak kullanımını ölç
monitor.measure("my-plugin", ResourceType::CpuUsage).await?;

// Kullanım profilini al
if let Some(profile) = monitor.get_usage_profile("my-plugin") {
    if let Some(cpu) = profile.current_usage(ResourceType::CpuUsage) {
        println!("CPU Kullanımı: {}%", cpu);
    }
    
    if let Some(memory) = profile.current_usage(ResourceType::MemoryUsage) {
        println!("Bellek Kullanımı: {} MB", memory);
    }
}

// Limit aşım olaylarını kontrol et
let events = monitor.get_limit_events("my-plugin");
for event in events {
    println!(
        "{} - {}: {} > {} ({}% aşım)",
        event.timestamp.format("%H:%M:%S"),
        event.resource_type.description(),
        event.actual_value,
        event.limit,
        event.overage_percent
    );
}

// İzlemeyi durdur
monitor.stop_monitoring("my-plugin").await?;

// Kaynak izleyiciyi kapat
monitor.shutdown().await?;
```

### JavaScript API

```javascript
// Plugin'i izlemeye başla
await window.__TAURI__.invoke('plugin:resource_monitor|start_monitoring', {
  pluginId: 'my-plugin',
  processId: 12345
});

// Kaynak kullanımını al
const usageJson = await window.__TAURI__.invoke('plugin:resource_monitor|get_resource_usage', {
  pluginId: 'my-plugin'
});
const usage = JSON.parse(usageJson);
console.log(`CPU: ${usage.current_measurements.CpuUsage?.value}%`);
console.log(`Memory: ${usage.current_measurements.MemoryUsage?.value} MB`);

// Kaynak limitlerini al
const limitsJson = await window.__TAURI__.invoke('plugin:resource_monitor|get_resource_limits');
const limits = JSON.parse(limitsJson);
console.log('Resource Limits:', limits);

// Limit aşım olaylarını al
const eventsJson = await window.__TAURI__.invoke('plugin:resource_monitor|get_limit_events', {
  pluginId: 'my-plugin'
});
const events = JSON.parse(eventsJson);
console.log('Limit Events:', events);

// Kaynak limitlerini güncelle
await window.__TAURI__.invoke('plugin:resource_monitor|update_resource_limits', {
  limits: [
    {
      resource_type: 'CpuUsage',
      soft_limit: 60.0,
      hard_limit: 80.0,
      measurement_period: 5,
      action: 'Throttle'
    }
  ]
});

// İzlemeyi durdur
await window.__TAURI__.invoke('plugin:resource_monitor|stop_monitoring', {
  pluginId: 'my-plugin'
});
```

## Kaynak Tipleri

Bu modül aşağıdaki kaynak tiplerini izler:

| Kaynak Tipi | Açıklama | Birim |
|-------------|----------|-------|
| CpuUsage | CPU kullanımı | Yüzde (0-100) |
| MemoryUsage | Bellek kullanımı | MB |
| ProcessCount | Aktif process sayısı | Adet |
| DiskRead | Disk okuma hızı | KB/s |
| DiskWrite | Disk yazma hızı | KB/s |
| NetworkDownload | Ağ indirme hızı | KB/s |
| NetworkUpload | Ağ yükleme hızı | KB/s |
| DiskSpace | Disk alanı kullanımı | MB |
| ThreadCount | Thread sayısı | Adet |
| HandleCount | Handle sayısı | Adet |
| GdiObjectCount | GDI nesne sayısı | Adet |
| PageFaults | Sayfa hatası oranı | Adet/s |
| SystemCalls | Sistem çağrısı oranı | Adet/s |

## Limit Eylemleri

Limit aşıldığında aşağıdaki eylemlerden biri uygulanabilir:

| Eylem | Açıklama |
|-------|----------|
| Warn | Sadece uyarı gönderir, herhangi bir kısıtlama uygulamaz |
| Throttle | Plugin kaynak kullanımını kısıtlar (CPU, bellek vb.) |
| Suspend | Plugin'i geçici olarak askıya alır |
| Terminate | Plugin'i sonlandırır |

## Mimari Entegrasyon

Resource Monitor modülü, Tauri Windows Plugin System'in diğer bileşenleriyle şu şekilde entegre çalışır:

1. **Sandbox Manager**: İzlenen process'ler üzerinde kaynak kısıtlaması uygulamak için Windows Job Objects kullanır
2. **Permission System**: Belirli kaynak limitleri için gerekli izinleri kontrol eder
3. **Plugin Manager**: Plugin yaşam döngüsü olaylarını (başlatma, durdurma) izler

## Örnekler

`examples` dizininde bulunan `resource_monitor_test.rs` dosyası, Notepad gibi basit bir uygulamayı izleyerek modülün nasıl kullanılacağını gösterir:

```bash
cargo run --example resource_monitor_test
```

## Performans Notları

- İzleme aralığı (`monitoring_interval_ms`) değeri düşük tutulduğunda (örn. 100ms) sistem yükü artabilir
- Çok sayıda process izlendiğinde bellek kullanımı artabilir
- Windows API çağrıları için native bağlantılar kullanıldığından, hata durumlarına karşı güçlü hata işleme mekanizmaları eklenmiştir

## Gelecek Geliştirmeler

- Alarm ve bildirim sistemi genişletilmesi
- Daha ayrıntılı kaynak kullanım grafikleri ve raporları
- Adaptif kaynak yönetimi (kullanım modellerine göre limitleri otomatik ayarlama)
- WebAssembly (WASM) plugin'leri için kaynak izleme desteği
- Platform bağımsız bir altyapı için ek destek (Linux, macOS)
