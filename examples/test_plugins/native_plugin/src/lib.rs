// Tauri Windows Plugin System - Resource Usage Test Plugin
//
// Bu plugin, Windows API'leri kullanarak sistem kaynak kullanımını ölçer.
// Sandbox ve Resource Monitor modüllerini test etmek için kullanılır.

use plugin_interface::{
    PluginError, PluginInterface, PluginMetadata, PluginType, 
    ResourceMonitorPlugin, ResourceUsage,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use windows::Win32::System::{
    ProcessStatus::GetProcessMemoryInfo,
    SystemInformation::GetSystemInfo,
    Threading::{GetCurrentProcess, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
};

// Dinamik kitaplık ihraç sembolleri
#[no_mangle]
pub extern "C" fn create_plugin() -> *mut dyn PluginInterface {
    let plugin = ResourceUsagePlugin::new();
    Box::into_raw(Box::new(plugin))
}

#[no_mangle]
pub extern "C" fn destroy_plugin(plugin: *mut dyn PluginInterface) {
    if !plugin.is_null() {
        unsafe {
            drop(Box::from_raw(plugin));
        }
    }
}

/// Kaynak Kullanımı Plugin uygulaması
pub struct ResourceUsagePlugin {
    /// Plugin ID'si
    id: String,
    /// Plugin metadatası
    metadata: PluginMetadata,
    /// Plugin başlatıldı mı?
    initialized: bool,
    /// İzleme durumu
    monitoring: bool,
    /// İzleme iş parçacığı handle'ı
    monitor_thread: Option<thread::JoinHandle<()>>,
    /// Kaynak kullanımı
    resource_usage: Arc<Mutex<ResourceUsage>>,
    /// CPU sınırı
    cpu_limit: Option<f64>,
    /// Bellek sınırı
    memory_limit: Option<u64>,
    /// Başlangıç zamanı
    start_time: Option<Instant>,
}

impl ResourceUsagePlugin {
    /// Yeni bir kaynak kullanımı plugin'i oluşturur
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            id: "com.tauri.plugins.resource-usage".to_string(),
            name: "Resource Usage Plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "Windows sistem kaynak kullanımını ölçen test plugin".to_string(),
            plugin_type: PluginType::Native,
            vendor: "Tauri Windows Plugin System Team".to_string(),
            vendor_url: Some("https://tauri.app".to_string()),
            permissions: vec![
                "process.query".to_string(),
                "system.info".to_string(),
                "network.status".to_string(),
            ],
            min_host_version: Some("0.1.0".to_string()),
        };
        
        Self {
            id: metadata.id.clone(),
            metadata,
            initialized: false,
            monitoring: false,
            monitor_thread: None,
            resource_usage: Arc::new(Mutex::new(ResourceUsage {
                cpu_percent: 0.0,
                memory_bytes: 0,
                disk_bytes_per_sec: 0,
                network_bytes_per_sec: 0,
            })),
            cpu_limit: None,
            memory_limit: None,
            start_time: None,
        }
    }
    
    /// Windows performans sayaçları yoluyla CPU kullanımını ölçer
    fn measure_cpu_usage(&self) -> Result<f64, PluginError> {
        // Basit bir örnek olarak, sabit bir değer döndürelim
        // Gerçek uygulamada Windows Performance Counters API kullanılabilir
        Ok(25.5)
    }
    
    /// Mevcut process için bellek kullanımını ölçer
    fn measure_memory_usage(&self) -> Result<u64, PluginError> {
        unsafe {
            let process_handle = GetCurrentProcess();
            
            // PROCESS_MEMORY_COUNTERS_EX yapısını hazırla
            let mut mem_counters = std::mem::zeroed();
            let result = GetProcessMemoryInfo(
                process_handle,
                &mut mem_counters,
                std::mem::size_of::<windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS>() as u32,
            );
            
            if result.is_ok() {
                // WorkingSetSize, process'in fiziksel bellek kullanımını gösterir
                Ok(mem_counters.WorkingSetSize as u64)
            } else {
                Err(PluginError::Resource("Bellek kullanımı ölçülemedi".to_string()))
            }
        }
    }
    
    /// Disk I/O ölçümü
    fn measure_disk_usage(&self) -> Result<u64, PluginError> {
        // Basit bir örnek olarak, sabit bir değer döndürelim
        // Gerçek uygulamada Windows I/O performans sayaçları kullanılabilir
        Ok(1024 * 1024) // 1 MB/s
    }
    
    /// Ağ I/O ölçümü
    fn measure_network_usage(&self) -> Result<u64, PluginError> {
        // Basit bir örnek olarak, sabit bir değer döndürelim
        // Gerçek uygulamada Windows IPHelper API kullanılabilir
        Ok(512 * 1024) // 512 KB/s
    }
    
    /// Kaynak sınırlarını kontrol et
    fn check_resource_limits(&self, usage: &ResourceUsage) -> Result<(), PluginError> {
        // CPU sınırını kontrol et
        if let Some(cpu_limit) = self.cpu_limit {
            if usage.cpu_percent > cpu_limit {
                return Err(PluginError::Resource(format!(
                    "CPU kullanımı sınırı aşıldı: {:.1}% > {:.1}%", 
                    usage.cpu_percent, 
                    cpu_limit
                )));
            }
        }
        
        // Bellek sınırını kontrol et
        if let Some(memory_limit) = self.memory_limit {
            if usage.memory_bytes > memory_limit {
                return Err(PluginError::Resource(format!(
                    "Bellek kullanımı sınırı aşıldı: {} > {} bytes", 
                    usage.memory_bytes, 
                    memory_limit
                )));
            }
        }
        
        Ok(())
    }
}

