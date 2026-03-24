use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Listener, Manager, Window, menu::{Menu, MenuItem, PredefinedMenuItem}, tray::TrayIconBuilder};
use serde::{Deserialize, Serialize};
use chrono::Utc;

// 数据结构定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content_type: String, // "text" or "image"
    pub content: String,      // 文本内容，或图片的 base64 缩略图（可选）
    pub file_path: Option<String>, // 图片文件路径（仅图片类型）
    pub created_at: String,   // ISO 8601 format
    pub favorite: bool,       // 是否收藏
}

// 局域网设备数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkDevice {
    pub device_id: String,
    pub name: String,
    pub hostname: String,
    pub ip: String,
    pub port: u16,
    pub last_seen: String,
}

// mDNS 服务数据结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MdnsServiceInfo {
    pub service_type: String,
    pub service_name: String,
    pub port: u16,
    pub txt_records: Vec<(String, String)>,
}

// 内置 HTTP 服务器模块
mod server;
use server::{ServerState, ServerHandle, ServerStatus};

// HTTP 客户端模块
mod client;
use client::{HttpClient, PingResponse, PairResponse, SyncResponse, FileTransferResponse};

// 设备身份管理模块
mod device;
use device::{DeviceManager, DeviceInfo};

// 配对管理模块
mod pairing;
use pairing::{PairingManager, PairingStatus};

// 信任设备管理模块
mod trusted;
use trusted::{TrustedDevicesManager, TrustedDevice};

// 剪贴板同步协议模块
mod protocol;

// 剪贴板监听和同步模块
mod clipboard;
use clipboard::ClipboardSyncManager;

// SQLite 数据库模块
mod database {
    use super::*;
    use rusqlite::{Connection, Result as SqlResult};

    pub struct Database {
        conn: Connection,
    }

    impl Database {
        pub fn new() -> SqlResult<Self> {
            // 获取应用数据目录
            let app_data_dir = dirs::data_local_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("com.yangtanfang.tie-lifang");
            
            // 确保目录存在
            let _ = std::fs::create_dir_all(&app_data_dir);
            
            let db_path = app_data_dir.join("clipboard.db");
            println!("📁 Database path: {:?}", db_path);
            
            let conn = Connection::open(&db_path)?;

            // 创建新表结构（如果不存在）
            conn.execute(
                "CREATE TABLE IF NOT EXISTS clipboard_items (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content_type TEXT NOT NULL,
                    content TEXT,
                    file_path TEXT,
                    created_at TEXT NOT NULL,
                    favorite INTEGER DEFAULT 0
                )",
                [],
            )?;

            // 尝试添加 file_path 列（如果不存在）
            let _ = conn.execute(
                "ALTER TABLE clipboard_items ADD COLUMN file_path TEXT",
                [],
            );
            
            // 尝试添加 favorite 列（如果不存在）
            let _ = conn.execute(
                "ALTER TABLE clipboard_items ADD COLUMN favorite INTEGER DEFAULT 0",
                [],
            );

            Ok(Database { conn })
        }

        pub fn insert_item(&self, content_type: &str, content: &str, file_path: Option<&str>) -> SqlResult<i64> {
            let created_at = Utc::now().to_rfc3339();
            self.conn.execute(
                "INSERT INTO clipboard_items (content_type, content, file_path, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![content_type, content, file_path, &created_at],
            )?;
            Ok(self.conn.last_insert_rowid())
        }

        pub fn get_all_items(&self) -> SqlResult<Vec<ClipboardItem>> {
            let mut stmt = self.conn.prepare(
                "SELECT id, content_type, content, file_path, created_at, COALESCE(favorite, 0) FROM clipboard_items ORDER BY created_at DESC LIMIT 100"
            )?;
            let item_iter = stmt.query_map([], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    file_path: row.get(3)?,
                    created_at: row.get(4)?,
                    favorite: row.get::<_, i64>(5)? != 0,
                })
            })?;

            let mut items = Vec::new();
            for item in item_iter {
                items.push(item?);
            }
            Ok(items)
        }

        pub fn search_items(&self, query: &str) -> SqlResult<Vec<ClipboardItem>> {
            let mut stmt = self.conn.prepare(
                "SELECT id, content_type, content, file_path, created_at, COALESCE(favorite, 0) FROM clipboard_items
                 WHERE content_type = 'text' AND content LIKE ?1 ORDER BY created_at DESC LIMIT 100"
            )?;
            let search_pattern = format!("%{}%", query);
            let item_iter = stmt.query_map([&search_pattern], |row| {
                Ok(ClipboardItem {
                    id: row.get(0)?,
                    content_type: row.get(1)?,
                    content: row.get(2)?,
                    file_path: row.get(3)?,
                    created_at: row.get(4)?,
                    favorite: row.get::<_, i64>(5)? != 0,
                })
            })?;

            let mut items = Vec::new();
            for item in item_iter {
                items.push(item?);
            }
            Ok(items)
        }

        pub fn delete_item(&self, id: i64) -> SqlResult<Option<String>> {
            // 先获取文件路径
            let file_path: Option<String> = self.conn.query_row(
                "SELECT file_path FROM clipboard_items WHERE id = ?1",
                [id],
                |row| row.get(0),
            ).ok();

            // 删除数据库记录
            self.conn.execute("DELETE FROM clipboard_items WHERE id = ?1", [id])?;

            Ok(file_path)
        }
        
        pub fn toggle_favorite(&self, id: i64) -> SqlResult<bool> {
            // 获取当前收藏状态
            let current: i64 = self.conn.query_row(
                "SELECT COALESCE(favorite, 0) FROM clipboard_items WHERE id = ?1",
                [id],
                |row| row.get(0),
            )?;
            
            let new_state = if current == 0 { 1 } else { 0 };
            
            self.conn.execute(
                "UPDATE clipboard_items SET favorite = ?1 WHERE id = ?2",
                rusqlite::params![new_state, id],
            )?;
            
            Ok(new_state == 1)
        }

        pub fn clear_all(&self) -> SqlResult<()> {
            self.conn.execute("DELETE FROM clipboard_items", [])?;
            Ok(())
        }
    }

    impl Default for Database {
        fn default() -> Self {
            Self::new().expect("Failed to initialize database")
        }
    }
}

