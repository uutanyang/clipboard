//! 配对管理模块
//! 负责设备配对请求、状态管理和超时处理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 配对状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PairingStatus {
    Pending,    // 等待对方确认
    Accepted,   // 对方已接受
    Rejected,   // 对方已拒绝
    Cancelled,  // 已取消
    Timeout,    // 超时
    Failed,     // 失败
}

/// 配对请求信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairRequest {
    pub device_id: String,
    pub device_name: String,
}

/// 配对状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingState {
    pub target_device_id: String,
    pub target_device_name: String,
    pub status: PairingStatus,
    pub requested_at: String,
    pub updated_at: String,
}

/// 配对管理器
pub struct PairingManager {
    /// 存储配对请求状态: target_device_id -> PairingState
    pairings: Arc<RwLock<HashMap<String, PairingState>>>,
}

impl PairingManager {
    /// 创建新的配对管理器
    pub fn new() -> Self {
        PairingManager {
            pairings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 发起配对请求
    ///
    /// # Arguments
    /// * `target_device_id` - 目标设备 ID
    /// * `target_device_name` - 目标设备名称
    ///
    /// # Returns
    /// 配对状态信息
    pub async fn initiate_pairing(
        &self,
        target_device_id: String,
        target_device_name: String,
    ) -> Result<PairingState, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let pairing_state = PairingState {
            target_device_id: target_device_id.clone(),
            target_device_name,
            status: PairingStatus::Pending,
            requested_at: now.clone(),
            updated_at: now,
        };

        // 存储配对状态
        {
            let mut pairings = self.pairings.write().await;
            pairings.insert(target_device_id.clone(), pairing_state.clone());
        }

        println!("🔗 Pairing request sent to device: {}", target_device_id);

        // 启动超时检查任务（10秒超时）
        self.start_timeout_check(target_device_id.clone()).await;

        Ok(pairing_state)
    }

    /// 取消配对请求
    ///
    /// # Arguments
    /// * `target_device_id` - 目标设备 ID
    ///
    /// # Returns
    /// 是否成功取消
    pub async fn cancel_pairing(&self, target_device_id: String) -> Result<bool, String> {
        let mut pairings = self.pairings.write().await;

        if let Some(state) = pairings.get_mut(&target_device_id) {
            if matches!(state.status, PairingStatus::Pending) {
                state.status = PairingStatus::Cancelled;
                state.updated_at = chrono::Utc::now().to_rfc3339();
                println!("🚫 Pairing request cancelled for device: {}", target_device_id);
                return Ok(true);
            } else {
                return Err(format!("Cannot cancel pairing with status: {:?}", state.status));
            }
        }

        Err("Pairing request not found".to_string())
    }

    /// 更新配对状态
    ///
    /// # Arguments
    /// * `target_device_id` - 目标设备 ID
    /// * `status` - 新的配对状态
    ///
    /// # Returns
    /// 是否成功更新
    pub async fn update_pairing_status(
        &self,
        target_device_id: String,
        new_status: PairingStatus,
    ) -> Result<(), String> {
        let mut pairings = self.pairings.write().await;

        if let Some(state) = pairings.get_mut(&target_device_id) {
            state.status = new_status;
            state.updated_at = chrono::Utc::now().to_rfc3339();
            println!("✓ Pairing status updated for device {:?}", target_device_id);
            Ok(())
        } else {
            Err("Pairing request not found".to_string())
        }
    }

    /// 获取配对状态
    ///
    /// # Arguments
    /// * `target_device_id` - 目标设备 ID
    ///
    /// # Returns
    /// 配对状态信息，如果不存在则返回 None
    pub async fn get_pairing_status(&self, target_device_id: &str) -> Option<PairingState> {
        let pairings = self.pairings.read().await;
        pairings.get(target_device_id).cloned()
    }

    /// 获取所有配对状态
    pub async fn get_all_pairings(&self) -> Vec<PairingState> {
        let pairings = self.pairings.read().await;
        pairings.values().cloned().collect()
    }

    /// 清理已完成的配对请求
    ///
    /// 移除状态为 Accepted, Rejected, Cancelled, Timeout, Failed 的记录
    pub async fn cleanup_completed_pairings(&self) {
        let mut pairings = self.pairings.write().await;
        let mut to_remove = Vec::new();

        for (device_id, state) in pairings.iter() {
            if matches!(
                state.status,
                PairingStatus::Accepted
                    | PairingStatus::Rejected
                    | PairingStatus::Cancelled
                    | PairingStatus::Timeout
                    | PairingStatus::Failed
            ) {
                to_remove.push(device_id.clone());
            }
        }

        for device_id in to_remove {
            pairings.remove(&device_id);
            println!("🧹 Cleaned up pairing for device: {}", device_id);
        }
    }

    /// 启动超时检查任务
    ///
    /// 10秒后自动将 Pending 状态改为 Timeout
    async fn start_timeout_check(&self, target_device_id: String) {
        let pairings = self.pairings.clone();

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

            let mut pairings = pairings.write().await;
            if let Some(state) = pairings.get_mut(&target_device_id) {
                if matches!(state.status, PairingStatus::Pending) {
                    state.status = PairingStatus::Timeout;
                    state.updated_at = chrono::Utc::now().to_rfc3339();
                    println!("⏰ Pairing timeout for device: {}", target_device_id);
                }
            }
        });
    }
}

impl Default for PairingManager {
    fn default() -> Self {
        Self::new()
    }
}