impl PluginInterface for ResourceUsagePlugin {
    fn get_id(&self) -> &str {
        &self.id
    }
    
    fn get_metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    fn initialize(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Err(PluginError::General("Plugin zaten başlatılmış".to_string()));
        }
        
        // Başlangıç zamanını kaydet
        self.start_time = Some(Instant::now());
        self.initialized = true;
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> Result<(), PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        // İzleme yapılıyorsa durdur
        if self.monitoring {
            self.stop_monitoring()?;
        }
        
        self.initialized = false;
        self.start_time = None;
        
        Ok(())
    }
    
    fn execute_command(&mut self, command: &str, args: &str) -> Result<String, PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        match command {
            "get_usage" => {
                let usage = self.measure_resource_usage()?;
                Ok(serde_json::to_string(&usage).map_err(|e| 
                    PluginError::Serialization(format!("Serileştirme hatası: {}", e))
                )?)
            },
            "start_monitoring" => {
                self.start_monitoring()?;
                Ok("Monitoring started".to_string())
            },
            "stop_monitoring" => {
                self.stop_monitoring()?;
                Ok("Monitoring stopped".to_string())
            },
            "set_limits" => {
                #[derive(Deserialize)]
                struct Limits {
                    cpu_percent: Option<f64>,
                    memory_bytes: Option<u64>,
                }
                
                let limits: Limits = serde_json::from_str(args).map_err(|e| 
                    PluginError::Serialization(format!("Deserileştirme hatası: {}", e))
                )?;
                
                self.set_resource_limits(limits.cpu_percent, limits.memory_bytes)?;
                Ok("Limits set".to_string())
            },
            "simulate_high_cpu" => {
                // Yüksek CPU kullanımını simüle et
                thread::spawn(|| {
                    let start = Instant::now();
                    while start.elapsed() < Duration::from_secs(5) {
                        // CPU kullanımı yarat
                        let mut x = 0.0;
                        for i in 0..10000000 {
                            x += (i as f64).sin();
                        }
                    }
                });
                
                Ok("High CPU simulation started".to_string())
            },
            "simulate_high_memory" => {
                // Yüksek bellek kullanımını simüle et
                let memory_size = args.parse::<usize>().unwrap_or(100 * 1024 * 1024); // Varsayılan 100MB
                let data = vec![0u8; memory_size];
                thread::sleep(Duration::from_secs(5));
                std::mem::drop(data);
                
                Ok("High memory simulation completed".to_string())
            },
            _ => Err(PluginError::Api(format!("Bilinmeyen komut: {}", command))),
        }
    }
}

