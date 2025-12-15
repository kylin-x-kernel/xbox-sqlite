DROP INDEX IF EXISTS idx_ai_recommendations_priority;
DROP INDEX IF EXISTS idx_ai_recommendations_crash_log_id;
DROP INDEX IF EXISTS idx_crash_logs_severity;
DROP INDEX IF EXISTS idx_crash_logs_timestamp;
DROP INDEX IF EXISTS idx_crash_logs_server_id;
DROP INDEX IF EXISTS idx_threads_pid;
DROP INDEX IF EXISTS idx_threads_server_id;
DROP INDEX IF EXISTS idx_process_trends_timestamp;
DROP INDEX IF EXISTS idx_process_trends_pid;
DROP INDEX IF EXISTS idx_process_trends_server_id;
DROP INDEX IF EXISTS idx_processes_pid;
DROP INDEX IF EXISTS idx_processes_server_id;

DROP TABLE IF EXISTS ai_recommendations;
DROP TABLE IF EXISTS crash_logs;
DROP TABLE IF EXISTS threads;
DROP TABLE IF EXISTS process_trends;
DROP TABLE IF EXISTS processes;