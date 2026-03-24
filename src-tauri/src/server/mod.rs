//! 内置 HTTP 服务端模块
//! 提供设备发现、配对、同步和静态文件服务功能

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response, Json},
    routing::{get, post},
    body::Body,
    Router,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use tower_http::cors::CorsLayer;
use tokio::sync::broadcast;

/// 服务器配置
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub base_port: u16,
    pub max_attempts: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            base_port: 9527,
            max_attempts: 100,
        }
    }
}

impl ServerConfig {
    /// 从配置文件加载配置
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                let base_port = config.get("base_port")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as u16)
                    .unwrap_or(9527);
                
                // 验证端口范围
                let base_port = if base_port >= 1024 {
                    base_port
                } else {
                    9527
                };
                
                return Self {
                    base_port,
                    max_attempts: 100,
                };
            }
        }
        
        Self::default()
    }
    
    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::get_config_path();
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        
        let config = serde_json::json!({
            "base_port": self.base_port,
        });
        
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        std::fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config: {}", e))?;
        
        Ok(())
    }
    
    /// 获取配置文件路径
    fn get_config_path() -> std::path::PathBuf {
        let app_data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("com.yangtanfang.tie-lifang");
        
        app_data_dir.join("server_config.json")
    }
    
    /// 设置基础端口
    pub fn set_base_port(&mut self, port: u16) {
        // 验证端口范围
        if port >= 1024 {
            self.base_port = port;
        }
    }
}

/// 服务器运行状态
#[derive(Clone, Debug, Serialize)]
pub struct ServerStatus {
    pub is_running: bool,
    pub port: Option<u16>,
    pub started_at: Option<String>,
}

/// 全局服务器控制句柄
pub struct ServerHandle {
    pub shutdown_tx: broadcast::Sender<()>,
    pub is_running: Arc<AtomicBool>,
    pub port: Arc<std::sync::Mutex<Option<u16>>>,
}

impl ServerHandle {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            shutdown_tx,
            is_running: Arc::new(AtomicBool::new(false)),
            port: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn get_port(&self) -> Option<u16> {
        self.port.lock().ok()?.as_ref().copied()
    }
}

/// 全局服务器状态
#[derive(Clone)]
pub struct ServerState {
    pub device_id: Arc<str>,
    pub server_port: Arc<u16>,
    pub app_handle: AppHandle,
    pub clipboard_sync: Option<Arc<super::ClipboardSyncManager>>,
    pub trusted_devices: Option<Arc<std::sync::Mutex<super::TrustedDevicesManager>>>,
}

/// PING 响应
#[derive(Debug, Serialize)]
pub struct PingResponse {
    pub device_id: String,
    pub status: String,
    pub port: u16,
    pub timestamp: String,
}

/// 配对请求
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PairRequest {
    pub device_id: String,
    pub device_name: String,
}

/// 配对响应
#[derive(Debug, Serialize)]
pub struct PairResponse {
    pub status: String,
    pub message: String,
}

/// 同步请求（使用新协议）
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub messages: Vec<serde_json::Value>,
}

/// 同步响应
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub status: String,
    pub received_count: usize,
    pub errors: Vec<String>,
}

/// 文件传输数据（JSON + base64）
#[derive(Debug, Deserialize)]
pub struct FileTransferData {
    pub device_id: String,
    pub filename: String,
    pub file_data: String,  // base64 编码
}

/// 文件传输响应
#[derive(Debug, Serialize)]
pub struct FileTransferResponse {
    pub status: String,
    pub file_id: String,
    pub file_path: String,
    pub file_size: u64,
}

/// 最大文件大小限制（500MB）
const MAX_FILE_SIZE: u64 = 500 * 1024 * 1024;

/// 查找可用端口（随机选择，优先低端口）
fn find_available_port(base_port: u16, max_attempts: u16) -> Option<u16> {
    // 先尝试基础端口
    if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", base_port)) {
        drop(listener);
        return Some(base_port);
    }
    
    // 如果基础端口被占用，随机尝试范围内的端口
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut tried_ports = std::collections::HashSet::new();
    
    for _ in 0..max_attempts {
        // 在范围内随机选择端口
        let port = rng.gen_range(base_port..base_port + max_attempts);
        
        if tried_ports.contains(&port) {
            continue;
        }
        tried_ports.insert(port);
        
        if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
            drop(listener);
            println!("🔌 Found available port: {} (tried {} ports)", port, tried_ports.len());
            return Some(port);
        }
    }
    
    // 如果随机尝试失败，按顺序尝试
    for port in base_port..(base_port + max_attempts) {
        if tried_ports.contains(&port) {
            continue;
        }
        if let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{}", port)) {
            drop(listener);
            return Some(port);
        }
    }
    
    None
}

