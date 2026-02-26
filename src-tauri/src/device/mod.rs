//! 设备身份管理模块
//! 负责设备 ID 生成、存储和设备信息管理

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// 设备信息配置文件
const DEVICE_CONFIG_FILE: &str = "device_config.json";

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub created_at: String,
    pub last_updated: String,
}

/// 设备身份管理器
pub struct DeviceManager {
    config_path: PathBuf,
}

impl DeviceManager {
    /// 创建新的设备管理器
    ///
    /// # Arguments
    /// * `config_dir` - 配置文件所在目录
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

        Ok(DeviceManager { config_path })
    }

    /// 获取默认配置目录
    fn get_default_config_dir() -> Result<PathBuf, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Failed to get config directory")?;

        Ok(config_dir.join("clipboard-caoguo").join(DEVICE_CONFIG_FILE))
    }

    /// 生成唯一的设备 ID（UUID v4）
    pub fn generate_device_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// 获取或创建设备 ID
    ///
    /// 首次运行时生成 ID 并存储到本地配置文件
    /// 后续运行直接从配置文件读取
    pub fn get_or_create_device_id(&self) -> Result<String, String> {
        // 尝试从配置文件读取
        if let Some(info) = self.load_device_info()? {
            println!("📱 Loaded device ID from config: {}", info.device_id);
            return Ok(info.device_id);
        }

        // 首次运行，生成新的设备 ID
        let device_id = Self::generate_device_id();
        println!("🆔 Generated new device ID: {}", device_id);

        // 获取设备名称
        let device_name = self.get_device_name();

        // 创建设备信息
        let now = chrono::Utc::now().to_rfc3339();
        let device_info = DeviceInfo {
            device_id: device_id.clone(),
            device_name,
            created_at: now.clone(),
            last_updated: now,
        };

        // 保存到配置文件
        self.save_device_info(&device_info)?;

        Ok(device_id)
    }

    /// 获取设备名称
    ///
    /// 优先使用系统主机名，如果失败则返回默认名称
    pub fn get_device_name(&self) -> String {
        hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "Unknown Device".to_string())
    }

    /// 加载设备信息
    fn load_device_info(&self) -> Result<Option<DeviceInfo>, String> {
        if !self.config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.config_path)
            .map_err(|e| format!("Failed to read device config: {}", e))?;

        let info: DeviceInfo = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse device config: {}", e))?;

        Ok(Some(info))
    }

    /// 保存设备信息
    fn save_device_info(&self, info: &DeviceInfo) -> Result<(), String> {
        let content = serde_json::to_string_pretty(info)
            .map_err(|e| format!("Failed to serialize device info: {}", e))?;

        fs::write(&self.config_path, content)
            .map_err(|e| format!("Failed to write device config: {}", e))?;

        println!("💾 Device config saved to: {}", self.config_path.display());

        Ok(())
    }

    /// 更新设备信息
    pub fn update_device_info(&self, device_name: Option<String>) -> Result<DeviceInfo, String> {
        let mut info = self.load_device_info()?
            .ok_or("Device info not found. Please initialize first.")?;

        if let Some(name) = device_name {
            info.device_name = name;
        }
        info.last_updated = chrono::Utc::now().to_rfc3339();

        self.save_device_info(&info)?;
        Ok(info)
    }

    /// 获取完整的设备信息
    pub fn get_device_info(&self) -> Result<DeviceInfo, String> {
        self.load_device_info()?
            .ok_or_else(|| "Device info not found. Please initialize first.".to_string())
    }

    /// 重置设备 ID（谨慎使用）
    ///
    /// 删除配置文件，下次启动时会生成新的设备 ID
    pub fn reset_device_id(&self) -> Result<(), String> {
        if self.config_path.exists() {
            fs::remove_file(&self.config_path)
                .map_err(|e| format!("Failed to remove device config: {}", e))?;
            println!("🗑️ Device config removed");
        }
        Ok(())
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new(None).expect("Failed to create DeviceManager")
    }
}

/// 快捷函数：获取或创建设备 ID（使用默认配置目录）
pub fn get_or_create_device_id() -> Result<String, String> {
    DeviceManager::default().get_or_create_device_id()
}

/// 快捷函数：获取设备名称
pub fn get_device_name() -> String {
    DeviceManager::default().get_device_name()
}