use database::Database;

// 图片文件存储辅助函数
fn save_image_to_app_data(app_handle: &AppHandle, image_bytes: &[u8], item_id: i64) -> Result<String, String> {
    use std::fs;
    use std::io::Write;

    // 获取应用数据目录
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // 创建图片存储目录
    let images_dir = app_data_dir.join("images");
    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // 检测图片格式并确定扩展名
    let extension = detect_image_extension(image_bytes);
    
    // 生成文件名
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("img_{}_{}.{}", item_id, timestamp, extension);
    let file_path = images_dir.join(&filename);

    // 保存图片文件
    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create image file: {}", e))?;
    file.write_all(image_bytes)
        .map_err(|e| format!("Failed to write image data: {}", e))?;

    // 返回相对路径（相对于 app_data_dir）
    Ok(format!("images/{}", filename))
}

// 检测图片格式并返回扩展名
fn detect_image_extension(data: &[u8]) -> &'static str {
    // JPEG: FF D8 FF
    if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
        return "jpg";
    }
    // PNG: 89 50 4E 47 (即 \x89PNG)
    if data.len() >= 8 && &data[0..4] == &[0x89, 0x50, 0x4E, 0x47] {
        return "png";
    }
    // GIF: GIF87a 或 GIF89a
    if data.len() >= 6 && (&data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a") {
        return "gif";
    }
    // WebP: RIFF....WEBP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return "webp";
    }
    // BMP: BM
    if data.len() >= 2 && &data[0..2] == b"BM" {
        return "bmp";
    }
    // 默认使用 png（macOS 截图默认格式）
    "png"
}

// 删除图片文件
fn delete_image_file(app_handle: &AppHandle, relative_path: &str) -> Result<(), String> {
    use std::fs;

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let file_path = app_data_dir.join(relative_path);

    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to delete image file: {}", e))?;
    }

    Ok(())
}

// mDNS 服务发现模块
mod discovery;
use discovery::MdnsDiscovery;

// P2P 网络通信模块
mod p2p_server {
    use super::*;
    use axum::{
        http::StatusCode,
        response::Json,
        routing::get,
        Router,
    };
    use tower_http::cors::CorsLayer;
    use std::net::SocketAddr;

    const HTTP_SERVER_PORT: u16 = 54321;

    /// PING 响应结构
    #[derive(Debug, Serialize)]
    struct PingResponse {
        status: String,
        hostname: String,
        ip: String,
        version: String,
        timestamp: String,
    }

    /// /ping 端点 - 用于存活检测
    async fn ping_handler() -> Result<Json<PingResponse>, (StatusCode, String)> {
        let hostname = hostname::get()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .to_string_lossy()
            .to_string();

        let ip = local_ip_address::local_ip()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .to_string();

        Ok(Json(PingResponse {
            status: "ok".to_string(),
            hostname,
            ip,
            version: "0.1.0".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        }))
    }

    /// 启动 P2P HTTP 服务器
    pub fn start_server() -> Result<SocketAddr, String> {
        let rt = tauri::async_runtime::handle();

        rt.block_on(async {
            // 配置 CORS（允许跨域请求）
            let cors = CorsLayer::permissive();

            // 创建路由
            let app = Router::new()
                .route("/ping", get(ping_handler))
                .layer(cors);

            // 绑定地址
            let addr = SocketAddr::from(([0, 0, 0, 0], HTTP_SERVER_PORT));

            // 启动服务器
            tokio::spawn(async move {
                println!("🚀 P2P HTTP server listening on {}", addr);

                let listener = tokio::net::TcpListener::bind(addr).await
                    .expect("Failed to bind HTTP server");

                axum::serve(listener, app).await
                    .expect("HTTP server error");
            });

            Ok(addr)
        })
    }
}

