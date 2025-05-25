// Tauri Windows Plugin System - Plugin Paketleme Aracı
//
// Bu araç, plugin projelerini paketler ve imzalar.

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use ring::signature::{self, KeyPair, RsaKeyPair};
use base64::{encode, decode};
use sha2::{Sha256, Digest};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use zip::{ZipWriter, write::FileOptions};

// Plugin manifest
#[derive(Debug, Serialize, Deserialize)]
struct PluginManifest {
    // Plugin bilgileri
    id: String,
    name: String,
    version: String,
    description: String,
    plugin_type: String,
    vendor: String,
    vendor_url: Option<String>,
    
    // Teknik bilgiler
    permissions: Vec<String>,
    min_host_version: Option<String>,
    dependencies: Vec<Dependency>,
    
    // Paket bilgileri
    package: PackageInfo,
    
    // İmza bilgileri
    signature: Option<SignatureInfo>,
}

// Plugin bağımlılığı
#[derive(Debug, Serialize, Deserialize)]
struct Dependency {
    id: String,
    version: String,
    optional: bool,
}

// Paket bilgisi
#[derive(Debug, Serialize, Deserialize)]
struct PackageInfo {
    created_at: String,
    file_count: usize,
    total_size_bytes: u64,
    sha256_hash: String,
    plugin_binary: String,
    readme: Option<String>,
    license: Option<String>,
    changelog: Option<String>,
}

// İmza bilgisi
#[derive(Debug, Serialize, Deserialize)]
struct SignatureInfo {
    algorithm: String,
    signature: String,
    public_key_id: String,
    signed_at: String,
    signer: String,
}

