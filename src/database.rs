use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use anyhow::Result;
use std::env;

use crate::models::*;

pub fn establish_connection() -> Result<SqliteConnection> {
    establish_connection_with_url(None)
}

pub fn establish_connection_with_url(database_path: Option<&str>) -> Result<SqliteConnection> {
    dotenv::dotenv().ok();
    
    let database_url = if let Some(path) = database_path {
        // 如果提供了路径，构造 SQLite URL
        if path.starts_with("sqlite://") {
            path.to_string()
        } else {
            format!("sqlite://{}", path)
        }
    } else {
        // 否则从环境变量获取，如果没有则使用默认值
        env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://./database.db".to_string())
    };
    
    let connection = SqliteConnection::establish(&database_url)?;
    Ok(connection)
}

pub fn create_server(conn: &mut SqliteConnection, new_server: &NewServer) -> Result<Server> {
    use crate::schema::servers::dsl::*;
    
    diesel::insert_into(servers)
        .values(new_server)
        .execute(conn)?;
    
    // SQLite 不支持 RETURNING，所以需要单独查询
    let server = servers
        .filter(server_id.eq(&new_server.server_id))
        .first::<Server>(conn)?;
    
    Ok(server)
}

pub fn get_server_by_id(conn: &mut SqliteConnection, server_id_param: &str) -> Result<Option<Server>> {
    use crate::schema::servers::dsl::*;
    
    let server = servers
        .filter(server_id.eq(server_id_param))
        .first::<Server>(conn)
        .optional()?;
    
    Ok(server)
}

pub fn get_all_servers(conn: &mut SqliteConnection) -> Result<Vec<Server>> {
    use crate::schema::servers::dsl::*;
    
    let results = servers
        .load::<Server>(conn)?;
    
    Ok(results)
}

pub fn update_server_status(conn: &mut SqliteConnection, server_id_param: &str, new_status: &str) -> Result<Server> {
    use crate::schema::servers::dsl::*;
    
    diesel::update(servers.filter(server_id.eq(server_id_param)))
        .set((server_status.eq(new_status), updated_at.eq(chrono::Utc::now().naive_utc())))
        .execute(conn)?;
    
    // SQLite 不支持 RETURNING，所以需要单独查询
    let server = servers
        .filter(server_id.eq(server_id_param))
        .first::<Server>(conn)?;
    
    Ok(server)
}

pub fn create_system_metric(conn: &mut SqliteConnection, new_metric: &NewSystemMetric) -> Result<()> {
    use crate::schema::system_metrics::dsl::*;
    
    diesel::insert_into(system_metrics)
        .values(new_metric)
        .execute(conn)?;
    
    Ok(())
}

pub fn get_metrics_by_server(conn: &mut SqliteConnection, server_id_param: &str, limit: Option<i64>) -> Result<Vec<SystemMetric>> {
    use crate::schema::system_metrics::dsl::*;
    
    let mut query = system_metrics
        .filter(server_id.eq(server_id_param))
        .order(timestamp.desc())
        .into_boxed();
    
    if let Some(limit_val) = limit {
        query = query.limit(limit_val);
    }
    
    let results = query.load::<SystemMetric>(conn)?;
    Ok(results)
}

pub fn get_metrics_by_time_range(
    conn: &mut SqliteConnection, 
    server_id_param: &str, 
    start_time: i64, 
    end_time: i64
) -> Result<Vec<SystemMetric>> {
    use crate::schema::system_metrics::dsl::*;
    
    let results = system_metrics
        .filter(server_id.eq(server_id_param))
        .filter(timestamp.between(start_time, end_time))
        .order(timestamp.asc())
        .load::<SystemMetric>(conn)?;
    
    Ok(results)
}

pub fn delete_old_metrics(conn: &mut SqliteConnection, before_timestamp: i64) -> Result<usize> {
    use crate::schema::system_metrics::dsl::*;
    
    let deleted_count = diesel::delete(system_metrics.filter(timestamp.lt(before_timestamp)))
        .execute(conn)?;
    
    Ok(deleted_count)
}

// 进程相关操作
pub fn create_process(conn: &mut SqliteConnection, new_process: &NewProcess) -> Result<()> {
    use crate::schema::processes::dsl::*;
    
    diesel::insert_into(processes)
        .values(new_process)
        .execute(conn)?;
    
    Ok(())
}

pub fn get_processes_by_server(conn: &mut SqliteConnection, server_id_param: &str) -> Result<Vec<Process>> {
    use crate::schema::processes::dsl::*;
    
    let results = processes
        .filter(server_id.eq(server_id_param))
        .load::<Process>(conn)?;
    
    Ok(results)
}

// 进程趋势相关操作
pub fn create_process_trend(conn: &mut SqliteConnection, new_trend: &NewProcessTrend) -> Result<()> {
    use crate::schema::process_trends::dsl::*;
    
    diesel::insert_into(process_trends)
        .values(new_trend)
        .execute(conn)?;
    
    Ok(())
}

