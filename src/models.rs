use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::servers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Server {
    pub id: i32,
    pub server_id: String,
    pub server_name: String,
    pub server_ip: String,
    pub server_os: String,
    pub server_status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::servers)]
#[serde(rename_all = "camelCase")]
pub struct NewServer {
    pub server_id: String,
    pub server_name: String,
    pub server_ip: String,
    pub server_os: String,
    pub server_status: String,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::system_metrics)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SystemMetric {
    pub id: i32,
    pub server_id: String,
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub io_read: f32,
    pub io_write: f32,
    pub network_in: f32,
    pub network_out: f32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::system_metrics)]
#[serde(rename_all = "camelCase")]
pub struct NewSystemMetric {
    pub server_id: String,
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub io_read: f32,
    pub io_write: f32,
    pub network_in: f32,
    pub network_out: f32,
}

// 进程模型
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::processes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Process {
    pub id: i32,
    pub server_id: String,
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::processes)]
#[serde(rename_all = "camelCase")]
pub struct NewProcess {
    pub server_id: String,
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
}

// 进程趋势模型
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::process_trends)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProcessTrend {
    pub id: i32,
    pub server_id: String,
    pub pid: i32,
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub thread_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::process_trends)]
#[serde(rename_all = "camelCase")]
pub struct NewProcessTrend {
    pub server_id: String,
    pub pid: i32,
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub thread_count: i32,
}

// 线程模型
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::threads)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Thread {
    pub id: i32,
    pub server_id: String,
    pub pid: i32,
    pub thread_id: i32,
    pub user_name: String,
    pub priority: i32,
    pub nice_value: i32,
    pub virtual_memory: String,
    pub resident_memory: String,
    pub shared_memory: String,
    pub status: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub runtime: String,
    pub command: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::threads)]
#[serde(rename_all = "camelCase")]
pub struct NewThread {
    pub server_id: String,
    pub pid: i32,
    pub thread_id: i32,
    pub user_name: String,
    pub priority: i32,
    pub nice_value: i32,
    pub virtual_memory: String,
    pub resident_memory: String,
    pub shared_memory: String,
    pub status: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub runtime: String,
    pub command: String,
}

// 崩溃日志模型
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::crash_logs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CrashLog {
    pub id: i32,
    pub server_id: String,
    pub log_id: i64,
    pub timestamp: i64,
    pub crash_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub resolved: bool,
    pub ai_summary: Option<String>,
    pub ai_analysis: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::crash_logs)]
#[serde(rename_all = "camelCase")]
pub struct NewCrashLog {
    pub server_id: String,
    pub log_id: i64,
    pub timestamp: i64,
    pub crash_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub resolved: bool,
    pub ai_summary: Option<String>,
    pub ai_analysis: Option<String>,
}

// AI 建议模型
#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::ai_recommendations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AiRecommendation {
    pub id: i32,
    pub crash_log_id: i32,
    pub priority: i32,
    pub action: String,
    pub command: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::ai_recommendations)]
#[serde(rename_all = "camelCase")]
pub struct NewAiRecommendation {
    pub crash_log_id: i32,
    pub priority: i32,
    pub action: String,
    pub command: String,
}

