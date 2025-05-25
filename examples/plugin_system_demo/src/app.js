// Tauri Windows Plugin System Demo - Frontend Script
const { invoke } = window.__TAURI__.tauri;
const { listen } = window.__TAURI__.event;

// DOM elementleri
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('search-btn');
const searchResults = document.getElementById('search-results');
const installedPlugins = document.getElementById('installed-plugins');
const pluginDetails = document.getElementById('plugin-details');
const resourceStats = document.getElementById('resource-stats');
const logContainer = document.getElementById('log-container');
const checkUpdatesBtn = document.getElementById('check-updates-btn');
const refreshBtn = document.getElementById('refresh-btn');
const notificationContainer = document.getElementById('notification-container');

// Test elementleri
const startTestBtn = document.getElementById('start-test-btn');
const testStatus = document.getElementById('test-status');
const testPluginsContainer = document.getElementById('test-plugins-container');
const progressBarFill = document.getElementById('progress-bar-fill');
const testStepInfo = document.getElementById('test-step-info');

// Seçili plugin
let selectedPluginId = null;
// Çalışan pluginler için kaynak izleme
let resourceMonitors = {};
// Test durumu
let testData = null;
// Test adımları için progress değerleri
const testStepProgress = {
  'Initialize': 0,
  'ConnectToStore': 10,
  'Login': 20,
  'SearchPlugins': 30,
  'DownloadPlugins': 50,
  'InstallPlugins': 70,
  'RunPlugins': 90,
  'Complete': 100
};

// Sayfa yüklendiğinde
document.addEventListener('DOMContentLoaded', async () => {
  // Log ekle
  addLog('Uygulama başlatıldı', 'info');
  addLog('Plugin System modülleri yükleniyor...', 'info');
  
  // Plugin listesini yükle
  await refreshPluginList();
  
  // Event listener'ları ekle
  setupEventListeners();
  
  // Tauri event dinleyicileri
  await setupTauriEventListeners();
  
  addLog('Plugin System Demo hazır', 'success');
  showNotification('Plugin System Demo hazır', 'success');
});

// Event listener'ları kur
function setupEventListeners() {
  // Arama butonu
  searchBtn.addEventListener('click', async () => {
    const query = searchInput.value.trim();
    if (query) {
      await searchPlugins(query);
    }
  });
  
  // Enter tuşu ile arama
  searchInput.addEventListener('keypress', async (e) => {
    if (e.key === 'Enter') {
      const query = searchInput.value.trim();
      if (query) {
        await searchPlugins(query);
      }
    }
  });
  
  // Güncelleme kontrolü
  checkUpdatesBtn.addEventListener('click', async () => {
    await checkForUpdates();
  });
  
  // Liste yenileme
  refreshBtn.addEventListener('click', async () => {
    await refreshPluginList();
  });
  
  // Test senaryosunu başlat
  startTestBtn.addEventListener('click', async () => {
    await startPluginTest();
  });
}

