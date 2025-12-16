//! 服务层 - 封装业务逻辑和公共操作

use anyhow::Result;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::fs;

use crate::database::*;
use crate::models::*;
const THREAD_EXCEPTION_THRESHOLD: i32 = 2000;

/// 插入操作结果
#[derive(Debug, Clone)]
pub struct InsertResult {
    pub success_count: usize,
    pub updated_count: usize,
    pub error_count: usize,
}

impl InsertResult {
    pub fn new() -> Self {
        Self {
            success_count: 0,
            updated_count: 0,
            error_count: 0,
        }
    }

    pub fn add_success(&mut self) {
        self.success_count += 1;
    }

    pub fn add_updated(&mut self) {
        self.updated_count += 1;
    }

    pub fn add_error(&mut self) {
        self.error_count += 1;
    }

    pub fn merge(&mut self, other: InsertResult) {
        self.success_count += other.success_count;
        self.updated_count += other.updated_count;
        self.error_count += other.error_count;
    }
}

/// 数据库连接管理器
pub struct DatabaseManager {
    db_path: Option<String>,
}

impl DatabaseManager {
    pub fn new(db_path: Option<String>) -> Self {
        Self { db_path }
    }

    /// 获取数据库路径
    pub fn get_db_path(&self) -> &Option<String> {
        &self.db_path
    }

    /// 获取数据库连接
    pub fn get_connection(&self) -> Result<SqliteConnection> {
        let db_url = self.build_database_url();
        establish_connection_with_url(Some(&db_url))
    }

    /// 构建数据库 URL
    fn build_database_url(&self) -> String {
        if let Some(path) = &self.db_path {
            if path.starts_with("sqlite://") {
                path.clone()
            } else {
                format!("sqlite://{}", path)
            }
        } else {
            "sqlite://./database.db".to_string()
        }
    }

    /// 获取数据库路径（用于连接）
    pub fn get_db_path_for_connection(&self) -> Option<String> {
        self.db_path.as_ref().map(|path| {
            if path.starts_with("sqlite://") {
                path.clone()
            } else {
                format!("sqlite://{}", path)
            }
        })
    }
}

/// 数据库初始化服务
pub struct DatabaseInitService;

impl DatabaseInitService {
    /// 初始化数据库
    pub fn init_database(db_manager: &DatabaseManager, force: bool) -> Result<()> {
        let database_url = db_manager.build_database_url();

        // 提取文件路径
        let file_path = database_url
            .strip_prefix("sqlite://")
            .unwrap_or(&database_url);

        // 检查文件是否已存在
        if std::path::Path::new(file_path).exists() {
            if !force {
                return Err(anyhow::anyhow!(
                    "数据库文件已存在: {}，使用 force=true 强制重新创建",
                    file_path
                ));
            } else {
                fs::remove_file(file_path)?;
            }
        }

        // 创建数据库连接（这会自动创建文件）
        let mut conn = db_manager.get_connection()?;

        // 执行建表 SQL
        Self::create_tables(&mut conn)?;
        Self::create_indexes(&mut conn)?;

        Ok(())
    }

    fn create_tables(conn: &mut SqliteConnection) -> Result<()> {
        use diesel::sql_query;

        // 创建 servers 表
        sql_query(
            r#"
            CREATE TABLE servers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL UNIQUE,
                server_name TEXT NOT NULL,
                server_ip TEXT NOT NULL,
                server_os TEXT NOT NULL,
                server_status TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#,
        )
        .execute(conn)?;

        // 创建 system_metrics 表
        sql_query(
            r#"
            CREATE TABLE system_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                timestamp BIGINT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage REAL NOT NULL,
                disk_usage REAL NOT NULL,
                io_read REAL NOT NULL,
                io_write REAL NOT NULL,
                network_in REAL NOT NULL,
                network_out REAL NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(server_id)
            )
        "#,
        )
        .execute(conn)?;

