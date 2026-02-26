//! 信任设备管理模块
//! 负责已配对设备的存储和管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 信任设备配置文件
const TRUSTED_DEVICES_FILE: &str = "trusted_devices.json";

/// 信任设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub paired_at: String,
    pub last_seen: String,
}

/// 信任设备管理器
pub struct TrustedDevicesManager {
    config_path: PathBuf,
    devices: HashMap<String, TrustedDevice>,
}

impl TrustedDevicesManager {
    /// 创建新的信任设备管理器
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self, String> {
        let config_path = match config_dir {
            Some(path) => path,
            None => Self::get_default_config_dir()?,
        };

        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let mut manager = TrustedDevicesManager {
            config_path,
            devices: HashMap::new(),
        };

        // 加载已保存的信任设备
        manager.load_devices()?;

        Ok(manager)
    }

    /// 获取默认配置目录
    fn get_default_config_dir() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?;

        Ok(config_dir.join("clipboard-caoguo").join(TRUSTED_DEVICES_FILE))
    }

    /// 加载信任设备列表
    fn load_devices(&mut self) -> Result<(), String> {
        if !self.config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read trusted devices config: {}", e))?;

        self.devices = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse trusted devices config: {}", e))?;

        println!("📚 Loaded {} trusted device(s)", self.devices.len());
        Ok(())
    }

    /// 保存信任设备列表
    fn save_devices(&self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.devices)
            .map_err(|e| format!("Failed to serialize trusted devices: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write trusted devices config: {}", e))?;

        println!("💾 Trusted devices saved to: {}", self.config_path.display());
        Ok(())
    }

    /// 添加信任设备
    pub fn add_device(&mut self, device_id: String, device_name: String) -> Result<(), String> {
        let now = chrono::Utc::now().to_rfc3339();

        let device = TrustedDevice {
            device_id: device_id.clone(),
            device_name,
            paired_at: now.clone(),
            last_seen: now,
        };

        self.devices.insert(device_id.clone(), device);
        self.save_devices()?;

        println!("✅ Device trusted: {}", device_id);
        Ok(())
    }

    /// 移除信任设备
    pub fn remove_device(&mut self, device_id: &str) -> Result<(), String> {
        if self.devices.remove(device_id).is_some() {
            self.save_devices()?;
            println!("🚫 Device removed from trusted list: {}", device_id);
            Ok(())
        } else {
            Err("Device not found in trusted list".to_string())
        }
    }

    /// 检查设备是否已信任
    pub fn is_trusted(&self, device_id: &str) -> bool {
        self.devices.contains_key(device_id)
    }

    /// 更新设备最后看见时间
    pub fn update_last_seen(&mut self, device_id: &str) -> Result<(), String> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.last_seen = chrono::Utc::now().to_rfc3339();
            self.save_devices()?;
            Ok(())
        } else {
            Err("Device not found in trusted list".to_string())
        }
    }

    /// 获取信任设备信息
    pub fn get_device(&self, device_id: &str) -> Option<&TrustedDevice> {
        self.devices.get(device_id)
    }

    /// 获取所有信任设备
    pub fn get_all_devices(&self) -> Vec<TrustedDevice> {
        self.devices.values().cloned().collect()
    }

    /// 清空所有信任设备
    pub fn clear_all(&mut self) -> Result<(), String> {
        self.devices.clear();
        self.save_devices()?;
        println!("🗑️ All trusted devices cleared");
        Ok(())
    }
}

impl Default for TrustedDevicesManager {
    fn default() -> Self {
        Self::new(None).expect("Failed to create TrustedDevicesManager")
    }
}
