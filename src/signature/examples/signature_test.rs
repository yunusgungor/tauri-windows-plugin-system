// Tauri Windows Plugin System - İmza Test Uygulaması
//
// Bu örnek uygulama, dijital imza oluşturma ve doğrulama işlemlerini gösterir.
// Test için geçici dosyalar oluşturur ve Ed25519 algoritması ile imzalama yapar.

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri_plugin_signature::{SignatureManager, TrustLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tauri Windows Plugin İmza Test Uygulaması");
    println!("==========================================");
    
    // Geçici dizin oluştur
    let temp_dir = env::temp_dir().join("tauri_signature_test");
    fs::create_dir_all(&temp_dir)?;
    println!("Geçici dizin oluşturuldu: {:?}", temp_dir);
    
    // Test plugin paketi oluştur (temsili)
    let package_path = temp_dir.join("test_plugin.zip");
    let test_content = b"Bu bir test plugin paketidir.";
    File::create(&package_path)?.write_all(test_content)?;
    println!("Test plugin paketi oluşturuldu: {:?}", package_path);
    
    // Test Ed25519 anahtar çifti oluştur
    println!("\nEd25519 anahtar çifti oluşturuluyor...");
    let (private_key_path, public_key_path, cert_path) = generate_test_keys(&temp_dir)?;
    println!("Özel anahtar: {:?}", private_key_path);
    println!("Sertifika: {:?}", cert_path);
    
    // İmza yöneticisi oluştur
    let signature_manager = SignatureManager::new();
    
    // Sertifikayı güvenilir olarak ekle
    let cert_content = fs::read_to_string(&cert_path)?;
    signature_manager.add_trusted_root(&cert_content)?;
    println!("\nSertifika güvenilir kök olarak eklendi");
    
    // Paketi imzala
    println!("\nPlugin paketi imzalanıyor...");
    let signature_info = signature_manager.sign_package(
        &package_path,
        &private_key_path,
        &cert_path,
    )?;
    
    // İmza bilgilerini göster
    println!("İmza oluşturuldu:");
    println!("  Algoritma: {:?}", signature_info.algorithm);
    println!("  İçerik Hash: {}", signature_info.content_hash);
    println!("  İmzalayan: {}", signature_info.signer_thumbprint);
    println!("  Zaman: {}", signature_info.timestamp);
    
    // İmza bilgilerini kaydet
    let signature_path = temp_dir.join("signature.json");
    let signature_json = serde_json::to_string_pretty(&signature_info)?;
    fs::write(&signature_path, signature_json)?;
    println!("İmza bilgileri kaydedildi: {:?}", signature_path);
    
    // İmzayı doğrula
    println!("\nİmza doğrulanıyor...");
    let verification_result = signature_manager.verify_signature(
        &package_path,
        &signature_info,
        &cert_content,
        Some(TrustLevel::Basic),
    )?;
    
    println!("Doğrulama sonucu: {:?}", verification_result);
    
    // Paketi doğrudan doğrula
    println!("\nPaket doğrudan doğrulanıyor...");
    let package_verification = signature_manager.verify_package(
        &package_path,
        &signature_path,
        &cert_path,
        Some(TrustLevel::Full),
    )?;
    
    println!("Paket doğrulama sonucu: {:?}", package_verification);
    
    // İçeriği değiştirerek doğrulama başarısızlığını göster
    println!("\nPaket içeriği değiştirilerek doğrulama test ediliyor...");
    let tampered_package_path = temp_dir.join("tampered_plugin.zip");
    let tampered_content = b"Bu bir değiştirilmiş pakettir!";
    File::create(&tampered_package_path)?.write_all(tampered_content)?;
    
    let tampered_verification = signature_manager.verify_signature(
        &tampered_package_path,
        &signature_info,
        &cert_content,
        Some(TrustLevel::Basic),
    );
    
    match tampered_verification {
        Ok(result) => println!("Değiştirilmiş paket doğrulama sonucu: {:?}", result),
        Err(e) => println!("Değiştirilmiş paket doğrulama hatası (beklenen): {}", e),
    }
    
    // Temizlik
    println!("\nGeçici dosyalar temizleniyor...");
    fs::remove_dir_all(temp_dir)?;
    
    println!("\nTest tamamlandı!");
    Ok(())
}

// Test için Ed25519 anahtar çifti oluştur
fn generate_test_keys(temp_dir: &Path) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
    use ring::rand::SystemRandom;
    use ring::signature::Ed25519KeyPair;
    
    // Rastgele sayı üreteci
    let rng = SystemRandom::new();
    
    // Ed25519 anahtar çifti oluştur
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    
    // Özel anahtarı kaydet
    let private_key_path = temp_dir.join("test_private_key.der");
    fs::write(&private_key_path, pkcs8_bytes.as_ref())?;
    
    // Public key'i çıkar
    let public_key = key_pair.public_key().as_ref();
    let public_key_path = temp_dir.join("test_public_key.der");
    fs::write(&public_key_path, public_key)?;
    
    // Temsili bir X.509 sertifika oluştur (gerçek uygulamada OpenSSL vb. kullanılabilir)
    // Bu sadece test amaçlı basitleştirilmiş bir sertifikadır
    let cert_content = format!(
        "-----BEGIN CERTIFICATE-----
MIIBxzCCAWygAwIBAgIUJhw89zHbo5KzHzQ+HOMlBQvDrj0wCgYIKoZIzj0EAwIw
RzELMAkGA1UEBhMCVFIxFzAVBgNVBAgMDlRhdXJpIFBsdWdpbnMxHzAdBgNVBAoM
FlRhdXJpIFBsdWdpbiBEZXZlbG9wZXJzMB4XDTIzMDUyNTAwMDAwMFoXDTI3MDUy
NTAwMDAwMFowRzELMAkGA1UEBhMCVFIxFzAVBgNVBAgMDlRhdXJpIFBsdWdpbnMx
HzAdBgNVBAoMFlRhdXJpIFBsdWdpbiBEZXZlbG9wZXJzMFkwEwYHKoZIzj0CAQYI
KoZIzj0DAQcDQgAE{public_key_b64}qANgwdQ1tBQpDvYolr1iGDHQxdBkXsko
NLtPrWU+OKjy9ujAeYYQSuixd0Uzpt5Vh9k50o6F4gSCRaNTMFEwHQYDVR0OBBYE
FEVoEpQUKNd5jNFRCFBV1BJDkNYzMB8GA1UdIwQYMBaAFEVoEpQUKNd5jNFRCFBV
1BJDkNYzMA8GA1UdEwEB/wQFMAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIhANs+852C
eUb+Sf9wW8JPrQoCMaqUv1DP6F7h4svOSG4/AiAeS47g5hKnGEnioj3NSRUWVupw
QOqPwCnUxGWHsLr6eQ==
-----END CERTIFICATE-----",
        public_key_b64 = base64::engine::general_purpose::STANDARD.encode(public_key)
    );
    
    let cert_path = temp_dir.join("test_certificate.pem");
    fs::write(&cert_path, cert_content)?;
    
    Ok((private_key_path, public_key_path, cert_path))
}
