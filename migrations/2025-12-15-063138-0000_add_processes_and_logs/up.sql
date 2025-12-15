-- 进程表
CREATE TABLE processes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL,
    pid INTEGER NOT NULL,
    name VARCHAR NOT NULL,
    user_name VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

-- 进程趋势表
CREATE TABLE process_trends (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL,
    pid INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage REAL NOT NULL,
    thread_count INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

-- 线程表
CREATE TABLE threads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL,
    pid INTEGER NOT NULL,
    thread_id INTEGER NOT NULL,
    user_name VARCHAR NOT NULL,
    priority INTEGER NOT NULL,
    nice_value INTEGER NOT NULL,
    virtual_memory VARCHAR NOT NULL,
    resident_memory VARCHAR NOT NULL,
    shared_memory VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    cpu_usage VARCHAR NOT NULL,
    memory_usage VARCHAR NOT NULL,
    runtime VARCHAR NOT NULL,
    command TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

-- 崩溃日志表
CREATE TABLE crash_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL,
    log_id BIGINT NOT NULL,
    timestamp BIGINT NOT NULL,
    crash_type VARCHAR NOT NULL,
    severity VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    message TEXT NOT NULL,
    stack_trace TEXT,
    resolved BOOLEAN NOT NULL DEFAULT FALSE,
    ai_summary TEXT,
    ai_analysis TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

-- AI 建议表
CREATE TABLE ai_recommendations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crash_log_id INTEGER NOT NULL,
    priority INTEGER NOT NULL,
    action TEXT NOT NULL,
    command TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (crash_log_id) REFERENCES crash_logs (id)
);

-- 创建索引
CREATE INDEX idx_processes_server_id ON processes (server_id);
CREATE INDEX idx_processes_pid ON processes (pid);
CREATE INDEX idx_process_trends_server_id ON process_trends (server_id);
CREATE INDEX idx_process_trends_pid ON process_trends (pid);
CREATE INDEX idx_process_trends_timestamp ON process_trends (timestamp);
CREATE INDEX idx_threads_server_id ON threads (server_id);
CREATE INDEX idx_threads_pid ON threads (pid);
CREATE INDEX idx_crash_logs_server_id ON crash_logs (server_id);
CREATE INDEX idx_crash_logs_timestamp ON crash_logs (timestamp);
CREATE INDEX idx_crash_logs_severity ON crash_logs (severity);
CREATE INDEX idx_ai_recommendations_crash_log_id ON ai_recommendations (crash_log_id);
CREATE INDEX idx_ai_recommendations_priority ON ai_recommendations (priority);