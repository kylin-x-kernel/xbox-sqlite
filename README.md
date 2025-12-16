# BlackBox - 服务器监控数据管理系统

一个基于 Rust 和 SQLite 的高性能服务器监控数据管理系统，支持智能数据插入、复杂查询分析和数据库管理。

## 📖 命令详解

### 1. 数据库初始化 (init)

初始化新的数据库文件，创建所有必要的表结构和索引：

```bash
# 创建新数据库
./target/debug/blackbox --db test.db init

# 强制重新创建数据库（会删除现有数据）
./target/debug/blackbox --db production.db init --force

# 使用默认数据库路径
./target/debug/blackbox init

# 查看初始化命令帮助
./target/debug/blackbox init --help
```

**功能特性**：
- 自动创建 7 个核心数据表（servers, system_metrics, processes, process_trends, threads, crash_logs, ai_recommendations）
- 创建优化查询性能的索引
- 支持强制重新创建数据库
- 显示详细的创建过程和使用示例

### 2. 智能数据插入 (insert)

支持复杂业务逻辑的智能数据插入，根据不同数据类型采用不同的插入策略：

```bash
# 插入服务器数据（已存在则更新状态）
./target/debug/blackbox --db test.db insert servers --file servers.json

# 插入系统指标（按时间戳智能更新/插入）
./target/debug/blackbox --db test.db insert system-metrics --file metrics.json

# 插入进程数据（按用户名和进程名智能处理，包含趋势和线程）
./target/debug/blackbox --db test.db insert processes --file processes.json

# 插入崩溃日志（按时间戳智能更新/插入）
./target/debug/blackbox --db test.db insert crash-logs --file crash_logs.json

# 🆕 组合插入（同时插入进程和系统指标数据）
./target/debug/blackbox --db test.db insert combined --file test_save.json

# 遇到错误时继续处理
./target/debug/blackbox --db test.db insert servers --file servers.json --continue-on-error

# 查看智能插入命令帮助
./target/debug/blackbox insert --help
```

**智能插入策略**：

- **servers**: 根据 `serverId` 判断，已存在则更新 `serverStatus`，不存在则创建新记录
- **system-metrics**: 根据 `serverId` + `timestamp` 判断，相同时间戳则更新指标值，否则新增记录
- **processes**: 根据 `serverId` + `name` + `userName` 判断，存在则更新状态并添加趋势数据，线程数据完全覆盖
- **crash-logs**: 根据 `serverId` + `timestamp` 判断，相同时间戳则更新日志内容，否则新增记录
- **🆕 combined**: 组合插入模式，同时处理进程和系统指标数据，自动创建服务器（如果不存在），智能处理数据关联

**支持的 JSON 数据格式**：

服务器数据 (`servers.json`):
```json
[
  {
    "serverId": "web-server-01",
    "serverName": "生产环境Web服务器",
    "serverIp": "192.168.1.100",
    "serverOs": "Ubuntu 22.04",
    "serverStatus": "running"
  }
]
```

系统指标数据 (`metrics.json`):
```json
[
  {
    "serverId": "web-server-01",
    "timestamp": 1734249600000,
    "cpuUsage": 45.2,
    "memoryUsage": 68.5,
    "diskUsage": 32.1,
    "ioRead": 1024.5,
    "ioWrite": 2048.3,
    "networkIn": 512.7,
    "networkOut": 256.9
  }
]
```

进程数据 (`processes.json`):
```json
[
  {
    "serverId": "web-server-01",
    "pid": 1001,
    "name": "nginx",
    "userName": "www-data",
    "status": "R",
    "timestamp": 1734249600000,
    "trend": [
      {
        "cpuUsage": 15.2,
        "memoryUsage": 5.8,
        "threadCount": 4
      }
    ],
    "threads": [
      {
        "threadId": 1001,
        "userName": "www-data",
        "priority": 20,
        "niceValue": 0,
        "virtualMemory": "512M",
        "residentMemory": "32M",
        "sharedMemory": "8M",
        "status": "R",
        "cpuUsage": "12.5",
        "memoryUsage": "4.2",
        "runtime": "02:45:18",
        "command": "nginx: master process /usr/sbin/nginx"
      }
    ]
  }
]
```

