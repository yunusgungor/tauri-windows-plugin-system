// Tauri Windows Plugin System - Sandbox Manager
// 
// Bu modül, Windows Job Objects kullanarak Tauri plugin'leri için güvenli izolasyon sağlar.
// Windows Job Objects, process gruplarını yönetmek ve sınırlandırmak için kullanılan 
// Windows-spesifik bir mekanizmadır.

#[cfg(test)]
mod tests;


use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicLimitInformation,
    JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_BASIC_LIMIT_INFORMATION,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
    JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_LIMIT_WORKINGSET,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, GetCurrentProcess, GetProcessId, OpenProcess, ResumeThread,
    PROCESS_ALL_ACCESS,
};

/// Sandbox yöneticisi hata türleri
#[derive(Error, Debug)]
pub enum SandboxError {
    #[error("Job Object oluşturma hatası: {0}")]
    JobObjectCreationError(String),

    #[error("Process Job Object'e eklenemedi: {0}")]
    ProcessAssignmentError(String),

    #[error("İşlem bulunamadı: {0}")]
    ProcessNotFoundError(String),

    #[error("Job Object limit ayarlama hatası: {0}")]
    SetLimitError(String),

    #[error("Sistem hatası: {0}")]
    SystemError(String),

    #[error("İzin hatası: {0}")]
    PermissionError(String),

    #[error("Geçersiz kaynak limiti: {0}")]
    InvalidResourceLimit(String),
}

/// Bir plugin için kaynak sınırlamaları
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maksimum bellek kullanımı (MB)
    pub max_memory_mb: u32,
    
    /// Maksimum CPU kullanımı (yüzde)
    pub max_cpu_percentage: u32,
    
    /// İzin verilen maksimum aktif process sayısı
    pub max_process_count: u32,
    
    /// Maksimum working set boyutu (MB)
    pub max_working_set_mb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,        // 512 MB varsayılan limit
            max_cpu_percentage: 30,    // %30 CPU kullanımı
            max_process_count: 5,      // maksimum 5 process
            max_working_set_mb: 256,   // 256 MB working set
        }
    }
}

/// Plugin izin düzeyleri
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionLevel {
    Core,       // Temel izinler
    Filesystem, // Dosya sistemi erişimi
    Network,    // Ağ erişimi
    UI,         // Kullanıcı arayüzü erişimi
    System,     // Sistem erişimi
    Interprocess, // Process'ler arası iletişim
}

/// Sandbox'lanmış bir plugin
#[derive(Debug)]
pub struct SandboxedPlugin {
    /// Benzersiz plugin tanımlayıcısı
    pub id: String,
    
    /// Plugin process ID
    pub process_id: u32,
    
    /// Windows Job Object handle
    job_handle: HANDLE,
    
    /// Kaynak sınırlamaları
    pub resource_limits: ResourceLimits,
    
    /// Plugin'in sahip olduğu izinler
    pub permissions: Vec<PermissionLevel>,
}

impl Drop for SandboxedPlugin {
    fn drop(&mut self) {
        // Job handle'ı kapat
        unsafe {
            if self.job_handle != INVALID_HANDLE_VALUE {
                let _ = CloseHandle(self.job_handle);
            }
        }
    }
}

/// Sandbox Manager - Tüm sandbox'ları yöneten ana sınıf
pub struct SandboxManager {
    /// Aktif sandbox'lanmış plugin'ler
    sandboxed_plugins: Arc<Mutex<HashMap<String, SandboxedPlugin>>>,
}

impl SandboxManager {
    /// Yeni bir Sandbox Manager oluştur
    pub fn new() -> Self {
        Self {
            sandboxed_plugins: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Bir plugin process'ini sandbox'la
    pub fn sandbox_plugin(
        &self,
        plugin_id: &str,
        process_id: u32,
        resource_limits: Option<ResourceLimits>,
        permissions: Vec<PermissionLevel>,
    ) -> Result<(), SandboxError> {
        let resource_limits = resource_limits.unwrap_or_default();
        
        // Benzersiz bir isimle Job Object oluştur
        let job_handle = unsafe {
            CreateJobObjectW(None, None)
        };
        
        if job_handle == INVALID_HANDLE_VALUE {
            return Err(SandboxError::JobObjectCreationError(
                "Job Object oluşturulamadı".to_string(),
            ));
        }
        
        // Job Object limitlerini ayarla
        self.set_job_limits(job_handle, &resource_limits)?;
        
        // Process'i aç
        let process_handle = unsafe {
            OpenProcess(PROCESS_ALL_ACCESS, false, process_id)
        };
        
        if process_handle == INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(job_handle);
            }
            return Err(SandboxError::ProcessNotFoundError(
                format!("Process ID {} açılamadı", process_id),
            ));
        }
        
        // Process'i Job Object'e ekle
        let result = unsafe {
            AssignProcessToJobObject(job_handle, process_handle)
        };
        
        // Process handle'ı kapat
        unsafe {
            CloseHandle(process_handle);
        }
        
        if !result.as_bool() {
            unsafe {
                CloseHandle(job_handle);
            }
            return Err(SandboxError::ProcessAssignmentError(
                "Process Job Object'e eklenemedi".to_string(),
            ));
        }
        
        // Sandbox'lanmış plugin'i kaydet
        let sandboxed_plugin = SandboxedPlugin {
            id: plugin_id.to_string(),
            process_id,
            job_handle,
            resource_limits,
            permissions,
        };
        
        let mut plugins = self.sandboxed_plugins.lock().unwrap();
        plugins.insert(plugin_id.to_string(), sandboxed_plugin);
        
        info!("Plugin {} başarıyla sandbox'landı (Process ID: {})", plugin_id, process_id);
        Ok(())
    }
    
