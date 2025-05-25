# Tauri Windows Plugin Gelişmiş İzin Sistemi

Bu modül, Tauri Windows Plugin System için kapsamlı bir izin yönetim sistemi sağlar. Plugin'lerin hangi sistem kaynaklarına ve API'lere erişebileceğini granüler şekilde kontrol eder ve kullanıcı onayı gerektiğinde etkileşimli izin isteme arayüzü sunar.

## Özellikler

- **Granüler İzin Kontrolü**: Kategoriler ve kapsamlar ile ayrıntılı erişim kontrolü
- **Ayrıntılı İzin Kategorileri**: Dosya sistemi, ağ, sistem, UI, donanım, plugin arası iletişim ve komutlar
- **Çoklu İzin Politikaları**: Her zaman sor, bir kere sor, otomatik kabul, otomatik red
- **Farklı Denetim Düzeyleri**: Sıkı, normal, gevşek ve devre dışı denetim modları
- **Kullanıcı Etkileşimli İzin İsteme**: Modallar, dialog'lar, bannerlar ve bildirimler
- **Kalıcı İzin Kararları**: Kullanıcı kararlarını depolama ve yükleme
- **Süreli İzin Belirteçleri**: Belirli bir süre için geçerli izinler
- **Tauri Entegrasyonu**: JavaScript API ile tam entegrasyon

## Kurulum

### Cargo.toml'a bağımlılığı ekleme

```toml
[dependencies]
tauri-plugin-permissions = { path = "../path/to/permissions" }
```

### Tauri uygulamasına entegre etme

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_permissions::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Kullanım

### Rust API

```rust
use tauri_plugin_permissions::{
    PermissionCategory, PermissionDescriptor, FilesystemScope, NetworkScope,
    PermissionManager,
};

// İzin tanımlayıcıları oluştur
let file_perm = PermissionDescriptor {
    category: PermissionCategory::Filesystem,
    scope: FilesystemScope::READ_PLUGIN_DATA.bits() | FilesystemScope::WRITE_PLUGIN_DATA.bits(),
    reason: "Plugin verileri için okuma/yazma erişimi gerekli".to_string(),
};

let net_perm = PermissionDescriptor {
    category: PermissionCategory::Network,
    scope: NetworkScope::HTTPS.bits(),
    reason: "API istekleri için HTTPS erişimi gerekli".to_string(),
};

// İzin iste (async)
let token = permission_manager.check_permissions(
    "my-plugin",
    vec![file_perm, net_perm]
).await?;

// Belirli bir izni kontrol et
let has_permission = permission_manager.has_permission(
    "my-plugin",
    PermissionCategory::Filesystem,
    FilesystemScope::READ_PLUGIN_DATA.bits()
)?;

if has_permission {
    // Dosya işlemlerini gerçekleştir
} else {
    // İzin yok, alternatif işlem yap veya hata göster
}
```

### JavaScript API

```javascript
// İzin iste
const permissionResult = await window.__TAURI__.invoke('plugin:permissions|request_permission', {
  pluginId: 'my-plugin',
  category: 'filesystem',
  scope: 1, // FilesystemScope::READ_PLUGIN_DATA
  reason: 'Veri dosyalarını okumak için'
});
console.log(permissionResult);

// İzin kontrolü
const hasPermission = await window.__TAURI__.invoke('plugin:permissions|check_permission', {
  pluginId: 'my-plugin',
  category: 'filesystem',
  scope: 1 // FilesystemScope::READ_PLUGIN_DATA
});

if (hasPermission) {
  // İzin var, işlemi gerçekleştir
} else {
  // İzin yok, kullanıcıya bilgi ver
}

// Plugin izinlerini görüntüle
const permissions = await window.__TAURI__.invoke('plugin:permissions|get_permissions', {
  pluginId: 'my-plugin'
});
console.log(JSON.parse(permissions));

// İzinleri iptal et
await window.__TAURI__.invoke('plugin:permissions|revoke_permission', {
  pluginId: 'my-plugin'
});
```

## İzin Kategorileri ve Kapsamları

### Dosya Sistemi İzinleri (FilesystemScope)
- `READ_PLUGIN_DATA`: Plugin veri dizininden okuma
- `WRITE_PLUGIN_DATA`: Plugin veri dizinine yazma
- `READ_EXTERNAL`: Harici dizinlerden okuma
- `WRITE_EXTERNAL`: Harici dizinlere yazma
- `READ_APP_DIR`: Uygulama dizininden okuma
- `WRITE_APP_DIR`: Uygulama dizinine yazma
- `READ_ANY`: Herhangi bir dosyayı okuma (yüksek risk)
- `WRITE_ANY`: Herhangi bir dosyaya yazma (yüksek risk)