        // 创建 processes 表
        sql_query(
            r#"
            CREATE TABLE processes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                pid INTEGER NOT NULL,
                name TEXT NOT NULL,
                user_name TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(server_id)
            )
        "#,
        )
        .execute(conn)?;

        // 创建 process_trends 表
        sql_query(
            r#"
            CREATE TABLE process_trends (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                pid INTEGER NOT NULL,
                timestamp BIGINT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage REAL NOT NULL,
                thread_count INTEGER NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(server_id)
            )
        "#,
        )
        .execute(conn)?;

        // 创建 threads 表
        sql_query(
            r#"
            CREATE TABLE threads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                pid INTEGER NOT NULL,
                thread_id INTEGER NOT NULL,
                user_name TEXT NOT NULL,
                priority INTEGER NOT NULL,
                nice_value INTEGER NOT NULL,
                virtual_memory TEXT NOT NULL,
                resident_memory TEXT NOT NULL,
                shared_memory TEXT NOT NULL,
                status TEXT NOT NULL,
                cpu_usage TEXT NOT NULL,
                memory_usage TEXT NOT NULL,
                runtime TEXT NOT NULL,
                command TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(server_id)
            )
        "#,
        )
        .execute(conn)?;

        // 创建 crash_logs 表
        sql_query(
            r#"
            CREATE TABLE crash_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                server_id TEXT NOT NULL,
                log_id BIGINT NOT NULL,
                timestamp BIGINT NOT NULL,
                crash_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                stack_trace TEXT,
                resolved BOOLEAN NOT NULL DEFAULT 0,
                ai_summary TEXT,
                ai_analysis TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (server_id) REFERENCES servers(server_id)
            )
        "#,
        )
        .execute(conn)?;

        // 创建 ai_recommendations 表
        sql_query(
            r#"
            CREATE TABLE ai_recommendations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crash_log_id INTEGER NOT NULL,
                priority INTEGER NOT NULL,
                action TEXT NOT NULL,
                command TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (crash_log_id) REFERENCES crash_logs(id)
            )
        "#,
        )
        .execute(conn)?;

        Ok(())
    }

    fn create_indexes(conn: &mut SqliteConnection) -> Result<()> {
        use diesel::sql_query;

        sql_query("CREATE INDEX idx_servers_server_id ON servers(server_id)").execute(conn)?;
        sql_query("CREATE INDEX idx_system_metrics_server_timestamp ON system_metrics(server_id, timestamp)").execute(conn)?;
        sql_query(
            "CREATE INDEX idx_processes_server_name_user ON processes(server_id, name, user_name)",
        )
        .execute(conn)?;
        sql_query("CREATE INDEX idx_process_trends_server_pid ON process_trends(server_id, pid)")
            .execute(conn)?;
        sql_query("CREATE INDEX idx_threads_server_pid ON threads(server_id, pid)")
            .execute(conn)?;
        sql_query(
            "CREATE INDEX idx_crash_logs_server_timestamp ON crash_logs(server_id, timestamp)",
        )
        .execute(conn)?;
        sql_query(
            "CREATE INDEX idx_ai_recommendations_crash_log ON ai_recommendations(crash_log_id)",
        )
        .execute(conn)?;

        Ok(())
    }
}

/// 智能插入服务
pub struct SmartInsertService;