/// GET /ping - 设备存活检测
async fn ping_handler(State(state): State<ServerState>) -> Json<PingResponse> {
    println!("📡 Received ping request");
    Json(PingResponse {
        device_id: state.device_id.to_string(),
        status: "ok".to_string(),
        port: *state.server_port,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// POST /pair-request - 设备配对请求（仅打印日志）
async fn pair_request_handler(
    State(state): State<ServerState>,
    Json(req): Json<PairRequest>,
) -> Json<PairResponse> {
    println!("🔗 Pair request received from device: {}", req.device_id);
    println!("   Device name: {}", req.device_name);
    println!("   Target device: {}", state.device_id);

    // 发送事件到前端，让用户决定是否接受
    if let Some(window) = state.app_handle.get_webview_window("main") {
        let _ = window.emit("pair-request-received", req);
    }

    Json(PairResponse {
        status: "pending".to_string(),
        message: "Pair request received, waiting for user confirmation".to_string(),
    })
}

/// POST /sync - 剪贴板数据同步
async fn sync_handler(
    State(state): State<ServerState>,
    Json(req): Json<SyncRequest>,
) -> Json<SyncResponse> {
    println!("📤 Received sync from device: {}", req.device_id);
    println!("   Messages count: {}", req.messages.len());
    println!("   Target device ID: {}", state.device_id);

    let mut errors = Vec::new();
    let mut received_count = 0;

    // 验证设备是否信任
    let is_trusted = if let Some(ref trusted_devices) = state.trusted_devices {
        match trusted_devices.lock() {
            Ok(trusted) => trusted.is_trusted(&req.device_id),
            Err(e) => {
                errors.push(format!("Failed to check trust status: {}", e));
                false
            }
        }
    } else {
        errors.push("Trusted devices manager not available".to_string());
        false
    };

    if !is_trusted {
        println!("⚠️ Device {} is not trusted, rejecting sync", req.device_id);
        errors.push(format!("Device {} is not trusted", req.device_id));
        return Json(SyncResponse {
            status: "rejected".to_string(),
            received_count: 0,
            errors,
        });
    }

    // 处理每个消息
    for msg_value in &req.messages {
        // 解析 ClipSyncMessage
        match serde_json::from_value::<super::protocol::ClipSyncMessage>(msg_value.clone()) {
            Ok(message) => {
                println!("📋 Processing message from {} at {}",
                    message.source_device, message.timestamp);

                // 验证哈希
                if !message.verify_hash() {
                    let err_msg = format!("Hash verification failed for message from {}", message.source_device);
                    println!("❌ {}", err_msg);
                    errors.push(err_msg);
                    continue;
                }

                // 处理不同类型的内容
                if let Some(ref clipboard_sync) = state.clipboard_sync {
                    match handle_clipboard_message(
                        &state.app_handle,
                        clipboard_sync,
                        message.clone(),
                    ) {
                        Ok(_) => {
                            received_count += 1;
                            println!("✓ Clipboard updated");
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to update clipboard: {}", e);
                            println!("❌ {}", err_msg);
                            errors.push(err_msg);
                        }
                    }
                } else {
                    errors.push("Clipboard sync manager not available".to_string());
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to parse message: {}", e);
                println!("❌ {}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    // 发送通知到前端
    if let Some(window) = state.app_handle.get_webview_window("main") {
        let _ = window.emit("clipboard-synced", serde_json::json!({
            "device_id": req.device_id,
            "received_count": received_count,
            "errors": errors,
        }));
    }

    Json(SyncResponse {
        status: if errors.is_empty() { "success".to_string() } else { "partial".to_string() },
        received_count,
        errors,
    })
}

/// 处理剪贴板消息
fn handle_clipboard_message(
    _handle: &AppHandle,
    _clipboard_sync: &Arc<super::ClipboardSyncManager>,
    message: super::protocol::ClipSyncMessage,
) -> Result<(), String> {
    use super::protocol::ClipType;
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    match message.msg_type {
        ClipType::Text => {
            // 文本类型：直接写入剪贴板
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    clipboard.set_text(&message.content)
                        .map_err(|e| format!("Failed to set text clipboard: {}", e))?;
                }
                Err(e) => return Err(format!("Failed to initialize clipboard: {}", e)),
            }
        }
        ClipType::Image => {
            // 图片类型：解码 base64 并写入剪贴板
            let image_bytes = STANDARD.decode(&message.content)
                .map_err(|e| format!("Failed to decode base64 image: {}", e))?;

            // 解码图像并创建 ImageData
            let img = image::load_from_memory(&image_bytes)
                .map_err(|e| format!("Failed to decode image: {}", e))?;

            let rgba = img.to_rgba8();
            let (width, height) = (rgba.width(), rgba.height());
            let bytes = rgba.into_raw();
            let image_data = arboard::ImageData {
                bytes: bytes.into(),
                width: width as usize,
                height: height as usize,
            };

            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    clipboard.set_image(image_data)
                        .map_err(|e| format!("Failed to set image clipboard: {}", e))?;
                }
                Err(e) => return Err(format!("Failed to initialize clipboard: {}", e)),
            }
        }
        ClipType::File => {
            // 文件类型：保存到临时目录
            let temp_dir = std::env::temp_dir();
            let file_name = message.file_name.as_deref()
                .unwrap_or("clipboard_file.txt");
            let file_path = temp_dir.join(file_name);

            // 如果内容是 base64，先解码
            let file_content = if is_base64(&message.content) {
                STANDARD.decode(&message.content)
                    .map_err(|e| format!("Failed to decode base64 content: {}", e))?
            } else {
                message.content.into_bytes()
            };

            std::fs::write(&file_path, file_content)
                .map_err(|e| format!("Failed to write file: {}", e))?;

            // 将文件路径写入剪贴板
            let path_str = file_path.to_string_lossy().to_string();
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    clipboard.set_text(&path_str)
                        .map_err(|e| format!("Failed to set file path to clipboard: {}", e))?;
                }
                Err(e) => return Err(format!("Failed to initialize clipboard: {}", e)),
            }

            println!("📁 File saved to: {}", file_path.display());
        }
    }

    Ok(())
}

/// 检查字符串是否是有效的 base64
fn is_base64(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.len() % 4 != 0 {
        return false;
    }
    trimmed.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='
    })
}