// 全局状态
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Database>>,
    mdns: Arc<Mutex<Option<MdnsDiscovery>>>,
    server_state: Arc<Mutex<Option<ServerState>>>,
    server_handle: Arc<Mutex<Arc<ServerHandle>>>,
    device_info: Arc<DeviceManager>,
    pairing_manager: Arc<PairingManager>,
    trusted_devices: Arc<Mutex<TrustedDevicesManager>>,
    clipboard_sync: Arc<ClipboardSyncManager>,
}

// Tauri 命令：获取所有剪贴板记录
#[tauri::command]
async fn get_all_items(app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<Vec<ClipboardItem>, String> {
    let mut items = state.db.lock()
        .map_err(|e| e.to_string())?
        .get_all_items()
        .map_err(|e| e.to_string())?;

    // 将相对路径转换为绝对路径
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        for item in &mut items {
            if let Some(ref path) = item.file_path {
                // 将相对路径转换为绝对路径
                let absolute_path = app_data_dir.join(path);
                item.file_path = Some(absolute_path.to_string_lossy().to_string());
            }
        }
    }

    Ok(items)
}

// Tauri 命令：搜索剪贴板记录
#[tauri::command]
fn search_items(query: String, state: tauri::State<'_, AppState>) -> Result<Vec<ClipboardItem>, String> {
    state.db.lock()
        .map_err(|e| e.to_string())?
        .search_items(&query)
        .map_err(|e| e.to_string())
}

// Tauri 命令：删除记录
#[tauri::command]
fn delete_item(id: i64, app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // 先获取文件路径并删除记录
    let file_path = state.db.lock()
        .map_err(|e| e.to_string())?
        .delete_item(id)
        .map_err(|e| e.to_string())?;

    // 如果有关联的图片文件，删除它
    if let Some(path) = file_path {
        let _ = delete_image_file(&app_handle, &path);
    }

    Ok(())
}

// Tauri 命令：切换收藏状态
#[tauri::command]
fn toggle_favorite(id: i64, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    state.db.lock()
        .map_err(|e| e.to_string())?
        .toggle_favorite(id)
        .map_err(|e| e.to_string())
}

// Tauri 命令：清空所有记录
#[tauri::command]
fn clear_all(app_handle: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    use std::fs;

    // 先获取所有图片文件路径
    let items = state.db.lock()
        .map_err(|e| e.to_string())?
        .get_all_items()
        .map_err(|e| e.to_string())?;

    // 删除所有图片文件
    for item in items {
        if let Some(path) = item.file_path {
            let _ = delete_image_file(&app_handle, &path);
        }
    }

    // 清空数据库
    state.db.lock()
        .map_err(|e| e.to_string())?
        .clear_all()
        .map_err(|e| e.to_string())?;

    // 尝试删除整个图片目录
    if let Ok(app_data_dir) = app_handle.path().app_data_dir() {
        let images_dir = app_data_dir.join("images");
        if images_dir.exists() {
            let _ = fs::remove_dir_all(&images_dir);
        }
    }

    Ok(())
}

// Tauri 命令：获取图片文件的可访问 URL
#[tauri::command]
async fn get_image_url(relative_path: String, app_handle: AppHandle) -> Result<String, String> {
    // 获取应用数据目录
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // 构建完整文件路径
    let file_path = app_data_dir.join(&relative_path);

    // 返回文件绝对路径，前端使用 convertFileSrc
    Ok(file_path.to_string_lossy().to_string())
}

// Tauri 命令：复制文本到剪贴板
#[tauri::command]
fn copy_text(text: String) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(&text).map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri 命令：隐藏窗口
#[tauri::command]
fn hide_window(window: Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri 命令：开始拖动窗口
#[tauri::command]
async fn start_drag(window: Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())?;
    Ok(())
}

// Tauri 命令：启动局域网设备发现
#[tauri::command]
async fn start_mdns_discovery(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut mdns_guard = state.mdns.lock()
        .map_err(|e| format!("Failed to acquire mdns lock: {}", e))?;

    if mdns_guard.is_some() {
        return Err("mDNS discovery already started".to_string());
    }

    // 获取 device_id
    let device_id = state.device_info.get_or_create_device_id()?;

    // 获取服务器状态
    let server_state = state.server_state.lock()
        .map_err(|e| format!("Failed to acquire server state lock: {}", e))?;

    if let Some(ref server_state) = *server_state {
        // 使用已有的服务器状态启动 mDNS
        let mdns = discovery::MdnsDiscovery::with_server_state(server_state.clone())?;
        mdns.register_service(app_handle.clone(), device_id)?;
        mdns.start_browsing(app_handle)?;

        let _ = server_state;
        *mdns_guard = Some(mdns);
    } else {
        // 服务器未启动，返回错误
        return Err("HTTP server not started. Please restart the application.".to_string());
    }

    Ok(())
}

