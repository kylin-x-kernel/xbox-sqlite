// @generated automatically by Diesel CLI.

diesel::table! {
    servers (id) {
        id -> Integer,
        server_id -> Text,
        server_name -> Text,
        server_ip -> Text,
        server_os -> Text,
        server_status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    system_metrics (id) {
        id -> Integer,
        server_id -> Text,
        timestamp -> BigInt,
        cpu_usage -> Float,
        memory_usage -> Float,
        disk_usage -> Float,
        io_read -> Float,
        io_write -> Float,
        network_in -> Float,
        network_out -> Float,
        created_at -> Timestamp,
    }
}

diesel::table! {
    processes (id) {
        id -> Integer,
        server_id -> Text,
        pid -> Integer,
        name -> Text,
        user_name -> Text,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    process_trends (id) {
        id -> Integer,
        server_id -> Text,
        pid -> Integer,
        timestamp -> BigInt,
        cpu_usage -> Float,
        memory_usage -> Float,
        thread_count -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    threads (id) {
        id -> Integer,
        server_id -> Text,
        pid -> Integer,
        thread_id -> Integer,
        user_name -> Text,
        priority -> Integer,
        nice_value -> Integer,
        virtual_memory -> Text,
        resident_memory -> Text,
        shared_memory -> Text,
        status -> Text,
        cpu_usage -> Text,
        memory_usage -> Text,
        runtime -> Text,
        command -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    crash_logs (id) {
        id -> Integer,
        server_id -> Text,
        log_id -> BigInt,
        timestamp -> BigInt,
        crash_type -> Text,
        severity -> Text,
        title -> Text,
        message -> Text,
        stack_trace -> Nullable<Text>,
        resolved -> Bool,
        ai_summary -> Nullable<Text>,
        ai_analysis -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    ai_recommendations (id) {
        id -> Integer,
        crash_log_id -> Integer,
        priority -> Integer,
        action -> Text,
        command -> Text,
        created_at -> Timestamp,
    }
}

// SQLite 外键关联，但不使用 joinable 宏，因为字段类型不匹配

diesel::allow_tables_to_appear_in_same_query!(
    servers,
    system_metrics,
    processes,
    process_trends,
    threads,
    crash_logs,
    ai_recommendations,
);