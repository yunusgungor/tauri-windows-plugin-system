// Tauri Windows Plugin System - Mock Plugin Store Server
//
// Bu sunucu, Plugin Store Client için test amaçlı bir API sağlar.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    net::SocketAddr,
    path::{Path as FilePath, PathBuf},
    sync::{Arc, RwLock},
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use uuid::Uuid;

// Plugin kategorileri
#[derive(Debug, Clone, Serialize, Deserialize)]
enum PluginCategory {
    Security,
    Development,
    Utility,
    Media,
    Network,
    System,
    Other,
}

// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginMetadata {
    id: String,
    name: String,
    version: String,
    description: String,
    plugin_type: String,
    vendor: String,
    vendor_url: Option<String>,
    permissions: Vec<String>,
    min_host_version: Option<String>,
    categories: Vec<PluginCategory>,
    tags: Vec<String>,
    rating: Option<f32>,
    download_count: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    icon_url: Option<String>,
    price: Option<PluginPrice>,
}

// Plugin fiyatlandırma
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginPrice {
    amount: f64,
    currency: String,
    model: PricingModel,
}

// Fiyatlandırma modeli
#[derive(Debug, Clone, Serialize, Deserialize)]
enum PricingModel {
    Free,
    OneTime,
    Subscription,
}

// Plugin paket bilgisi
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignatureInfo {
    algorithm: String,
    signature: String,
    public_key_id: String,
    signed_at: String,
    signer: String,
}

// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    description: String,
    plugin_type: String,
    vendor: String,
    vendor_url: Option<String>,
    permissions: Vec<String>,
    min_host_version: Option<String>,
    dependencies: Vec<Dependency>,
    package: PackageInfo,
    signature: Option<SignatureInfo>,
}

// Plugin bağımlılığı
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Dependency {
    id: String,
    version: String,
    optional: bool,
}

// Plugin arama filtresi
#[derive(Debug, Deserialize)]
struct PluginSearchFilter {
    query: Option<String>,
    category: Option<PluginCategory>,
    plugin_type: Option<String>,
    min_rating: Option<f32>,
    limit: Option<usize>,
    offset: Option<usize>,
    sort_by: Option<String>,
}

// Plugin arama sonucu
#[derive(Debug, Serialize)]
struct PluginSearchResult {
    total_count: usize,
    offset: usize,
    limit: usize,
    plugins: Vec<PluginMetadata>,
}

// Plugin indirme bilgisi
#[derive(Debug, Serialize)]
struct PluginDownloadInfo {
    download_url: String,
    file_size: u64,
    sha256_hash: String,
}

// Plugin indirme isteği
#[derive(Debug, Deserialize)]
struct PluginDownloadRequest {
    plugin_id: String,
    version: String,
    platform: String,
    arch: String,
}

// API hata
#[derive(Debug, Serialize)]
struct ApiError {
    code: String,
    message: String,
}

// API yanıtı
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ApiError>,
}

// API token
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiToken {
    token: String,
    user_id: String,
    expires_at: DateTime<Utc>,
}

// Kullanıcı bilgisi
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    username: String,
    email: String,
    created_at: DateTime<Utc>,
}

// Giriş isteği
#[derive(Debug, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

// Giriş yanıtı
#[derive(Debug, Serialize)]
struct LoginResponse {
    token: String,
    user: User,
}

// Uygulama durumu
#[derive(Debug, Clone)]
struct AppState {
    plugins: Arc<RwLock<HashMap<String, PluginMetadata>>>,
    plugin_files: Arc<RwLock<HashMap<String, PathBuf>>>,
    tokens: Arc<RwLock<HashMap<String, ApiToken>>>,
    users: Arc<RwLock<HashMap<String, User>>>,
    data_dir: PathBuf,
}

// Özel API hata yanıtı
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code.as_str() {
            "not_found" => StatusCode::NOT_FOUND,
            "unauthorized" => StatusCode::UNAUTHORIZED,
            "bad_request" => StatusCode::BAD_REQUEST,
            "internal_error" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        let api_response = ApiResponse {
            success: false,
            data: None::<()>,
            error: Some(self),
        };
        
        let body = Json(api_response);
        (status, body).into_response()
    }
}