崩溃日志数据 (`crash_logs.json`):
```json
[
  {
    "serverId": "web-server-01",
    "logId": 2001,
    "timestamp": 1734249700000,
    "crashType": "segmentation_fault",
    "severity": "high",
    "title": "Nginx 进程崩溃",
    "message": "nginx worker process crashed with segmentation fault",
    "stackTrace": "#0 0x00007f8b2c4a5b70 in nginx_worker_process()",
    "resolved": false,
    "aiSummary": "进程内存访问错误导致崩溃",
    "aiAnalysis": "可能是配置文件错误或内存泄漏导致的问题"
  }
]
```

🆕 **组合数据** (`test_save.json` - 同时包含进程和系统指标):
```json
{
  "process": [
    {
      "serverId": "test-server-001",
      "serverName": "测试服务器1",
      "serverIp": "192.168.1.100",
      "serverOs": "Ubuntu 22.04",
      "serverStatus": "running",
      "pid": 1001,
      "name": "ukui-panel",
      "userName": "ukui",
      "status": "S",
      "timestamp": 1734249600000,
      "trend": [
        {
          "cpuUsage": 8.89,
          "memoryUsage": 2.99,
          "threadCount": 12
        }
      ],
      "threads": [
        {
          "threadId": 1001,
          "userName": "ukui",
          "priority": 20,
          "niceValue": 0,
          "virtualMemory": "1.2G",
          "residentMemory": "45M",
          "sharedMemory": "12M",
          "status": "S",
          "cpuUsage": "2.1",
          "memoryUsage": "1.5",
          "runtime": "00:15:32",
          "command": "/usr/bin/ukui-panel --display=:0"
        }
      ]
    }
  ],
  "metrics": [
    {
      "serverId": "test-server-001",
      "timestamp": 1734249600000,
      "cpuUsage": 45.2,
      "memoryUsage": 68.5,
      "diskUsage": 32.1,
      "ioRead": 1024.5,
      "ioWrite": 2048.3,
      "networkIn": 512.7,
      "networkOut": 256.9
    }
  ]
}
```

**组合插入的优势**：
- 🔄 **一次性处理**: 同时插入进程和系统指标数据，保证数据一致性
- 🏗️ **自动创建服务器**: 如果服务器不存在，会根据进程数据中的服务器信息自动创建
- 🧠 **智能关联**: 自动处理进程、趋势、线程和系统指标之间的关联关系
- ⚡ **高效处理**: 减少多次调用，提高数据插入效率
- 📊 **完整监控**: 适合监控系统一次性上报完整的服务器状态数据

### 3. 数据导入 (import)

从 JSON 文件批量导入完整的监控数据：

```bash
# 基本导入
./target/debug/blackbox import

# 指定文件导入
./target/debug/blackbox import --file data.json

# 指定数据库和文件
./target/debug/blackbox --db production.db import --file monitoring_data.json

# 清空现有数据后导入
./target/debug/blackbox import --file data_new.json --clean

# 查看导入命令帮助
./target/debug/blackbox import --help
```

### 4. 数据导出 (export)

将数据库中的所有数据导出为 JSON 格式：

```bash
# 基本导出（格式化输出）
./target/debug/blackbox export

# 指定输出文件
./target/debug/blackbox export --file backup.json

# 指定数据库导出
./target/debug/blackbox --db production.db export --file prod_backup.json

# 紧凑格式导出（节省空间）
./target/debug/blackbox export --file compact.json --pretty false

# 查看导出命令帮助
./target/debug/blackbox export --help
```

### 5. 数据查询 (query)

查询和分析数据库中的监控数据：

```bash
# 查询所有服务器数据
./target/debug/blackbox query

# 查询特定数据库
./target/debug/blackbox --db production.db query

# 查询特定服务器（支持 ID 和名称模糊匹配）
./target/debug/blackbox query --server web-server-01
./target/debug/blackbox --db test.db query --server "Web-Server"

# 限制显示记录数
./target/debug/blackbox query --limit 10

# 组合查询
./target/debug/blackbox --db monitoring.db query --server nginx --limit 5

# 查看查询命令帮助
./target/debug/blackbox query --help
```