/// POST /file-transfer - 文件传输接口
async fn file_transfer_handler(
    State(state): State<ServerState>,
    Json(data): Json<FileTransferData>,
) -> Result<Json<FileTransferResponse>, StatusCode> {
    println!("📦 Received file transfer request");

    let device_id = data.device_id;
    let filename = data.filename;

    // 解码 base64 文件数据
    let file_data = match STANDARD.decode(&data.file_data) {
        Ok(data) => data,
        Err(e) => {
            println!("❌ Failed to decode base64: {}", e);
            return Ok(Json(FileTransferResponse {
                status: "error".to_string(),
                file_id: String::new(),
                file_path: String::new(),
                file_size: 0,
            }));
        }
    };

    let file_size = file_data.len() as u64;

    println!("   Device ID: {}", device_id);
    println!("   File: {}, size: {} bytes", filename, file_size);

    // 检查是否信任
    let is_trusted = if let Some(ref trusted_devices) = state.trusted_devices {
        match trusted_devices.lock() {
            Ok(trusted_mgr) => trusted_mgr.is_trusted(&device_id),
            Err(e) => {
                println!("❌ Failed to lock trusted devices: {}", e);
                false
            }
        }
    } else {
        println!("❌ Trusted devices manager not available");
        false
    };

    if !is_trusted {
        println!("⚠️ Device {} is not trusted, rejecting file transfer", device_id);
        return Ok(Json(FileTransferResponse {
            status: "rejected".to_string(),
            file_id: String::new(),
            file_path: String::new(),
            file_size: 0,
        }));
    }

    // 生成文件 ID（时间戳 + 原始文件名）
    let timestamp = chrono::Utc::now().timestamp_millis();
    let safe_filename = sanitize_filename(&filename);
    let file_id = format!("{}_{}", timestamp, safe_filename);

    // 保存文件
    let file_path = match save_file(&file_id, &file_data) {
        Ok(path) => path,
        Err(e) => {
            println!("❌ Failed to save file: {}", e);
            return Ok(Json(FileTransferResponse {
                status: "error".to_string(),
                file_id: String::new(),
                file_path: String::new(),
                file_size: 0,
            }));
        }
    };

    println!("✓ File saved to: {}", file_path);

    // 获取发送设备名称
    let device_name = if let Some(ref trusted_devices) = state.trusted_devices {
        match trusted_devices.lock() {
            Ok(trusted_mgr) => {
                trusted_mgr.get_device(&device_id)
                    .map(|d| d.device_name.clone())
                    .unwrap_or_else(|| device_id.clone())
            }
            Err(e) => {
                println!("❌ Failed to lock trusted devices: {}", e);
                device_id.clone()
            }
        }
    } else {
        device_id.clone()
    };

    // 发送系统通知
    if let Err(e) = state.app_handle.emit("file-notification", serde_json::json!({
        "title": "文件接收完成",
        "body": format!("收到来自 {} 的文件：{}", device_name, filename)
    })) {
        println!("❌ Failed to send notification event: {}", e);
    }

    // 发送事件到前端（按照要求的格式）
    if let Some(window) = state.app_handle.get_webview_window("main") {
        let _ = window.emit("file-received", serde_json::json!({
            "file_name": filename,
            "file_size": file_size,
            "file_path": file_path,
            "source_device": device_name,
            "timestamp": timestamp
        }));
    }

    Ok(Json(FileTransferResponse {
        status: "success".to_string(),
        file_id: file_id.clone(),
        file_path,
        file_size,
    }))
}