// Tauri 命令：获取已发现的设备列表
#[tauri::command]
async fn get_discovered_devices(state: tauri::State<'_, AppState>) -> Result<Vec<NetworkDevice>, String> {
    let mdns_guard = state.mdns.lock()
        .map_err(|e| format!("Failed to acquire mdns lock: {}", e))?;

    if let Some(ref mdns) = *mdns_guard {
        Ok(mdns.get_devices())
    } else {
        Ok(vec![])
    }
}

// Tauri 命令：停止局域网设备发现
#[tauri::command]
async fn stop_mdns_discovery(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut mdns_guard = state.mdns.lock()
        .map_err(|e| format!("Failed to acquire mdns lock: {}", e))?;

    *mdns_guard = None;

    Ok(())
}

// Tauri 命令：Ping 远程设备
#[tauri::command]
async fn ping_device(ip: String, port: u16) -> Result<PingResponse, String> {
    let client = HttpClient::new();
    client.send_ping(ip, port).await
}

// Tauri 命令：发送配对请求
#[tauri::command]
async fn send_pair_request(
    ip: String,
    port: u16,
    device_id: String,
    device_name: String,
) -> Result<PairResponse, String> {
    let client = HttpClient::new();
    client.request_pair(ip, port, device_id, device_name).await
}

// Tauri 命令：发送同步数据
#[tauri::command]
async fn send_sync(
    ip: String,
    port: u16,
    device_id: String,
    items: Vec<ClipboardItem>,
) -> Result<SyncResponse, String> {
    let client = HttpClient::new();
    client.send_sync_data(ip, port, device_id, items).await
}

// Tauri 命令：获取本地服务器端口
#[tauri::command]
async fn get_server_port(state: tauri::State<'_, AppState>) -> Result<Option<u16>, String> {
    let server_state = state.server_state.lock()
        .map_err(|e| e.to_string())?;

    Ok(server_state.as_ref().map(|s| *s.server_port))
}

// Tauri 命令：获取服务器状态
#[tauri::command]
async fn get_server_status(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let handle = state.server_handle.lock()
        .map_err(|e| e.to_string())?;
    
    Ok(ServerStatus {
        is_running: handle.is_running(),
        port: handle.get_port(),
        started_at: None,
    })
}

// Tauri 命令：获取服务器配置端口
#[tauri::command]
async fn get_server_config_port() -> Result<u16, String> {
    let config = server::ServerConfig::load();
    Ok(config.base_port)
}

// Tauri 命令：设置服务器配置端口
#[tauri::command]
async fn set_server_config_port(port: u16) -> Result<(), String> {
    // 验证端口范围
    if port < 1024 || port > 65535 {
        return Err("端口范围必须在 1024-65535 之间".to_string());
    }
    
    let mut config = server::ServerConfig::load();
    config.set_base_port(port);
    config.save()?;
    
    println!("✓ Server port config saved: {}", port);
    Ok(())
}

