//! HTTP 客户端模块
//! 用于向其他设备发送请求

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use super::ClipboardItem;
use base64::Engine as _;
use crate::server;

/// PING 响应
#[derive(Debug, Deserialize, Serialize)]
pub struct PingResponse {
    pub device_id: String,
    pub status: String,
    pub port: u16,
    pub timestamp: String,
}

/// 配对请求
#[derive(Debug, Serialize)]
pub struct PairRequest {
    pub device_id: String,
    pub device_name: String,
}

/// 配对响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PairResponse {
    pub status: String,
    pub message: String,
}

/// 同步数据
#[derive(Debug, Serialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub items: Vec<ClipboardItem>,
}

/// 同步响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub status: String,
    pub received_count: usize,
}

/// 文件传输响应
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTransferResponse {
    pub status: String,
    pub file_id: String,
    pub file_path: String,
    pub file_size: u64,
}

/// 进度回调类型
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

/// 文件上传器（支持进度回调）
pub struct FileUploader {
    device_id: String,
    file_path: String,
    progress_callback: Option<ProgressCallback>,
}

impl FileUploader {
    /// 创建新的文件上传器
    pub fn new(device_id: String, file_path: String) -> Self {
        Self {
            device_id,
            file_path,
            progress_callback: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    /// 执行文件上传
    pub async fn upload(&self, client: &Client, url: String) -> Result<FileTransferResponse, String> {
        // 读取文件
        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let metadata = file.metadata()
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        let file_size = metadata.len();

        // 获取文件名
        let filename = Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        println!("📤 Uploading file: {}", filename);
        println!("   Size: {} bytes", file_size);

        // 读取文件到内存（支持进度回调）
        let mut buffer = Vec::with_capacity(file_size as usize);
        std::io::copy(&mut file, &mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // 将文件数据编码为 base64
        let file_data_base64 = base64::engine::general_purpose::STANDARD_NO_PAD.encode(&buffer);

        // 创建 JSON 请求体
        let request_body = serde_json::json!({
            "device_id": self.device_id,
            "filename": filename,
            "file_data": file_data_base64
        });

        // 如果有进度回调，先调用一次完成回调
        if let Some(ref callback) = self.progress_callback {
            callback(file_size, file_size);
        }

        // 发送请求
        let response = client
            .post(&url)
            .json(&request_body)
            .timeout(Duration::from_secs(300)) // 5 分钟超时用于大文件
            .send()
            .await
            .map_err(|e| format!("Failed to send file: {}", e))?;

        println!("✓ Upload completed, status: {}", response.status());

        // 解析响应
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await
                .unwrap_or_else(|_| "Unable to read response body".to_string());
            return Err(format!("Upload failed: {} - {}", status, body));
        }

        response.json::<FileTransferResponse>().await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

/// HTTP 客户端
pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    /// 创建新的 HTTP 客户端
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .expect("Failed to create HTTP client");

        HttpClient { client }
    }

    /// 发送 PING 请求，测试连接
    ///
    /// # Arguments
    /// * `ip` - 目标设备 IP 地址
    /// * `port` - 目标设备端口
    ///
    /// # Returns
    /// PING 响应数据
    pub async fn send_ping(&self, ip: String, port: u16) -> Result<PingResponse, String> {
        let url = format!("http://{}:{}/ping", ip, port);

        println!("📡 Sending PING to {}", url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PingResponse>().await {
                        Ok(ping) => {
                            println!("✓ PING success from {}", ping.device_id);
                            Ok(ping)
                        }
                        Err(e) => Err(format!("Failed to parse PING response: {}", e)),
                    }
                } else {
                    Err(format!(
                        "PING failed with status: {}",
                        response.status()
                    ))
                }
            }
            Err(e) => Err(format!("Failed to send PING: {}", e)),
        }
    }