/// 获取下载目录
fn get_download_dir() -> Result<std::path::PathBuf, String> {
    let downloads = dirs::download_dir()
        .ok_or("Failed to get downloads directory")?;

    let clip_sync_dir = downloads.join("clipboard-caoguo");
    std::fs::create_dir_all(&clip_sync_dir)
        .map_err(|e| format!("Failed to create download directory: {}", e))?;

    Ok(clip_sync_dir)
}

/// 保存文件
fn save_file(filename: &str, data: &[u8]) -> Result<String, String> {
    let download_dir = get_download_dir()?;
    let file_path = download_dir.join(filename);

    std::fs::write(&file_path, data)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(file_path.to_string_lossy().to_string())
}

/// 清理文件名（移除危险字符）
fn sanitize_filename(filename: &str) -> String {
    let dangerous_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut result = String::new();

    for c in filename.chars() {
        if dangerous_chars.contains(&c) || c.is_control() {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    // 限制文件名长度
    if result.len() > 100 {
        result.truncate(100);
    }

    result
}

/// GET /images/{filename} - 静态图片文件服务
async fn serve_image(
    State(state): State<ServerState>,
    Path(filename): Path<String>
) -> impl IntoResponse {
    // 使用与 save_image_to_app_data 相同的路径获取方式
    let app_data_dir = match state.app_handle.path().app_data_dir() {
        Ok(dir) => dir,
        Err(e) => {
            println!("❌ Failed to get app data directory: {}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Failed to get app data directory"))
                .unwrap();
        }
    };
    
    let images_dir = app_data_dir.join("images");
    let file_path = images_dir.join(&filename);
    
    println!("📷 Serving image: {:?}", file_path);
    
    // 安全检查：确保文件在 images 目录内
    if !file_path.starts_with(&images_dir) {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Access denied"))
            .unwrap();
    }
    
    // 读取文件
    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            // 根据文件内容检测 MIME 类型（更可靠）
            let mime_type = if data.len() >= 8 && &data[0..4] == &[0x89, 0x50, 0x4E, 0x47] {
                "image/png"
            } else if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
                "image/jpeg"
            } else if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
                "image/gif"
            } else if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
                "image/webp"
            } else if data.len() >= 2 && &data[0..2] == b"BM" {
                "image/bmp"
            } else {
                // 回退到扩展名检测
                if filename.ends_with(".png") {
                    "image/png"
                } else if filename.ends_with(".gif") {
                    "image/gif"
                } else if filename.ends_with(".webp") {
                    "image/webp"
                } else {
                    "image/jpeg"
                }
            };
            
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime_type)
                .header(header::CACHE_CONTROL, "max-age=86400")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(data))
                .unwrap()
        }
        Err(e) => {
            println!("❌ Failed to read image file: {}", e);
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Image not found"))
                .unwrap()
        }
    }
}

/// GET / - 根路径处理（用于健康检查）
async fn root_handler() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("Clipboard Sync Server"))
        .unwrap()
}