#[tokio::main]
async fn main() {
    // Tracing ayarla
    tracing_subscriber::fmt::init();
    
    let data_dir = PathBuf::from("data");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Veri dizini oluşturulamadı");
    }
    
    // Test verilerini oluştur
    let plugins = Arc::new(RwLock::new(HashMap::new()));
    let plugin_files = Arc::new(RwLock::new(HashMap::new()));
    let tokens = Arc::new(RwLock::new(HashMap::new()));
    let users = Arc::new(RwLock::new(HashMap::new()));
    
    // Test kullanıcısı ekle
    let test_user = User {
        id: "user-123".to_string(),
        username: "test".to_string(),
        email: "test@tauri.app".to_string(),
        created_at: Utc::now(),
    };
    
    users.write().unwrap().insert(test_user.id.clone(), test_user.clone());
    
    // Test token'ı ekle
    let test_token = ApiToken {
        token: "test-token-123".to_string(),
        user_id: "user-123".to_string(),
        expires_at: Utc::now() + chrono::Duration::days(7),
    };
    
    tokens.write().unwrap().insert(test_token.token.clone(), test_token);
    
    // Plugins dizinindeki tüm plugin paketlerini yükle
    let plugins_dir = data_dir.join("plugins");
    if !plugins_dir.exists() {
        fs::create_dir_all(&plugins_dir).expect("Plugins dizini oluşturulamadı");
    }
    
    load_plugins_from_dir(&plugins_dir, &plugins, &plugin_files);
    
    // Uygulama durumu
    let app_state = AppState {
        plugins,
        plugin_files,
        tokens,
        users,
        data_dir,
    };
    
    // CORS yapılandırması
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    // API router'ı
    let api_routes = Router::new()
        .route("/plugins/search", get(search_plugins))
        .route("/plugins/:id", get(get_plugin))
        .route("/plugins/:id/download", post(download_plugin))
        .route("/auth/login", post(login));
    
    // Ana router
    let app = Router::new()
        .nest("/api/v1", api_routes)
        .nest_service("/files", ServeDir::new(data_dir.join("plugins")))
        .with_state(app_state)
        .layer(cors);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    
    println!("Plugin Store API Sunucusu çalışıyor: http://{}", addr);
    println!("Örnek API kullanımı:");
    println!("  - Plugin arama: http://{}/api/v1/plugins/search", addr);
    println!("  - Plugin detayı: http://{}/api/v1/plugins/com.tauri.plugins.resource-usage-plugin", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Plugins dizinindeki tüm plugin paketlerini yükle
fn load_plugins_from_dir(
    dir: &PathBuf,
    plugins: &Arc<RwLock<HashMap<String, PluginMetadata>>>,
    plugin_files: &Arc<RwLock<HashMap<String, PathBuf>>>,
) {
    // Gerçek uygulamada burada .zip dosyalarını tarayıp içlerindeki manifest'i okuyarak plugin'leri yükleriz
    // Bu örnekte, mock veriler oluşturacağız
    
    let mut plugins_map = plugins.write().unwrap();
    let mut files_map = plugin_files.write().unwrap();
    
    // Native Windows Plugin (Resource Usage)
    let resource_plugin_id = "com.tauri.plugins.resource-usage".to_string();
    plugins_map.insert(
        resource_plugin_id.clone(),
        PluginMetadata {
            id: resource_plugin_id.clone(),
            name: "Resource Usage Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Windows sistem kaynak kullanımını ölçen test plugin".to_string(),
            plugin_type: "native".to_string(),
            vendor: "Tauri Windows Plugin System Team".to_string(),
            vendor_url: Some("https://tauri.app".to_string()),
            permissions: vec![
                "process.query".to_string(),
                "system.info".to_string(),
                "network.status".to_string(),
            ],
            min_host_version: Some("0.1.0".to_string()),
            categories: vec![PluginCategory::System, PluginCategory::Utility],
            tags: vec!["system".to_string(), "resources".to_string(), "monitoring".to_string()],
            rating: Some(4.8),
            download_count: 1250,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            icon_url: Some("/files/resource-usage-plugin/icon.png".to_string()),
            price: Some(PluginPrice {
                amount: 0.0,
                currency: "USD".to_string(),
                model: PricingModel::Free,
            }),
        },
    );
    
    files_map.insert(
        resource_plugin_id,
        dir.join("resource-usage-plugin-0.1.0.zip"),
    );
    
    // WASM Plugin (Data Processor)
    let wasm_plugin_id = "com.tauri.plugins.data-processor".to_string();
    plugins_map.insert(
        wasm_plugin_id.clone(),
        PluginMetadata {
            id: wasm_plugin_id.clone(),
            name: "Data Processor Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "WASM tabanlı veri işleme plugin'i".to_string(),
            plugin_type: "wasm".to_string(),
            vendor: "Tauri Windows Plugin System Team".to_string(),
            vendor_url: Some("https://tauri.app".to_string()),
            permissions: vec![
                "data.read".to_string(),
                "data.write".to_string(),
            ],
            min_host_version: Some("0.1.0".to_string()),
            categories: vec![PluginCategory::Utility, PluginCategory::Development],
            tags: vec!["wasm".to_string(), "data".to_string(), "processing".to_string()],
            rating: Some(4.5),
            download_count: 850,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            icon_url: Some("/files/data-processor-plugin/icon.png".to_string()),
            price: Some(PluginPrice {
                amount: 0.0,
                currency: "USD".to_string(),
                model: PricingModel::Free,
            }),
        },
    );
    
    files_map.insert(
        wasm_plugin_id,
        dir.join("data-processor-plugin-0.1.0.zip"),
    );
    
    // Security Scanner Plugin
    let security_plugin_id = "com.tauri.plugins.security-scanner".to_string();
    plugins_map.insert(
        security_plugin_id.clone(),
        PluginMetadata {
            id: security_plugin_id.clone(),
            name: "Security Scanner Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Güvenlik taraması, imza doğrulama ve izin sistemi test plugin'i".to_string(),
            plugin_type: "native".to_string(),
            vendor: "Tauri Windows Plugin System Team".to_string(),
            vendor_url: Some("https://tauri.app".to_string()),
            permissions: vec![
                "fs.read".to_string(),
                "process.query".to_string(),
                "network.check".to_string(),
                "registry.read".to_string(),
            ],
            min_host_version: Some("0.1.0".to_string()),
            categories: vec![PluginCategory::Security, PluginCategory::System],
            tags: vec!["security".to_string(), "scanner".to_string(), "permissions".to_string()],
            rating: Some(4.9),
            download_count: 2100,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            icon_url: Some("/files/security-scanner-plugin/icon.png".to_string()),
            price: Some(PluginPrice {
                amount: 0.0,
                currency: "USD".to_string(),
                model: PricingModel::Free,
            }),
        },
    );
    
    files_map.insert(
        security_plugin_id,
        dir.join("security-scanner-plugin-0.1.0.zip"),
    );
}

// Plugin arama
async fn search_plugins(
    Query(filter): Query<PluginSearchFilter>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let plugins_map = app_state.plugins.read().unwrap();
    
    let mut filtered_plugins: Vec<_> = plugins_map.values().cloned().collect();
    
    // Filtrele
    if let Some(ref query) = filter.query {
        let query = query.to_lowercase();
        filtered_plugins.retain(|p| {
            p.name.to_lowercase().contains(&query) ||
            p.description.to_lowercase().contains(&query) ||
            p.vendor.to_lowercase().contains(&query) ||
            p.tags.iter().any(|t| t.to_lowercase().contains(&query))
        });
    }
    
    if let Some(ref category) = filter.category {
        filtered_plugins.retain(|p| p.categories.contains(category));
    }
    
    if let Some(ref plugin_type) = filter.plugin_type {
        filtered_plugins.retain(|p| p.plugin_type == *plugin_type);
    }
    
    if let Some(min_rating) = filter.min_rating {
        filtered_plugins.retain(|p| p.rating.unwrap_or(0.0) >= min_rating);
    }
    
    // Sırala
    if let Some(ref sort_by) = filter.sort_by {
        match sort_by.as_str() {
            "name" => filtered_plugins.sort_by(|a, b| a.name.cmp(&b.name)),
            "rating" => filtered_plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)),
            "downloads" => filtered_plugins.sort_by(|a, b| b.download_count.cmp(&a.download_count)),
            _ => {} // Varsayılan sıralama
        }
    } else {
        // Varsayılan olarak puana göre sırala
        filtered_plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    // Sayfalama
    let total_count = filtered_plugins.len();
    let offset = filter.offset.unwrap_or(0);
    let limit = filter.limit.unwrap_or(10).min(100); // Maksimum 100 sonuç
    
    let plugins = filtered_plugins
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    let result = PluginSearchResult {
        total_count,
        offset,
        limit,
        plugins,
    };
    
    Json(ApiResponse {
        success: true,
        data: Some(result),
        error: None,
    })
}