impl ResourceMonitorPlugin for ResourceUsagePlugin {
    fn measure_resource_usage(&self) -> Result<ResourceUsage, PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        let cpu = self.measure_cpu_usage()?;
        let memory = self.measure_memory_usage()?;
        let disk = self.measure_disk_usage()?;
        let network = self.measure_network_usage()?;
        
        let usage = ResourceUsage {
            cpu_percent: cpu,
            memory_bytes: memory,
            disk_bytes_per_sec: disk,
            network_bytes_per_sec: network,
        };
        
        // Sınırları kontrol et
        self.check_resource_limits(&usage)?;
        
        // Mevcut kullanımı güncelle
        if let Ok(mut current_usage) = self.resource_usage.lock() {
            *current_usage = usage.clone();
        }
        
        Ok(usage)
    }
    
    fn set_resource_limits(&mut self, cpu_percent: Option<f64>, memory_bytes: Option<u64>) -> Result<(), PluginError> {
        self.cpu_limit = cpu_percent;
        self.memory_limit = memory_bytes;
        
        Ok(())
    }
    
    fn start_monitoring(&mut self) -> Result<(), PluginError> {
        if !self.initialized {
            return Err(PluginError::General("Plugin başlatılmamış".to_string()));
        }
        
        if self.monitoring {
            return Err(PluginError::General("İzleme zaten aktif".to_string()));
        }
        
        self.monitoring = true;
        let resource_usage = self.resource_usage.clone();
        let plugin_id = self.id.clone();
        
        // Periyodik izleme iş parçacığı
        self.monitor_thread = Some(thread::spawn(move || {
            let plugin = ResourceUsagePlugin::new();
            
            while let Ok(true) = thread::scope(|_| {
                // Her 1 saniyede bir ölçüm yap
                thread::sleep(Duration::from_secs(1));
                
                // Kaynak kullanımını ölç
                match plugin.measure_cpu_usage() {
                    Ok(cpu) => {
                        match plugin.measure_memory_usage() {
                            Ok(memory) => {
                                match plugin.measure_disk_usage() {
                                    Ok(disk) => {
                                        match plugin.measure_network_usage() {
                                            Ok(network) => {
                                                // Kaynak kullanımını güncelle
                                                if let Ok(mut usage) = resource_usage.lock() {
                                                    usage.cpu_percent = cpu;
                                                    usage.memory_bytes = memory;
                                                    usage.disk_bytes_per_sec = disk;
                                                    usage.network_bytes_per_sec = network;
                                                }
                                            },
                                            Err(e) => log::error!("[{}] Ağ kullanımı ölçüm hatası: {:?}", plugin_id, e),
                                        }
                                    },
                                    Err(e) => log::error!("[{}] Disk kullanımı ölçüm hatası: {:?}", plugin_id, e),
                                }
                            },
                            Err(e) => log::error!("[{}] Bellek kullanımı ölçüm hatası: {:?}", plugin_id, e),
                        }
                    },
                    Err(e) => log::error!("[{}] CPU kullanımı ölçüm hatası: {:?}", plugin_id, e),
                }
                
                // İzleme devam ediyor mu kontrol et
                Ok(true)
            }) {}
        }));
        
        Ok(())
    }
    
    fn stop_monitoring(&mut self) -> Result<(), PluginError> {
        if !self.monitoring {
            return Ok(());
        }
        
        self.monitoring = false;
        
        // İzleme iş parçacığını sonlandır
        if let Some(handle) = self.monitor_thread.take() {
            // Thread'in kendiliğinden sonlanmasını bekle
            match handle.join() {
                Ok(_) => (),
                Err(e) => log::error!("İzleme iş parçacığı sonlandırma hatası: {:?}", e),
            }
        }
        
        Ok(())
    }
}
