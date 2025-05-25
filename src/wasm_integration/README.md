# Tauri Windows Plugin WebAssembly Entegrasyonu

Bu modül, Tauri Windows Plugin System için WebAssembly (WASM) desteği sağlar. Bu sayede plugin'lerin cross-platform olarak çalışabilmesi, platform bağımsız eklenti geliştirilebilmesi ve gelişmiş güvenlik izolasyonu sağlanması mümkün olur.

## Özellikler

- **Cross-Platform Plugin Desteği**: Rust, C++, C#, AssemblyScript gibi dillerde yazılmış kodları WASM'e derleyerek tüm platformlarda çalıştırabilme
- **Wasmtime Entegrasyonu**: Performanslı ve güvenli WASM runtime
- **Granüler İzin Modeli**: Dosya sistemi, ağ, API ve kaynak erişimleri için ayrıntılı izin kontrolü
- **Bellek İzolasyonu**: WASM modülleri için tam bellek izolasyonu ve kaynak sınırlaması
- **Tauri Entegrasyonu**: JavaScript ve Rust API'leri üzerinden tam entegrasyon
- **API Köprüsü**: Host ve WASM modülleri arasında iki yönlü API iletişimi
- **Performans Optimizasyonu**: JIT derleme, bellek yönetimi ve zaman sınırlaması

## Kurulum

### Cargo.toml'a bağımlılığı ekleme

```toml
[dependencies]
tauri-plugin-wasm = { path = "../path/to/wasm_integration" }
```

### Tauri uygulamasına entegre etme

```rust
fn main() {
    // WASM plugin konfigürasyonu
    let wasm_config = tauri_plugin_wasm::WasmPluginConfig {
        security_policy: Some(tauri_plugin_wasm::WasmSecurityPolicy::AskOnce),
        ..Default::default()
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_wasm::init(wasm_config))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Mimariye Genel Bakış

WASM entegrasyonu aşağıdaki bileşenlerden oluşur:

1. **WASM Runtime**: Wasmtime üzerine kurulu modül yükleme ve çalıştırma sistemi
2. **API Bridge**: Host ve WASM modülleri arasında iki yönlü iletişim sağlayan köprü
3. **Security Layer**: WASM modüllerinin erişim izinlerini kontrol eden güvenlik katmanı
4. **Tauri Integration**: Tauri ile entegrasyonu sağlayan bileşen

## Kullanım

### Rust API

```rust
use tauri_plugin_wasm::{
    WasmRuntimeManager, WasmLoadOptions, WasmModuleConfig, WasmSecurityManager,
    WasmPermissionType, WasmPermissionScope, WasmSecurityPolicy,
};

// WASM runtime yöneticisini oluştur
let mut runtime = WasmRuntimeManager::new(Some(WasmModuleConfig::default()))?;

// Güvenlik yöneticisini oluştur
let security = WasmSecurityManager::new(WasmSecurityPolicy::AskOnce);

// WASM modülünü yükle
let options = WasmLoadOptions {
    name: Some("my_plugin".to_string()),
    auto_start: true,
    ..Default::default()
};

let module_id = runtime.load_module_from_file("path/to/plugin.wasm", Some(options)).await?;

// Fonksiyonu çağır
let args = vec![wasmtime::Val::I32(5), wasmtime::Val::I32(7)];
let result = runtime.call_function(&module_id, "add", args).await?;
println!("add(5, 7) = {:?}", result[0]);

// Modülü durdur
runtime.stop_module(&module_id).await?;
```

### JavaScript API

```javascript
// WASM modülünü yükle
const moduleId = await window.__TAURI__.invoke('plugin:wasm|load_module_from_file', {
  path: '/path/to/plugin.wasm',
  options: {
    name: 'my_plugin',
    autoStart: true
  }
});

// Fonksiyonu çağır
const result = await window.__TAURI__.invoke('plugin:wasm|call_wasm_function', {
  moduleId: moduleId,
  functionName: 'add',
  params: [5, 7]
});
console.log(`add(5, 7) = ${result}`);