// Tauri event dinleyicileri
async function setupTauriEventListeners() {
  // Test durum değişikliği eventi
  await listen('test_status_changed', (event) => {
    updateTestStatus(event.payload);
  });
  
  // Plugin kurulum olayları
  await listen('plugin_install_started', (event) => {
    addLog(`Plugin kurulumu başlatıldı: ${event.payload}`, 'info');
  });
  
  await listen('plugin_installed', (event) => {
    addLog(`Plugin kuruldu: ${event.payload}`, 'success');
    showNotification(`Plugin başarıyla kuruldu: ${event.payload}`, 'success');
    refreshPluginList();
  });
  
  await listen('plugin_install_failed', (event) => {
    addLog(`Plugin kurulumu başarısız: ${event.payload}`, 'error');
    showNotification(`Plugin kurulumu başarısız: ${event.payload}`, 'error');
  });
  
  // Plugin kaldırma olayları
  await listen('plugin_uninstall_started', (event) => {
    addLog(`Plugin kaldırılıyor: ${event.payload}`, 'info');
  });
  
  await listen('plugin_uninstalled', (event) => {
    addLog(`Plugin kaldırıldı: ${event.payload}`, 'success');
    showNotification(`Plugin başarıyla kaldırıldı: ${event.payload}`, 'success');
    refreshPluginList();
  });
  
  // Plugin çalıştırma olayları
  await listen('plugin_start_requested', (event) => {
    addLog(`Plugin başlatma isteği: ${event.payload}`, 'info');
  });
  
  await listen('plugin_permissions_requested', (event) => {
    addLog(`Plugin izinleri isteniyor: ${event.payload}`, 'info');
  });
  
  await listen('plugin_permissions_granted', (event) => {
    addLog(`Plugin izinleri verildi: ${event.payload}`, 'success');
  });
  
  await listen('plugin_permissions_denied', (event) => {
    addLog(`Plugin izinleri reddedildi: ${event.payload}`, 'warning');
    showNotification(`Plugin izinleri reddedildi: ${event.payload}`, 'warning');
  });
  
  await listen('plugin_sandbox_creating', (event) => {
    addLog(`Plugin sandbox oluşturuluyor: ${event.payload}`, 'info');
  });
  
  await listen('plugin_sandbox_created', (event) => {
    addLog(`Plugin sandbox oluşturuldu: ${event.payload}`, 'success');
  });
  
  await listen('plugin_process_started', (event) => {
    addLog(`Plugin süreci başlatıldı: ${event.payload}`, 'success');
  });
  
  await listen('plugin_monitoring_started', (event) => {
    addLog(`Plugin kaynak izleme başlatıldı: ${event.payload}`, 'info');
    startResourceMonitoring(event.payload);
  });
  
  await listen('plugin_running', (event) => {
    addLog(`Plugin çalışıyor: ${event.payload}`, 'success');
    showNotification(`Plugin başarıyla çalıştırıldı: ${event.payload}`, 'success');
    refreshPluginList();
  });
  
  // Plugin durdurma olayları
  await listen('plugin_stop_requested', (event) => {
    addLog(`Plugin durdurma isteği: ${event.payload}`, 'info');
  });
  
  await listen('plugin_monitoring_stopped', (event) => {
    addLog(`Plugin kaynak izleme durduruldu: ${event.payload}`, 'info');
    stopResourceMonitoring(event.payload);
  });
  
  await listen('plugin_process_terminated', (event) => {
    addLog(`Plugin süreci sonlandırıldı: ${event.payload}`, 'info');
  });
  
  await listen('plugin_sandbox_destroyed', (event) => {
    addLog(`Plugin sandbox kaldırıldı: ${event.payload}`, 'info');
  });
  
  await listen('plugin_stopped', (event) => {
    addLog(`Plugin durduruldu: ${event.payload}`, 'success');
    showNotification(`Plugin başarıyla durduruldu: ${event.payload}`, 'success');
    refreshPluginList();
  });
  
  // Güncelleme olayları
  await listen('update_check_started', () => {
    addLog('Güncelleme kontrolü başlatıldı', 'info');
  });
  
  await listen('updates_available', (event) => {
    addLog(`${event.payload} plugin için güncelleme mevcut`, 'info');
    showNotification(`${event.payload} plugin için güncelleme mevcut`, 'info');
  });
  
  await listen('no_updates_available', () => {
    addLog('Tüm pluginler güncel', 'success');
    showNotification('Tüm pluginler güncel', 'success');
  });
}

// Plugin listesini yenile
async function refreshPluginList() {
  addLog('Plugin listesi yenileniyor...', 'info');
  
  try {
    const plugins = await invoke('list_plugins');
    renderInstalledPlugins(plugins);
    addLog('Plugin listesi yenilendi', 'success');
  } catch (error) {
    addLog(`Plugin listesi alınamadı: ${error}`, 'error');
    showNotification('Plugin listesi alınamadı', 'error');
  }
}

// Pluginleri ara
async function searchPlugins(query) {
  addLog(`Plugin aranıyor: ${query}`, 'info');
  
  try {
    const plugins = await invoke('search_plugins', { query });
    renderSearchResults(plugins);
    addLog(`${plugins.length} plugin bulundu`, 'success');
  } catch (error) {
    addLog(`Plugin arama hatası: ${error}`, 'error');
    showNotification('Plugin arama hatası', 'error');
  }
}

// Plugin kur
async function installPlugin(pluginId) {
  addLog(`Plugin kurulumu başlatılıyor: ${pluginId}`, 'info');
  
  try {
    const success = await invoke('install_plugin', { pluginId });
    if (success) {
      addLog(`Plugin kuruldu: ${pluginId}`, 'success');
      await refreshPluginList();
    }
  } catch (error) {
    addLog(`Plugin kurulum hatası: ${error}`, 'error');
    showNotification(`Plugin kurulum hatası: ${pluginId}`, 'error');
  }
}

