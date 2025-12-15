mod schema;
mod models;
mod database;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use diesel::prelude::*;
use diesel::{Connection, RunQueryDsl};
use models::*;
use database::*;

#[derive(Parser)]
#[command(name = "blackbox")]
#[command(about = "服务器监控数据管理系统", long_about = None)]
#[command(version = "1.0")]
struct Cli {
    /// 数据库文件路径
    #[arg(long, short, global = true, help = "指定数据库文件路径")]
    db: Option<String>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 导入 JSON 数据到数据库
    Import {
        /// 输入文件路径
        #[arg(short, long, default_value = "data.json")]
        file: String,
        /// 是否清空现有数据
        #[arg(long)]
        clean: bool,
    },
    /// 从数据库导出数据到 JSON 文件
    Export {
        /// 输出文件路径
        #[arg(short, long, default_value = "export.json")]
        file: String,
        /// 是否格式化输出
        #[arg(long, default_value = "true")]
        pretty: bool,
    },
    /// 查询并显示数据库内容
    Query {
        /// 指定服务器 ID
        #[arg(short, long)]
        server: Option<String>,
        /// 限制显示的记录数
        #[arg(short, long)]
        limit: Option<i64>,
    },
    /// 初始化数据库文件
    Init {
        /// 是否强制重新创建数据库 (会删除现有数据)
        #[arg(long)]
        force: bool,
    },
    /// 智能插入数据记录 (支持复杂业务逻辑)
    Insert {
        /// 数据类型 (servers, system_metrics, processes, crash_logs)
        #[arg(value_enum)]
        data_type: SmartDataType,
        /// JSON 文件路径
        #[arg(short, long)]
        file: String,
        /// 遇到错误时是否继续处理
        #[arg(long, default_value = "false")]
        continue_on_error: bool,
    },
    /// 数据库统计信息
    Stats,
    /// 清理旧数据
    Clean {
        /// 保留最近 N 天的数据
        #[arg(short, long, default_value = "30")]
        days: i64,
        /// 确认执行清理
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum SmartDataType {
    /// 服务器信息 (已存在则更新状态)
    Servers,
    /// 系统指标数据 (按时间戳智能更新/插入)
    SystemMetrics,
    /// 进程信息 (按用户名和进程名智能处理，包含趋势和线程)
    Processes,
    /// 崩溃日志 (按时间戳智能更新/插入)
    CrashLogs,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 建立数据库连接
    let mut conn = establish_connection_with_url(cli.db.as_deref())?;
    
    match cli.command {
        Some(Commands::Import { file, clean }) => {
            if clean {
                println!("🗑️  清空现有数据...");
                clean_database(&mut conn)?;
            }
            import_json_data(&mut conn, &file)?;
        }
        Some(Commands::Export { file, pretty }) => {
            export_to_json(&mut conn, &file, pretty)?;
        }
        Some(Commands::Query { server, limit }) => {
            query_data(&mut conn, server.as_deref(), limit)?;
        }
        Some(Commands::Init { force }) => {
            init_database(&cli.db, force)?;
        }
        Some(Commands::Insert { data_type, file, continue_on_error }) => {
            smart_insert_from_file(&mut conn, data_type, &file, continue_on_error)?;
        }
        Some(Commands::Stats) => {
            show_statistics(&mut conn)?;
        }
        Some(Commands::Clean { days, confirm }) => {
            clean_old_data(&mut conn, days, confirm)?;
        }
        None => {
            // 默认行为：显示统计信息
            println!("🖥️  服务器监控数据管理系统");
            show_statistics(&mut conn)?;
            println!("\n💡 使用 --help 查看所有可用命令");
        }
    }
    
    Ok(())
}

fn clean_database(conn: &mut diesel::SqliteConnection) -> Result<()> {
    use crate::schema::*;
    use diesel::prelude::*;
    
    diesel::delete(ai_recommendations::table).execute(conn)?;
    diesel::delete(crash_logs::table).execute(conn)?;
    diesel::delete(threads::table).execute(conn)?;
    diesel::delete(process_trends::table).execute(conn)?;
    diesel::delete(processes::table).execute(conn)?;
    diesel::delete(system_metrics::table).execute(conn)?;
    diesel::delete(servers::table).execute(conn)?;
    
    println!("✅ 数据库已清空");
    Ok(())
}

fn show_statistics(conn: &mut diesel::SqliteConnection) -> Result<()> {
    let servers = get_all_servers(conn)?;
    
    println!("\n📊 数据库统计信息");
    println!("═══════════════════");
    
    if servers.is_empty() {
        println!("📭 数据库为空，请先导入数据");
        return Ok(());
    }
    
    println!("🖥️  服务器总数: {}", servers.len());
    
    let mut total_metrics = 0;
    let mut total_processes = 0;
    let mut total_crashes = 0;
    
    for server in &servers {
        let metrics = get_metrics_by_server(conn, &server.server_id, None)?;
        let processes = get_processes_by_server(conn, &server.server_id)?;
        let crashes = get_crash_logs_by_server(conn, &server.server_id)?;
        
        total_metrics += metrics.len();
        total_processes += processes.len();
        total_crashes += crashes.len();
        
        println!("\n🔸 {} ({})", server.server_name, server.server_status);
        println!("   📈 系统指标: {} 条", metrics.len());
        println!("   ⚙️  进程数量: {} 个", processes.len());
        println!("   🚨 崩溃日志: {} 条", crashes.len());
        
        if !metrics.is_empty() {
            let latest = &metrics[0];
            let datetime = chrono::DateTime::from_timestamp_millis(latest.timestamp)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S");
            println!("   🕒 最新数据: {}", datetime);
        }
    }
    
    println!("\n📋 总计统计");
    println!("   📊 系统指标: {} 条", total_metrics);
    println!("   🔄 进程记录: {} 个", total_processes);
    println!("   ⚠️  崩溃日志: {} 条", total_crashes);
    
    let unresolved = get_unresolved_crash_logs(conn)?;
    if !unresolved.is_empty() {
        println!("   🔴 未解决问题: {} 个", unresolved.len());
    }
    
    Ok(())
}

fn clean_old_data(conn: &mut diesel::SqliteConnection, days: i64, confirm: bool) -> Result<()> {
    if !confirm {
        println!("⚠️  此操作将删除 {} 天前的数据", days);
        println!("   请使用 --confirm 参数确认执行");
        return Ok(());
    }
    
    let cutoff_time = chrono::Utc::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);
    let deleted = delete_old_metrics(conn, cutoff_time)?;
    
    println!("🗑️  已删除 {} 条旧的系统指标数据", deleted);
    Ok(())
}

fn import_json_data(conn: &mut diesel::SqliteConnection, filename: &str) -> Result<()> {
    println!("正在读取 {} 文件...", filename);
    
    let json_content = fs::read_to_string(filename)?;
    let json_data: JsonData = serde_json::from_str(&json_content)?;
    
    println!("找到 {} 个服务器", json_data.servers.len());
    
    for json_server in json_data.servers {
        // 检查服务器是否已存在
        match get_server_by_id(conn, &json_server.server_id)? {
            Some(existing_server) => {
                println!("服务器 {} 已存在，更新状态", existing_server.server_name);
                update_server_status(conn, &json_server.server_id, &json_server.server_status)?;
            }
            None => {
                println!("创建新服务器: {}", json_server.server_name);
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
        println!("导入 {} 条系统指标数据", json_server.system_metrics.len());
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
            println!("导入 {} 个进程", processes.len());
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
            println!("导入 {} 条崩溃日志", crash_logs.len());
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
    
    println!("数据导入完成！");
    Ok(())
}

fn query_data(conn: &mut diesel::SqliteConnection, server_filter: Option<&str>, limit: Option<i64>) -> Result<()> {
    println!("\n🔍 数据查询结果");
    println!("═══════════════");
    
    let servers = get_all_servers(conn)?;
    
    // 根据过滤条件选择服务器
    let target_servers: Vec<_> = if let Some(filter) = server_filter {
        servers.into_iter()
            .filter(|s| s.server_id == filter || s.server_name.contains(filter))
            .collect()
    } else {
        servers
    };
    
    if target_servers.is_empty() {
        println!("❌ 未找到匹配的服务器");
        return Ok(());
    }
    
    println!("\n🖥️  匹配的服务器 ({} 个):", target_servers.len());
    for server in &target_servers {
        println!("  🔸 {} ({}) - 状态: {}", 
                server.server_name, 
                server.server_ip, 
                server.server_status);
    }
    
    // 显示详细信息
    for server in &target_servers {
        println!("\n═══ {} 详细信息 ═══", server.server_name);
        
        // 系统指标
        let display_limit = limit.unwrap_or(5);
        println!("\n📊 最新 {} 条系统指标:", display_limit);
        let metrics = get_metrics_by_server(conn, &server.server_id, Some(display_limit))?;
        for metric in metrics {
            let datetime = chrono::DateTime::from_timestamp_millis(metric.timestamp)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S");
            
            println!("  时间: {} | CPU: {:.1}% | 内存: {:.1}% | 磁盘: {:.1}%", 
                    datetime,
                    metric.cpu_usage, 
                    metric.memory_usage, 
                    metric.disk_usage);
        }
        
        // 进程信息
        let processes = get_processes_by_server(conn, &server.server_id)?;
        if !processes.is_empty() {
            println!("\n🔄 运行中的进程 ({} 个):", processes.len());
            for process in &processes {
                println!("  PID: {} | 名称: {} | 用户: {} | 状态: {}", 
                        process.pid, 
                        process.name, 
                        process.user_name, 
                        process.status);
                
                // 显示进程的线程信息
                let threads = get_threads_by_process(conn, &server.server_id, process.pid)?;
                if !threads.is_empty() {
                    println!("    └─ 线程数: {}", threads.len());
                    for thread in threads.iter().take(2) { // 只显示前2个线程
                        println!("      └─ TID: {} | CPU: {}% | 内存: {}% | 命令: {}", 
                                thread.thread_id,
                                thread.cpu_usage,
                                thread.memory_usage,
                                thread.command.chars().take(50).collect::<String>());
                    }
                    if threads.len() > 2 {
                        println!("      └─ ... 还有 {} 个线程", threads.len() - 2);
                    }
                }
                
                // 显示进程趋势
                let trends = get_process_trends(conn, &server.server_id, process.pid)?;
                if !trends.is_empty() {
                    let latest_trend = &trends[0];
                    let datetime = chrono::DateTime::from_timestamp_millis(latest_trend.timestamp)
                        .unwrap_or_default()
                        .format("%H:%M:%S");
                    println!("    └─ 最新趋势 ({}): CPU: {:.1}% | 内存: {:.1}% | 线程数: {}", 
                            datetime,
                            latest_trend.cpu_usage,
                            latest_trend.memory_usage,
                            latest_trend.thread_count);
                }
            }
        }
        
        // 崩溃日志
        let crash_logs = get_crash_logs_by_server(conn, &server.server_id)?;
        if !crash_logs.is_empty() {
            println!("\n🚨 崩溃日志 ({} 条):", crash_logs.len());
            for log in crash_logs.iter().take(3) { // 只显示前3条
                let datetime = chrono::DateTime::from_timestamp_millis(log.timestamp)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S");
                
                println!("  时间: {} | 类型: {} | 严重性: {} | 已解决: {}", 
                        datetime,
                        log.crash_type,
                        log.severity,
                        if log.resolved { "是" } else { "否" });
                println!("    标题: {}", log.title);
                println!("    消息: {}", log.message.chars().take(100).collect::<String>());
                
                // 显示 AI 建议
                let recommendations = get_recommendations_by_crash_log(conn, log.id)?;
                if !recommendations.is_empty() {
                    println!("    🤖 AI 建议 ({} 条):", recommendations.len());
                    for rec in recommendations.iter().take(2) {
                        println!("      {}. {} (优先级: {})", 
                                rec.priority, 
                                rec.action,
                                rec.priority);
                        println!("         命令: {}", rec.command.chars().take(80).collect::<String>());
                    }
                }
                println!();
            }
        }
        
        // 统计信息
        let all_metrics = get_metrics_by_server(conn, &server.server_id, None)?;
        if !all_metrics.is_empty() {
            let avg_cpu: f32 = all_metrics.iter().map(|m| m.cpu_usage).sum::<f32>() / all_metrics.len() as f32;
            let avg_memory: f32 = all_metrics.iter().map(|m| m.memory_usage).sum::<f32>() / all_metrics.len() as f32;
            let avg_disk: f32 = all_metrics.iter().map(|m| m.disk_usage).sum::<f32>() / all_metrics.len() as f32;
            
            println!("\n📈 统计摘要:");
            println!("  平均 CPU 使用率: {:.1}%", avg_cpu);
            println!("  平均内存使用率: {:.1}%", avg_memory);
            println!("  平均磁盘使用率: {:.1}%", avg_disk);
            println!("  系统指标数量: {}", all_metrics.len());
            println!("  进程数量: {}", processes.len());
            println!("  崩溃日志数量: {}", crash_logs.len());
        }
    }
    
    // 全局统计
    let unresolved_crashes = get_unresolved_crash_logs(conn)?;
    if !unresolved_crashes.is_empty() {
        println!("\n⚠️  未解决的崩溃问题: {} 个", unresolved_crashes.len());
        for crash in unresolved_crashes.iter().take(3) {
            println!("  - {} ({})", crash.title, crash.severity);
        }
    }
    
    Ok(())
}
fn export_to_json(conn: &mut diesel::SqliteConnection, filename: &str, pretty: bool) -> Result<()> {
    println!("📤 正在导出数据到 {} 文件...", filename);
    
    let export_data = export_all_data(conn)?;
    
    let json_content = if pretty {
        serde_json::to_string_pretty(&export_data)?
    } else {
        serde_json::to_string(&export_data)?
    };
    
    fs::write(filename, json_content)?;
    
    println!("✅ 数据导出完成！");
    println!("\n📊 导出统计:");
    println!("   🖥️  服务器数量: {}", export_data.servers.len());
    
    let mut total_metrics = 0;
    let mut total_processes = 0;
    let mut total_crashes = 0;
    
    for server in &export_data.servers {
        total_metrics += server.system_metrics.len();
        total_processes += server.processes.len();
        total_crashes += server.crash_logs.len();
        
        println!("   🔸 {}: {} 条指标, {} 个进程, {} 条崩溃日志", 
                server.server_name,
                server.system_metrics.len(),
                server.processes.len(),
                server.crash_logs.len());
    }
    
    println!("\n📋 总计: {} 条指标, {} 个进程, {} 条崩溃日志", 
            total_metrics, total_processes, total_crashes);
    
    let file_size = fs::metadata(filename)?.len();
    println!("📁 文件大小: {:.2} MB", file_size as f64 / 1024.0 / 1024.0);
    
    Ok(())
}

fn smart_insert_from_file(conn: &mut diesel::SqliteConnection, data_type: SmartDataType, filename: &str, continue_on_error: bool) -> Result<()> {
    println!("🧠 正在智能插入 {:?} 类型的数据 (文件: {})...", data_type, filename);
    
    let json_content = fs::read_to_string(filename)
        .map_err(|e| anyhow::anyhow!("无法读取文件 {}: {}", filename, e))?;
    
    match data_type {
        SmartDataType::Servers => {
            let servers: Vec<NewServer> = serde_json::from_str(&json_content)
                .map_err(|e| anyhow::anyhow!("JSON 解析错误: {}", e))?;
            
            smart_insert_servers(conn, servers, continue_on_error)?;
        }
        
        SmartDataType::SystemMetrics => {
            let metrics: Vec<SmartSystemMetric> = serde_json::from_str(&json_content)
                .map_err(|e| anyhow::anyhow!("JSON 解析错误: {}", e))?;
            
            smart_insert_system_metrics(conn, metrics, continue_on_error)?;
        }
        
        SmartDataType::Processes => {
            let processes: Vec<SmartProcessInsert> = serde_json::from_str(&json_content)
                .map_err(|e| anyhow::anyhow!("JSON 解析错误: {}", e))?;
            
            smart_insert_processes(conn, processes, continue_on_error)?;
        }
        
        SmartDataType::CrashLogs => {
            let crash_logs: Vec<SmartCrashLog> = serde_json::from_str(&json_content)
                .map_err(|e| anyhow::anyhow!("JSON 解析错误: {}", e))?;
            
            smart_insert_crash_logs(conn, crash_logs, continue_on_error)?;
        }
    }
    
    Ok(())
}

fn smart_insert_servers(conn: &mut diesel::SqliteConnection, servers: Vec<NewServer>, continue_on_error: bool) -> Result<()> {
    println!("📋 处理 {} 个服务器记录", servers.len());
    
    let mut success_count = 0;
    let mut updated_count = 0;
    let mut error_count = 0;
    
    for server in servers {
        match get_server_by_id(conn, &server.server_id)? {
            Some(existing) => {
                match update_server_status(conn, &server.server_id, &server.server_status) {
                    Ok(_) => {
                        updated_count += 1;
                        println!("🔄 更新服务器: {} -> 状态: {}", existing.server_name, server.server_status);
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 更新服务器 {} 失败: {}", server.server_name, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
            None => {
                match create_server(conn, &server) {
                    Ok(_) => {
                        success_count += 1;
                        println!("✅ 创建服务器: {}", server.server_name);
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 创建服务器 {} 失败: {}", server.server_name, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n📊 服务器处理完成:");
    println!("   ✅ 新建: {} 个", success_count);
    println!("   🔄 更新: {} 个", updated_count);
    println!("   ❌ 失败: {} 个", error_count);
    
    Ok(())
}

fn smart_insert_system_metrics(conn: &mut diesel::SqliteConnection, metrics: Vec<SmartSystemMetric>, continue_on_error: bool) -> Result<()> {
    println!("📋 处理 {} 条系统指标记录", metrics.len());
    
    let mut success_count = 0;
    let mut updated_count = 0;
    let mut error_count = 0;
    
    for metric in metrics {
        // 验证服务器是否存在
        if get_server_by_id(conn, &metric.server_id)?.is_none() {
            error_count += 1;
            eprintln!("❌ 服务器 {} 不存在", metric.server_id);
            if !continue_on_error {
                return Err(anyhow::anyhow!("服务器 {} 不存在", metric.server_id));
            }
            continue;
        }
        
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
                match update_system_metric(conn, &metric.server_id, metric.timestamp, &new_metric) {
                    Ok(_) => {
                        updated_count += 1;
                        if updated_count % 10 == 0 {
                            println!("🔄 已更新 {} 条指标...", updated_count);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 更新指标失败 (时间戳: {}): {}", metric.timestamp, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
            None => {
                match create_system_metric(conn, &new_metric) {
                    Ok(_) => {
                        success_count += 1;
                        if success_count % 10 == 0 {
                            println!("✅ 已插入 {} 条指标...", success_count);
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 插入指标失败 (时间戳: {}): {}", metric.timestamp, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n📊 系统指标处理完成:");
    println!("   ✅ 新建: {} 条", success_count);
    println!("   🔄 更新: {} 条", updated_count);
    println!("   ❌ 失败: {} 条", error_count);
    
    Ok(())
}

fn smart_insert_processes(conn: &mut diesel::SqliteConnection, processes: Vec<SmartProcessInsert>, continue_on_error: bool) -> Result<()> {
    println!("📋 处理 {} 个进程记录", processes.len());
    
    let mut success_count = 0;
    let mut updated_count = 0;
    let mut error_count = 0;
    
    for process_data in processes {
        // 验证服务器是否存在，如果不存在则尝试自动创建
        if get_server_by_id(conn, &process_data.server_id)?.is_none() {
            // 检查是否提供了服务器信息用于自动创建
            if let (Some(server_name), Some(server_ip), Some(server_os), Some(server_status)) = (
                &process_data.server_name,
                &process_data.server_ip,
                &process_data.server_os,
                &process_data.server_status,
            ) {
                println!("🔧 服务器 {} 不存在，正在自动创建...", process_data.server_id);
                let new_server = NewServer {
                    server_id: process_data.server_id.clone(),
                    server_name: server_name.clone(),
                    server_ip: server_ip.clone(),
                    server_os: server_os.clone(),
                    server_status: server_status.clone(),
                };
                
                match create_server(conn, &new_server) {
                    Ok(_) => {
                        println!("✅ 自动创建服务器: {} ({})", server_name, process_data.server_id);
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 自动创建服务器 {} 失败: {}", process_data.server_id, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                        continue;
                    }
                }
            } else {
                error_count += 1;
                eprintln!("❌ 服务器 {} 不存在且未提供服务器信息用于自动创建", process_data.server_id);
                if !continue_on_error {
                    return Err(anyhow::anyhow!("服务器 {} 不存在且未提供服务器信息用于自动创建", process_data.server_id));
                }
                continue;
            }
        }
        
        match get_process_by_name_and_user(conn, &process_data.server_id, &process_data.name, &process_data.user_name)? {
            Some(existing_process) => {
                // 进程已存在，更新状态并添加趋势数据
                match update_process_status(conn, existing_process.id, &process_data.status) {
                    Ok(_) => {
                        updated_count += 1;
                        println!("🔄 更新进程: {} (用户: {}) -> 状态: {}", process_data.name, process_data.user_name, process_data.status);
                        
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
                            
                            if let Err(e) = create_process_trend(conn, &new_trend) {
                                eprintln!("⚠️  添加趋势数据失败: {}", e);
                            }
                        }
                        
                        // 覆盖线程数据
                        if let Err(e) = delete_threads_by_process(conn, &process_data.server_id, process_data.pid) {
                            eprintln!("⚠️  删除旧线程数据失败: {}", e);
                        }
                        
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
                            
                            if let Err(e) = create_thread(conn, &new_thread) {
                                eprintln!("⚠️  添加线程数据失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 更新进程 {} 失败: {}", process_data.name, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
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
                
                match create_process(conn, &new_process) {
                    Ok(_) => {
                        success_count += 1;
                        println!("✅ 创建进程: {} (用户: {}, PID: {})", process_data.name, process_data.user_name, process_data.pid);
                        
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
                            
                            if let Err(e) = create_process_trend(conn, &new_trend) {
                                eprintln!("⚠️  添加趋势数据失败: {}", e);
                            }
                        }
                        
                        // 添加线程数据
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
                            
                            if let Err(e) = create_thread(conn, &new_thread) {
                                eprintln!("⚠️  添加线程数据失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 创建进程 {} 失败: {}", process_data.name, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n📊 进程处理完成:");
    println!("   ✅ 新建: {} 个", success_count);
    println!("   🔄 更新: {} 个", updated_count);
    println!("   ❌ 失败: {} 个", error_count);
    
    Ok(())
}

fn smart_insert_crash_logs(conn: &mut diesel::SqliteConnection, crash_logs: Vec<SmartCrashLog>, continue_on_error: bool) -> Result<()> {
    println!("📋 处理 {} 条崩溃日志记录", crash_logs.len());
    
    let mut success_count = 0;
    let mut updated_count = 0;
    let mut error_count = 0;
    
    for log_data in crash_logs {
        // 验证服务器是否存在
        if get_server_by_id(conn, &log_data.server_id)?.is_none() {
            error_count += 1;
            eprintln!("❌ 服务器 {} 不存在", log_data.server_id);
            if !continue_on_error {
                return Err(anyhow::anyhow!("服务器 {} 不存在", log_data.server_id));
            }
            continue;
        }
        
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
                match update_crash_log(conn, existing_log.id, &new_log) {
                    Ok(_) => {
                        updated_count += 1;
                        println!("🔄 更新崩溃日志: {} (时间戳: {})", log_data.title, log_data.timestamp);
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 更新崩溃日志失败 (时间戳: {}): {}", log_data.timestamp, e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
            None => {
                match create_crash_log(conn, &new_log) {
                    Ok(_) => {
                        success_count += 1;
                        println!("✅ 创建崩溃日志: {}", log_data.title);
                    }
                    Err(e) => {
                        error_count += 1;
                        eprintln!("❌ 创建崩溃日志失败: {}", e);
                        if !continue_on_error {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
    
    println!("\n📊 崩溃日志处理完成:");
    println!("   ✅ 新建: {} 条", success_count);
    println!("   🔄 更新: {} 条", updated_count);
    println!("   ❌ 失败: {} 条", error_count);
    
    Ok(())
}
fn init_database(db_path: &Option<String>, force: bool) -> Result<()> {
    let database_url = if let Some(path) = db_path {
        if path.starts_with("sqlite://") {
            path.clone()
        } else {
            format!("sqlite://{}", path)
        }
    } else {
        "sqlite://./database.db".to_string()
    };
    
    // 提取文件路径
    let file_path = database_url.strip_prefix("sqlite://").unwrap_or(&database_url);
    
    println!("🔧 正在初始化数据库: {}", file_path);
    
    // 检查文件是否已存在
    if Path::new(file_path).exists() {
        if !force {
            println!("⚠️  数据库文件已存在: {}", file_path);
            println!("   使用 --force 参数强制重新创建数据库");
            return Ok(());
        } else {
            println!("🗑️  删除现有数据库文件...");
            fs::remove_file(file_path)?;
        }
    }
    
    // 创建数据库连接（这会自动创建文件）
    println!("📁 创建数据库文件...");
    let mut conn = diesel::SqliteConnection::establish(&database_url)?;
    
    // 执行建表 SQL
    println!("🏗️  创建数据表...");
    
    // 创建 servers 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 system_metrics 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 processes 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 process_trends 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 threads 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 crash_logs 表
    diesel::sql_query(r#"
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
    "#).execute(&mut conn)?;
    
    // 创建 ai_recommendations 表
    diesel::sql_query(r#"
        CREATE TABLE ai_recommendations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            crash_log_id INTEGER NOT NULL,
            priority INTEGER NOT NULL,
            action TEXT NOT NULL,
            command TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (crash_log_id) REFERENCES crash_logs(id)
        )
    "#).execute(&mut conn)?;
    
    // 创建索引以提高查询性能
    println!("📊 创建索引...");
    
    diesel::sql_query("CREATE INDEX idx_servers_server_id ON servers(server_id)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_system_metrics_server_timestamp ON system_metrics(server_id, timestamp)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_processes_server_name_user ON processes(server_id, name, user_name)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_process_trends_server_pid ON process_trends(server_id, pid)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_threads_server_pid ON threads(server_id, pid)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_crash_logs_server_timestamp ON crash_logs(server_id, timestamp)").execute(&mut conn)?;
    diesel::sql_query("CREATE INDEX idx_ai_recommendations_crash_log ON ai_recommendations(crash_log_id)").execute(&mut conn)?;
    
    println!("✅ 数据库初始化完成！");
    println!("\n📋 创建的表:");
    println!("   🖥️  servers - 服务器信息");
    println!("   📊 system_metrics - 系统指标数据");
    println!("   ⚙️  processes - 进程信息");
    println!("   📈 process_trends - 进程趋势数据");
    println!("   🧵 threads - 线程信息");
    println!("   🚨 crash_logs - 崩溃日志");
    println!("   🤖 ai_recommendations - AI 建议");
    
    println!("\n💡 使用示例:");
    println!("   blackbox --db {} smart-insert servers --file servers.json", file_path);
    println!("   blackbox --db {} query", file_path);
    println!("   blackbox --db {} stats", file_path);
    
    Ok(())
}