// Tauri 命令：启动服务器
#[tauri::command]
async fn start_server_service(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ServerStatus, String> {
    // 获取设备 ID
    let device_id = state.device_info.get_or_create_device_id()?;
    
    // 获取当前句柄并检查状态
    {
        let handle = state.server_handle.lock()
            .map_err(|e| e.to_string())?;
        if handle.is_running() {
            return Ok(ServerStatus {
                is_running: true,
                port: handle.get_port(),
                started_at: None,
            });
        }
    }
    
    // 创建新的 ServerHandle
    let new_handle = Arc::new(ServerHandle::new());
    
    // 加载配置
    let config = server::ServerConfig::load();
    
    // 使用异步版本启动服务器
    let result = server::start_server_async(
        config,
        device_id,
        app_handle,
        Some(new_handle.clone()),
    ).await;
    
    match result {
        Ok((addr, server_state)) => {
            // 更新服务器句柄
            {
                let mut handle_guard = state.server_handle.lock()
                    .map_err(|e| e.to_string())?;
                *handle_guard = new_handle;
            }
            
            // 更新服务器状态
            let mut state_guard = state.server_state.lock()
                .map_err(|e| e.to_string())?;
            *state_guard = Some(server_state);
            
            Ok(ServerStatus {
                is_running: true,
                port: Some(addr.port()),
                started_at: Some(chrono::Utc::now().to_rfc3339()),
            })
        }
        Err(e) => Err(e),
    }
}

// Tauri 命令：停止服务器
#[tauri::command]
async fn stop_server_service(state: tauri::State<'_, AppState>) -> Result<ServerStatus, String> {
    let handle = state.server_handle.lock()
        .map_err(|e| e.to_string())?;
    server::stop_server(&handle)?;
    
    // 清空服务器状态
    let mut state_guard = state.server_state.lock()
        .map_err(|e| e.to_string())?;
    *state_guard = None;
    
    Ok(ServerStatus {
        is_running: false,
        port: None,
        started_at: None,
    })
}

// Tauri 命令：重启服务器
#[tauri::command]
async fn restart_server_service(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<ServerStatus, String> {
    // 获取设备 ID
    let device_id = state.device_info.get_or_create_device_id()?;
    
    // 停止服务器
    {
        let handle = state.server_handle.lock()
            .map_err(|e| e.to_string())?;
        server::stop_server(&handle)?;
    }
    
    // 等待端口释放
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    
    // 创建新的 ServerHandle
    let new_handle = Arc::new(ServerHandle::new());
    
    // 加载配置
    let config = server::ServerConfig::load();
    
    // 使用异步版本启动服务器
    let result = server::start_server_async(
        config,
        device_id,
        app_handle,
        Some(new_handle.clone()),
    ).await;
    
    match result {
        Ok((addr, server_state)) => {
            // 更新服务器句柄
            {
                let mut handle_guard = state.server_handle.lock()
                    .map_err(|e| e.to_string())?;
                *handle_guard = new_handle;
            }
            
            // 更新服务器状态
            let mut state_guard = state.server_state.lock()
                .map_err(|e| e.to_string())?;
            *state_guard = Some(server_state);
            
            Ok(ServerStatus {
                is_running: true,
                port: Some(addr.port()),
                started_at: Some(chrono::Utc::now().to_rfc3339()),
            })
        }
        Err(e) => Err(e),
    }
}

// Tauri 命令：获取本地设备 ID
#[tauri::command]
async fn get_device_id(state: tauri::State<'_, AppState>) -> Result<String, String> {
    state.device_info.get_or_create_device_id()
}

// Tauri 命令：获取设备信息
#[tauri::command]
async fn get_device_info(state: tauri::State<'_, AppState>) -> Result<DeviceInfo, String> {
    state.device_info.get_device_info()
}

// Tauri 命令：更新设备名称
#[tauri::command]
async fn update_device_name(
    state: tauri::State<'_, AppState>,
    device_name: String,
) -> Result<DeviceInfo, String> {
    state.device_info.update_device_info(Some(device_name))
}

// Tauri 命令：重置设备 ID
#[tauri::command]
async fn reset_device_id(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.device_info.reset_device_id()
}

// Tauri 命令：发起配对请求
#[tauri::command]
async fn request_pair(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    target_device_id: String,
) -> Result<String, String> {
    // 从设备列表中查找目标设备
    let target_device = {
        let mdns_guard = state.mdns.lock()
            .map_err(|e| e.to_string())?;

        if let Some(ref mdns) = *mdns_guard {
            let devices = mdns.get_devices();
            devices
                .iter()
                .find(|d| d.hostname == target_device_id)
                .ok_or_else(|| format!("Target device not found: {}", target_device_id))?
                .clone()
        } else {
            return Err("Device discovery not started. Please start mDNS discovery first.".to_string());
        }
    };

    // 获取本设备信息
    let my_device_id = state.device_info.get_or_create_device_id()?;
    let my_device_name = state.device_info.get_device_name();

    // 使用配对管理器创建配对状态
    let pairing_state = state.pairing_manager
        .initiate_pairing(target_device_id.clone(), target_device.name.clone())
        .await?;

    // 发送配对请求
    let client = HttpClient::new();
    match client.request_pair(
        target_device.ip.clone(),
        target_device.port,
        my_device_id,
        my_device_name,
    ).await {
        Ok(response) => {
            println!("📨 Pair request sent successfully: {}", response.status);

            // 发送事件到前端
            if let Some(window) = app_handle.get_webview_window("main") {
                let _ = window.emit("pair-request-sent", pairing_state);
            }

            Ok(response.status)
        }
        Err(e) => {
            // 更新配对状态为失败
            let _ = state.pairing_manager
                .update_pairing_status(target_device_id.clone(), PairingStatus::Failed)
                .await;

            Err(e)
        }
    }
}

// Tauri 命令：取消配对请求
#[tauri::command]
async fn cancel_pair(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    target_device_id: String,
) -> Result<bool, String> {
    match state.pairing_manager.cancel_pairing(target_device_id.clone()).await {
        Ok(cancelled) => {
            if cancelled {
                // 发送取消事件到前端
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.emit("pair-cancelled", target_device_id);
                }
            }
            Ok(cancelled)
        }
        Err(e) => Err(e)
    }
}

// Tauri 命令：获取配对状态
#[tauri::command]
async fn get_pairing_status(
    state: tauri::State<'_, AppState>,
    target_device_id: String,
) -> Result<Option<serde_json::Value>, String> {
    let status = state.pairing_manager.get_pairing_status(&target_device_id).await;

    Ok(status.map(|s| serde_json::to_value(s).unwrap()))
}

// Tauri 命令：获取所有配对状态
#[tauri::command]
async fn get_all_pairings(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let pairings = state.pairing_manager.get_all_pairings().await;

    Ok(pairings
        .into_iter()
        .map(|p| serde_json::to_value(p).unwrap())
        .collect())
}

// Tauri 命令：清理已完成的配对请求
#[tauri::command]
async fn cleanup_pairings(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.pairing_manager.cleanup_completed_pairings().await;
    Ok(())
}

// Tauri 命令：接受配对请求
#[tauri::command]
async fn accept_pair(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    device_id: String,
    device_name: String,
) -> Result<String, String> {
    // 生成确认 Token（使用 UUID）
    let token = uuid::Uuid::new_v4().to_string();

    // 添加到信任设备列表
    {
        let mut trusted = state.trusted_devices.lock()
            .map_err(|e| e.to_string())?;
        trusted.add_device(device_id.clone(), device_name.clone())?;
    }

    println!("✅ Pair accepted for device: {} (token: {})", device_id, token);

    // 更新配对状态
    let _ = state.pairing_manager
        .update_pairing_status(device_id.clone(), PairingStatus::Accepted)
        .await;

    // 发送事件到前端
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("pair-accepted", serde_json::json!({
            "device_id": device_id,
            "device_name": device_name,
            "token": token
        }));
    }

    Ok(token)
}