// Plugin kaldır
async function uninstallPlugin(pluginId) {
  if (confirm(`${pluginId} plugin'ini kaldırmak istediğinize emin misiniz?`)) {
    addLog(`Plugin kaldırılıyor: ${pluginId}`, 'info');
    
    try {
      const success = await invoke('uninstall_plugin', { pluginId });
      if (success) {
        addLog(`Plugin kaldırıldı: ${pluginId}`, 'success');
        await refreshPluginList();
      }
    } catch (error) {
      addLog(`Plugin kaldırma hatası: ${error}`, 'error');
      showNotification(`Plugin kaldırma hatası: ${pluginId}`, 'error');
    }
  }
}

// Plugin çalıştır
async function runPlugin(pluginId) {
  addLog(`Plugin çalıştırılıyor: ${pluginId}`, 'info');
  
  try {
    const success = await invoke('run_plugin', { pluginId });
    if (success) {
      addLog(`Plugin çalıştırıldı: ${pluginId}`, 'success');
      await refreshPluginList();
    }
  } catch (error) {
    addLog(`Plugin çalıştırma hatası: ${error}`, 'error');
    showNotification(`Plugin çalıştırma hatası: ${pluginId}`, 'error');
  }
}

// Plugin durdur
async function stopPlugin(pluginId) {
  addLog(`Plugin durduruluyor: ${pluginId}`, 'info');
  
  try {
    const success = await invoke('stop_plugin', { pluginId });
    if (success) {
      addLog(`Plugin durduruldu: ${pluginId}`, 'success');
      await refreshPluginList();
    }
  } catch (error) {
    addLog(`Plugin durdurma hatası: ${error}`, 'error');
    showNotification(`Plugin durdurma hatası: ${pluginId}`, 'error');
  }
}

// Güncelleme kontrolü
async function checkForUpdates() {
  addLog('Güncellemeler kontrol ediliyor...', 'info');
  
  try {
    const updates = await invoke('check_for_updates');
    const updateCount = Object.keys(updates).length;
    
    if (updateCount > 0) {
      addLog(`${updateCount} plugin için güncelleme mevcut`, 'info');
      showNotification(`${updateCount} plugin için güncelleme mevcut`, 'info');
      
      // Güncelleme listesini göster
      let updateList = '<ul>';
      for (const [pluginId, update] of Object.entries(updates)) {
        updateList += `<li>${pluginId}: ${update.current_version} -> ${update.version}</li>`;
      }
      updateList += '</ul>';
      
      pluginDetails.innerHTML = `
        <h3>Mevcut Güncellemeler</h3>
        ${updateList}
        <button id="update-all-btn" class="btn-update">Tümünü Güncelle</button>
      `;
      
      document.getElementById('update-all-btn').addEventListener('click', async () => {
        for (const pluginId of Object.keys(updates)) {
          await updatePlugin(pluginId);
        }
      });
    } else {
      addLog('Tüm pluginler güncel', 'success');
      showNotification('Tüm pluginler güncel', 'success');
    }
  } catch (error) {
    addLog(`Güncelleme kontrolü hatası: ${error}`, 'error');
    showNotification('Güncelleme kontrolü hatası', 'error');
  }
}

// Plugin güncelle
async function updatePlugin(pluginId) {
  addLog(`Plugin güncelleniyor: ${pluginId}`, 'info');
  
  try {
    // Burada update_plugin çağrısını ekleyebilirsiniz
    // const status = await invoke('update_plugin', { pluginId });
    // Şimdilik dummy bir güncelleme
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    addLog(`Plugin güncellendi: ${pluginId}`, 'success');
    showNotification(`Plugin başarıyla güncellendi: ${pluginId}`, 'success');
    await refreshPluginList();
  } catch (error) {
    addLog(`Plugin güncelleme hatası: ${error}`, 'error');
    showNotification(`Plugin güncelleme hatası: ${pluginId}`, 'error');
  }
}

// Kaynak kullanımını izlemeye başla
function startResourceMonitoring(pluginId) {
  if (resourceMonitors[pluginId]) {
    clearInterval(resourceMonitors[pluginId]);
  }
  
  // Her 1 saniyede bir kaynak kullanımını güncelle
  resourceMonitors[pluginId] = setInterval(async () => {
    try {
      const usage = await invoke('get_resource_usage', { pluginId });
      updateResourceStats(usage);
    } catch (error) {
      console.error('Kaynak kullanımı alınamadı:', error);
    }
  }, 1000);
}

