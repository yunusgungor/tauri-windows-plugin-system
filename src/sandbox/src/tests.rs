#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    
    // Test yardımcı fonksiyonu: Test için process oluşturma
    fn create_test_process() -> u32 {
        // Windows'ta basit bir test uygulaması başlat (notepad)
        let child = Command::new("notepad.exe")
            .spawn()
            .expect("Test process başlatılamadı");
        
        // Process ID'yi al
        child.id()
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_sandbox_creation() {
        let sandbox_manager = SandboxManager::new();
        
        // Test process'i oluştur
        let process_id = create_test_process();
        
        // Özel kaynak sınırlamaları belirle
        let resource_limits = ResourceLimits {
            max_memory_mb: 128,
            max_cpu_percentage: 20,
            max_process_count: 3,
            max_working_set_mb: 64,
        };
        
        // Process'i sandbox'la
        let result = sandbox_manager.sandbox_plugin(
            "test-plugin",
            process_id,
            Some(resource_limits),
            vec![PermissionLevel::Core],
        );
        
        // Sandbox oluşturma başarılı olmalı
        assert!(result.is_ok(), "Sandbox oluşturma başarısız: {:?}", result);
        
        // Plugin listesinde olmalı
        let plugins = sandbox_manager.list_sandboxed_plugins();
        assert!(plugins.contains(&"test-plugin".to_string()));
        
        // Plugin bilgilerini kontrol et
        let status = sandbox_manager.get_plugin_sandbox_status("test-plugin");
        assert!(status.is_some());
        
        if let Some((pid, limits, permissions)) = status {
            assert_eq!(pid, process_id);
            assert_eq!(limits.max_memory_mb, 128);
            assert_eq!(permissions.len(), 1);
            assert_eq!(permissions[0], PermissionLevel::Core);
        }
        
        // Sandbox'ı temizle
        let cleanup_result = sandbox_manager.remove_sandbox("test-plugin");
        assert!(cleanup_result.is_ok());
        
        // Process'i sonlandır
        let _ = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .output();
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_permission_check() {
        let sandbox_manager = SandboxManager::new();
        
        // Test process'i oluştur
        let process_id = create_test_process();
        
        // Belirli izinlerle sandbox'la
        let result = sandbox_manager.sandbox_plugin(
            "permission-test-plugin",
            process_id,
            None,
            vec![PermissionLevel::Core, PermissionLevel::Filesystem],
        );
        
        assert!(result.is_ok());
        
        // İzin kontrollerini yap
        let has_core = sandbox_manager.check_permission(
            "permission-test-plugin", 
            PermissionLevel::Core
        );
        assert!(has_core.is_ok());
        assert!(has_core.unwrap());
        
        let has_filesystem = sandbox_manager.check_permission(
            "permission-test-plugin", 
            PermissionLevel::Filesystem
        );
        assert!(has_filesystem.is_ok());
        assert!(has_filesystem.unwrap());
        
        // Olmayan bir izni kontrol et
        let has_network = sandbox_manager.check_permission(
            "permission-test-plugin", 
            PermissionLevel::Network
        );
        assert!(has_network.is_ok());
        assert!(!has_network.unwrap());
        
        // Sandbox'ı temizle
        let _ = sandbox_manager.remove_sandbox("permission-test-plugin");
        
        // Process'i sonlandır
        let _ = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .output();
    }
    
    #[test]
    #[cfg(target_os = "windows")]
    fn test_resource_limits() {
        let sandbox_manager = SandboxManager::new();
        
        // Test process'i oluştur
        let process_id = create_test_process();
        
        // Çok kısıtlı kaynak sınırlamaları belirle
        let resource_limits = ResourceLimits {
            max_memory_mb: 32,       // Çok düşük bellek limiti
            max_cpu_percentage: 10,
            max_process_count: 1,    // Yeni process oluşturmayı engelle
            max_working_set_mb: 16,
        };
        
        // Process'i sandbox'la
        let result = sandbox_manager.sandbox_plugin(
            "resource-test-plugin",
            process_id,
            Some(resource_limits),
            vec![PermissionLevel::Core],
        );
        
        assert!(result.is_ok());
        
        // Kısa bir bekleme süresi
        thread::sleep(Duration::from_secs(2));
        
        // Sandbox'ı temizle
        let _ = sandbox_manager.remove_sandbox("resource-test-plugin");
        
        // Process'i sonlandır
        let _ = Command::new("taskkill")
            .args(["/PID", &process_id.to_string(), "/F"])
            .output();
    }
}