fn main() -> io::Result<()> {
    println!("Tauri Windows Plugin System - Plugin Paketleme Aracı");
    println!("=============================================");
    
    // Komut satırı argümanlarını al
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Kullanım: {} <plugin_dizini> <çıktı_dizini> [imzala]", args[0]);
        println!("  <plugin_dizini>: Plugin projesinin kök dizini");
        println!("  <çıktı_dizini>: Paket çıktısının yazılacağı dizin");
        println!("  [imzala]: 'true' ise paketi imzalar (varsayılan: false)");
        return Ok(());
    }
    
    let plugin_dir = PathBuf::from(&args[1]);
    let output_dir = PathBuf::from(&args[2]);
    let sign_package = args.get(3).map_or(false, |arg| arg == "true");
    
    // Dizinlerin varlığını kontrol et
    if !plugin_dir.is_dir() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Plugin dizini bulunamadı: {:?}", plugin_dir)));
    }
    
    if !output_dir.is_dir() {
        fs::create_dir_all(&output_dir)?;
    }
    
    // Cargo.toml dosyasını oku
    let cargo_toml_path = plugin_dir.join("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Cargo.toml bulunamadı: {:?}", cargo_toml_path)));
    }
    
    let cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let cargo_data: toml::Value = toml::from_str(&cargo_toml)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Cargo.toml ayrıştırma hatası: {}", e)))?;
    
    // Plugin bilgilerini topla
    let package_data = cargo_data.get("package")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Cargo.toml'da [package] bölümü bulunamadı"))?;
    
    let plugin_id = package_data.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Plugin adı bulunamadı"))?;
    
    let plugin_name = package_data.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or(plugin_id);
    
    let plugin_version = package_data.get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Plugin versiyonu bulunamadı"))?;
    
    let plugin_description = package_data.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("No description");
    
    let plugin_authors = package_data.get("authors")
        .and_then(|v| v.as_array())
        .map_or(Vec::new(), |a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>());
    
    let plugin_vendor = plugin_authors.first().cloned().unwrap_or_else(|| "Unknown".to_string());
    
    let plugin_type = if plugin_id.contains("wasm") {
        "wasm"
    } else {
        "native"
    };
    
    // Cargo build ile plugin'i derle
    println!("\nPlugin derleniyor: {}", plugin_id);
    let status = Command::new("cargo")
        .current_dir(&plugin_dir)
        .args(&["build", "--release"])
        .status()?;
    
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "Plugin derleme hatası"));
    }
    println!("Plugin başarıyla derlendi.");
    
    // Plugin ikili dosyasının yolunu belirle
    let target_dir = plugin_dir.join("target").join("release");
    let lib_prefix = if cfg!(target_os = "windows") { "" } else { "lib" };
    let lib_ext = if cfg!(target_os = "windows") { "dll" } else if cfg!(target_os = "macos") { "dylib" } else { "so" };
    
    let plugin_binary_name = format!("{}{}.{}", lib_prefix, plugin_id.replace('-', "_"), lib_ext);
    let plugin_binary_path = target_dir.join(&plugin_binary_name);
    
    if !plugin_binary_path.exists() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("Plugin ikili dosyası bulunamadı: {:?}", plugin_binary_path)));
    }
    
    // Paket dizini oluştur
    let package_dir = output_dir.join(format!("{}-{}", plugin_id, plugin_version));
    fs::create_dir_all(&package_dir)?;
    
    // README.md, LICENSE ve CHANGELOG.md dosyalarını kontrol et
    let readme_path = plugin_dir.join("README.md");
    let license_path = plugin_dir.join("LICENSE");
    let changelog_path = plugin_dir.join("CHANGELOG.md");
    
    let readme_exists = readme_path.exists();
    let license_exists = license_path.exists();
    let changelog_exists = changelog_path.exists();
    
    // Plugin ikili dosyasını kopyala
    let target_binary_path = package_dir.join(&plugin_binary_name);
    fs::copy(&plugin_binary_path, &target_binary_path)?;
    
    // Diğer dosyaları kopyala
    if readme_exists {
        fs::copy(&readme_path, package_dir.join("README.md"))?;
    }
    
    if license_exists {
        fs::copy(&license_path, package_dir.join("LICENSE"))?;
    }
    
    if changelog_exists {
        fs::copy(&changelog_path, package_dir.join("CHANGELOG.md"))?;
    }
    
    // Gerekli izinleri belirle
    let permissions = determine_required_permissions(&plugin_dir)?;
    
    // Plugin bağımlılıklarını belirle
    let dependencies = determine_dependencies(&cargo_toml)?;
    
    // SHA-256 hash hesapla
    let binary_content = fs::read(&target_binary_path)?;
    let mut hasher = Sha256::new();
    hasher.update(&binary_content);
    let hash = hasher.finalize();
    let sha256_hash = format!("{:x}", hash);
    
    // Paket bilgisi oluştur
    let now: DateTime<Utc> = Utc::now();
    let package_info = PackageInfo {
        created_at: now.to_rfc3339(),
        file_count: 1 + readme_exists as usize + license_exists as usize + changelog_exists as usize,
        total_size_bytes: binary_content.len() as u64 + 
            (if readme_exists { fs::metadata(&readme_path)?.len() } else { 0 }) +
            (if license_exists { fs::metadata(&license_path)?.len() } else { 0 }) +
            (if changelog_exists { fs::metadata(&changelog_path)?.len() } else { 0 }),
        sha256_hash,
        plugin_binary: plugin_binary_name,
        readme: if readme_exists { Some("README.md".to_string()) } else { None },
        license: if license_exists { Some("LICENSE".to_string()) } else { None },
        changelog: if changelog_exists { Some("CHANGELOG.md".to_string()) } else { None },
    };
    
    // Manifest oluştur
    let mut manifest = PluginManifest {
        id: format!("com.tauri.plugins.{}", plugin_id),
        name: plugin_name.to_string(),
        version: plugin_version.to_string(),
        description: plugin_description.to_string(),
        plugin_type: plugin_type.to_string(),
        vendor: plugin_vendor,
        vendor_url: package_data.get("license").and_then(|v| v.as_str()).map(|s| s.to_string()),
        permissions,
        min_host_version: Some("0.1.0".to_string()),
        dependencies,
        package: package_info,
        signature: None,
    };
    
    // İmzalama gerekiyorsa
    if sign_package {
        // Test için sabit bir private key kullanıyoruz
        // Gerçek uygulamada, güvenli bir yerden private key yüklenmelidir
        let private_key_base64 = "MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC+Z2mi8shZ6Qhi\
                               5htJi1q3dI5rVjSKdcH+LFTFr/OzbFIjhONJ1cuA+kN/MjMF5BoQVdDRvtZn9QFF\
                               Yx31fawm+nuVBJH5jIU9Vy7/iQp4PHOd7mZbYVlIZwQqw3RrYH4jYAl9DSyfBPYv\
                               O9KbSOm/uqUFmKN+MwqJe7N5VUxKzX1xfM8JCEUyQOxBZ2LQ+mLBxkG4VTXfW8QA\
                               kQvQHnKR2j8yKOWctYyZ+8xNFcL8C8n1q+Xy1Se9e4FXz/J8DYLIwzlOlFZDYYz\
                               jUG9W6jt3IkW+ycnADndrX2sJq0U/mN/LsR1+YayVgITI0D2A/Iv+gLQBxHgGJVe\
                               L+CA7URGfAgMBAAECggEAMgFIACiOv1/GHa8dejo3mB0bEtyGRpkSN9wPQhx5cgL";
        
        // Private key'i decode et
        let private_key = decode(private_key_base64)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Private key decode hatası: {}", e)))?;
        
        // İmzalama
        println!("\nPlugin imzalanıyor...");
        
        // Manifest içeriğini bir string olarak al
        manifest.signature = None; // İmzalama öncesi signature alanını temizle
        let manifest_json = serde_json::to_string(&manifest)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Manifest JSON hatası: {}", e)))?;
        
        // RSA key pair oluştur
        let key_pair = RsaKeyPair::from_pkcs8(&private_key)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("RSA key pair hatası: {}", e)))?;
        
        // İmzalama için hash hesapla
        let mut manifest_hasher = Sha256::new();
        manifest_hasher.update(manifest_json.as_bytes());
        let manifest_hash = manifest_hasher.finalize();
        
        // İmzala
        let mut signature = vec![0; key_pair.public_modulus_len()];
        key_pair.sign(&signature::RSA_PKCS1_SHA256, &manifest_hash, &mut signature)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("İmzalama hatası: {}", e)))?;
        
        // İmza bilgisini ekle
        manifest.signature = Some(SignatureInfo {
            algorithm: "RSA-SHA256".to_string(),
            signature: encode(&signature),
            public_key_id: "tauri-test-key-2025".to_string(),
            signed_at: Utc::now().to_rfc3339(),
            signer: "Tauri Windows Plugin System".to_string(),
        });
        
        println!("Plugin başarıyla imzalandı.");
    }
    
    // Manifest dosyasını yaz
    let manifest_path = package_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Manifest JSON hatası: {}", e)))?;
    
    fs::write(&manifest_path, manifest_json)?;
    println!("\nPlugin manifest dosyası oluşturuldu: {:?}", manifest_path);
    
    // ZIP paketi oluştur
    let zip_path = output_dir.join(format!("{}-{}.zip", plugin_id, plugin_version));
    create_zip_package(&package_dir, &zip_path)?;
    println!("Plugin paketlendi: {:?}", zip_path);
    
    println!("\nPlugin paketleme tamamlandı!");
    Ok(())
}