// Tauri 命令：拒绝配对请求
#[tauri::command]
async fn reject_pair(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    println!("🚫 Pair rejected for device: {}", device_id);

    // 更新配对状态
    let _ = state.pairing_manager
        .update_pairing_status(device_id.clone(), PairingStatus::Rejected)
        .await;

    // 发送事件到前端
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("pair-rejected", device_id);
    }

    Ok(())
}

// Tauri 命令：获取信任设备列表
#[tauri::command]
async fn get_trusted_devices(state: tauri::State<'_, AppState>) -> Result<Vec<TrustedDevice>, String> {
    let trusted = state.trusted_devices.lock()
        .map_err(|e| e.to_string())?;

    Ok(trusted.get_all_devices())
}

// Tauri 命令：移除信任设备
#[tauri::command]
async fn remove_trusted_device(
    state: tauri::State<'_, AppState>,
    device_id: String,
) -> Result<(), String> {
    let mut trusted = state.trusted_devices.lock()
        .map_err(|e| e.to_string())?;

    trusted.remove_device(&device_id)
}

// Tauri 命令：检查设备是否已信任
#[tauri::command]
async fn is_device_trusted(
    state: tauri::State<'_, AppState>,
    device_id: String,
) -> Result<bool, String> {
    let trusted = state.trusted_devices.lock()
        .map_err(|e| e.to_string())?;

    Ok(trusted.is_trusted(&device_id))
}

// Tauri 命令：发送文件到指定设备
#[tauri::command]
async fn send_file_to_device(
    app_handle: AppHandle,
    state: tauri::State<'_, AppState>,
    device_id: String,
    file_path: String,
) -> Result<FileTransferResponse, String> {
    // 从 mDNS 发现的设备列表中查找目标设备
    let target_device = {
        let mdns_guard = state.mdns.lock()
            .map_err(|e| format!("Failed to acquire mdns lock: {}", e))?;

        if let Some(ref mdns) = *mdns_guard {
            let devices = mdns.get_devices();
            devices
                .iter()
                .find(|d| d.device_id == device_id)
                .ok_or_else(|| format!("Target device not found: {}", device_id))?
                .clone()
        } else {
            return Err("Device discovery not started. Please start mDNS discovery first.".to_string());
        }
    };

    // 检查设备是否在线（通过 PING）
    let client = HttpClient::new();
    match client.send_ping(target_device.ip.clone(), target_device.port).await {
        Ok(_) => {
            println!("✓ Device {} is online", device_id);
        }
        Err(e) => {
            return Err(format!("Device {} is not reachable: {}", device_id, e));
        }
    }

    // 获取本设备 ID
    let my_device_id = state.device_info.get_or_create_device_id()?;

    // 发送文件
    println!("📤 Sending file {} to device {}", file_path, device_id);
    let response = client
        .send_file(
            target_device.ip.clone(),
            target_device.port,
            my_device_id,
            file_path.clone(),
            None,
        )
        .await?;

    // 发送文件发送完成事件到前端
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit("file-sent", serde_json::json!({
            "device_id": device_id,
            "file_path": file_path,
            "file_id": response.file_id,
            "file_size": response.file_size
        }));
    }

    Ok(response)
}

// Tauri 命令：获取下载目录路径
#[tauri::command]
async fn get_download_directory() -> Result<String, String> {
    let download_dir = dirs::download_dir()
        .ok_or("Failed to get download directory")?;

    Ok(download_dir.to_string_lossy().to_string())
}

// Tauri 命令：打开下载目录
#[tauri::command]
async fn open_download_directory() -> Result<(), String> {
    let download_dir = dirs::download_dir()
        .ok_or("Failed to get download directory")?;

    // 使用 opener crate 打开目录
    opener::open(&download_dir)
        .map_err(|e| format!("Failed to open download directory: {}", e))?;

    Ok(())
}

// Tauri 命令：打开图片资源目录
#[tauri::command]
async fn open_images_directory(app_handle: AppHandle) -> Result<String, String> {
    // 获取应用数据目录
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let images_dir = app_data_dir.join("images");

    // 确保目录存在
    std::fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // 使用 opener crate 打开目录
    opener::open(&images_dir)
        .map_err(|e| format!("Failed to open images directory: {}", e))?;

    Ok(images_dir.to_string_lossy().to_string())
}