// Plugin detayı
async fn get_plugin(
    Path(id): Path<String>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    let plugins_map = app_state.plugins.read().unwrap();
    
    if let Some(plugin) = plugins_map.get(&id) {
        Json(ApiResponse {
            success: true,
            data: Some(plugin.clone()),
            error: None,
        })
    } else {
        ApiError {
            code: "not_found".to_string(),
            message: format!("Plugin bulunamadı: {}", id),
        }
        .into_response()
    }
}

// Plugin indirme
async fn download_plugin(
    Json(request): Json<PluginDownloadRequest>,
    headers: HeaderMap,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // API token'ı kontrol et
    let auth_header = headers.get(header::AUTHORIZATION);
    if auth_header.is_none() {
        return ApiError {
            code: "unauthorized".to_string(),
            message: "API token gerekiyor".to_string(),
        }
        .into_response();
    }
    
    let auth_header = auth_header.unwrap().to_str().unwrap_or("");
    if !auth_header.starts_with("Bearer ") {
        return ApiError {
            code: "unauthorized".to_string(),
            message: "Geçersiz API token formatı".to_string(),
        }
        .into_response();
    }
    
    let token = &auth_header[7..];
    
    let tokens_map = app_state.tokens.read().unwrap();
    if !tokens_map.contains_key(token) {
        return ApiError {
            code: "unauthorized".to_string(),
            message: "Geçersiz API token".to_string(),
        }
        .into_response();
    }
    
    // Plugin'i kontrol et
    let plugins_map = app_state.plugins.read().unwrap();
    let plugin_files_map = app_state.plugin_files.read().unwrap();
    
    if let Some(plugin) = plugins_map.get(&request.plugin_id) {
        if plugin.version != request.version {
            return ApiError {
                code: "bad_request".to_string(),
                message: format!("İstenen versiyon bulunamadı: {}", request.version),
            }
            .into_response();
        }
        
        if let Some(plugin_file) = plugin_files_map.get(&plugin.id) {
            // Download URL oluştur
            let file_name = plugin_file.file_name().unwrap().to_string_lossy();
            let download_url = format!("/files/{}", file_name);
            
            // Dosya boyutunu al
            let file_size = if plugin_file.exists() {
                fs::metadata(plugin_file).map(|m| m.len()).unwrap_or(0)
            } else {
                // Mock veri olduğu için varsayılan bir boyut döndür
                1024 * 1024 // 1 MB
            };
            
            let download_info = PluginDownloadInfo {
                download_url,
                file_size,
                sha256_hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(), // Örnek hash
            };
            
            Json(ApiResponse {
                success: true,
                data: Some(download_info),
                error: None,
            })
            .into_response()
        } else {
            ApiError {
                code: "not_found".to_string(),
                message: format!("Plugin dosyası bulunamadı: {}", request.plugin_id),
            }
            .into_response()
        }
    } else {
        ApiError {
            code: "not_found".to_string(),
            message: format!("Plugin bulunamadı: {}", request.plugin_id),
        }
        .into_response()
    }
}

// Giriş
async fn login(
    Json(request): Json<LoginRequest>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    // Test kullanıcısı kontrolü
    if request.username == "test" && request.password == "test123" {
        let users_map = app_state.users.read().unwrap();
        let user = users_map.get("user-123").unwrap().clone();
        
        // Yeni token oluştur
        let token = format!("token-{}", Uuid::new_v4());
        let api_token = ApiToken {
            token: token.clone(),
            user_id: user.id.clone(),
            expires_at: Utc::now() + chrono::Duration::days(7),
        };
        
        app_state.tokens.write().unwrap().insert(token.clone(), api_token);
        
        let response = LoginResponse {
            token,
            user,
        };
        
        Json(ApiResponse {
            success: true,
            data: Some(response),
            error: None,
        })
        .into_response()
    } else {
        ApiError {
            code: "unauthorized".to_string(),
            message: "Geçersiz kullanıcı adı veya şifre".to_string(),
        }
        .into_response()
    }
}
