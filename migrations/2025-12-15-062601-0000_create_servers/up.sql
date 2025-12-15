CREATE TABLE servers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL UNIQUE,
    server_name VARCHAR NOT NULL,
    server_ip VARCHAR NOT NULL,
    server_os VARCHAR NOT NULL,
    server_status VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE system_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id VARCHAR NOT NULL,
    timestamp BIGINT NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage REAL NOT NULL,
    disk_usage REAL NOT NULL,
    io_read REAL NOT NULL,
    io_write REAL NOT NULL,
    network_in REAL NOT NULL,
    network_out REAL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (server_id) REFERENCES servers (server_id)
);

CREATE INDEX idx_system_metrics_server_id ON system_metrics (server_id);
CREATE INDEX idx_system_metrics_timestamp ON system_metrics (timestamp);