// Kaynak kullanımı izlemeyi durdur
function stopResourceMonitoring(pluginId) {
  if (resourceMonitors[pluginId]) {
    clearInterval(resourceMonitors[pluginId]);
    delete resourceMonitors[pluginId];
  }
  
  // Kaynak kullanımı panelini temizle
  resourceStats.innerHTML = `<p class="placeholder">Aktif plugin yok</p>`;
}

// Kaynak kullanımı istatistiklerini güncelle
function updateResourceStats(usage) {
  resourceStats.innerHTML = `
    <div class="stat-item cpu">
      <div class="stat-label">CPU</div>
      <div class="stat-value">${usage.cpu_usage.toFixed(1)}%</div>
    </div>
    <div class="stat-item memory">
      <div class="stat-label">Bellek</div>
      <div class="stat-value">${usage.memory_usage_mb.toFixed(1)} MB</div>
    </div>
    <div class="stat-item network">
      <div class="stat-label">Ağ</div>
      <div class="stat-value">${usage.network_usage_kbps.toFixed(1)} KB/s</div>
    </div>
  `;
}

// Arama sonuçlarını render et
function renderSearchResults(plugins) {
  searchResults.innerHTML = '';
  
  if (plugins.length === 0) {
    searchResults.innerHTML = '<p class="placeholder">Sonuç bulunamadı</p>';
    return;
  }
  
  plugins.forEach(plugin => {
    const item = document.createElement('div');
    item.className = 'plugin-item';
    item.innerHTML = `
      <div class="plugin-item-header">
        <span class="plugin-name">${plugin.name}</span>
        <span class="plugin-version">v${plugin.version}</span>
      </div>
      <div class="plugin-description">${plugin.description.substring(0, 100)}${plugin.description.length > 100 ? '...' : ''}</div>
      <div class="plugin-vendor">${plugin.vendor.name}</div>
      <div class="plugin-actions">
        <button class="btn-install">Kur</button>
      </div>
    `;
    
    // Kurulum butonu
    const installButton = item.querySelector('.btn-install');
    installButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      await installPlugin(plugin.id);
    });
    
    // Plugin seç
    item.addEventListener('click', () => {
      showPluginDetails(plugin);
    });
    
    searchResults.appendChild(item);
  });
}

// Kurulu pluginleri render et
function renderInstalledPlugins(plugins) {
  installedPlugins.innerHTML = '';
  
  if (plugins.length === 0) {
    installedPlugins.innerHTML = '<p class="placeholder">Kurulu plugin yok</p>';
    return;
  }
  
  plugins.forEach(plugin => {
    const item = document.createElement('div');
    item.className = `plugin-item ${plugin.running ? 'running' : ''}`;
    if (plugin.id === selectedPluginId) {
      item.classList.add('selected');
    }
    
    item.innerHTML = `
      <div class="plugin-item-header">
        <span class="plugin-name">${plugin.name}</span>
        <span class="plugin-version">v${plugin.version}</span>
      </div>
      <div class="plugin-status">
        <span class="status-dot ${plugin.running ? 'active' : ''}"></span>
        ${plugin.running ? 'Çalışıyor' : 'Durduruldu'}
      </div>
      <div class="plugin-actions">
        ${plugin.running 
          ? '<button class="btn-stop">Durdur</button>' 
          : '<button class="btn-run">Çalıştır</button>'}
        <button class="btn-uninstall">Kaldır</button>
      </div>
    `;
    
    // Çalıştırma butonu
    const runButton = item.querySelector('.btn-run');
    if (runButton) {
      runButton.addEventListener('click', async (e) => {
        e.stopPropagation();
        await runPlugin(plugin.id);
      });
    }
    
    // Durdurma butonu
    const stopButton = item.querySelector('.btn-stop');
    if (stopButton) {
      stopButton.addEventListener('click', async (e) => {
        e.stopPropagation();
        await stopPlugin(plugin.id);
      });
    }
    
    // Kaldırma butonu
    const uninstallButton = item.querySelector('.btn-uninstall');
    uninstallButton.addEventListener('click', async (e) => {
      e.stopPropagation();
      await uninstallPlugin(plugin.id);
    });
    
    // Plugin seç
    item.addEventListener('click', () => {
      selectedPluginId = plugin.id;
      const allItems = installedPlugins.querySelectorAll('.plugin-item');
      allItems.forEach(el => el.classList.remove('selected'));
      item.classList.add('selected');
      
      showPluginDetails(plugin);
    });
    
    installedPlugins.appendChild(item);
  });
}

