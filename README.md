# Agent Bark API

[![Built with 小顺子](https://img.shields.io/badge/Built%20with-%E5%B0%8F%E9%A1%BA%E5%AD%90-3b82f6?style=flat-square&logo=robotframework&logoColor=white)](https://kimi.moonshot.cn)

专为 AI Agent 设计的 Bark 推送通知 API，支持即时推送、定时循环推送和一次性定时推送。让 AI Agent 可以通过简单的 HTTP 调用来向用户手机发送通知。

## 为什么需要这个项目？

很多 AI Agent 运行在用户的笔记本电脑上，而非台式机或服务器。笔记本电脑存在**休眠、断网、关机**等情况，无法保证 24 小时在线。

如果你的 Agent 需要：
- 📅 给用户设置提醒（"30分钟后开会"）
- ⏰ 定时推送信息（"每天9点推送日报"）
- 🔁 循环通知（"每小时提醒喝水"）

**问题**：Agent 自己无法可靠地完成这些任务，因为它可能在关键时刻处于离线状态。

**解决方案**：把定时任务交给外部服务！Agent 只需要调用一次 API，后续的定时/提醒逻辑由这个服务负责，不受 Agent 本身在线状态的影响。

## 功能特性

- **即时推送** - 通过接口立即发送通知到手机
- **定时循环** - 支持定时表达式，按周期重复执行
- **次数限制** - 可设置最大执行次数，达到后自动停止
- **一次性定时** - 指定时间点执行一次
- **精确删除** - 删除周期性任务后立即停止，无残留执行
- **密码保护** - 接口密码验证，可安全部署到公网
- **单文件部署** - 单二进制文件，无需额外依赖

## 前置要求

- Rust 1.75+（[安装指南](https://www.rust-lang.org/tools/install)）
- Linux 系统需要 OpenSSL 开发库：
  ```bash
  # Ubuntu/Debian
  sudo apt-get install libssl-dev pkg-config
  
  # CentOS/RHEL
  sudo yum install openssl-devel pkgconfig
  ```

## 快速开始

### 1. 克隆并编译

```bash
git clone <仓库地址>
cd agent-bark-api
cargo build --release
```

编译产物位于 `target/release/agent-bark-api`

### 2. 配置

服务支持两种配置方式：环境变量或配置文件。

**方式一：环境变量（适合容器/脚本部署）**

```bash
export BARK_DEVICE_KEY="你的设备密钥"
export BARK_PASSWORD="你的密码"
export BARK_HOST="0.0.0.0"      # 可选，默认 0.0.0.0
export BARK_PORT=3000           # 可选，默认 3000
export BARK_BARK_URL="https://api.day.app"  # 可选，默认 https://api.day.app

./agent-bark-api
```

**方式二：配置文件（适合传统部署）**

创建 `config.toml`：

```toml
# 服务监听配置
host = "0.0.0.0"
port = 3000

# Bark 服务地址
bark_url = "https://api.day.app"

# 设备密钥（必填，从 Bark App 获取）
device_key = "你的设备密钥"

# 访问密码（建议公网部署时设置）
password = "你的密码"
```

然后直接运行：

```bash
./agent-bark-api
```

**配置优先级**：环境变量 > 配置文件 > 默认值

**安全提醒**：部署到公网时，**务必**设置 `password`，否则接口完全公开。

### 3. 后台运行（非 Systemd 方式）

```bash
# 使用 nohup
nohup ./agent-bark-api > server.log 2>&1 &

# 查看日志
tail -f server.log

# 停止服务
pkill agent-bark-api
```

## 接口说明

### 认证方式

公开接口无需认证：
- `GET /` - 服务信息
- `GET /health` - 健康检查

其他接口需要携带密码，支持两种方式：

**请求头认证（推荐）**
```bash
-H "Authorization: Bearer 你的密码"
```

**地址参数认证**
```bash
?token=你的密码
```

### 立即发送通知

```bash
POST /notify
Content-Type: application/json
Authorization: Bearer 你的密码

{
  "title": "通知标题",
  "body": "通知内容",
  "sound": "bell",           // 可选，提示音
  "group": "分组名称",        // 可选，通知分组
  "level": "timeSensitive",  // 可选：active（默认）、timeSensitive、passive
  "icon": "https://example.com/icon.png",  // 可选，图标地址
  "url": "https://example.com",            // 可选，点击跳转
  "copy": "复制内容",          // 可选，复制到剪贴板
  "auto_copy": true,          // 可选，自动复制
  "badge": 1                  // 可选，角标数字
}
```

**响应示例**：
```json
{
  "success": true,
  "data": {
    "code": 200,
    "message": "success"
  }
}
```

### 定时循环发送

```bash
POST /schedule/cron
Content-Type: application/json
Authorization: Bearer 你的密码

{
  "title": "定时提醒",
  "body": "该喝水了",
  "cron": "0 */5 * * * *",  // 每5分钟
  "max_count": 3            // 可选，最多执行3次，达到后自动删除
}
```

**响应示例**：
```json
{
  "success": true,
  "data": {
    "job_id": "db253fcc-669e-49b1-a251-ab2e7dbb5357"
  }
}
```

**定时表达式格式**（6位）：`秒 分 时 日 月 星期`

常用示例：
- `0 */5 * * * *` - 每5分钟
- `0 0 9 * * *` - 每天上午9点
- `0 0 9 * * 1` - 每周一上午9点
- `0 0 9,18 * * *` - 每天上午9点和下午6点

### 一次性定时发送

指定未来时间点执行一次，执行后自动删除。

```bash
POST /schedule/once
Content-Type: application/json
Authorization: Bearer 你的密码

{
  "title": "会议提醒",
  "body": "15分钟后开会",
  "at": "2024-01-15T14:45:00Z"  // UTC时间格式
}
```

**注意**：
- 时间必须是未来时间
- 使用 UTC 时区（带 `Z` 后缀）

### 查看定时任务

```bash
# 查看所有任务
GET /jobs
Authorization: Bearer 你的密码

# 响应示例
{
  "success": true,
  "data": [
    {
      "id": "61634a91-3e2c-4540-a9ea-65696034cc21",
      "cron": null,
      "at": "2026-02-03T08:49:51Z",
      "notify": {"title": "一次性提醒", "body": "30秒后收到"},
      "created_at": "2026-02-03T08:49:21.466Z",
      "max_count": 1
    }
  ]
}

# 查看单个任务
GET /jobs/任务ID
Authorization: Bearer 你的密码
```

### 删除定时任务

```bash
DELETE /jobs/任务ID
Authorization: Bearer 你的密码
```

删除后立即生效，任务不会再次执行。

**响应示例**：
```json
{
  "success": true,
  "data": null
}
```

## 部署示例

### Systemd 服务

创建 `/etc/systemd/system/agent-bark-api.service`：

```ini
[Unit]
Description=Agent Bark API
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/opt/agent-bark-api
ExecStart=/opt/agent-bark-api/agent-bark-api
Restart=always
Environment="BARK_DEVICE_KEY=你的设备密钥"
Environment="BARK_PASSWORD=你的密码"

[Install]
WantedBy=multi-user.target
```

启动和管理：

```bash
# 重载配置
sudo systemctl daemon-reload

# 开机自启
sudo systemctl enable agent-bark-api

# 启动服务
sudo systemctl start agent-bark-api

# 查看状态
sudo systemctl status agent-bark-api

# 查看日志
sudo journalctl -u agent-bark-api -f

# 停止服务
sudo systemctl stop agent-bark-api
```

### Docker

**构建镜像**（需要先本地编译）：

```bash
# 先编译
cargo build --release

# 构建镜像
docker build -t agent-bark-api .
```

`Dockerfile`：

```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY target/release/agent-bark-api /usr/local/bin/
ENV BARK_DEVICE_KEY=你的设备密钥
ENV BARK_PASSWORD=你的密码
EXPOSE 3000
CMD ["agent-bark-api"]
```

**运行容器**：

```bash
# 前台运行
docker run -p 3000:3000 agent-bark-api

# 后台运行
docker run -d --name agent-bark-api -p 3000:3000 agent-bark-api

# 查看日志
docker logs -f agent-bark-api

# 停止容器
docker stop agent-bark-api
docker rm agent-bark-api
```

### PM2

```bash
# 安装 PM2
npm install -g pm2

# 启动
export BARK_DEVICE_KEY="你的设备密钥"
export BARK_PASSWORD="你的密码"
pm2 start ./agent-bark-api --name agent-bark-api

# 保存配置
pm2 save
pm2 startup

# 管理
pm2 logs agent-bark-api    # 查看日志
pm2 stop agent-bark-api    # 停止
pm2 restart agent-bark-api # 重启
pm2 delete agent-bark-api  # 删除
```

## 使用场景

### 服务器告警

```bash
curl -X POST http://localhost:3000/notify \
  -H "Authorization: Bearer 你的密码" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "服务器告警",
    "body": "磁盘空间不足，仅剩 10%",
    "sound": "alarm",
    "level": "timeSensitive"
  }'
```

### 定时提醒（执行3次后自动停止）

```bash
curl -X POST http://localhost:3000/schedule/cron \
  -H "Authorization: Bearer 你的密码" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "健康提醒",
    "body": "该喝水了",
    "cron": "0 0 * * * *",
    "max_count": 3
  }'
```

### 会议提醒

```bash
curl -X POST http://localhost:3000/schedule/once \
  -H "Authorization: Bearer 你的密码" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "会议提醒",
    "body": "15分钟后与客户开会",
    "at": "2024-01-15T14:45:00Z"
  }'
```

## 响应格式

成功响应：

```json
{
  "success": true,
  "data": { ... }
}
```

错误响应：

```json
{
  "success": false,
  "error": "错误信息"
}
```

认证失败返回状态码 401：

```
Unauthorized: invalid or missing token
```

## 错误码说明

| HTTP 状态码 | 说明 | 常见原因 |
|------------|------|---------|
| 200 | 成功 | 请求处理成功 |
| 400 | 请求参数错误 | JSON 格式错误、缺少必填字段 |
| 401 | 认证失败 | 未提供密码或密码错误 |
| 404 | 资源不存在 | 任务 ID 不存在 |
| 422 | 无法处理 | 定时时间必须是未来时间 |
| 500 | 服务端错误 | 内部错误或 Bark 服务异常 |

**业务错误**（`success: false` 时）：

```json
{
  "success": false,
  "error": "设备密钥 device_key 不能为空"
}
```

## 相关链接

**Bark 官方：**
- [Bark 官网](https://bark.day.app/) - 官方文档和 App 下载
- [Bark GitHub](https://github.com/Finb/Bark) - iOS App 源码
- [Bark 服务端 GitHub](https://github.com/Finb/bark-server) - 官方服务端参考实现

**本项目技术栈：**
- [Axum](https://github.com/tokio-rs/axum) - Web 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [tokio-cron-scheduler](https://github.com/mvniekerk/tokio-cron-scheduler) - 定时任务调度
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端

## 安全建议

公网部署时建议：

1. **启用密码认证**（必需）
   ```bash
   export BARK_PASSWORD="16位以上随机字符串"
   ```

2. **启用 HTTPS**（配合 Nginx 或 Caddy）

3. **限制访问来源**（防火墙）
   ```bash
   ufw allow from 你的IP to any port 3000
   ```

4. **定期更换设备密钥**（Bark App 中可重新注册）

## 常见问题

**Q: 编译失败，提示找不到 OpenSSL？**

A: 安装 OpenSSL 开发库：
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl
```

**Q: 服务启动后立即退出？**

A: 检查日志，通常是 `device_key` 未配置。必填项必须通过配置文件或环境变量提供。

**Q: 定时任务没有按时执行？**

A: 检查服务器时区设置。服务内部使用 UTC 时间，但 cron 表达式按服务器本地时间解析。

**Q: 如何查看运行日志？**

A: 
- Systemd: `sudo journalctl -u agent-bark-api -f`
- Docker: `docker logs -f agent-bark-api`
- PM2: `pm2 logs agent-bark-api`
- 手动: `tail -f server.log`

---

<p align="center">Built with ❤️ by <a href="https://kimi.moonshot.cn">小顺子</a></p>
