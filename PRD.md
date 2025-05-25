Product Requirements Document (PRD) for Tauri Windows Plugin System

1. Introduction 1.1. Purpose - Tanımlanan PRD, Windows üzerinde çalışan Tauri tabanlı uygulamamız için dinamik plugin yükleme ve yönetimi mimarisini eksiksiz şekilde tanımlar. - Amaç: Geliştiricilerin ve kullanıcıların eklentileri kolayca yükleyip kaldırabilecekleri, güvenli ve ölçeklenebilir bir altyapı sunmak.

1.2. Kapsam - Plugin paket formatı - Yükleme, kaldırma, güncelleme iş akışları - Host uygulama tarafı yükleme/çalıştırma mekanizmaları - GUI entegrasyonu - Güvenlik, izin, imza doğrulama - Versioning ve uyumluluk kontrolleri - Test, CI/CD süreci

1.3. Tanımlar ve Kısaltmalar - PRD: Product Requirements Document - IPC: Inter-Process Communication - ABI: Application Binary Interface - DLL: Dynamic Link Library - WASM: WebAssembly


2. Hedefler ve Başarı Kriterleri 2.1. Hedefler - Kullanıcı dostu plugin yönetimi GUI’si - Plugin’lerin runtime’da sorunsuz yüklenip unload edilmesi - Sıkı güvenlik politikaları (sandbox, imza doğrulama) - Performanslı ve düşük gecikmeli IPC - Kolay test edilebilir, bakım dostu kod tabanı

2.2. Başarı Kriterleri - %100 birim ve entegrasyon test kapsamı - İlk sürümde en az 3 örnek plugin çalıştırma - Yükleme/kaldırma işlemi < 2 saniye - Güvenlik taramalarında kritik açık olmaması


3. Paydaşlar

Ürün Sahibi (Product Owner)

Yazılım Mühendisleri (Rust, Frontend)

QA Ekibi

DevOps/CI/CD Mühendisleri

Son Kullanıcı (Plugin geliştiricileri ve uygulama kullanıcıları)



4. Kullanıcı Hikayeleri

US1: Kullanıcı, dahili mağazadan bir plugin seçip yüklemek ister.

US2: Geliştirici, manuel ZIP yükleyerek plugin test etmek ister.

US3: Kullanıcı, yüklü pluginleri listeler ve herhangi birini devre dışı bırakmak ister.

US4: Sistem, manifest uyumsuzluğu tespit ederse kullanıcıyı uyarır.

US5: CI/CD pipeline, her plugin PR’si için otomatik yükleme ve test yapar.



5. Fonksiyonel Gereksinimler 5.1. Plugin Paket Formatı - ZIP arşiv, içinde plugin.json manifest ve plugin.dll - plugin.json alanları: name, version, entry, api_version, permissions 5.2. Yükleme Süreci - GUI: Dosya seçici veya mağaza arayüzü - Doğrulama: Şema, izin, imza - Çıkarma: %LOCALAPPDATA%/MyApp/plugins/{plugin_name} - Metadata güncelleme: installed_plugins.json 5.3. Kaldırma Süreci - GUI’de seçim - Dosya sistemi temizliği ve metadata güncelleme - Runtime unload (API aboneliklerinin iptali) 5.4. Plugin Host - libloading tabanlı DLL yükleme - Ortak C ABI fonksiyonları: plugin_init, plugin_teardown - PluginHost trait arayüzü 5.5. GUI Entegrasyonu - WebView: window.plugins listesi, yükle/araştır butonları - Tauri invoke aracılığıyla komut gönderme 5.6. Güncelleme Mekanizması - Mağaza API: Versiyon kontrolü - Otomatik veya manuel güncelleme seçeneği


6. Teknik ve Teknik Olmayan Gereksinimler 6.1. Performans - Ortalama yükleme süresi < 2s - Plugin başına bellek kullanımı < 20MB 6.2. Güvenlik - Her plugin izin kontrolünden geçmeli - Opsiyonel SHA256 imza doğrulaması - WASM sandbox geleceğe dönük roadmap’te 6.3. Uyumluluk - Windows 10 ve üzeri - Tauri v1.6+ 6.4. Erişebilirlik - GUI WCAG 2.1 AA uyumlu temel öğeler


7. Mimari ve Veri Tasarımı 7.1. Katmanlı Mimari - UI (WebView) - IPC katmanı (Tauri Commands/Events) - Plugin Loader (Rust libloading) - Plugin API (C ABI) - Dosya Sistemi (plugin dizini, metadata, log) 7.2. Veri Yapıları - installed_plugins.json: Liste halinde plugin objeleri - plugin.json manifest şeması


8. Test ve Kalite Güvencesi

Birim test: PluginHost trait implementasyonları

Entegrasyon test: Örnek plugin yükleme, unload

Fuzz test: Bozuk manifest, hatalı DLL

CI/CD: GitHub Actions veya benzeri pipeline



9. Yol Haritası ve Zaman Çizelgesi

Sprint 1: Paket formatı ve manifest şeması oluşturma (2 hafta)

Sprint 2: Plugin loader ve libloading entegrasyonu (2 hafta)

Sprint 3: GUI yükleme/kaldırma arayüzü geliştirme (2 hafta)

Sprint 4: Güvenlik ve imza doğrulama (1 hafta)

Sprint 5: Testler ve CI/CD pipeline (2 hafta)

Sprint 6: Dokümantasyon ve örnek pluginler (1 hafta)



10. Kabul Kriterleri



Tüm fonksiyonel gereksinimler karşılanmalı

Dokümantasyon eksiksiz olmalı

QA onayı ve kullanıcı kabul testi başarıyla tamamlanmalı


11. Ekler



Manifest JSON şeması (appendiks)

Örnek plugin repository bağlantısı

Referanslar: Tauri Plugin System Guide, libloading crate docs