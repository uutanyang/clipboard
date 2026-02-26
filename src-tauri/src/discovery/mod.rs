//! mDNS 服务发现模块
//! 负责设备的局域网发现和广播

use super::*;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::collections::HashMap;

const SERVICE_TYPE: &str = "_clip_sync._tcp.local.";
const DEFAULT_SERVICE_PORT: u16 = 54321;

/// mDNS 发现管理器
pub struct MdnsDiscovery {
    daemon: ServiceDaemon,
    devices: Arc<Mutex<HashMap<String, NetworkDevice>>>,
    server_state: Option<ServerState>,
}

impl MdnsDiscovery {
    /// 创建新的 mDNS 发现实例
    pub fn new() -> Result<Self, String> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

        Ok(MdnsDiscovery {
            daemon,
            devices: Arc::new(Mutex::new(HashMap::new())),
            server_state: None,
        })
    }

    /// 创建带有服务器状态的 mDNS 发现实例
    pub fn with_server_state(server_state: ServerState) -> Result<Self, String> {
        let daemon = ServiceDaemon::new()
            .map_err(|e| format!("Failed to create mDNS daemon: {}", e))?;

        Ok(MdnsDiscovery {
            daemon,
            devices: Arc::new(Mutex::new(HashMap::new())),
            server_state: Some(server_state),
        })
    }

    /// 注册 mDNS 服务，使本设备可被发现
    pub fn register_service(&self, app_handle: AppHandle) -> Result<(), String> {
        let hostname = hostname::get()
            .map_err(|e| format!("Failed to get hostname: {}", e))?
            .to_string_lossy()
            .to_string();

        let ip = local_ip_address::local_ip()
            .map_err(|e| format!("Failed to get local IP: {}", e))?
            .to_string();

        // 使用服务器实际监听的端口，如果未启动服务器则使用默认端口
        let port = self.server_state
            .as_ref()
            .map(|s| *s.server_port)
            .unwrap_or(DEFAULT_SERVICE_PORT);

        // 创建 TXT 记录（包含端口信息和设备元数据）
        let txt_props = vec![
            ("hostname".to_string(), hostname.clone()),
            ("ip".to_string(), ip.clone()),
            ("port".to_string(), port.to_string()),
            ("version".to_string(), "0.1.0".to_string()),
        ];

        // 注册服务（mdns-sd 0.11 需要使用 ServiceInfo）
        let service_info = ServiceInfo::new(
            SERVICE_TYPE,
            &hostname,
            &hostname,
            &format!("{}:{}", ip, port),
            port,
            &*txt_props,
        ).map_err(|e| format!("Failed to create service info: {}", e))?;

        let my_service = self.daemon.register(service_info)
            .map_err(|e| format!("Failed to register mDNS service: {}", e))?;

        println!("✓ mDNS service registered:");
        println!("   Device: {}@{}", hostname, ip);
        println!("   Port: {}", port);
        println!("   Service: {}", SERVICE_TYPE);

        // 保存服务句柄（不立即释放）
        let _ = my_service;

        // 发送自身信息到前端
        let device = NetworkDevice {
            name: hostname.clone(),
            hostname: hostname.clone(),
            ip,
            port,
            last_seen: Utc::now().to_rfc3339(),
        };

        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.emit("device-discovered", device);
        }

        Ok(())
    }

    /// 开始扫描局域网设备
    pub fn start_browsing(&self, app_handle: AppHandle) -> Result<(), String> {
        let receiver = self.daemon.browse(SERVICE_TYPE)
            .map_err(|e| format!("Failed to browse mDNS service: {}", e))?;

        let devices = self.devices.clone();

        tauri::async_runtime::spawn(async move {
            println!("🔍 Started browsing for devices...");

            while let Ok(event) = receiver.recv() {
                match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let hostname = info.get_hostname();
                        let addresses = info.get_addresses();
                        let port = info.get_port();

                        if let Some(ip) = addresses.iter().next() {
                            let device = NetworkDevice {
                                name: info.get_fullname().to_string(),
                                hostname: hostname.to_string(),
                                ip: ip.to_string(),
                                port,
                                last_seen: Utc::now().to_rfc3339(),
                            };

                            println!("📱 Device discovered:");
                            println!("   Hostname: {}", device.hostname);
                            println!("   IP: {}", device.ip);
                            println!("   Port: {}", device.port);

                            // 存储设备信息
                            {
                                let mut devices_map = devices.lock().unwrap();
                                devices_map.insert(device.hostname.clone(), device.clone());
                            }

                            // 发送事件到前端
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.emit("device-discovered", device);
                            }
                        }
                    }
                    ServiceEvent::ServiceRemoved(_service_type, fullname) => {
                        println!("🚫 Device removed: {}", fullname);

                        // 从存储中移除
                        let hostname = fullname.trim_end_matches(&format!(".{}", SERVICE_TYPE));

                        {
                            let mut devices_map = devices.lock().unwrap();
                            devices_map.remove(hostname);
                        }

                        // 发送移除事件到前端
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.emit("device-removed", hostname.to_string());
                        }
                    }
                    ServiceEvent::SearchStarted(_service_type) => {
                        println!("🔎 mDNS search started");
                    }
                    ServiceEvent::SearchStopped(_service_type) => {
                        println!("⏹️ mDNS search stopped");
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    /// 获取已发现的所有设备
    pub fn get_devices(&self) -> Vec<NetworkDevice> {
        self.devices.lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }
}
