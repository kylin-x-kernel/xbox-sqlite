//! BlackBox - 服务器监控数据管理系统库
//! 
//! 提供高性能的服务器监控数据管理功能，支持智能数据插入、复杂查询分析和数据库管理。

pub mod schema;
pub mod models;
pub mod database;
pub mod services;

use anyhow::Result;
use std::fs;

pub use models::*;
pub use database::*;
pub use services::*;

/// 智能数据插入类型
#[derive(Debug, Clone)]
pub enum SmartDataType {
    /// 服务器信息 (已存在则更新状态)
    Servers,
    /// 系统指标数据 (按时间戳智能更新/插入)
    SystemMetrics,
    /// 进程信息 (按用户名和进程名智能处理，包含趋势和线程)
    Processes,
    /// 崩溃日志 (按时间戳智能更新/插入)
    CrashLogs,
    /// 组合数据 (同时插入进程和系统指标数据)
    Combined,
}

/// BlackBox 核心库结构
pub struct BlackBox {
    db_manager: DatabaseManager,
}

impl BlackBox {
    /// 创建新的 BlackBox 实例
    /// 
    /// # 参数
    /// * `db_path` - 数据库文件路径，None 则使用默认路径
    /// 
    /// # 示例
    /// ```rust
    /// use blackbox::BlackBox;
    /// 
    /// // 使用默认数据库路径
    /// let blackbox = BlackBox::new(None);
    /// 
    /// // 使用指定数据库路径
    /// let blackbox = BlackBox::new(Some("monitoring.db".to_string()));
    /// ```
    pub fn new(db_path: Option<String>) -> Self {
        Self { 
            db_manager: DatabaseManager::new(db_path)
        }
    }

    /// 获取当前数据库路径
    pub fn get_db_path(&self) -> &Option<String> {
        self.db_manager.get_db_path()
    }

    /// 设置数据库路径
    pub fn set_db_path(&mut self, db_path: Option<String>) {
        self.db_manager = DatabaseManager::new(db_path);
    }
    /// 初始化数据库
    /// 
    /// # 参数
    /// * `force` - 是否强制重新创建数据库
    /// 
    /// # 示例
    /// ```rust
    /// use blackbox::BlackBox;
    /// 
    /// // 初始化默认数据库
    /// let blackbox = BlackBox::new(None);
    /// blackbox.init_database(false)?;
    /// 
    /// // 初始化指定数据库
    /// let blackbox = BlackBox::new(Some("monitoring.db".to_string()));
    /// blackbox.init_database(false)?;
    /// 
    /// // 强制重新创建数据库
    /// let blackbox = BlackBox::new(Some("test.db".to_string()));
    /// blackbox.init_database(true)?;
    /// ```
    pub fn init_database(&self, force: bool) -> Result<()> {
        DatabaseInitService::init_database(&self.db_manager, force)
    }

    /// 智能插入数据
    /// 
    /// # 参数
    /// * `data_type` - 数据类型
    /// * `json_data` - JSON 格式的数据字符串
    /// * `continue_on_error` - 遇到错误时是否继续处理
    /// 
    /// # 示例
    /// ```rust
    /// use blackbox::{BlackBox, SmartDataType};
    /// 
    /// let blackbox = BlackBox::new(Some("test.db".to_string()));
    /// let json_data = r#"[{"serverId":"srv-01","serverName":"测试服务器","serverIp":"192.168.1.100","serverOs":"Ubuntu 22.04","serverStatus":"running"}]"#;
    /// let result = blackbox.smart_insert(SmartDataType::Servers, json_data, false)?;
    /// ```
    pub fn smart_insert(
        &self,
        data_type: SmartDataType, 
        json_data: &str, 
        continue_on_error: bool
    ) -> Result<InsertResult> {
        let mut conn = self.db_manager.get_connection()?;
        
        match data_type {
            SmartDataType::Servers => {
                let servers: Vec<NewServer> = serde_json::from_str(json_data)?;
                SmartInsertService::insert_servers(&mut conn, servers, continue_on_error)
            }
            SmartDataType::SystemMetrics => {
                let metrics: Vec<SmartSystemMetric> = serde_json::from_str(json_data)?;
                SmartInsertService::insert_system_metrics(&mut conn, metrics, continue_on_error)
            }
            SmartDataType::Processes => {
                let processes: Vec<SmartProcessInsert> = serde_json::from_str(json_data)?;
                SmartInsertService::insert_processes(&mut conn, processes, continue_on_error)
            }
            SmartDataType::CrashLogs => {
                let crash_logs: Vec<SmartCrashLog> = serde_json::from_str(json_data)?;
                SmartInsertService::insert_crash_logs(&mut conn, crash_logs, continue_on_error)
            }
            SmartDataType::Combined => {
                let combined_data: CombinedInsertData = serde_json::from_str(json_data)?;
                SmartInsertService::insert_combined_data(&mut conn, combined_data, continue_on_error)
            }
        }
    }

    /// 从文件智能插入数据
    /// 
    /// # 参数
    /// * `data_type` - 数据类型
    /// * `file_path` - JSON 文件路径
    /// * `continue_on_error` - 遇到错误时是否继续处理
    pub fn smart_insert_from_file(
        &self,
        data_type: SmartDataType, 
        file_path: &str, 
        continue_on_error: bool
    ) -> Result<InsertResult> {
        let json_content = fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("无法读取文件 {}: {}", file_path, e))?;
        