**查询功能**：
- 📊 系统指标趋势分析
- 🔄 进程和线程监控详情
- 🚨 崩溃日志和 AI 建议展示
- 📈 统计摘要信息
- 🔍 支持服务器名称和 ID 模糊匹配

### 6. 统计信息 (stats)

显示数据库的详细统计信息：

```bash
# 显示统计信息
./target/debug/blackbox stats

# 查看特定数据库统计
./target/debug/blackbox --db production.db stats
./target/debug/blackbox --db /var/lib/monitoring/archive.db stats
```

**统计内容**：
- 服务器数量和状态分布
- 各类数据记录总数
- 每个服务器的详细指标
- 最新数据时间戳
- 未解决崩溃问题汇总

### 7. 数据清理 (clean)

清理指定时间之前的旧数据：

```bash
# 清理 30 天前的数据（需要确认）
./target/debug/blackbox clean --days 30 --confirm

# 清理特定数据库的旧数据
./target/debug/blackbox --db production.db clean --days 7 --confirm

# 预览清理操作（不加 --confirm）
./target/debug/blackbox clean --days 15

# 查看清理命令帮助
./target/debug/blackbox clean --help
```

## 🚀 完整使用示例

### 基本工作流程

```bash
# 1. 初始化新数据库
./target/debug/blackbox --db monitoring.db init

# 2. 插入服务器信息
./target/debug/blackbox --db monitoring.db insert servers --file servers.json

# 3. 插入系统指标数据
./target/debug/blackbox --db monitoring.db insert system-metrics --file metrics.json

# 4. 插入进程监控数据
./target/debug/blackbox --db monitoring.db insert processes --file processes.json

# 5. 插入崩溃日志
./target/debug/blackbox --db monitoring.db insert crash-logs --file crash_logs.json

# 🆕 或者使用组合插入（一次性插入进程和系统指标）
./target/debug/blackbox --db monitoring.db insert combined --file test_save.json

# 6. 查看统计信息
./target/debug/blackbox --db monitoring.db stats

# 7. 查询特定服务器详情
./target/debug/blackbox --db monitoring.db query --server web-server-01 --limit 10

# 8. 导出备份数据
./target/debug/blackbox --db monitoring.db export --file backup_$(date +%Y%m%d).json

# 9. 清理旧数据
./target/debug/blackbox --db monitoring.db clean --days 30 --confirm
```

### 智能更新示例

```bash
# 第一次插入服务器
echo '[{"serverId":"srv-01","serverName":"Web服务器","serverIp":"192.168.1.100","serverOs":"Ubuntu 22.04","serverStatus":"running"}]' > server.json
./target/debug/blackbox --db test.db insert servers --file server.json

# 更新服务器状态（相同 serverId 会自动更新）
echo '[{"serverId":"srv-01","serverName":"Web服务器","serverIp":"192.168.1.100","serverOs":"Ubuntu 22.04","serverStatus":"maintenance"}]' > server_update.json
./target/debug/blackbox --db test.db insert servers --file server_update.json

# 插入相同时间戳的指标数据会更新现有记录
echo '[{"serverId":"srv-01","timestamp":1734249600000,"cpuUsage":45.2,"memoryUsage":68.5,"diskUsage":32.1,"ioRead":1024.5,"ioWrite":2048.3,"networkIn":512.7,"networkOut":256.9}]' > metrics1.json
./target/debug/blackbox --db test.db insert system-metrics --file metrics1.json

# 相同时间戳，不同指标值 - 会更新现有记录
echo '[{"serverId":"srv-01","timestamp":1734249600000,"cpuUsage":55.8,"memoryUsage":72.1,"diskUsage":33.5,"ioRead":1200.0,"ioWrite":2500.0,"networkIn":600.0,"networkOut":300.0}]' > metrics2.json
./target/debug/blackbox --db test.db insert system-metrics --file metrics2.json
```