    /// Job Object için kaynak limitlerini ayarla
    fn set_job_limits(&self, job_handle: HANDLE, limits: &ResourceLimits) -> Result<(), SandboxError> {
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        
        // Temel limitler
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_PROCESS_MEMORY 
            | JOB_OBJECT_LIMIT_JOB_MEMORY 
            | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_WORKINGSET;
        
        // Memory limitleri (MB -> byte dönüşümü)
        info.ProcessMemoryLimit = (limits.max_memory_mb as u64) * 1024 * 1024;
        info.JobMemoryLimit = (limits.max_memory_mb as u64) * 1024 * 1024;
        
        // Working set limiti
        info.BasicLimitInformation.MinimumWorkingSetSize = 0;
        info.BasicLimitInformation.MaximumWorkingSetSize = (limits.max_working_set_mb as usize) * 1024 * 1024;
        
        // Process sayısı limiti
        info.BasicLimitInformation.ActiveProcessLimit = limits.max_process_count;
        
        // CPU kullanım limiti (şu anda Windows API sınırlaması nedeniyle uygulanmıyor)
        // Bu özellik için PeriodicMonitoring gerekecek
        
        // Limitleri Job Object'e uygula
        let result = unsafe {
            SetInformationJobObject(
                job_handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        
        if !result.as_bool() {
            return Err(SandboxError::SetLimitError(
                "Job Object limitleri ayarlanamadı".to_string(),
            ));
        }
        
        Ok(())
    }
    
    /// Sandbox'lanmış plugin'i kaldır
    pub fn remove_sandbox(&self, plugin_id: &str) -> Result<(), SandboxError> {
        let mut plugins = self.sandboxed_plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.remove(plugin_id) {
            info!("Plugin {} sandbox'ı kaldırıldı", plugin_id);
            // SandboxedPlugin için Drop trait'i otomatik olarak job_handle'ı kapatacak
            Ok(())
        } else {
            Err(SandboxError::ProcessNotFoundError(
                format!("Plugin {} bulunamadı", plugin_id),
            ))
        }
    }
    
    /// İzin kontrolü yap
    pub fn check_permission(&self, plugin_id: &str, required_permission: PermissionLevel) -> Result<bool, SandboxError> {
        let plugins = self.sandboxed_plugins.lock().unwrap();
        
        if let Some(plugin) = plugins.get(plugin_id) {
            Ok(plugin.permissions.contains(&required_permission))
        } else {
            Err(SandboxError::ProcessNotFoundError(
                format!("Plugin {} bulunamadı", plugin_id),
            ))
        }
    }
    
    /// Tüm sandbox'lanmış plugin'leri listele
    pub fn list_sandboxed_plugins(&self) -> Vec<String> {
        let plugins = self.sandboxed_plugins.lock().unwrap();
        plugins.keys().cloned().collect()
    }
    
    /// Plugin'in sandbox durumunu kontrol et
    pub fn get_plugin_sandbox_status(&self, plugin_id: &str) -> Option<(u32, ResourceLimits, Vec<PermissionLevel>)> {
        let plugins = self.sandboxed_plugins.lock().unwrap();
        
        plugins.get(plugin_id).map(|plugin| {
            (
                plugin.process_id,
                plugin.resource_limits.clone(),
                plugin.permissions.clone(),
            )
        })
    }
}

// Tauri plugin entegrasyonu
#[tauri::command]
pub fn check_sandbox_status(plugin_id: String, sandbox_manager: tauri::State<'_, Arc<SandboxManager>>) -> Result<String, String> {
    let plugins = sandbox_manager.list_sandboxed_plugins();
    
    if plugins.contains(&plugin_id) {
        if let Some((process_id, limits, _)) = sandbox_manager.get_plugin_sandbox_status(&plugin_id) {
            Ok(format!(
                "Plugin {} sandbox'lanmış (PID: {}, Bellek Limiti: {} MB)",
                plugin_id,
                process_id,
                limits.max_memory_mb
            ))
        } else {
            Err(format!("Plugin {} durumu alınamadı", plugin_id))
        }
    } else {
        Err(format!("Plugin {} sandbox'lanmamış", plugin_id))
    }
}

// Tauri plugin oluşturma
pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let sandbox_manager = Arc::new(SandboxManager::new());
    
    tauri::plugin::Builder::new("sandbox")
        .invoke_handler(tauri::generate_handler![check_sandbox_status])
        .setup(move |app| {
            app.manage(sandbox_manager.clone());
            Ok(())
        })
        .build()
}