/// 创建 Axum 路由器
fn create_router(state: ServerState) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/ping", get(ping_handler))
        .route("/pair-request", post(pair_request_handler))
        .route("/sync", post(sync_handler))
        .route("/file-transfer", post(file_transfer_handler))
        .route("/images/:filename", get(serve_image))
        .with_state(state)
        .layer(CorsLayer::permissive())
}

/// 启动 HTTP 服务器（同步版本，用于 setup）
pub fn start_server(
    config: ServerConfig,
    device_id: String,
    app_handle: AppHandle,
    server_handle: Option<Arc<ServerHandle>>,
) -> Result<(SocketAddr, ServerState), String> {
    let rt = tauri::async_runtime::handle();

    rt.block_on(async move {
        start_server_async(config, device_id, app_handle, server_handle).await
    })
}

/// 启动 HTTP 服务器（异步版本，用于 Tauri 命令）
pub async fn start_server_async(
    config: ServerConfig,
    device_id: String,
    app_handle: AppHandle,
    server_handle: Option<Arc<ServerHandle>>,
) -> Result<(SocketAddr, ServerState), String> {
    // 查找可用端口
    let port = find_available_port(config.base_port, config.max_attempts)
        .ok_or_else(|| format!("No available ports starting from {}", config.base_port))?;

    println!("🔌 Selected port: {}", port);

    // 创建服务器状态（使用传入的设备 ID 和 AppHandle）
    let server_state = ServerState {
        device_id: Arc::from(device_id.clone()),
        server_port: Arc::new(port),
        app_handle,
        clipboard_sync: None,
        trusted_devices: None,
    };

    // 创建路由器
    let app = create_router(server_state.clone());

    // 绑定地址
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // 获取 shutdown receiver（如果有的话）
    let shutdown_rx = server_handle.as_ref().map(|h| h.shutdown_tx.subscribe());

    // 如果有服务器句柄，设置状态
    if let Some(ref handle) = server_handle {
        handle.is_running.store(true, Ordering::SeqCst);
        if let Ok(mut p) = handle.port.lock() {
            *p = Some(port);
        }
    }

    // 启动服务器
    let device_id_clone = device_id.clone();
    let handle_clone = server_handle.clone();
    
    tokio::spawn(async move {
        println!("🚀 HTTP server listening on {}", addr);
        println!("   Device ID: {}", device_id_clone);

        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind HTTP server: {}", e);
                if let Some(ref handle) = handle_clone {
                    handle.is_running.store(false, Ordering::SeqCst);
                }
                return;
            }
        };

        // 使用 graceful shutdown
        match shutdown_rx {
            Some(mut rx) => {
                let server = axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        let _ = rx.recv().await;
                        println!("🛑 Server shutting down gracefully...");
                    });
                
                if let Err(e) = server.await {
                    eprintln!("HTTP server error: {}", e);
                }
            }
            None => {
                let server = axum::serve(listener, app);
                if let Err(e) = server.await {
                    eprintln!("HTTP server error: {}", e);
                }
            }
        }
        
        if let Some(ref handle) = handle_clone {
            handle.is_running.store(false, Ordering::SeqCst);
            if let Ok(mut p) = handle.port.lock() {
                *p = None;
            }
        }
        println!("✓ HTTP server stopped");
    });

    Ok((addr, server_state))
}

/// 启动服务器并返回服务器状态
pub fn start_with_default_config(device_id: String, app_handle: AppHandle) -> Result<ServerState, String> {
    start_server(ServerConfig::default(), device_id, app_handle, None).map(|(_, state)| state)
}

/// 停止服务器
pub fn stop_server(handle: &ServerHandle) -> Result<(), String> {
    if !handle.is_running() {
        return Ok(());
    }
    
    handle.shutdown();
    println!("✓ Server stop signal sent");
    Ok(())
}

/// 重启服务器
pub fn restart_server(
    config: ServerConfig,
    device_id: String,
    app_handle: AppHandle,
    handle: &ServerHandle,
) -> Result<(SocketAddr, ServerState), String> {
    // 先停止
    stop_server(handle)?;
    
    // 等待一小段时间确保端口释放
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    // 重新启动，使用新的 ServerHandle（因为旧的 shutdown_tx 已经触发过）
    let handle_arc = Arc::new(ServerHandle::new());
    start_server(config, device_id, app_handle, Some(handle_arc))
}

impl Clone for ServerHandle {
    fn clone(&self) -> Self {
        Self {
            shutdown_tx: self.shutdown_tx.clone(),
            is_running: self.is_running.clone(),
            port: self.port.clone(),
        }
    }
}