// Tauri 命令：使用系统默认应用打开图片
#[tauri::command]
async fn open_image_file(app_handle: AppHandle, filename: String) -> Result<(), String> {
    // 获取应用数据目录
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let image_path = app_data_dir.join("images").join(&filename);
    
    // 检查文件是否存在
    if !image_path.exists() {
        return Err(format!("Image file not found: {}", filename));
    }

    // 使用系统默认应用打开图片
    opener::open(&image_path)
        .map_err(|e| format!("Failed to open image file: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn save_image_to_file(base64_data: String, filename: Option<String>) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    // 解码 base64 数据
    let image_bytes = STANDARD.decode(&base64_data)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    // 确定保存目录（使用下载目录）
    let save_dir = dirs::download_dir()
        .ok_or("Failed to get download directory")?;

    // 确保目录存在
    std::fs::create_dir_all(&save_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // 生成文件名
    let filename = filename.unwrap_or_else(|| {
        let now = Utc::now();
        format!("screenshot_{}.jpg", now.format("%Y%m%d_%H%M%S"))
    });

    // 如果文件名没有扩展名，根据图片数据添加
    let filename = if filename.contains('.') {
        filename
    } else {
        // 使用统一的格式检测函数
        let ext = detect_image_extension(&image_bytes);
        format!("{}.{}", filename, ext)
    };

    // 完整路径
    let file_path = save_dir.join(&filename);

    // 保存文件
    std::fs::write(&file_path, &image_bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    println!("✓ Image saved to: {:?}", file_path);

    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
fn clear_hash_cache(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.clipboard_sync.clear_hash_cache();
    Ok(())
}

// Tauri 命令：读取图片文件并返回 base64 数据
#[tauri::command]
async fn get_image_base64(file_path: String, app_handle: AppHandle) -> Result<String, String> {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    println!("📷 get_image_base64 called with: {:?}", file_path);
    
    // 判断是绝对路径还是相对路径
    let path = std::path::Path::new(&file_path);
    let file_path = if path.is_absolute() {
        // 已经是绝对路径，直接使用
        path.to_path_buf()
    } else {
        // 相对路径，拼接应用数据目录
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {}", e))?;
        app_data_dir.join(&file_path)
    };
    
    println!("📷 Reading image from: {:?}", file_path);
    
    // 检查文件是否存在
    if !file_path.exists() {
        return Err(format!("Image file not found: {:?}", file_path));
    }
    
    // 读取文件内容
    let image_bytes = std::fs::read(&file_path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;
    
    // 检测图片类型
    let mime_type = if image_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if image_bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if image_bytes.starts_with(b"GIF87a") || image_bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if image_bytes.starts_with(b"RIFF") && image_bytes.len() > 12 && &image_bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/jpeg" // 默认使用 JPEG
    };
    
    println!("📷 Image loaded: {} bytes, type: {}", image_bytes.len(), mime_type);
    
    // 编码为 base64
    let base64_data = STANDARD.encode(&image_bytes);
    
    // 返回 data URL
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

// 后台监听剪贴板变化 (已废弃，使用 clipboard 模块)
fn start_clipboard_monitor(_handle: AppHandle, _state: Arc<AppState>) {
    // 此函数已被 clipboard::ClipboardSyncManager 替代
    println!("⚠️ start_clipboard_monitor is deprecated, using ClipboardSyncManager instead");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化设备管理器
    let device_manager = Arc::new(DeviceManager::default());
    let device_id = device_manager.get_or_create_device_id()
        .expect("Failed to initialize device ID");
    let device_name = device_manager.get_device_name();

    println!("📱 Device initialized:");
    println!("   ID: {}", device_id);
    println!("   Name: {}", device_name);

    let db = Arc::new(Mutex::new(Database::new().expect("Failed to initialize database")));

    let clipboard_sync = Arc::new(ClipboardSyncManager::new(device_id.clone()));

    let server_handle = Arc::new(ServerHandle::new());

    let state = AppState {
        db: db.clone(),
        mdns: Arc::new(Mutex::new(None)),
        server_state: Arc::new(Mutex::new(None)),
        server_handle: Arc::new(Mutex::new(server_handle.clone())),
        device_info: device_manager,
        pairing_manager: Arc::new(PairingManager::default()),
        trusted_devices: Arc::new(Mutex::new(TrustedDevicesManager::default())),
        clipboard_sync: clipboard_sync.clone(),
    };

    let state_arc = Arc::new(state.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_all_items,
            search_items,
            delete_item,
            toggle_favorite,
            clear_all,
            copy_text,
            hide_window,
            start_drag,
            start_mdns_discovery,
            get_discovered_devices,
            stop_mdns_discovery,
            ping_device,
            send_pair_request,
            send_sync,
            get_server_port,
            get_server_status,
            get_server_config_port,
            set_server_config_port,
            start_server_service,
            stop_server_service,
            restart_server_service,
            get_device_id,
            get_device_info,
            update_device_name,
            reset_device_id,
            request_pair,
            cancel_pair,
            get_pairing_status,
            get_all_pairings,
            cleanup_pairings,
            accept_pair,
            reject_pair,
            get_trusted_devices,
            remove_trusted_device,
            is_device_trusted,
            send_file_to_device,
            get_download_directory,
            open_download_directory,
            open_images_directory,
            open_image_file,
            save_image_to_file,
            clear_hash_cache,
            get_image_base64
        ])
        .setup(move |app| {
            let handle = app.handle().clone();

            // 监听窗口关闭事件，仅隐藏窗口而不是退出应用
            let app_handle = app.handle().clone();
            app.listen("tauri://close-requested", move |_event: tauri::Event| {
                println!("🔒 Window close requested, hiding window instead of quitting");
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.hide();
                }
            });

            // 获取设备 ID 并启动 HTTP 服务器
            let device_id = app.state::<AppState>().device_info.get_or_create_device_id()
                .expect("Failed to get device ID");

            let server_handle_arc = app.state::<AppState>().server_handle.clone();
            let server_handle = {
                let handle_guard = server_handle_arc.lock().unwrap();
                handle_guard.clone()
            };
            
            // 加载配置
            let config = server::ServerConfig::load();
            
            match server::start_server(
                config,
                device_id,
                handle.clone(),
                Some(server_handle),
            ) {
                Ok((addr, server_state)) => {
                    println!("✓ HTTP server started on port {}", addr.port());

                    // 更新服务器状态，注入 clipboard_sync 和 trusted_devices
                    let mut updated_server_state = server_state.clone();
                    updated_server_state.clipboard_sync = Some(clipboard_sync.clone());
                    updated_server_state.trusted_devices = Some(
                        app.state::<AppState>().trusted_devices.clone()
                    );

                    *app.state::<AppState>().server_state.lock().unwrap() = Some(updated_server_state);
                }
                Err(e) => {
                    eprintln!("Failed to start HTTP server: {}", e);
                }
            }

            // 启动剪贴板监听和同步
            clipboard_sync.start_monitoring(
                handle.clone(),
                state_arc.clone(),
                app.state::<AppState>().mdns.clone(),
            );

            // 设置全局快捷键 (Command/Ctrl + Shift + V)
            #[cfg(not(target_os = "macos"))]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

                let shortcut = Shortcut::new(Some(tauri_plugin_global_shortcut::Modifiers::CONTROL | tauri_plugin_global_shortcut::Modifiers::SHIFT), tauri_plugin_global_shortcut::Code::KeyV);

                let _ = app.global_shortcut().on_shortcut(shortcut, move |app_handle, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });
            }

            // macOS 上的全局快捷键
            #[cfg(target_os = "macos")]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

                let shortcut = Shortcut::new(Some(tauri_plugin_global_shortcut::Modifiers::SUPER | tauri_plugin_global_shortcut::Modifiers::SHIFT), tauri_plugin_global_shortcut::Code::KeyV);

                let _ = app.global_shortcut().on_shortcut(shortcut, move |app_handle, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                });
            }

            // 创建系统托盘菜单
            println!("📦 Creating tray menu...");
            let show = MenuItem::new(&handle, "打开", true, None::<&str>).unwrap();
            let separator1 = PredefinedMenuItem::separator(&handle).unwrap();
            let devices = MenuItem::new(&handle, "设备管理", true, None::<&str>).unwrap();
            let file_transfer = MenuItem::new(&handle, "文件传输", true, None::<&str>).unwrap();
            let separator2 = PredefinedMenuItem::separator(&handle).unwrap();
            let quit = MenuItem::new(&handle, "退出", true, None::<&str>).unwrap();

            let tray_menu = Menu::with_items(&handle, &[
                &show,
                &separator1,
                &devices,
                &file_transfer,
                &separator2,
                &quit,
            ]).unwrap();
            println!("✓ Tray menu created");

            // 加载 PNG 并转换为 RGBA
            println!("🎨 Loading tray icon...");
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let image = image::load_from_memory(icon_bytes).unwrap();
            let rgba = image.to_rgba8();
            let icon = tauri::image::Image::new_owned(rgba.to_vec(), rgba.width(), rgba.height());
            println!("✓ Tray icon loaded");

            println!("🔔 Building tray icon...");

            let app_handle_clone = handle.clone();

            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&tray_menu)
                .icon(icon)
                .icon_as_template(true)
                .show_menu_on_left_click(false)
                .tooltip("贴立方")
                .on_menu_event(move |app_handle, event| {
                    match event.id.as_ref() {
                        "打开" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "设备管理" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("open-devices", ());
                            }
                        }
                        "文件传输" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("open-file-transfer", ());
                            }
                        }
                        "退出" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(move |_tray_id, event| {
                    // 点击托盘图标时显示窗口
                    if matches!(event, tauri::tray::TrayIconEvent::Click { .. }) {
                        if let Some(window) = app_handle_clone.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(&handle)
                .unwrap();
            println!("✓ Tray icon created successfully!");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
