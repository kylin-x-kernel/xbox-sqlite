use anyhow::Result;
use clap::{Parser, Subcommand};
use blackbox::{BlackBox, SmartDataType as LibSmartDataType};

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
    /// 组合数据 (同时插入进程和系统指标数据)
    Combined,
}

impl From<SmartDataType> for LibSmartDataType {
    fn from(cli_type: SmartDataType) -> Self {
        match cli_type {
            SmartDataType::Servers => LibSmartDataType::Servers,
            SmartDataType::SystemMetrics => LibSmartDataType::SystemMetrics,
            SmartDataType::Processes => LibSmartDataType::Processes,
            SmartDataType::CrashLogs => LibSmartDataType::CrashLogs,
            SmartDataType::Combined => LibSmartDataType::Combined,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // 创建 BlackBox 实例
    let blackbox = BlackBox::new(cli.db.clone());
    
    match cli.command {
        Some(Commands::Import { file, clean }) => {
            println!("📥 正在导入数据...");
            blackbox.import_json_data(&file, clean)?;
            println!("✅ 数据导入完成！");
        }
        Some(Commands::Export { file, pretty }) => {
            println!("📤 正在导出数据...");
            blackbox.export_to_json(&file, pretty)?;
            println!("✅ 数据导出完成！");
        }
        Some(Commands::Query { server, limit }) => {
            query_data(&blackbox, server.as_deref(), limit)?;
        }
        Some(Commands::Init { force }) => {
            println!("🔧 正在初始化数据库...");
            blackbox.init_database(force)?;
            println!("✅ 数据库初始化完成！");
        }
        Some(Commands::Insert { data_type, file, continue_on_error }) => {
            smart_insert_from_file(&blackbox, data_type, &file, continue_on_error)?;
        }
        Some(Commands::Stats) => {
            show_statistics(&blackbox)?;
        }
        Some(Commands::Clean { days, confirm }) => {
            clean_old_data(&blackbox, days, confirm)?;
        }
        None => {
            // 默认行为：显示统计信息
            println!("🖥️  服务器监控数据管理系统");
            show_statistics(&blackbox)?;
            println!("\n💡 使用 --help 查看所有可用命令");
        }
    }
    
    Ok(())
}

fn show_statistics(blackbox: &BlackBox) -> Result<()> {
    let stats = blackbox.get_statistics()?;
    
    println!("\n📊 数据库统计信息");
    println!("═══════════════════");
    
    if stats.server_count == 0 {
        println!("📭 数据库为空，请先导入数据");
        return Ok(());
    }
    
    println!("🖥️  服务器总数: {}", stats.server_count);
    
    let mut total_metrics = 0;
    let mut total_processes = 0;
    let mut total_crashes = 0;
    
    for server_stat in &stats.servers {
        total_metrics += server_stat.metrics_count;
        total_processes += server_stat.processes_count;
        total_crashes += server_stat.crashes_count;
        
        println!("\n🔸 {} ({})", server_stat.server.server_name, server_stat.server.server_status);
        println!("   📈 系统指标: {} 条", server_stat.metrics_count);
        println!("   ⚙️  进程数量: {} 个", server_stat.processes_count);
        println!("   🚨 崩溃日志: {} 条", server_stat.crashes_count);
        
        if let Some(latest_time) = server_stat.latest_metric_time {
            let datetime = chrono::DateTime::from_timestamp_millis(latest_time)
                .unwrap_or_default()
                .format("%Y-%m-%d %H:%M:%S");
            println!("   🕒 最新数据: {}", datetime);
        }
    }
    
    println!("\n📋 总计统计");
    println!("   📊 系统指标: {} 条", total_metrics);
    println!("   🔄 进程记录: {} 个", total_processes);
    println!("   ⚠️  崩溃日志: {} 条", total_crashes);
    
    Ok(())
}

fn query_data(blackbox: &BlackBox, server_filter: Option<&str>, limit: Option<i64>) -> Result<()> {
    println!("\n🔍 数据查询结果");
    println!("═══════════════");
    
    let server_details = blackbox.query_servers(server_filter, limit)?;
    
    if server_details.is_empty() {
        println!("❌ 未找到匹配的服务器");
        return Ok(());
    }
    
    println!("\n🖥️  匹配的服务器 ({} 个):", server_details.len());
    for detail in &server_details {
        println!("  🔸 {} ({}) - 状态: {}", 
                detail.server.server_name, 
                detail.server.server_ip, 
                detail.server.server_status);
    }
    
    // 显示详细信息
    for detail in &server_details {
        println!("\n═══ {} 详细信息 ═══", detail.server.server_name);
        
        // 系统指标
        let display_limit = limit.unwrap_or(5);
        println!("\n📊 最新 {} 条系统指标:", display_limit);
        for metric in &detail.metrics {
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
        if !detail.processes.is_empty() {
            println!("\n🔄 运行中的进程 ({} 个):", detail.processes.len());
            for process_detail in &detail.processes {
                println!("  PID: {} | 名称: {} | 用户: {} | 状态: {}", 
                        process_detail.process.pid, 
                        process_detail.process.name, 
                        process_detail.process.user_name, 
                        process_detail.process.status);
                
                // 显示进程的线程信息
                if !process_detail.threads.is_empty() {
                    println!("    └─ 线程数: {}", process_detail.threads.len());
                    for thread in process_detail.threads.iter().take(2) { // 只显示前2个线程
                        println!("      └─ TID: {} | CPU: {}% | 内存: {}% | 命令: {}", 
                                thread.thread_id,
                                thread.cpu_usage,
                                thread.memory_usage,
                                thread.command.chars().take(50).collect::<String>());
                    }
                    if process_detail.threads.len() > 2 {
                        println!("      └─ ... 还有 {} 个线程", process_detail.threads.len() - 2);
                    }
                }
                
                // 显示进程趋势
                if !process_detail.trends.is_empty() {
                    let latest_trend = &process_detail.trends[0];
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
        if !detail.crashes.is_empty() {
            println!("\n🚨 崩溃日志 ({} 条):", detail.crashes.len());
            for crash_detail in detail.crashes.iter().take(3) { // 只显示前3条
                let datetime = chrono::DateTime::from_timestamp_millis(crash_detail.crash_log.timestamp)
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S");
                
                println!("  时间: {} | 类型: {} | 严重性: {} | 已解决: {}", 
                        datetime,
                        crash_detail.crash_log.crash_type,
                        crash_detail.crash_log.severity,
                        if crash_detail.crash_log.resolved { "是" } else { "否" });
                println!("    标题: {}", crash_detail.crash_log.title);
                println!("    消息: {}", crash_detail.crash_log.message.chars().take(100).collect::<String>());
                
                // 显示 AI 建议
                if !crash_detail.recommendations.is_empty() {
                    println!("    🤖 AI 建议 ({} 条):", crash_detail.recommendations.len());
                    for rec in crash_detail.recommendations.iter().take(2) {
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
        if !detail.metrics.is_empty() {
            let avg_cpu: f32 = detail.metrics.iter().map(|m| m.cpu_usage).sum::<f32>() / detail.metrics.len() as f32;
            let avg_memory: f32 = detail.metrics.iter().map(|m| m.memory_usage).sum::<f32>() / detail.metrics.len() as f32;
            let avg_disk: f32 = detail.metrics.iter().map(|m| m.disk_usage).sum::<f32>() / detail.metrics.len() as f32;
            
            println!("\n📈 统计摘要:");
            println!("  平均 CPU 使用率: {:.1}%", avg_cpu);
            println!("  平均内存使用率: {:.1}%", avg_memory);
            println!("  平均磁盘使用率: {:.1}%", avg_disk);
            println!("  系统指标数量: {}", detail.metrics.len());
            println!("  进程数量: {}", detail.processes.len());
            println!("  崩溃日志数量: {}", detail.crashes.len());
        }
    }
    
    Ok(())
}

fn smart_insert_from_file(blackbox: &BlackBox, data_type: SmartDataType, filename: &str, continue_on_error: bool) -> Result<()> {
    println!("🧠 正在智能插入 {:?} 类型的数据 (文件: {})...", data_type, filename);
    
    let result = blackbox.smart_insert_from_file(data_type.into(), filename, continue_on_error)?;
    
    println!("\n📊 智能插入处理完成:");
    println!("   ✅ 新建: {} 条记录", result.success_count);
    println!("   🔄 更新: {} 条记录", result.updated_count);
    println!("   ❌ 失败: {} 条记录", result.error_count);
    
    if result.error_count == 0 {
        println!("   🎉 所有数据处理成功！");
    } else if result.success_count + result.updated_count > 0 {
        println!("   ⚠️  部分数据处理成功，请检查错误信息");
    } else {
        println!("   💥 数据处理失败，请检查输入格式和错误信息");
    }
    
    Ok(())
}

fn clean_old_data(blackbox: &BlackBox, days: i64, confirm: bool) -> Result<()> {
    if !confirm {
        println!("⚠️  此操作将删除 {} 天前的数据", days);
        println!("   请使用 --confirm 参数确认执行");
        return Ok(());
    }
    
    let deleted = blackbox.clean_old_data(days)?;
    
    println!("🗑️  已删除 {} 条旧的系统指标数据", deleted);
    Ok(())
}