        self.smart_insert(data_type, &json_content, continue_on_error)
    }

    /// 导入 JSON 数据到数据库
    /// 
    /// # 参数
    /// * `file_path` - JSON 文件路径
    /// * `clean` - 是否清空现有数据
    pub fn import_json_data(&self, file_path: &str, clean: bool) -> Result<()> {
        let mut conn = self.db_manager.get_connection()?;
        
        if clean {
            DataCleanService::clean_database(&mut conn)?;
        }
        
        let json_content = fs::read_to_string(file_path)?;
        let json_data: JsonData = serde_json::from_str(&json_content)?;
        
        JsonImportService::import_json_data(&mut conn, json_data)?;
        Ok(())
    }

    /// 导出数据到 JSON 文件
    /// 
    /// # 参数
    /// * `output_path` - 输出文件路径
    /// * `pretty` - 是否格式化输出
    pub fn export_to_json(&self, output_path: &str, pretty: bool) -> Result<()> {
        let mut conn = self.db_manager.get_connection()?;
        
        let export_data = export_all_data(&mut conn)?;
        
        let json_content = if pretty {
            serde_json::to_string_pretty(&export_data)?
        } else {
            serde_json::to_string(&export_data)?
        };
        
        fs::write(output_path, json_content)?;
        Ok(())
    }

    /// 查询数据库统计信息
    /// 
    /// # 返回
    /// 返回数据库统计信息
    pub fn get_statistics(&self) -> Result<DatabaseStats> {
        let mut conn = self.db_manager.get_connection()?;
        
        let servers = get_all_servers(&mut conn)?;
        let mut stats = DatabaseStats {
            server_count: servers.len(),
            servers: Vec::new(),
        };
        
        for server in servers {
            let metrics = get_metrics_by_server(&mut conn, &server.server_id, None)?;
            let processes = get_processes_by_server(&mut conn, &server.server_id)?;
            let crashes = get_crash_logs_by_server(&mut conn, &server.server_id)?;
            
            let server_stat = ServerStats {
                server: server,
                metrics_count: metrics.len(),
                processes_count: processes.len(),
                crashes_count: crashes.len(),
                latest_metric_time: metrics.first().map(|m| m.timestamp),
            };
            
            stats.servers.push(server_stat);
        }
        
        Ok(stats)
    }

    /// 查询服务器详细信息
    /// 
    /// # 参数
    /// * `server_filter` - 服务器过滤条件（ID 或名称）
    /// * `limit` - 限制返回的记录数
    pub fn query_servers(
        &self,
        server_filter: Option<&str>, 
        limit: Option<i64>
    ) -> Result<Vec<ServerDetail>> {
        let mut conn = self.db_manager.get_connection()?;
        
        let servers = get_all_servers(&mut conn)?;
        
        // 根据过滤条件选择服务器
        let target_servers: Vec<_> = if let Some(filter) = server_filter {
            servers.into_iter()
                .filter(|s| s.server_id == filter || s.server_name.contains(filter))
                .collect()
        } else {
            servers
        };
        
        let mut results = Vec::new();
        
        for server in target_servers {
            let display_limit = limit.unwrap_or(5);
            let metrics = get_metrics_by_server(&mut conn, &server.server_id, Some(display_limit))?;
            let processes = get_processes_by_server(&mut conn, &server.server_id)?;
            let crashes = get_crash_logs_by_server(&mut conn, &server.server_id)?;
            
            let mut process_details = Vec::new();
            for process in &processes {
                let trends = get_process_trends(&mut conn, &server.server_id, process.pid)?;
                let threads = get_threads_by_process(&mut conn, &server.server_id, process.pid)?;
                
                process_details.push(ProcessDetail {
                    process: process.clone(),
                    trends,
                    threads,
                });
            }
            
            let mut crash_details = Vec::new();
            for crash in &crashes {
                let recommendations = get_recommendations_by_crash_log(&mut conn, crash.id)?;
                crash_details.push(CrashDetail {
                    crash_log: crash.clone(),
                    recommendations,
                });
            }
            
            results.push(ServerDetail {
                server,
                metrics,
                processes: process_details,
                crashes: crash_details,
            });
        }
        
        Ok(results)
    }

    /// 清理旧数据
    /// 
    /// # 参数
    /// * `days` - 保留最近 N 天的数据
    pub fn clean_old_data(&self, days: i64) -> Result<usize> {
        let mut conn = self.db_manager.get_connection()?;
        
        let cutoff_time = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);
        let deleted = delete_old_metrics(&mut conn, cutoff_time)?;
        
        Ok(deleted)
    }

}

/// 数据库统计信息
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub server_count: usize,
    pub servers: Vec<ServerStats>,
}

/// 服务器统计信息
#[derive(Debug, Clone)]
pub struct ServerStats {
    pub server: Server,
    pub metrics_count: usize,
    pub processes_count: usize,
    pub crashes_count: usize,
    pub latest_metric_time: Option<i64>,
}

/// 服务器详细信息
#[derive(Debug, Clone)]
pub struct ServerDetail {
    pub server: Server,
    pub metrics: Vec<SystemMetric>,
    pub processes: Vec<ProcessDetail>,
    pub crashes: Vec<CrashDetail>,
}

/// 进程详细信息
#[derive(Debug, Clone)]
pub struct ProcessDetail {
    pub process: Process,
    pub trends: Vec<ProcessTrend>,
    pub threads: Vec<Thread>,
}

/// 崩溃详细信息
#[derive(Debug, Clone)]
pub struct CrashDetail {
    pub crash_log: CrashLog,
    pub recommendations: Vec<AiRecommendation>,
}