// Plugin detaylarını göster
function showPluginDetails(plugin) {
  // Burada gerçek bir uygulamada plugin.id ile plugin detaylarını almak için API çağrısı yapılabilir
  
  pluginDetails.innerHTML = `
    <h3>${plugin.name} v${plugin.version}</h3>
    <div class="details-section">
      <h4>Genel Bilgiler</h4>
      <p><strong>ID:</strong> ${plugin.id}</p>
      <p><strong>Durum:</strong> ${plugin.installed ? (plugin.running ? 'Çalışıyor' : 'Kurulu, Durduruldu') : 'Kurulu Değil'}</p>
      <p><strong>Sağlayıcı:</strong> ${plugin.vendor?.name || 'Bilinmiyor'}</p>
    </div>
    
    <div class="details-section">
      <h4>İzinler</h4>
      ${plugin.permissions && plugin.permissions.length > 0 
        ? `<ul>${plugin.permissions.map(p => `<li>${p}</li>`).join('')}</ul>` 
        : '<p>İzin bilgisi yok veya izin gerektirmiyor</p>'}
    </div>
    
    <div class="details-section">
      <h4>İşlemler</h4>
      <div class="plugin-actions">
        ${!plugin.installed 
          ? '<button class="btn-install">Kur</button>' 
          : plugin.running 
            ? '<button class="btn-stop">Durdur</button>' 
            : '<button class="btn-run">Çalıştır</button>'}
        ${plugin.installed ? '<button class="btn-uninstall">Kaldır</button>' : ''}
        <button class="btn-update">Güncelle</button>
      </div>
    </div>
  `;
  
  // Buton event listener'ları
  const installBtn = pluginDetails.querySelector('.btn-install');
  if (installBtn) {
    installBtn.addEventListener('click', () => installPlugin(plugin.id));
  }
  
  const runBtn = pluginDetails.querySelector('.btn-run');
  if (runBtn) {
    runBtn.addEventListener('click', () => runPlugin(plugin.id));
  }
  
  const stopBtn = pluginDetails.querySelector('.btn-stop');
  if (stopBtn) {
    stopBtn.addEventListener('click', () => stopPlugin(plugin.id));
  }
  
  const uninstallBtn = pluginDetails.querySelector('.btn-uninstall');
  if (uninstallBtn) {
    uninstallBtn.addEventListener('click', () => uninstallPlugin(plugin.id));
  }
  
  const updateBtn = pluginDetails.querySelector('.btn-update');
  if (updateBtn) {
    updateBtn.addEventListener('click', () => updatePlugin(plugin.id));
  }
}

// Log ekle
function addLog(message, type = 'info') {
  const logEntry = document.createElement('div');
  logEntry.className = `log-entry ${type}`;
  
  const timestamp = new Date().toLocaleTimeString();
  logEntry.innerHTML = `[${timestamp}] ${message}`;
  
  logContainer.appendChild(logEntry);
  logContainer.scrollTop = logContainer.scrollHeight;
}

// Bildirim göster
function showNotification(message, type = 'info') {
  const notification = document.createElement('div');
  notification.classList.add('notification', `notification-${type}`);
  
  const icon = document.createElement('span');
  icon.classList.add('notification-icon');
  
  switch (type) {
    case 'success':
      icon.textContent = '✓';
      break;
    case 'error':
      icon.textContent = '✕';
      break;
    case 'warning':
      icon.textContent = '⚠';
      break;
    default:
      icon.textContent = 'ℹ';
  }
  
  const text = document.createElement('span');
  text.textContent = message;
  
  notification.appendChild(icon);
  notification.appendChild(text);
  
  notificationContainer.appendChild(notification);
  
  // 5 saniye sonra bildirim kaybolsun
  setTimeout(() => {
    notification.classList.add('notification-hide');
    setTimeout(() => {
      notificationContainer.removeChild(notification);
    }, 300);
  }, 5000);
}