pub fn get_process_trends(conn: &mut SqliteConnection, server_id_param: &str, pid_param: i32) -> Result<Vec<ProcessTrend>> {
    use crate::schema::process_trends::dsl::*;
    
    let results = process_trends
        .filter(server_id.eq(server_id_param))
        .filter(pid.eq(pid_param))
        .order(timestamp.desc())
        .load::<ProcessTrend>(conn)?;
    
    Ok(results)
}

// 线程相关操作
pub fn create_thread(conn: &mut SqliteConnection, new_thread: &NewThread) -> Result<()> {
    use crate::schema::threads::dsl::*;
    
    diesel::insert_into(threads)
        .values(new_thread)
        .execute(conn)?;
    
    Ok(())
}

pub fn get_threads_by_process(conn: &mut SqliteConnection, server_id_param: &str, pid_param: i32) -> Result<Vec<Thread>> {
    use crate::schema::threads::dsl::*;
    
    let results = threads
        .filter(server_id.eq(server_id_param))
        .filter(pid.eq(pid_param))
        .load::<Thread>(conn)?;
    
    Ok(results)
}

// 崩溃日志相关操作
pub fn create_crash_log(conn: &mut SqliteConnection, new_log: &NewCrashLog) -> Result<i32> {
    use crate::schema::crash_logs::dsl::*;
    
    diesel::insert_into(crash_logs)
        .values(new_log)
        .execute(conn)?;
    
    // 获取插入的记录ID
    let log = crash_logs
        .filter(server_id.eq(&new_log.server_id))
        .filter(log_id.eq(new_log.log_id))
        .first::<CrashLog>(conn)?;
    
    Ok(log.id)
}

pub fn get_crash_logs_by_server(conn: &mut SqliteConnection, server_id_param: &str) -> Result<Vec<CrashLog>> {
    use crate::schema::crash_logs::dsl::*;
    
    let results = crash_logs
        .filter(server_id.eq(server_id_param))
        .order(timestamp.desc())
        .load::<CrashLog>(conn)?;
    
    Ok(results)
}

pub fn get_unresolved_crash_logs(conn: &mut SqliteConnection) -> Result<Vec<CrashLog>> {
    use crate::schema::crash_logs::dsl::*;
    
    let results = crash_logs
        .filter(resolved.eq(false))
        .order(timestamp.desc())
        .load::<CrashLog>(conn)?;
    
    Ok(results)
}

// AI 建议相关操作
pub fn create_ai_recommendation(conn: &mut SqliteConnection, new_recommendation: &NewAiRecommendation) -> Result<()> {
    use crate::schema::ai_recommendations::dsl::*;
    
    diesel::insert_into(ai_recommendations)
        .values(new_recommendation)
        .execute(conn)?;
    
    Ok(())
}

pub fn get_recommendations_by_crash_log(conn: &mut SqliteConnection, crash_log_id_param: i32) -> Result<Vec<AiRecommendation>> {
    use crate::schema::ai_recommendations::dsl::*;
    
    let results = ai_recommendations
        .filter(crash_log_id.eq(crash_log_id_param))
        .order(priority.asc())
        .load::<AiRecommendation>(conn)?;
    
    Ok(results)
}
// 导出功能
pub fn export_all_data(conn: &mut SqliteConnection) -> Result<ExportData> {
    let servers = get_all_servers(conn)?;
    let mut export_servers = Vec::new();
    
    for server in servers {
        // 获取系统指标
        let metrics = get_metrics_by_server(conn, &server.server_id, None)?;
        let export_metrics: Vec<ExportSystemMetric> = metrics.into_iter().map(|m| ExportSystemMetric {
            timestamp: m.timestamp,
            cpu_usage: m.cpu_usage,
            memory_usage: m.memory_usage,
            disk_usage: m.disk_usage,
            io_read: m.io_read,
            io_write: m.io_write,
            network_in: m.network_in,
            network_out: m.network_out,
        }).collect();
        
        // 获取进程信息
        let processes = get_processes_by_server(conn, &server.server_id)?;
        let mut export_processes = Vec::new();
        
        for process in processes {
            // 获取进程趋势
            let trends = get_process_trends(conn, &server.server_id, process.pid)?;
            let export_trends: Vec<ExportProcessTrend> = trends.into_iter().map(|t| ExportProcessTrend {
                timestamp: t.timestamp,
                cpu_usage: t.cpu_usage,
                memory_usage: t.memory_usage,
                thread_count: t.thread_count,
            }).collect();
            
            // 获取线程信息
            let threads = get_threads_by_process(conn, &server.server_id, process.pid)?;
            let export_threads: Vec<ExportThread> = threads.into_iter().map(|t| ExportThread {
                thread_id: t.thread_id,
                user_name: t.user_name,
                priority: t.priority,
                nice_value: t.nice_value,
                virtual_memory: t.virtual_memory,
                resident_memory: t.resident_memory,
                shared_memory: t.shared_memory,
                status: t.status,
                cpu_usage: t.cpu_usage,
                memory_usage: t.memory_usage,
                runtime: t.runtime,
                command: t.command,
            }).collect();
            
            export_processes.push(ExportProcess {
                pid: process.pid,
                name: process.name,
                user_name: process.user_name,
                status: process.status,
                trend: export_trends,
                threads: export_threads,
            });
        }
        
        // 获取崩溃日志
        let crash_logs = get_crash_logs_by_server(conn, &server.server_id)?;
        let mut export_crash_logs = Vec::new();
        
        for log in crash_logs {
            // 获取 AI 建议
            let recommendations = get_recommendations_by_crash_log(conn, log.id)?;
            let export_recommendations: Vec<ExportRecommendation> = recommendations.into_iter().map(|r| ExportRecommendation {
                priority: r.priority,
                action: r.action,
                command: r.command,
            }).collect();
            
            export_crash_logs.push(ExportCrashLog {
                id: log.log_id,
                timestamp: log.timestamp,
                crash_type: log.crash_type,
                severity: log.severity,
                title: log.title,
                message: log.message,
                stack_trace: log.stack_trace.unwrap_or_default(),
                resolved: log.resolved,
                ai_suggestion: ExportAiSuggestion {
                    summary: log.ai_summary.unwrap_or_default(),
                    analysis: log.ai_analysis.unwrap_or_default(),
                    recommendations: export_recommendations,
                },
            });
        }
        
        export_servers.push(ExportServer {
            server_id: server.server_id,
            server_name: server.server_name,
            server_ip: server.server_ip,
            server_os: server.server_os,
            server_status: server.server_status,
            system_metrics: export_metrics,
            processes: export_processes,
            crash_logs: export_crash_logs,
        });
    }
    
    Ok(ExportData {
        servers: export_servers,
    })
}

