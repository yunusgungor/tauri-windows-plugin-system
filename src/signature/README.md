# Tauri Windows Plugin İmza Doğrulama

Bu modül, Tauri Windows Plugin System için güçlü bir dijital imza altyapısı sağlar. RSA ve ECC tabanlı imzalama algoritmaları desteklenir ve paketlerin bütünlüğünü ve güvenilirliğini doğrulamak için kullanılır.

## Özellikler

- **Çoklu İmza Algoritmaları**: RSA (PKCS#1, PSS) ve ECC (ECDSA P-256, Ed25519) desteği
- **Sertifika Doğrulama**: X.509 sertifikaları ile tam entegrasyon
- **Güven Zinciri**: Farklı güven düzeyleri ile esnek doğrulama
- **İptal Kontrolü**: İptal edilmiş sertifikaların tespiti
- **Tauri Entegrasyonu**: Tauri uygulamalarına doğrudan entegrasyon

## Gereksinimler

- Rust 1.64 veya üzeri
- Tauri 1.4 veya üzeri

## Kullanım

### Cargo.toml'a bağımlılığı ekleme

```toml
[dependencies]
tauri-plugin-signature = { path = "../path/to/signature" }
```

### Tauri uygulamasına entegre etme

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_signature::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### İmza Yöneticisi kullanımı

```rust
use tauri_plugin_signature::{SignatureManager, TrustLevel, VerificationResult};
use std::path::Path;

// İmza Yöneticisi oluştur
let signature_manager = SignatureManager::new();

// Güvenilir bir kök sertifika ekle
signature_manager.add_trusted_root(&cert_content)?;

// Bir paketi imzala
let signature_info = signature_manager.sign_package(
    &Path::new("plugin.zip"),
    &Path::new("private_key.der"),
    &Path::new("certificate.pem"),
)?;

// İmza bilgilerini JSON olarak kaydet
let signature_json = serde_json::to_string_pretty(&signature_info)?;
std::fs::write("signature.json", signature_json)?;

// İmzayı doğrula
let verification_result = signature_manager.verify_signature(
    &Path::new("plugin.zip"),
    &signature_info,
    &cert_content,
    Some(TrustLevel::Full),
)?;

// Doğrulama sonucunu kontrol et
match verification_result {
    VerificationResult::Valid => println!("İmza geçerli ve güvenilir"),
    VerificationResult::ValidButUntrusted => println!("İmza geçerli fakat sertifika güvenilir değil"),
    VerificationResult::Invalid => println!("İmza geçersiz"),
    VerificationResult::Expired => println!("Sertifika süresi dolmuş"),
    VerificationResult::Revoked => println!("Sertifika iptal edilmiş"),
}
```

## Tauri JS API

Bu modül, plugin imzalarını doğrulamak için bir JavaScript API'si de sağlar:

```javascript
// Plugin imzasını doğrula
const verificationResult = await window.__TAURI__.invoke('plugin:signature|verify_plugin_signature', {
  pluginPath: '/path/to/plugin.zip',
  signaturePath: '/path/to/signature.json',
  certificatePath: '/path/to/certificate.pem'
});
console.log(verificationResult);
```

## İmza Algoritmaları

Modül şu imza algoritmalarını destekler:

1. **RSA-PKCS#1v15**: Klasik RSA imzalama, PKCS#1 v1.5 dolgusu ile
2. **RSA-PSS**: Modern RSA imzalama, PSS dolgusu ile (önerilen)
3. **ECDSA-P256**: Elliptik eğri imzalama, NIST P-256 eğrisi ile
4. **Ed25519**: Modern ve hızlı eliptik eğri imzalama (önerilen)

## Güven Düzeyleri

Üç farklı güven düzeyi sağlanır:

- **TrustLevel::None**: Hiçbir güven kontrolü yapmaz, sadece imzayı doğrular
- **TrustLevel::Basic**: Temel sertifika geçerlilik kontrollerini yapar
- **TrustLevel::Full**: Tam güven zinciri doğrulaması yapar

## Teknik Mimari

İmza doğrulama süreci şu adımlardan oluşur:

1. Paket içeriğinin hash'i hesaplanır (SHA-256)
2. İmza dosyasından imza bilgileri ve kullanılan algoritma alınır
3. Sertifika doğrulanır (geçerlilik tarihi, iptal durumu)
4. İmza, sertifikadaki public key ile doğrulanır
5. İsteğe bağlı olarak, sertifikanın güven zinciri doğrulanır

## Güvenlik Notları

- Eklentileri daima imzalı olarak dağıtın
- Özel anahtarları güvenli bir şekilde saklayın
- Sertifika iptali için düzenli olarak CRL veya OCSP kontrolü yapın
- Güvenlik açısından Ed25519 veya RSA-PSS kullanımı önerilir

## Test

Test etmek için örnek uygulamayı çalıştırın:

```
cargo run --example signature_test
```

## Gelecek Geliştirmeler

- OCSP entegrasyonu
- Zaman damgası desteği
- Çevrimiçi sertifika durumu kontrolü
- Donanım güvenlik modülü (HSM) desteği
- İmza sunucusu entegrasyonu