// Gerekli izinleri belirle
fn determine_required_permissions(plugin_dir: &Path) -> io::Result<Vec<String>> {
    let mut permissions = Vec::new();
    
    // src/ dizinindeki Rust dosyalarını tara
    let src_dir = plugin_dir.join("src");
    if !src_dir.is_dir() {
        return Ok(permissions);
    }
    
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
            let content = fs::read_to_string(&path)?;
            
            // Dosya içeriğine göre izinleri belirle
            if content.contains("fs::") || content.contains("File::") || content.contains("fs.") {
                permissions.push("fs.read".to_string());
                permissions.push("fs.write".to_string());
            }
            
            if content.contains("process::") || content.contains("Command::") {
                permissions.push("process.spawn".to_string());
            }
            
            if content.contains("reqwest::") || content.contains("http::") || content.contains("net::") {
                permissions.push("network.connect".to_string());
            }
            
            if content.contains("Registry::") || content.contains("RegKey::") {
                permissions.push("registry.read".to_string());
                permissions.push("registry.write".to_string());
            }
        }
    }
    
    // Tekrar eden izinleri kaldır
    permissions.sort();
    permissions.dedup();
    
    Ok(permissions)
}

// Bağımlılıkları belirle
fn determine_dependencies(cargo_toml: &str) -> io::Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    
    let cargo_data: toml::Value = toml::from_str(cargo_toml)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Cargo.toml ayrıştırma hatası: {}", e)))?;
    
    if let Some(deps) = cargo_data.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            // Plugin sistem bağımlılıklarını ekle
            if name.starts_with("tauri-") || name.starts_with("plugin-") {
                let version = if value.is_str() {
                    value.as_str().unwrap().to_string()
                } else if let Some(table) = value.as_table() {
                    table.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("*").to_string()
                } else {
                    "*".to_string()
                };
                
                dependencies.push(Dependency {
                    id: format!("com.tauri.{}", name),
                    version,
                    optional: false,
                });
            }
        }
    }
    
    Ok(dependencies)
}

// ZIP paketi oluştur
fn create_zip_package(source_dir: &Path, zip_path: &Path) -> io::Result<()> {
    let zip_file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(zip_file);
    
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);
    
    fn add_dir_to_zip(
        zip: &mut ZipWriter<File>,
        options: &FileOptions,
        dir: &Path,
        base_path: &Path,
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            let name = path.strip_prefix(base_path)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Path strip hatası: {}", e)))?;
            
            if path.is_file() {
                zip.start_file(name.to_string_lossy(), *options)?;
                let mut file = File::open(&path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            } else if path.is_dir() {
                zip.add_directory(name.to_string_lossy(), *options)?;
                add_dir_to_zip(zip, options, &path, base_path)?;
            }
        }
        
        Ok(())
    }
    
    add_dir_to_zip(&mut zip, &options, source_dir, source_dir)?;
    zip.finish()?;
    
    Ok(())
}