impl SmartInsertService {
    /// 智能插入服务器数据
    pub fn insert_servers(
        conn: &mut SqliteConnection,
        servers: Vec<NewServer>,
        continue_on_error: bool,
    ) -> Result<InsertResult> {
        // 插入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        let mut result = InsertResult::new();

        for server in servers {
            match Self::handle_server_insert(conn, server) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 智能插入系统指标数据
    pub fn insert_system_metrics(
        conn: &mut SqliteConnection,
        metrics: Vec<SmartSystemMetric>,
        continue_on_error: bool,
    ) -> Result<InsertResult> {
        // 插入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        let mut result = InsertResult::new();

        for metric in metrics {
            // 验证服务器是否存在
            if get_server_by_id(conn, &metric.server_id)?.is_none() {
                result.add_error();
                if !continue_on_error {
                    return Err(anyhow::anyhow!("服务器 {} 不存在", metric.server_id));
                }
                continue;
            }

            match Self::handle_metric_insert(conn, metric) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 智能插入进程数据
    pub fn insert_processes(
        conn: &mut SqliteConnection,
        processes: Vec<SmartProcessInsert>,
        continue_on_error: bool,
    ) -> Result<InsertResult> {
        // 插入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        let mut result = InsertResult::new();

        for process_data in processes {
            match Self::handle_process_insert(conn, process_data, continue_on_error) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 智能插入崩溃日志数据
    pub fn insert_crash_logs(
        conn: &mut SqliteConnection,
        crash_logs: Vec<SmartCrashLog>,
        continue_on_error: bool,
    ) -> Result<InsertResult> {
        // 插入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        let mut result = InsertResult::new();

        for log_data in crash_logs {
            // 验证服务器是否存在
            if get_server_by_id(conn, &log_data.server_id)?.is_none() {
                result.add_error();
                if !continue_on_error {
                    return Err(anyhow::anyhow!("服务器 {} 不存在", log_data.server_id));
                }
                continue;
            }

            match Self::handle_crash_log_insert(conn, log_data) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        Ok(result)
    }

    /// 智能插入组合数据
    pub fn insert_combined_data(
        conn: &mut SqliteConnection,
        combined_data: CombinedInsertData,
        continue_on_error: bool,
    ) -> Result<InsertResult> {
        // 插入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        let mut result = InsertResult::new();

        // 先获取第一个进程的服务器ID，用于后续的崩溃日志处理
        let first_server_id = combined_data.process.first().map(|p| p.server_id.clone());

        // 先处理进程数据以确保服务器存在，然后检测线程数异常
        for process_data in &combined_data.process {
            // 先确保服务器存在
            match get_server_by_id(conn, &process_data.server_id)? {
                Some(_) => {
                    // 服务器存在，更新状态
                    let _ = update_server_status(
                        conn,
                        &process_data.server_id,
                        &process_data.server_status,
                    );
                }
                None => {
                    // 服务器不存在，创建新服务器
                    let new_server = NewServer {
                        server_id: process_data.server_id.clone(),
                        server_name: process_data.server_name.clone(),
                        server_ip: process_data.server_ip.clone(),
                        server_os: process_data.server_os.clone(),
                        server_status: process_data.server_status.clone(),
                    };
                    let _ = create_server(conn, &new_server);
                }
            }

            // 检测线程数异常
            if Self::has_thread_exception(&process_data) {
                println!(
                    "检测到线程数异常，进程 PID={} NAME={} 线程数={}",
                    process_data.pid,
                    process_data.name,
                    process_data.trend.last().map_or(0, |t| t.thread_count)
                );
                match Self::handle_thread_exception_crash_log(conn, &process_data) {
                    Ok(_) => {
                        result.add_success();
                    }
                    Err(e) => {
                        result.add_error();
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }

        // 处理进程数据（包含服务器信息）
        for process_data in combined_data.process {
            match Self::handle_combined_process_insert(conn, process_data, continue_on_error) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        // 处理系统指标数据
        for metric in combined_data.metrics {
            // 验证服务器是否存在
            if get_server_by_id(conn, &metric.server_id)?.is_none() {
                result.add_error();
                if !continue_on_error {
                    return Err(anyhow::anyhow!("服务器 {} 不存在", metric.server_id));
                }
                continue;
            }

            match Self::handle_metric_insert(conn, metric) {
                Ok(is_update) => {
                    if is_update {
                        result.add_updated();
                    } else {
                        result.add_success();
                    }
                }
                Err(e) => {
                    result.add_error();
                    if !continue_on_error {
                        return Err(e);
                    }
                }
            }
        }

        // 处理 dmesg 数据，检测系统崩溃信息
        if let Some(dmesg_content) = combined_data.dmesg {
            if Self::is_system_crash(&dmesg_content) {
                // 使用之前保存的服务器ID
                if let Some(server_id) = first_server_id {
                    match Self::handle_crash_log_from_dmesg(conn, &server_id, &dmesg_content) {
                        Ok(_) => {
                            result.add_success();
                        }
                        Err(e) => {
                            result.add_error();
                            if !continue_on_error {
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    // 私有辅助方法
    fn handle_server_insert(conn: &mut SqliteConnection, server: NewServer) -> Result<bool> {
        match get_server_by_id(conn, &server.server_id)? {
            Some(_) => {
                update_server_status(conn, &server.server_id, &server.server_status)?;
                Ok(true) // 是更新操作
            }
            None => {
                create_server(conn, &server)?;
                Ok(false) // 是新建操作
            }
        }
    }

    fn handle_metric_insert(
        conn: &mut SqliteConnection,
        metric: SmartSystemMetric,
    ) -> Result<bool> {
        let new_metric = NewSystemMetric {
            server_id: metric.server_id.clone(),
            timestamp: metric.timestamp,
            cpu_usage: metric.cpu_usage,
            memory_usage: metric.memory_usage,
            disk_usage: metric.disk_usage,
            io_read: metric.io_read,
            io_write: metric.io_write,
            network_in: metric.network_in,
            network_out: metric.network_out,
        };

        match get_system_metric_by_timestamp(conn, &metric.server_id, metric.timestamp)? {
            Some(_) => {
                update_system_metric(conn, &metric.server_id, metric.timestamp, &new_metric)?;
                Ok(true) // 是更新操作
            }
            None => {
                create_system_metric(conn, &new_metric)?;
                Ok(false) // 是新建操作
            }
        }
    }

    fn handle_process_insert(
        conn: &mut SqliteConnection,
        process_data: SmartProcessInsert,
        continue_on_error: bool,
    ) -> Result<bool> {
        // 验证服务器是否存在，如果不存在则尝试自动创建
        Self::ensure_server_exists(conn, &process_data, continue_on_error)?;

        let is_update = match get_process_by_name_and_user(
            conn,
            &process_data.server_id,
            &process_data.name,
            &process_data.user_name,
        )? {
            Some(existing_process) => {
                // 进程已存在，更新状态
                update_process_status(conn, existing_process.id, &process_data.status)?;
                true
            }
            None => {
                // 进程不存在，创建新进程
                let new_process = NewProcess {
                    server_id: process_data.server_id.clone(),
                    pid: process_data.pid,
                    name: process_data.name.clone(),
                    user_name: process_data.user_name.clone(),
                    status: process_data.status.clone(),
                };
                create_process(conn, &new_process)?;
                false
            }
        };

        // 添加趋势数据和线程数据
        Self::add_process_related_data(conn, &process_data)?;

        Ok(is_update)
    }

    fn handle_combined_process_insert(
        conn: &mut SqliteConnection,
        process_data: CombinedProcessData,
        _continue_on_error: bool,
    ) -> Result<bool> {
        // 检查并创建服务器（如果不存在）
        match get_server_by_id(conn, &process_data.server_id)? {
            Some(_) => {
                // 服务器存在，更新状态
                update_server_status(conn, &process_data.server_id, &process_data.server_status)?;
            }
            None => {
                // 服务器不存在，创建新服务器
                let new_server = NewServer {
                    server_id: process_data.server_id.clone(),
                    server_name: process_data.server_name.clone(),
                    server_ip: process_data.server_ip.clone(),
                    server_os: process_data.server_os.clone(),
                    server_status: process_data.server_status.clone(),
                };
                create_server(conn, &new_server)?;
            }
        }

        // 处理进程信息
        let is_update = match get_process_by_name_and_user(
            conn,
            &process_data.server_id,
            &process_data.name,
            &process_data.user_name,
        )? {
            Some(existing_process) => {
                // 进程存在，更新状态
                update_process_status(conn, existing_process.id, &process_data.status)?;
                true
            }
            None => {
                // 进程不存在，创建新进程
                let new_process = NewProcess {
                    server_id: process_data.server_id.clone(),
                    pid: process_data.pid,
                    name: process_data.name.clone(),
                    user_name: process_data.user_name.clone(),
                    status: process_data.status.clone(),
                };
                create_process(conn, &new_process)?;
                false
            }
        };

        // 添加进程趋势数据
        for trend in &process_data.trend {
            let new_trend = NewProcessTrend {
                server_id: process_data.server_id.clone(),
                pid: process_data.pid,
                timestamp: process_data.timestamp,
                cpu_usage: trend.cpu_usage,
                memory_usage: trend.memory_usage,
                thread_count: trend.thread_count,
            };
            let _ = create_process_trend(conn, &new_trend);
        }

        // 删除旧的线程数据并添加新的线程数据
        let _ = delete_threads_by_process(conn, &process_data.server_id, process_data.pid);

        for thread in &process_data.threads {
            let new_thread = NewThread {
                server_id: process_data.server_id.clone(),
                pid: process_data.pid,
                thread_id: thread.thread_id,
                user_name: thread.user_name.clone(),
                priority: thread.priority,
                nice_value: thread.nice_value,
                virtual_memory: thread.virtual_memory.clone(),
                resident_memory: thread.resident_memory.clone(),
                shared_memory: thread.shared_memory.clone(),
                status: thread.status.clone(),
                cpu_usage: thread.cpu_usage.clone(),
                memory_usage: thread.memory_usage.clone(),
                runtime: thread.runtime.clone(),
                command: thread.command.clone(),
            };
            let _ = create_thread(conn, &new_thread);
        }

        Ok(is_update)
    }

    fn handle_crash_log_insert(
        conn: &mut SqliteConnection,
        log_data: SmartCrashLog,
    ) -> Result<bool> {
        let new_log = NewCrashLog {
            server_id: log_data.server_id.clone(),
            log_id: log_data.log_id,
            timestamp: log_data.timestamp,
            crash_type: log_data.crash_type.clone(),
            severity: log_data.severity.clone(),
            title: log_data.title.clone(),
            message: log_data.message.clone(),
            stack_trace: log_data.stack_trace.clone(),
            resolved: log_data.resolved,
            ai_summary: log_data.ai_summary.clone(),
            ai_analysis: log_data.ai_analysis.clone(),
        };

        match get_crash_log_by_timestamp(conn, &log_data.server_id, log_data.timestamp)? {
            Some(existing_log) => {
                update_crash_log(conn, existing_log.id, &new_log)?;
                Ok(true) // 是更新操作
            }
            None => {
                create_crash_log(conn, &new_log)?;
                Ok(false) // 是新建操作
            }
        }
    }

    fn ensure_server_exists(
        conn: &mut SqliteConnection,
        process_data: &SmartProcessInsert,
        _continue_on_error: bool,
    ) -> Result<()> {
        if get_server_by_id(conn, &process_data.server_id)?.is_none() {
            // 检查是否提供了服务器信息用于自动创建
            if let (Some(server_name), Some(server_ip), Some(server_os), Some(server_status)) = (
                &process_data.server_name,
                &process_data.server_ip,
                &process_data.server_os,
                &process_data.server_status,
            ) {
                let new_server = NewServer {
                    server_id: process_data.server_id.clone(),
                    server_name: server_name.clone(),
                    server_ip: server_ip.clone(),
                    server_os: server_os.clone(),
                    server_status: server_status.clone(),
                };
                create_server(conn, &new_server)?;
            } else {
                return Err(anyhow::anyhow!(
                    "服务器 {} 不存在且未提供服务器信息用于自动创建",
                    process_data.server_id
                ));
            }
        }
        Ok(())
    }

    fn add_process_related_data(
        conn: &mut SqliteConnection,
        process_data: &SmartProcessInsert,
    ) -> Result<()> {
        // 添加趋势数据
        for trend in &process_data.trend {
            let new_trend = NewProcessTrend {
                server_id: process_data.server_id.clone(),
                pid: process_data.pid,
                timestamp: process_data.timestamp,
                cpu_usage: trend.cpu_usage,
                memory_usage: trend.memory_usage,
                thread_count: trend.thread_count,
            };
            let _ = create_process_trend(conn, &new_trend);
        }

        // 删除旧线程数据并添加新的线程数据
        let _ = delete_threads_by_process(conn, &process_data.server_id, process_data.pid);

        for thread in &process_data.threads {
            let new_thread = NewThread {
                server_id: process_data.server_id.clone(),
                pid: process_data.pid,
                thread_id: thread.thread_id,
                user_name: thread.user_name.clone(),
                priority: thread.priority,
                nice_value: thread.nice_value,
                virtual_memory: thread.virtual_memory.clone(),
                resident_memory: thread.resident_memory.clone(),
                shared_memory: thread.shared_memory.clone(),
                status: thread.status.clone(),
                cpu_usage: thread.cpu_usage.clone(),
                memory_usage: thread.memory_usage.clone(),
                runtime: thread.runtime.clone(),
                command: thread.command.clone(),
            };
            let _ = create_thread(conn, &new_thread);
        }

        Ok(())
    }

    /// 检测 dmesg 内容是否包含系统崩溃信息
    fn is_system_crash(dmesg_content: &str) -> bool {
        let crash_indicators = [
            "kernel BUG at",
            "Internal error: Oops",
            "segmentation fault",
            "kernel panic",
            "Call trace:",
            "---[ end trace",
            "BUG:",
            "WARNING:",
        ];

        crash_indicators
            .iter()
            .any(|indicator| dmesg_content.contains(indicator))
    }

    /// 从 dmesg 内容创建崩溃日志
    fn handle_crash_log_from_dmesg(
        conn: &mut SqliteConnection,
        server_id: &str,
        dmesg_content: &str,
    ) -> Result<()> {
        use chrono::Utc;

        let timestamp = Utc::now().timestamp_millis();
        let log_id = timestamp; // 使用时间戳作为 log_id

        let new_crash_log = NewCrashLog {
            server_id: server_id.to_string(),
            log_id,
            timestamp,
            crash_type: "kernel_exception".to_string(),
            severity: "high".to_string(),
            title: "内核异常日志信息".to_string(),
            message: "正在等待 AI 生成".to_string(),
            stack_trace: Some(dmesg_content.to_string()),
            resolved: false,
            ai_summary: Some("正在等待 AI 生成".to_string()),
            ai_analysis: Some("正在等待 AI 生成".to_string()),
        };

        create_crash_log(conn, &new_crash_log)?;
        Ok(())
    }

    /// 检测进程是否有线程数异常
    fn has_thread_exception(process_data: &CombinedProcessData) -> bool {
        // 检查进程趋势中的线程数
        for trend in &process_data.trend {
            if trend.thread_count > THREAD_EXCEPTION_THRESHOLD {
                return true;
            }
        }

        // 检查实际线程数量
        if process_data.threads.len() as i32 > THREAD_EXCEPTION_THRESHOLD {
            return true;
        }

        false
    }

    /// 处理线程数异常，创建崩溃日志
    fn handle_thread_exception_crash_log(
        conn: &mut SqliteConnection,
        process_data: &CombinedProcessData,
    ) -> Result<()> {
        use chrono::Utc;

        // 检查是否已经存在相同进程的线程异常崩溃日志，防止重复添加

        // 检查是否已存在相同的线程异常日志（通过 stack_trace 中的特殊标记来识别）
        if Self::thread_exception_crash_log_exists(conn, &process_data.server_id, process_data.pid)?
        {
            return Ok(()); // 已存在，不重复添加
        }

        let timestamp = Utc::now().timestamp_millis();
        let log_id = timestamp; // 使用时间戳作为 log_id

        // 构建包含进程信息的 stack_trace
        let stack_trace = Self::build_thread_exception_stack_trace(process_data);

        let new_crash_log = NewCrashLog {
            server_id: process_data.server_id.clone(),
            log_id,
            timestamp,
            crash_type: "thread_exception".to_string(),
            severity: "high".to_string(),
            title: "Thread Exception".to_string(),
            message: format!("Thread exception detected in process PID={} NAME={} Count={}", process_data.pid, process_data.name, process_data.trend.last().map_or(0, |t| t.thread_count)),
            stack_trace: Some(stack_trace),
            resolved: false,
            ai_summary: Some("线程数达到上限，建议增加线程限制并重启桌面服务".to_string()),
            ai_analysis: Some("## 🔍 问题分析\n\nUKUI 3.0 桌面环境的 ukui-panel 进程因系统线程数达到上限（4096）而无法创建新的工作线程，导致桌面面板服务崩溃。\n\n### 📊 关键发现\n- **线程限制**: 当前系统线程限制为 4096，已达上限\n- **影响进程**: ukui-panel (PID: 1001) 桌面面板服务\n- **失败原因**: pthread_create 调用失败，资源暂时不可用\n\n---\n\n## 💡 解决方案\n\n### 1. 立即修复 `优先级: P1`\n\n增加系统线程限制：\n\n```bash\n# 临时增加线程限制\necho \"* soft nproc 8192\" >> /etc/security/limits.conf\necho \"* hard nproc 8192\" >> /etc/security/limits.conf\n\n# 重启桌面服务\nsystemctl --user restart ukui-panel.service\n```\n\n### 2. 长期优化 `优先级: P2`\n\n检查并优化 UKUI 桌面环境：\n\n```bash\n# 检查当前线程使用情况\nps -eLf | wc -l\n\n# 监控 ukui-panel 线程数\nwatch -n 1 \"ps -o pid,nlwp,comm -p 1001\"\n```\n\n> ⚠️ **注意**: 修改系统限制后需要重新登录或重启系统才能完全生效。".to_string()),
        };

        create_crash_log(conn, &new_crash_log)?;
        Ok(())
    }

    /// 检查是否已存在相同进程的线程异常崩溃日志
    fn thread_exception_crash_log_exists(
        conn: &mut SqliteConnection,
        target_server_id: &str,
        pid: i32,
    ) -> Result<bool> {
        use crate::schema::crash_logs::dsl::*;

        let process_marker = format!("PROCESS_INFO: PID={}", pid);

        let count: i64 = crash_logs
            .filter(server_id.eq(target_server_id))
            .filter(crash_type.eq("thread_exception"))
            .filter(stack_trace.like(format!("%{}%", process_marker)))
            .count()
            .get_result(conn)?;

        Ok(count > 0)
    }

    /// 构建线程异常的 stack_trace，包含进程信息
    fn build_thread_exception_stack_trace(process_data: &CombinedProcessData) -> String {
        let mut stack_trace = String::new();

        // 添加进程信息标记
        stack_trace.push_str(&format!("THREAD_EXCEPTION_DETECTED\n"));
        stack_trace.push_str(&format!(
            "PROCESS_INFO: PID={}, NAME={}, USER={}\n",
            process_data.pid, process_data.name, process_data.user_name
        ));
        stack_trace.push_str(&format!(
            "SERVER_INFO: ID={}, NAME={}\n",
            process_data.server_id, process_data.server_name
        ));
        stack_trace.push_str(&format!("TIMESTAMP: {}\n\n", process_data.timestamp));

        // 添加线程数统计信息
        stack_trace.push_str("THREAD_COUNT_ANALYSIS:\n");
        stack_trace.push_str(&format!(
            "  Actual threads count: {}\n",
            process_data.threads.len()
        ));

        for (i, trend) in process_data.trend.iter().enumerate() {
            stack_trace.push_str(&format!(
                "  Trend[{}] thread_count: {}\n",
                i, trend.thread_count
            ));
        }

        stack_trace.push_str("\nTHREAD_DETAILS:\n");

        // 添加前10个线程的详细信息
        for (i, thread) in process_data.threads.iter().take(10).enumerate() {
            stack_trace.push_str(&format!(
                "  Thread[{}]: TID={}, CPU={}, MEM={}, CMD={}\n",
                i,
                thread.thread_id,
                thread.cpu_usage,
                thread.memory_usage,
                thread.command.chars().take(50).collect::<String>()
            ));
        }

        if process_data.threads.len() > 10 {
            stack_trace.push_str(&format!(
                "  ... and {} more threads\n",
                process_data.threads.len() - 10
            ));
        }

        stack_trace
            .push_str("\nRECOMMENDATION: Check for thread leaks or infinite thread creation");

        stack_trace
    }
}

/// 数据清理服务
pub struct DataCleanService;

impl DataCleanService {
    /// 清理旧数据（保留最近24小时的数据）
    pub fn cleanup_old_data(conn: &mut SqliteConnection) -> Result<()> {
        use crate::schema::{system_metrics, process_trends, crash_logs, threads, ai_recommendations};
        use diesel::prelude::*;
        use chrono::{Utc, Duration};

        let cutoff_timestamp = Utc::now().timestamp() - 24 * 3600;
        let cutoff_datetime = Utc::now().naive_utc() - Duration::hours(24);

        // 清理系统指标
        diesel::delete(system_metrics::table.filter(system_metrics::timestamp.lt(cutoff_timestamp)))
            .execute(conn)?;

        // 清理进程趋势
        diesel::delete(process_trends::table.filter(process_trends::timestamp.lt(cutoff_timestamp)))
            .execute(conn)?;

        // 清理没有趋势数据的进程（即不活跃超过24小时的进程）
        diesel::sql_query("DELETE FROM processes WHERE NOT EXISTS (SELECT 1 FROM process_trends WHERE process_trends.server_id = processes.server_id AND process_trends.pid = processes.pid)")
            .execute(conn)?;

        // 清理孤儿线程数据（所属进程已被删除）
        diesel::sql_query("DELETE FROM threads WHERE NOT EXISTS (SELECT 1 FROM processes WHERE processes.server_id = threads.server_id AND processes.pid = threads.pid)")
            .execute(conn)?;

        // 清理崩溃日志
        diesel::delete(crash_logs::table.filter(crash_logs::timestamp.lt(cutoff_timestamp)))
            .execute(conn)?;
            
        // 清理线程快照
        diesel::delete(threads::table.filter(threads::created_at.lt(cutoff_datetime)))
            .execute(conn)?;
            
        // 清理 AI 建议
        diesel::delete(ai_recommendations::table.filter(ai_recommendations::created_at.lt(cutoff_datetime)))
            .execute(conn)?;

        Ok(())
    }

    /// 清空数据库
    pub fn clean_database(conn: &mut SqliteConnection) -> Result<()> {
        use crate::schema::*;
        use diesel::prelude::*;

        diesel::delete(ai_recommendations::table).execute(conn)?;
        diesel::delete(crash_logs::table).execute(conn)?;
        diesel::delete(threads::table).execute(conn)?;
        diesel::delete(process_trends::table).execute(conn)?;
        diesel::delete(processes::table).execute(conn)?;
        diesel::delete(system_metrics::table).execute(conn)?;
        diesel::delete(servers::table).execute(conn)?;

        Ok(())
    }
}

/// JSON 数据导入服务
pub struct JsonImportService;

impl JsonImportService {
    /// 导入 JSON 数据
    pub fn import_json_data(conn: &mut SqliteConnection, json_data: JsonData) -> Result<()> {
        // 导入前清理旧数据
        let _ = DataCleanService::cleanup_old_data(conn);

        for json_server in json_data.servers {
            // 检查服务器是否已存在
            match get_server_by_id(conn, &json_server.server_id)? {
                Some(_) => {
                    update_server_status(conn, &json_server.server_id, &json_server.server_status)?;
                }
                None => {
                    let new_server = NewServer {
                        server_id: json_server.server_id.clone(),
                        server_name: json_server.server_name.clone(),
                        server_ip: json_server.server_ip.clone(),
                        server_os: json_server.server_os.clone(),
                        server_status: json_server.server_status.clone(),
                    };
                    create_server(conn, &new_server)?;
                }
            }

            // 导入系统指标数据
            for json_metric in json_server.system_metrics {
                let new_metric = NewSystemMetric {
                    server_id: json_server.server_id.clone(),
                    timestamp: json_metric.timestamp,
                    cpu_usage: json_metric.cpu_usage,
                    memory_usage: json_metric.memory_usage,
                    disk_usage: json_metric.disk_usage,
                    io_read: json_metric.io_read,
                    io_write: json_metric.io_write,
                    network_in: json_metric.network_in,
                    network_out: json_metric.network_out,
                };

                create_system_metric(conn, &new_metric)?;
            }

            // 导入进程数据
            if let Some(processes) = json_server.processes {
                for json_process in processes {
                    let new_process = NewProcess {
                        server_id: json_server.server_id.clone(),
                        pid: json_process.pid,
                        name: json_process.name.clone(),
                        user_name: json_process.user_name.clone(),
                        status: json_process.status.clone(),
                    };

                    create_process(conn, &new_process)?;

                    // 导入进程趋势数据
                    if let Some(trends) = json_process.trend {
                        for json_trend in trends {
                            let new_trend = NewProcessTrend {
                                server_id: json_server.server_id.clone(),
                                pid: json_process.pid,
                                timestamp: json_trend.timestamp,
                                cpu_usage: json_trend.cpu_usage,
                                memory_usage: json_trend.memory_usage,
                                thread_count: json_trend.thread_count,
                            };

                            create_process_trend(conn, &new_trend)?;
                        }
                    }

                    // 导入线程数据
                    if let Some(threads) = json_process.threads {
                        for json_thread in threads {
                            let new_thread = NewThread {
                                server_id: json_server.server_id.clone(),
                                pid: json_process.pid,
                                thread_id: json_thread.thread_id,
                                user_name: json_thread.user_name.clone(),
                                priority: json_thread.priority,
                                nice_value: json_thread.nice_value,
                                virtual_memory: json_thread.virtual_memory.clone(),
                                resident_memory: json_thread.resident_memory.clone(),
                                shared_memory: json_thread.shared_memory.clone(),
                                status: json_thread.status.clone(),
                                cpu_usage: json_thread.cpu_usage.clone(),
                                memory_usage: json_thread.memory_usage.clone(),
                                runtime: json_thread.runtime.clone(),
                                command: json_thread.command.clone(),
                            };

                            create_thread(conn, &new_thread)?;
                        }
                    }
                }
            }

            // 导入崩溃日志数据
            if let Some(crash_logs) = json_server.crash_logs {
                for json_log in crash_logs {
                    let new_log = NewCrashLog {
                        server_id: json_server.server_id.clone(),
                        log_id: json_log.id,
                        timestamp: json_log.timestamp,
                        crash_type: json_log.crash_type.clone(),
                        severity: json_log.severity.clone(),
                        title: json_log.title.clone(),
                        message: json_log.message.clone(),
                        stack_trace: Some(json_log.stack_trace.clone()),
                        resolved: json_log.resolved,
                        ai_summary: json_log.ai_suggestion.as_ref().map(|s| s.summary.clone()),
                        ai_analysis: json_log.ai_suggestion.as_ref().map(|s| s.analysis.clone()),
                    };

                    let crash_log_id = create_crash_log(conn, &new_log)?;

                    // 导入 AI 建议
                    if let Some(ai_suggestion) = json_log.ai_suggestion {
                        for recommendation in ai_suggestion.recommendations {
                            let new_recommendation = NewAiRecommendation {
                                crash_log_id,
                                priority: recommendation.priority,
                                action: recommendation.action.clone(),
                                command: recommendation.command.clone(),
                            };

                            create_ai_recommendation(conn, &new_recommendation)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
