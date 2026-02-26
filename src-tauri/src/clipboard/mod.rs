//! 剪贴板监听和同步模块
//! 负责监听本地剪贴板变化并同步到其他设备

use super::*;
use super::client::HttpClient;
use super::discovery::MdnsDiscovery;
use super::protocol::{ClipSyncMessage, ClipType};
use arboard::Clipboard;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// 最近的哈希缓存大小
const HASH_CACHE_SIZE: usize = 100;

/// 剪贴板同步管理器
pub struct ClipboardSyncManager {
    /// 本设备 ID
    device_id: String,
    /// 最近接收的哈希缓存（用于防止回声）
    hash_cache: Arc<Mutex<VecDeque<String>>>,
    /// 本机更新标志（防止将远程写入的数据再次同步出去）
    is_local_update: Arc<Mutex<bool>>,
}

impl ClipboardSyncManager {
    /// 创建新的剪贴板同步管理器
    pub fn new(device_id: String) -> Self {
        ClipboardSyncManager {
            device_id,
            hash_cache: Arc::new(Mutex::new(VecDeque::with_capacity(HASH_CACHE_SIZE))),
            is_local_update: Arc::new(Mutex::new(true)),
        }
    }

    /// 检测图片格式并返回扩展名
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

    /// 检查哈希是否在缓存中（回声检测）
    fn is_hash_cached(&self, hash: &str) -> bool {
        let cache = self.hash_cache.lock().unwrap();
        cache.contains(&hash.to_string())
    }

    /// 将哈希添加到缓存中
    fn add_hash_to_cache(&self, hash: String) {
        let mut cache = self.hash_cache.lock().unwrap();
        cache.push_back(hash);
        // 保持缓存大小限制
        while cache.len() > HASH_CACHE_SIZE {
            cache.pop_front();
        }
    }

    /// 清空哈希缓存
    pub fn clear_hash_cache(&self) {
        let mut cache = self.hash_cache.lock().unwrap();
        cache.clear();
        println!("🗑️ Hash cache cleared");
    }