// Modülleri listele
const modules = await window.__TAURI__.invoke('plugin:wasm|list_modules');
console.log('Loaded modules:', modules);

// Modülü durdur
await window.__TAURI__.invoke('plugin:wasm|stop_module', {
  moduleId: moduleId
});
```

## İzin Yönetimi

WASM modülleri, varsayılan olarak hiçbir sistem kaynağına erişemez. Erişim için açık izinler gereklidir:

```rust
// İzin iste
let result = security.request_permission(
    &module_id,
    WasmPermission::new(
        WasmPermissionType::Filesystem,
        WasmPermissionScope::Path(PathBuf::from("/tmp")),
        "Geçici dosyalar için erişim gerekli",
        50 // Risk seviyesi
    )
).await?;

if result {
    println!("İzin verildi");
} else {
    println!("İzin reddedildi");
}
```

JavaScript'ten:

```javascript
// İzin iste
const granted = await window.__TAURI__.invoke('plugin:wasm|request_permission', {
  moduleId: moduleId,
  permissionType: 'Filesystem',
  scope: { Path: '/tmp' },
  description: 'Geçici dosyalar için erişim gerekli',
  riskLevel: 50
});

if (granted) {
  console.log('İzin verildi');
} else {
  console.log('İzin reddedildi');
}
```

## Güvenlik Politikaları

WASM modülleri için dört farklı güvenlik politikası desteklenir:

1. **AlwaysAsk**: Her izin talebi için kullanıcıya sor
2. **AskOnce**: İlk kullanımda sor, sonra kararı hatırla
3. **AutoAccept**: Tüm izinleri otomatik olarak ver (geliştirme için)
4. **AutoDeny**: Tüm izinleri otomatik olarak reddet (güvenlik testi için)
5. **RiskBased**: Risk seviyesine göre karar ver (düşük riskli izinleri otomatik ver)

## WASM Modülü Geliştirme

### Rust'tan WASM Modülü

```rust
// lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn hello() -> String {
    "Hello, World!".to_string()
}
```

Cargo.toml:

```toml
[package]
name = "my-wasm-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
```

Derleme:

```bash
wasm-pack build --target web
```

### AssemblyScript'ten WASM Modülü

```typescript
// index.ts
export function add(a: i32, b: i32): i32 {
  return a + b;
}

export function hello(): string {
  return "Hello, World!";
}
```

Derleme:

```bash
asc index.ts -o plugin.wasm --optimize
```

## Örnek Kullanım

`examples` dizininde bulunan `plugin_host.rs` dosyası, temel bir WASM modülünü yükleyip çalıştıran bir örnek içerir:

```bash
cargo run --example plugin_host
```

## Entegrasyon Noktaları

WASM entegrasyonu, Tauri Windows Plugin System'in diğer bileşenleriyle şu şekilde entegre çalışır:

1. **Sandbox Manager**: WASM modülleri kendi doğal sandbox'ları içinde çalışır ve ek bir izolasyona gerek duymaz
2. **Permission System**: WASM modülleri için izinler, genel izin sistemi ile entegre çalışır
3. **Resource Monitor**: WASM modüllerinin kaynak kullanımı izlenebilir ve sınırlandırılabilir
4. **Digital Signature**: WASM modülleri de imzalanabilir ve doğrulanabilir

## Performans Notları

- WASM JIT derlemesi ilk yüklemede biraz zaman alabilir ancak sonraki çalıştırmalarda önbelleğe alınır
- Bellek transferleri, host ve WASM modülleri arasında ek yük oluşturabilir
- Büyük veri yapıları için shared memory kullanımı önerilir
- CPU-intensive işlemler için WASM SIMD uzantıları kullanılabilir

## Gelecek Geliştirmeler

- WASM Component Model desteği
- Thread desteği ile paralel işlemler
- WebGPU entegrasyonu
- Daha zengin API köprüsü ve otomatik tip dönüşümleri
- Hot reload desteği
