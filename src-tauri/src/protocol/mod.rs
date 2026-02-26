//! 剪贴板同步数据协议模块
//! 定义网络传输的消息格式和数据处理函数

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::SystemTime;

/// 剪贴板内容类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClipType {
    /// 纯文本
    Text,
    /// 图片 (base64 编码)
    Image,
    /// 文件 (暂未实现)
    File,
}

impl ClipType {
    /// 从字符串解析 ClipType
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "image" => ClipType::Image,
            "file" => ClipType::File,
            _ => ClipType::Text,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipType::Text => "text",
            ClipType::Image => "image",
            ClipType::File => "file",
        }
    }
}

/// 剪贴板同步消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipSyncMessage {
    /// 消息类型
    #[serde(rename = "type")]
    pub msg_type: ClipType,
    /// 内容（文本内容或图片的 base64）
    pub content: String,
    /// 内容的 SHA256 哈希
    pub hash: String,
    /// 时间戳 (Unix timestamp in milliseconds)
    pub timestamp: i64,
    /// 来源设备 ID
    pub source_device: String,
    /// 文件名（如果是文件类型）
    pub file_name: Option<String>,
    /// 文件大小（字节）
    pub file_size: Option<u64>,
}

impl ClipSyncMessage {
    /// 创建新的剪贴板消息
    pub fn new(
        msg_type: ClipType,
        content: String,
        source_device: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let hash = Self::calculate_hash(&content);

        ClipSyncMessage {
            msg_type,
            content,
            hash,
            timestamp,
            source_device,
            file_name: None,
            file_size: None,
        }
    }

    /// 创建文件类型的消息
    pub fn new_file(
        content: String,
        file_name: String,
        file_size: u64,
        source_device: String,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let hash = Self::calculate_hash(&content);

        ClipSyncMessage {
            msg_type: ClipType::File,
            content,
            hash,
            timestamp,
            source_device,
            file_name: Some(file_name),
            file_size: Some(file_size),
        }
    }

    /// 计算内容的 SHA256 哈希
    pub fn calculate_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// 验证内容哈希是否匹配
    pub fn verify_hash(&self) -> bool {
        let calculated_hash = Self::calculate_hash(&self.content);
        calculated_hash == self.hash
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// 序列化为 JSON 字符串（美化格式）
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 获取内容大小（字节）
    pub fn content_size(&self) -> usize {
        self.content.len()
    }

    /// 检查消息是否过期（默认 5 分钟）
    pub fn is_expired(&self, max_age_ms: Option<i64>) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        let age = now - self.timestamp;
        age > max_age_ms.unwrap_or(5 * 60 * 1000)
    }

    /// 克隆并更新时间戳
    pub fn clone_with_new_timestamp(&self) -> Self {
        let mut msg = self.clone();
        msg.timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        msg
    }
}

/// 同步请求（用于 HTTP 请求体）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub messages: Vec<ClipSyncMessage>,
}

/// 同步响应（用于 HTTP 响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub status: String,
    pub received_count: usize,
    pub messages: Vec<ClipSyncMessage>,
}

/// 心跳消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub device_id: String,
    pub device_name: String,
    pub timestamp: i64,
    pub status: String,
}

impl HeartbeatMessage {
    pub fn new(device_id: String, device_name: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        HeartbeatMessage {
            device_id,
            device_name,
            timestamp,
            status: "online".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hash() {
        let hash1 = ClipSyncMessage::calculate_hash("hello world");
        let hash2 = ClipSyncMessage::calculate_hash("hello world");
        let hash3 = ClipSyncMessage::calculate_hash("hello rust");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA256 输出 64 字符
    }

    #[test]
    fn test_clip_type_from_str() {
        assert_eq!(ClipType::from_str("text"), ClipType::Text);
        assert_eq!(ClipType::from_str("TEXT"), ClipType::Text);
        assert_eq!(ClipType::from_str("image"), ClipType::Image);
        assert_eq!(ClipType::from_str("file"), ClipType::File);
        assert_eq!(ClipType::from_str("unknown"), ClipType::Text);
    }

    #[test]
    fn test_clip_sync_message_serialization() {
        let msg = ClipSyncMessage::new(
            ClipType::Text,
            "test content".to_string(),
            "device-123".to_string(),
        );

        let json = msg.to_json().unwrap();
        let msg2 = ClipSyncMessage::from_json(&json).unwrap();

        assert_eq!(msg.msg_type, msg2.msg_type);
        assert_eq!(msg.content, msg2.content);
        assert_eq!(msg.hash, msg2.hash);
        assert_eq!(msg.source_device, msg2.source_device);
    }

    #[test]
    fn test_verify_hash() {
        let mut msg = ClipSyncMessage::new(
            ClipType::Text,
            "original content".to_string(),
            "device-123".to_string(),
        );

        assert!(msg.verify_hash());

        // 修改内容但不修改哈希
        msg.content = "modified content".to_string();
        assert!(!msg.verify_hash());
    }

    #[test]
    fn test_file_message() {
        let msg = ClipSyncMessage::new_file(
            "file content".to_string(),
            "test.txt".to_string(),
            12,
            "device-123".to_string(),
        );

        assert_eq!(msg.msg_type, ClipType::File);
        assert_eq!(msg.file_name, Some("test.txt".to_string()));
        assert_eq!(msg.file_size, Some(12));
    }

    #[test]
    fn test_heartbeat_message() {
        let heartbeat = HeartbeatMessage::new(
            "device-123".to_string(),
            "My Device".to_string(),
        );

        assert_eq!(heartbeat.device_id, "device-123");
        assert_eq!(heartbeat.device_name, "My Device");
        assert_eq!(heartbeat.status, "online");
    }
}