    /// 启动剪贴板监听
    pub fn start_monitoring(
        &self,
        handle: AppHandle,
        state: Arc<AppState>,
        mdns: Arc<Mutex<Option<MdnsDiscovery>>>,
    ) {
        let device_id = self.device_id.clone();
        let hash_cache = self.hash_cache.clone();
        let is_local_update = self.is_local_update.clone();
        let client = HttpClient::new();

        tauri::async_runtime::spawn(async move {
            let mut last_text = String::new();
            let mut clipboard: Option<Clipboard> = None;

            println!("📋 Clipboard monitor started");

            loop {
                // 每隔 500ms 检查一次剪贴板
                tokio::time::sleep(Duration::from_millis(500)).await;

                // 如果是远程更新，跳过这次检查
                {
                    let local_flag = is_local_update.lock().unwrap();
                    if !*local_flag {
                        // 重置标志，下次检查恢复本地检测
                        drop(local_flag);
                        let mut flag = is_local_update.lock().unwrap();
                        *flag = true;
                        continue;
                    }
                }

                // 初始化或重新创建剪贴板实例
                if clipboard.is_none() {
                    clipboard = match Clipboard::new() {
                        Ok(cb) => Some(cb),
                        Err(_) => {
                            continue;
                        }
                    };
                }

                if let Some(ref mut cb) = clipboard {
                    // 先检查文本内容
                    if let Ok(text) = cb.get_text() {
                        if !text.is_empty() && text != last_text {
                            last_text = text.clone();

                            // 计算哈希
                            let hash = ClipSyncMessage::calculate_hash(&text);

                            // 检查是否是回声（检查哈希缓存）
                            {
                                let cache = hash_cache.lock().unwrap();
                                if cache.contains(&hash) {
                                    println!("🔁 Echo detected (hash in cache), skipping sync");
                                    continue;
                                }
                            }

                            println!("📋 Clipboard changed (text), hash: {}", hash);

                            // 保存到数据库
                            if let Ok(id) = state.db.lock().unwrap().insert_item("text", &text, None) {
                                // 发送事件到前端
                                let item = ClipboardItem {
                                    id,
                                    content_type: "text".to_string(),
                                    content: text.clone(),
                                    file_path: None,
                                    created_at: Utc::now().to_rfc3339(),
                                    favorite: false,
                                };

                                if let Some(window) = handle.get_webview_window("main") {
                                    let _ = window.emit("clipboard-update", item);
                                }
                            }

                            // 同步到信任设备
                            Self::sync_to_trusted_devices(
                                &handle,
                                &state,
                                &mdns,
                                &client,
                                &device_id,
                                text.clone(),
                                &hash_cache,
                            ).await;
                        }
                    }

                    // 再检查图片内容
                    if let Ok(image) = cb.get_image() {
                        let width = image.width;
                        let height = image.height;
                        let bytes = image.bytes.clone();
                        let byte_count = bytes.len();

                        // 简单哈希：使用图片尺寸和前 100 字节
                        let preview_bytes = bytes.iter().take(100).cloned().collect::<Vec<u8>>();
                        let image_hash = format!("img_{}_{}_{}", width, height,
                            ClipSyncMessage::calculate_hash(&STANDARD.encode(&preview_bytes))
                        );

                        // 检查是否是回声
                        {
                            let cache = hash_cache.lock().unwrap();
                            if cache.contains(&image_hash) {
                                println!("🔁 Echo detected (image hash in cache), skipping sync");
                                continue;
                            }
                        }

                        println!("📸 Image detected: {}x{}, size: {} bytes", width, height, byte_count);

                        // arboard 返回的是 RGBA 原始像素数据，需要编码为 PNG 格式
                        use image::{RgbaImage, ImageEncoder};
                        use image::codecs::png::PngEncoder;

                        let (final_bytes, extension) = if let Some(img_buffer) = RgbaImage::from_raw(
                            width as u32,
                            height as u32,
                            bytes.to_vec()
                        ) {
                            // 编码为 PNG 格式
                            let mut png_bytes = Vec::new();
                            if let Ok(()) = PngEncoder::new(&mut png_bytes).write_image(
                                &img_buffer,
                                width as u32,
                                height as u32,
                                image::ExtendedColorType::Rgba8
                            ) {
                                println!("✓ Image encoded as PNG: {}x{}, {} -> {} bytes",
                                    width, height, bytes.len(), png_bytes.len());
                                (png_bytes, "png")
                            } else {
                                // PNG 编码失败，尝试保存原始数据
                                println!("⚠️ PNG encoding failed, saving raw data");
                                let ext = Self::detect_image_extension(&bytes);
                                (bytes.to_vec(), ext)
                            }
                        } else {
                            // 无法创建图片缓冲区，保存原始数据
                            println!("⚠️ Failed to create image buffer, saving raw data");
                            let ext = Self::detect_image_extension(&bytes);
                            (bytes.to_vec(), ext)
                        };

                        // 生成唯一文件名并保存图片文件
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let random_suffix: u32 = rand::random();
                        let filename = format!("img_{}_{}.{}", timestamp, random_suffix, extension);
                        let relative_path = format!("images/{}", filename);

                        // 获取应用数据目录
                        if let Ok(app_data_dir) = handle.path().app_data_dir() {
                            let images_dir = app_data_dir.join("images");
                            let _ = std::fs::create_dir_all(&images_dir);
                            let file_path = images_dir.join(&filename);

                            // 保存图片文件
                            if let Err(e) = std::fs::write(&file_path, &final_bytes) {
                                eprintln!("❌ Failed to save image file: {}", e);
                            } else {
                                println!("✓ Image saved to: {:?}", file_path);
                            }
                        }

                        // 保存到数据库（存储文件路径）
                        if let Ok(id) = state.db.lock().unwrap().insert_item("image", "", Some(&relative_path)) {
                            // 发送事件到前端
                            let item = ClipboardItem {
                                id,
                                content_type: "image".to_string(),
                                content: String::new(), // 图片不再存储 base64
                                file_path: Some(relative_path.clone()),
                                created_at: Utc::now().to_rfc3339(),
                                favorite: false,
                            };

                            if let Some(window) = handle.get_webview_window("main") {
                                let _ = window.emit("clipboard-update", item);
                            }
                        }

                        // 添加哈希到缓存
                        {
                            let mut cache = hash_cache.lock().unwrap();
                            cache.push_back(image_hash);
                            while cache.len() > HASH_CACHE_SIZE {
                                cache.pop_front();
                            }
                        }

                        // 同步到信任设备（使用 base64 编码的图片数据）
                        let base64_for_sync = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &final_bytes);
                        Self::sync_image_to_trusted_devices(
                            &handle,
                            &state,
                            &mdns,
                            &client,
                            &device_id,
                            &base64_for_sync,
                        ).await;
                    }
                }
            }
        });
    }

    /// 同步图片到所有信任设备
    async fn sync_image_to_trusted_devices(
        _handle: &AppHandle,
        state: &Arc<AppState>,
        mdns: &Arc<Mutex<Option<MdnsDiscovery>>>,
        client: &HttpClient,
        device_id: &str,
        base64_image: &str,
    ) {
        // 获取所有信任设备
        let trusted_devices = {
            let trusted = state.trusted_devices.lock().unwrap();
            trusted.get_all_devices()
        };

        if trusted_devices.is_empty() {
            println!("📭 No trusted devices to sync image");
            return;
        }

        // 获取已发现的设备
        let discovered_devices = {
            let mdns_guard = mdns.lock().unwrap();
            if let Some(ref mdns_obj) = *mdns_guard {
                mdns_obj.get_devices()
            } else {
                Vec::new()
            }
        };

        println!("📸 Syncing image to {} trusted device(s)", trusted_devices.len());

        // 为每个信任设备发送图片数据
        for trusted_device in &trusted_devices {
            if let Some(discovered) = discovered_devices.iter().find(|d| {
                d.hostname.contains(&trusted_device.device_id) ||
                d.name.contains(&trusted_device.device_id)
            }) {
                println!("📤 Syncing image to device: {} @ {}:{}",
                    trusted_device.device_name,
                    discovered.ip,
                    discovered.port
                );

                // 创建图片消息
                let image_message = ClipSyncMessage::new(
                    ClipType::Image,
                    device_id.to_string(),
                    base64_image.to_string(),
                );

                // 发送同步数据
                match client.send_clipboard_data(
                    discovered.ip.clone(),
                    discovered.port,
                    image_message,
                ).await {
                    Ok(response) => {
                        println!("✓ Image sync success: {}", response.status);
                    }
                    Err(e) => {
                        println!("✗ Image sync failed: {}", e);
                    }
                }
            } else {
                println!("⚠️ Trusted device {} not found in discovered devices",
                    trusted_device.device_name);
            }
        }
    }

    /// 同步剪贴板内容到所有信任设备
    async fn sync_to_trusted_devices(
        _handle: &AppHandle,
        state: &Arc<AppState>,
        mdns: &Arc<Mutex<Option<MdnsDiscovery>>>,
        client: &HttpClient,
        device_id: &str,
        content: String,
        hash_cache: &Arc<Mutex<VecDeque<String>>>,
    ) {
        // 计算内容哈希
        let hash = ClipSyncMessage::calculate_hash(&content);

        // 检查哈希是否已在缓存中（防止循环同步）
        {
            let cache = hash_cache.lock().unwrap();
            if cache.contains(&hash) {
                println!("🔁 Hash already in cache, skipping sync");
                return;
            }
        }

        // 获取所有信任设备
        let trusted_devices = {
            let trusted = state.trusted_devices.lock().unwrap();
            trusted.get_all_devices()
        };

        if trusted_devices.is_empty() {
            println!("📭 No trusted devices to sync");
            return;
        }

        // 获取已发现的设备
        let discovered_devices = {
            let mdns_guard = mdns.lock().unwrap();
            if let Some(ref mdns_obj) = *mdns_guard {
                mdns_obj.get_devices()
            } else {
                Vec::new()
            }
        };

        println!("📊 Syncing to {} trusted device(s)", trusted_devices.len());

        // 为每个信任设备发送同步数据
        for trusted_device in &trusted_devices {
            // 查找设备在已发现列表中的地址
            if let Some(discovered) = discovered_devices.iter().find(|d| {
                // 通过主机名匹配（这里简化处理，实际应该通过设备 ID 匹配）
                d.hostname.contains(&trusted_device.device_id) ||
                d.name.contains(&trusted_device.device_id)
            }) {
                println!("📤 Syncing to device: {} @ {}:{}",
                    trusted_device.device_name,
                    discovered.ip,
                    discovered.port
                );

                // 发送同步数据
                match client.send_sync_data(
                    discovered.ip.clone(),
                    discovered.port,
                    device_id.to_string(),
                    vec![], // 暂时使用空的旧格式
                ).await {
                    Ok(response) => {
                        println!("✓ Sync success: {}", response.status);
                    }
                    Err(e) => {
                        println!("✗ Sync failed: {}", e);
                    }
                }
            } else {
                println!("⚠️ Trusted device {} not found in discovered devices",
                    trusted_device.device_name);
            }
        }
    }

    /// 接收远程剪贴板数据
    pub fn receive_clipboard(
        &self,
        handle: &AppHandle,
        state: &AppState,
        message: ClipSyncMessage,
    ) -> Result<(), String> {
        // 验证哈希
        if !message.verify_hash() {
            return Err("Hash verification failed".to_string());
        }

        // 检查是否来自信任设备
        let is_trusted = {
            let trusted = state.trusted_devices.lock().unwrap();
            trusted.is_trusted(&message.source_device)
        };

        if !is_trusted {
            return Err("Device not trusted".to_string());
        }

        // 检查是否已在缓存中（防止重复处理）
        if self.is_hash_cached(&message.hash) {
            println!("🔁 Duplicate message detected (hash already in cache)");
            return Err("Duplicate message".to_string());
        }

        // 将哈希添加到缓存中（在写入剪贴板之前）
        self.add_hash_to_cache(message.hash.clone());

        // 设置为远程更新标志
        {
            let mut local_flag = self.is_local_update.lock().unwrap();
            *local_flag = false;
        }

        // 写入剪贴板
        match Clipboard::new() {
            Ok(mut cb) => {
                // 根据消息类型处理
                if message.msg_type == ClipType::Image {
                    // 图片类型：解码 base64 并设置图片
                    let image_bytes = match STANDARD.decode(&message.content) {
                        Ok(bytes) => bytes,
                        Err(e) => return Err(format!("Failed to decode base64 image: {}", e)),
                    };

                    // 创建 ImageData - arboard 会自动从图片数据中读取宽高
                    let image = arboard::ImageData {
                        bytes: image_bytes.clone().into(),
                        width: 0,
                        height: 0,
                    };

                    // 先尝试直接设置
                    let result = cb.set_image(image);

                    if let Err(ref e) = result {
                        println!("⚠️ 原始图片设置失败: {}, 尝试转换格式", e);

                        // 尝试使用 image crate 转换格式
                        if let Ok(dynamic_img) = image::load_from_memory(&image_bytes) {
                            // 转换为 RGBA 格式
                            let rgba_img = dynamic_img.to_rgba8();
                            let (width, height) = rgba_img.dimensions();

                            let converted_image = arboard::ImageData {
                                bytes: rgba_img.into_raw().into(),
                                width: width as usize,
                                height: height as usize,
                            };

                            if let Err(e) = cb.set_image(converted_image) {
                                return Err(format!("Failed to set converted image clipboard: {}", e));
                            }

                            println!("✓ Image clipboard updated (converted) from device: {} ({}x{})",
                                message.source_device, width, height);
                        } else {
                            return Err(format!("Failed to load image from bytes: {}", e));
                        }
                    } else {
                        println!("✓ Image clipboard updated from device: {}", message.source_device);
                    }
                } else {
                    // 文本类型
                    if let Err(e) = cb.set_text(&message.content) {
                        return Err(format!("Failed to set clipboard: {}", e));
                    }
                    println!("✓ Clipboard updated from device: {}", message.source_device);
                }

                // 保存到数据库
                let (content, file_path) = if message.msg_type == ClipType::Image {
                    // 图片类型：解码 base64 并保存到文件
                    use base64::{Engine as _, engine::general_purpose::STANDARD};
                    if let Ok(image_bytes) = STANDARD.decode(&message.content) {
                        // 检测图片格式
                        let extension = Self::detect_image_extension(&image_bytes);
                        
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let random_suffix: u32 = rand::random();
                        let filename = format!("img_{}_{}.{}", timestamp, random_suffix, extension);
                        let relative_path = format!("images/{}", filename);

                        // 获取应用数据目录
                        if let Ok(app_data_dir) = handle.path().app_data_dir() {
                            let images_dir = app_data_dir.join("images");
                            let _ = std::fs::create_dir_all(&images_dir);
                            let file_path_full = images_dir.join(&filename);

                            // 保存图片文件
                            if let Err(e) = std::fs::write(&file_path_full, &image_bytes) {
                                eprintln!("❌ Failed to save remote image file: {}", e);
                            }
                        }

                        (String::new(), Some(relative_path))
                    } else {
                        (message.content.clone(), None)
                    }
                } else {
                    (message.content.clone(), None)
                };

                if let Ok(id) = state.db.lock().unwrap()
                    .insert_item(message.msg_type.as_str(), &content, file_path.as_deref()) {
                    // 发送事件到前端
                    let item = ClipboardItem {
                        id,
                        content_type: message.msg_type.as_str().to_string(),
                        content,
                        file_path,
                        created_at: Utc::now().to_rfc3339(),
                        favorite: false,
                    };

                    if let Some(window) = handle.get_webview_window("main") {
                        let _ = window.emit("clipboard-remote-update", item);
                    }
                }

                // 更新信任设备的最后在线时间
                if let Ok(mut trusted) = state.trusted_devices.lock() {
                    let _ = trusted.update_last_seen(&message.source_device);
                }

                Ok(())
            }
            Err(e) => Err(format!("Failed to initialize clipboard: {}", e))
        }
    }

    /// 手动触发同步
    pub async fn force_sync(
        &self,
        handle: &AppHandle,
        state: &Arc<AppState>,
        mdns: &Arc<Mutex<Option<MdnsDiscovery>>>,
        content: String,
    ) {
        let client = HttpClient::new();

        Self::sync_to_trusted_devices(
            handle,
            state,
            mdns,
            &client,
            &self.device_id,
            content,
            &self.hash_cache,
        ).await;
    }
}

impl Default for ClipboardSyncManager {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}