// JSON 数据结构，用于解析 data.json
#[derive(Deserialize, Debug)]
pub struct JsonData {
    pub servers: Vec<JsonServer>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonServer {
    pub server_id: String,
    pub server_name: String,
    pub server_ip: String,
    pub server_os: String,
    pub server_status: String,
    pub system_metrics: Vec<JsonSystemMetric>,
    pub processes: Option<Vec<JsonProcess>>,
    pub crash_logs: Option<Vec<JsonCrashLog>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonSystemMetric {
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub io_read: f32,
    pub io_write: f32,
    pub network_in: f32,
    pub network_out: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonProcess {
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
    pub trend: Option<Vec<JsonProcessTrend>>,
    pub threads: Option<Vec<JsonThread>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonProcessTrend {
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub thread_count: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonThread {
    pub thread_id: i32,
    pub user_name: String,
    pub priority: i32,
    pub nice_value: i32,
    pub virtual_memory: String,
    pub resident_memory: String,
    pub shared_memory: String,
    pub status: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub runtime: String,
    pub command: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonCrashLog {
    pub id: i64,
    pub timestamp: i64,
    pub crash_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub stack_trace: String,
    pub resolved: bool,
    pub ai_suggestion: Option<JsonAiSuggestion>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsonAiSuggestion {
    pub summary: String,
    pub analysis: String,
    pub recommendations: Vec<JsonRecommendation>,
}

#[derive(Deserialize, Debug)]
pub struct JsonRecommendation {
    pub priority: i32,
    pub action: String,
    pub command: String,
}

// 用于智能插入的数据结构
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartProcessInsert {
    pub server_id: String,
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
    pub timestamp: i64,
    pub trend: Vec<SmartProcessTrend>,
    pub threads: Vec<SmartThread>,
    // 服务器信息字段（仅用于自动创建服务器，不插入进程表）
    pub server_name: Option<String>,
    pub server_ip: Option<String>,
    pub server_os: Option<String>,
    pub server_status: Option<String>,
}

// 组合插入数据结构 - 同时包含进程和系统指标数据
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombinedInsertData {
    pub process: Vec<CombinedProcessData>,
    pub metrics: Vec<SmartSystemMetric>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombinedProcessData {
    pub server_id: String,
    pub server_name: String,
    pub server_ip: String,
    pub server_os: String,
    pub server_status: String,
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
    pub timestamp: i64,
    pub trend: Vec<SmartProcessTrend>,
    pub threads: Vec<SmartThread>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartProcessTrend {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub thread_count: i32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartThread {
    pub thread_id: i32,
    pub user_name: String,
    pub priority: i32,
    pub nice_value: i32,
    pub virtual_memory: String,
    pub resident_memory: String,
    pub shared_memory: String,
    pub status: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub runtime: String,
    pub command: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartSystemMetric {
    pub server_id: String,
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub io_read: f32,
    pub io_write: f32,
    pub network_in: f32,
    pub network_out: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartCrashLog {
    pub server_id: String,
    pub log_id: i64,
    pub timestamp: i64,
    pub crash_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub resolved: bool,
    pub ai_summary: Option<String>,
    pub ai_analysis: Option<String>,
}
// 导出用的数据结构
#[derive(Serialize, Debug)]
pub struct ExportData {
    pub servers: Vec<ExportServer>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportServer {
    pub server_id: String,
    pub server_name: String,
    pub server_ip: String,
    pub server_os: String,
    pub server_status: String,
    pub system_metrics: Vec<ExportSystemMetric>,
    pub processes: Vec<ExportProcess>,
    pub crash_logs: Vec<ExportCrashLog>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportSystemMetric {
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub io_read: f32,
    pub io_write: f32,
    pub network_in: f32,
    pub network_out: f32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportProcess {
    pub pid: i32,
    pub name: String,
    pub user_name: String,
    pub status: String,
    pub trend: Vec<ExportProcessTrend>,
    pub threads: Vec<ExportThread>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportProcessTrend {
    pub timestamp: i64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub thread_count: i32,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportThread {
    pub thread_id: i32,
    pub user_name: String,
    pub priority: i32,
    pub nice_value: i32,
    pub virtual_memory: String,
    pub resident_memory: String,
    pub shared_memory: String,
    pub status: String,
    pub cpu_usage: String,
    pub memory_usage: String,
    pub runtime: String,
    pub command: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportCrashLog {
    pub id: i64,
    pub timestamp: i64,
    pub crash_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub stack_trace: String,
    pub resolved: bool,
    pub ai_suggestion: ExportAiSuggestion,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportAiSuggestion {
    pub summary: String,
    pub analysis: String,
    pub recommendations: Vec<ExportRecommendation>,
}

#[derive(Serialize, Debug)]
pub struct ExportRecommendation {
    pub priority: i32,
    pub action: String,
    pub command: String,
}