// 智能插入相关的数据库操作
pub fn get_system_metric_by_timestamp(conn: &mut SqliteConnection, server_id_param: &str, timestamp_param: i64) -> Result<Option<SystemMetric>> {
    use crate::schema::system_metrics::dsl::*;
    
    let metric = system_metrics
        .filter(server_id.eq(server_id_param))
        .filter(timestamp.eq(timestamp_param))
        .first::<SystemMetric>(conn)
        .optional()?;
    
    Ok(metric)
}

pub fn update_system_metric(conn: &mut SqliteConnection, server_id_param: &str, timestamp_param: i64, new_metric: &NewSystemMetric) -> Result<()> {
    use crate::schema::system_metrics::dsl::*;
    
    diesel::update(system_metrics.filter(server_id.eq(server_id_param)).filter(timestamp.eq(timestamp_param)))
        .set((
            cpu_usage.eq(new_metric.cpu_usage),
            memory_usage.eq(new_metric.memory_usage),
            disk_usage.eq(new_metric.disk_usage),
            io_read.eq(new_metric.io_read),
            io_write.eq(new_metric.io_write),
            network_in.eq(new_metric.network_in),
            network_out.eq(new_metric.network_out),
        ))
        .execute(conn)?;
    
    Ok(())
}

pub fn get_process_by_name_and_user(conn: &mut SqliteConnection, server_id_param: &str, name_param: &str, user_name_param: &str) -> Result<Option<Process>> {
    use crate::schema::processes::dsl::*;
    
    let process = processes
        .filter(server_id.eq(server_id_param))
        .filter(name.eq(name_param))
        .filter(user_name.eq(user_name_param))
        .first::<Process>(conn)
        .optional()?;
    
    Ok(process)
}

pub fn update_process_status(conn: &mut SqliteConnection, process_id: i32, new_status: &str) -> Result<()> {
    use crate::schema::processes::dsl::*;
    
    diesel::update(processes.filter(id.eq(process_id)))
        .set((status.eq(new_status), updated_at.eq(chrono::Utc::now().naive_utc())))
        .execute(conn)?;
    
    Ok(())
}

pub fn delete_threads_by_process(conn: &mut SqliteConnection, server_id_param: &str, pid_param: i32) -> Result<()> {
    use crate::schema::threads::dsl::*;
    
    diesel::delete(threads.filter(server_id.eq(server_id_param)).filter(pid.eq(pid_param)))
        .execute(conn)?;
    
    Ok(())
}

pub fn get_crash_log_by_timestamp(conn: &mut SqliteConnection, server_id_param: &str, timestamp_param: i64) -> Result<Option<CrashLog>> {
    use crate::schema::crash_logs::dsl::*;
    
    let log = crash_logs
        .filter(server_id.eq(server_id_param))
        .filter(timestamp.eq(timestamp_param))
        .first::<CrashLog>(conn)
        .optional()?;
    
    Ok(log)
}

pub fn update_crash_log(conn: &mut SqliteConnection, crash_log_id: i32, new_log: &NewCrashLog) -> Result<()> {
    use crate::schema::crash_logs::dsl::*;
    
    diesel::update(crash_logs.filter(id.eq(crash_log_id)))
        .set((
            crash_type.eq(&new_log.crash_type),
            severity.eq(&new_log.severity),
            title.eq(&new_log.title),
            message.eq(&new_log.message),
            stack_trace.eq(&new_log.stack_trace),
            resolved.eq(new_log.resolved),
            ai_summary.eq(&new_log.ai_summary),
            ai_analysis.eq(&new_log.ai_analysis),
        ))
        .execute(conn)?;
    
    Ok(())
}