### Ağ İzinleri (NetworkScope)
- `HTTP`: HTTP istekleri
- `HTTPS`: HTTPS istekleri
- `WEBSOCKET`: WebSocket bağlantıları
- `TCP`: TCP soketleri
- `UDP`: UDP soketleri
- `LISTEN`: Gelen bağlantıları dinleme
- `LOCAL_NETWORK`: Yerel ağ erişimi
- `ANY_HOST`: Herhangi bir adrese erişim

### Sistem İzinleri (SystemScope)
- `READ_INFO`: Sistem bilgilerini okuma
- `READ_PROCESS`: Process bilgilerini okuma
- `READ_ENV`: Çevre değişkenlerini okuma
- `READ_MEMORY`: Bellek kullanımını okuma
- `READ_CPU`: CPU kullanımını okuma
- `READ_DISK`: Disk kullanımını okuma
- `READ_NETWORK`: Ağ kullanımını okuma
- `EXECUTE_COMMAND`: Shell komutları çalıştırma (yüksek risk)

### UI İzinleri (UIScope)
- `MAIN_WINDOW`: Ana pencereye erişim
- `DIALOG`: Dialog pencereleri gösterme
- `NOTIFICATION`: Bildirim gösterme
- `MENU`: Menü öğeleri ekleme
- `TRAY`: Sistem tepsisine erişim
- `NEW_WINDOW`: Yeni pencere oluşturma
- `GLOBAL_SHORTCUT`: Global kısayollar oluşturma
- `CLIPBOARD`: Pano erişimi

### Donanım İzinleri (HardwareScope)
- `CAMERA`: Kamera erişimi
- `MICROPHONE`: Mikrofon erişimi
- `BLUETOOTH`: Bluetooth erişimi
- `USB`: USB cihaz erişimi
- `SERIAL`: Seri port erişimi
- `HID`: HID cihaz erişimi
- `SCREEN_CAPTURE`: Ekran yakalama
- `AUDIO_OUTPUT`: Ses çıkış kontrolü

### Process Arası İzinler (InterprocessScope)
- `DISCOVER`: Diğer plugin'leri keşfetme
- `SEND_MESSAGE`: Belirli plugin'lere mesaj gönderme
- `SEND_ANY`: Herhangi bir plugin'e mesaj gönderme
- `SHARED_DATA`: Paylaşılan verilere erişim
- `LISTEN_EVENTS`: Olayları dinleme
- `EMIT_EVENTS`: Olayları gönderme
- `CALL_API`: Plugin API'lerine erişim
- `CONTROL`: Plugin'leri kontrol etme

### Komut İzinleri (CommandScope)
- `FS_COMMANDS`: Dosya sistemi komutları
- `SHELL_COMMANDS`: Shell komutları
- `HTTP_COMMANDS`: HTTP komutları
- `DIALOG_COMMANDS`: Dialog komutları
- `NOTIFICATION_COMMANDS`: Bildirim komutları
- `SHORTCUT_COMMANDS`: Kısayol komutları
- `CLIPBOARD_COMMANDS`: Pano komutları
- `WINDOW_COMMANDS`: Pencere komutları

## İzin Politikaları

- **AlwaysAsk**: Her izin talebi için kullanıcıya sor
- **AskOnce**: İlk kullanımda sor, sonra kararı hatırla
- **AutoGrant**: Tüm izinleri otomatik olarak ver (geliştirme için)
- **AutoDeny**: Tüm izinleri otomatik olarak reddet (güvenlik testi için)

## Denetim Düzeyleri

- **Strict**: Tüm izinler açıkça verilmelidir
- **Normal**: Bazı düşük riskli izinler otomatik verilebilir
- **Relaxed**: Yüksek riskli izinler dışında çoğu otomatik verilir
- **Disabled**: Tüm izinler otomatik verilir (sadece geliştirme için)

## İzin İsteme Stilleri

- **Modal**: Tam ekran modal dialog (kullanıcı yanıt vermeden devam edilemez)
- **InlineDialog**: Sayfa içi dialog (yarı-bloke edici)
- **Banner**: Sayfa üstünde veya altında banner (bloke etmez)
- **Notification**: Bildirim stili prompt (minimal müdahale)

## Test

Örnek uygulamayı çalıştırmak için:

```bash
cargo run --example permission_test
```

## Gelecek Geliştirmeler

- Dinamik izin seviyesi uyarlaması (plugin davranışına göre)
- Daha zengin kullanıcı izin arayüzleri
- İzin kullanım metriklerini izleme
- İzin analiz araçları
- İzin politikalarının uygulama düzeyinde yapılandırılması