    /// 发送同步数据到目标设备
    ///
    /// # Arguments
    /// * `ip` - 目标设备 IP 地址
    /// * `port` - 目标设备端口
    /// * `device_id` - 本设备 ID
    /// * `items` - 要同步的剪贴板项列表
    ///
    /// # Returns
    /// 同步响应数据
    pub async fn send_sync_data(
        &self,
        ip: String,
        port: u16,
        device_id: String,
        items: Vec<ClipboardItem>,
    ) -> Result<SyncResponse, String> {
        let url = format!("http://{}:{}/sync", ip, port);

        let sync_request = SyncRequest { device_id, items };

        println!("📤 Sending {} items to {}", sync_request.items.len(), url);

        match self
            .client
            .post(&url)
            .json(&sync_request)
            .timeout(Duration::from_secs(30))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<SyncResponse>().await {
                        Ok(sync_resp) => {
                            println!(
                                "✓ Sync completed: {} items received",
                                sync_resp.received_count
                            );
                            Ok(sync_resp)
                        }
                        Err(e) => Err(format!("Failed to parse sync response: {}", e)),
                    }
                } else {
                    let status = response.status();
                    let body = match response.text().await {
                        Ok(b) => b,
                        Err(_) => "Unable to read response body".to_string(),
                    };
                    Err(format!("Sync failed: {} - {}", status, body))
                }
            }
            Err(e) => Err(format!("Failed to send sync data: {}", e)),
        }
    }

    /// 发送剪贴板消息（支持文本、图片、文件）到目标设备
    ///
    /// # Arguments
    /// * `ip` - 目标设备 IP 地址
    /// * `port` - 目标设备端口
    /// * `message` - 剪贴板消息（ClipSyncMessage）
    ///
    /// # Returns
    /// 同步响应数据
    pub async fn send_clipboard_data(
        &self,
        ip: String,
        port: u16,
        message: super::protocol::ClipSyncMessage,
    ) -> Result<server::SyncResponse, String> {
        let url = format!("http://{}:{}/sync", ip, port);

        println!("📤 Sending clipboard message to {}: type={:?}", url, message.msg_type);

        // 创建 sync request，包含单个消息
        let sync_request = server::SyncRequest {
            device_id: message.source_device.clone(),
            messages: vec![serde_json::to_value(&message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?],
        };

        match self
            .client
            .post(&url)
            .json(&sync_request)
            .timeout(Duration::from_secs(60)) // 图片可能需要更长时间
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<server::SyncResponse>().await {
                        Ok(sync_resp) => {
                            println!("✓ Clipboard message sync completed: {} received", sync_resp.received_count);
                            Ok(sync_resp)
                        }
                        Err(e) => Err(format!("Failed to parse sync response: {}", e)),
                    }
                } else {
                    let status = response.status();
                    let body = match response.text().await {
                        Ok(b) => b,
                        Err(_) => "Unable to read response body".to_string(),
                    };
                    Err(format!("Sync failed: {} - {}", status, body))
                }
            }
            Err(e) => Err(format!("Failed to send clipboard data: {}", e)),
        }
    }

    /// 发送配对请求到目标设备
    ///
    /// # Arguments
    /// * `ip` - 目标设备 IP 地址
    /// * `port` - 目标设备端口
    /// * `device_id` - 本设备 ID
    /// * `device_name` - 本设备名称
    ///
    /// # Returns
    /// 配对响应数据
    pub async fn request_pair(
        &self,
        ip: String,
        port: u16,
        device_id: String,
        device_name: String,
    ) -> Result<PairResponse, String> {
        let url = format!("http://{}:{}/pair-request", ip, port);

        let pair_request = PairRequest {
            device_id,
            device_name,
        };

        println!("🔗 Sending pair request to {}", url);

        match self.client.post(&url).json(&pair_request).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PairResponse>().await {
                        Ok(pair) => {
                            println!("✓ Pair request sent: {}", pair.status);
                            Ok(pair)
                        }
                        Err(e) => Err(format!("Failed to parse pair response: {}", e)),
                    }
                } else {
                    let status = response.status();
                    let body = match response.text().await {
                        Ok(b) => b,
                        Err(_) => "Unable to read response body".to_string(),
                    };
                    Err(format!("Pair request failed: {} - {}", status, body))
                }
            }
            Err(e) => Err(format!("Failed to send pair request: {}", e)),
        }
    }

    /// 发送文件到目标设备
    ///
    /// # Arguments
    /// * `ip` - 目标设备 IP 地址
    /// * `port` - 目标设备端口
    /// * `device_id` - 本设备 ID
    /// * `file_path` - 文件路径
    /// * `progress_callback` - 可选的进度回调函数（已发送字节，总字节）
    ///
    /// # Returns
    /// 文件传输响应数据
    pub async fn send_file(
        &self,
        ip: String,
        port: u16,
        device_id: String,
        file_path: String,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<FileTransferResponse, String> {
        let url = format!("http://{}:{}/file-transfer", ip, port);

        println!("📤 Sending file to {}", url);

        let uploader = FileUploader::new(device_id, file_path);

        // 如果有进度回调，设置回调
        let uploader = match progress_callback {
            Some(callback) => uploader.with_progress(callback),
            None => uploader,
        };

        uploader.upload(&self.client, url).await
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 单例 HTTP 客户端
#[allow(static_mut_refs)]
static mut GLOBAL_CLIENT: Option<HttpClient> = None;

/// 获取全局 HTTP 客户端实例
///
/// # Safety
/// 此函数仅在单线程环境（Tauri 主线程）中调用，是安全的
#[allow(static_mut_refs)]
pub fn get_global_client() -> &'static HttpClient {
    unsafe {
        if GLOBAL_CLIENT.is_none() {
            GLOBAL_CLIENT = Some(HttpClient::new());
        }
        GLOBAL_CLIENT.as_ref().unwrap()
    }
}