### 🆕 组合插入示例

```bash
# 使用现有的 test_save.json 进行组合插入
./target/debug/blackbox --db test.db insert combined --file test_save.json

# 查看插入结果
./target/debug/blackbox --db test.db stats

# 查询特定服务器的详细信息
./target/debug/blackbox --db test.db query --server test-server-001

# 组合插入的优势演示：一次性插入多个服务器的完整监控数据
# 创建包含多个服务器的组合数据文件
cat > multi_server_data.json << 'EOF'
{
  "process": [
    {
      "serverId": "web-01",
      "serverName": "Web服务器1",
      "serverIp": "192.168.1.10",
      "serverOs": "Ubuntu 22.04",
      "serverStatus": "running",
      "pid": 1001,
      "name": "nginx",
      "userName": "www-data",
      "status": "R",
      "timestamp": 1734249600000,
      "trend": [{"cpuUsage": 15.2, "memoryUsage": 5.8, "threadCount": 4}],
      "threads": [
        {
          "threadId": 1001,
          "userName": "www-data",
          "priority": 20,
          "niceValue": 0,
          "virtualMemory": "512M",
          "residentMemory": "32M",
          "sharedMemory": "8M",
          "status": "R",
          "cpuUsage": "12.5",
          "memoryUsage": "4.2",
          "runtime": "02:45:18",
          "command": "nginx: master process"
        }
      ]
    },
    {
      "serverId": "db-01",
      "serverName": "数据库服务器1",
      "serverIp": "192.168.1.20",
      "serverOs": "CentOS 8",
      "serverStatus": "running",
      "pid": 2001,
      "name": "mysqld",
      "userName": "mysql",
      "status": "S",
      "timestamp": 1734249600000,
      "trend": [{"cpuUsage": 25.8, "memoryUsage": 45.2, "threadCount": 16}],
      "threads": [
        {
          "threadId": 2001,
          "userName": "mysql",
          "priority": 20,
          "niceValue": 0,
          "virtualMemory": "2.1G",
          "residentMemory": "512M",
          "sharedMemory": "64M",
          "status": "S",
          "cpuUsage": "20.1",
          "memoryUsage": "35.8",
          "runtime": "12:30:45",
          "command": "/usr/sbin/mysqld"
        }
      ]
    }
  ],
  "metrics": [
    {
      "serverId": "web-01",
      "timestamp": 1734249600000,
      "cpuUsage": 35.2,
      "memoryUsage": 58.5,
      "diskUsage": 28.1,
      "ioRead": 800.5,
      "ioWrite": 1200.3,
      "networkIn": 2048.7,
      "networkOut": 1024.9
    },
    {
      "serverId": "db-01",
      "timestamp": 1734249600000,
      "cpuUsage": 65.8,
      "memoryUsage": 78.2,
      "diskUsage": 45.6,
      "ioRead": 5120.8,
      "ioWrite": 3072.1,
      "networkIn": 1024.3,
      "networkOut": 512.7
    }
  ]
}
EOF

# 一次性插入两个服务器的完整监控数据
./target/debug/blackbox --db test.db insert combined --file multi_server_data.json

# 查看插入结果
./target/debug/blackbox --db test.db stats
```

### 多环境数据管理

```bash
# 开发环境
./target/debug/blackbox --db dev.db init
./target/debug/blackbox --db dev.db insert servers --file dev_servers.json

# 测试环境
./target/debug/blackbox --db test.db init
./target/debug/blackbox --db test.db insert servers --file test_servers.json

# 生产环境
./target/debug/blackbox --db production.db init
./target/debug/blackbox --db production.db insert servers --file prod_servers.json

# 查看各环境统计
./target/debug/blackbox --db dev.db stats
./target/debug/blackbox --db test.db stats
./target/debug/blackbox --db production.db stats
```

## 🛠️ 构建和安装

```bash
# 构建项目
cargo build --release

# 运行测试
cargo test

# 安装到系统
cargo install --path .

# 查看版本信息
./target/debug/blackbox --version
```

---

**BlackBox** - 让服务器监控数据管理变得简单高效！ 🚀