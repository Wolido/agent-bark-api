---
name: agent-bark-api
description: 使用 Agent Bark API 服务向用户手机发送推送通知。适用于需要定时提醒、即时通知、循环提醒等场景。支持即时推送、一次性定时推送、循环定时推送。
---

# Agent Bark API 使用指南

让 AI Agent 具备向用户手机发送推送通知的能力，即使 Agent 运行在可能离线的笔记本电脑上。

## 什么时候使用这个服务

✅ **适合使用**：
- 需要给用户设置提醒（"30分钟后开会"）
- 定时推送信息（"每天9点推送日报"）
- 循环通知（"每小时提醒喝水"）
- 重要事件即时通知（"任务完成"、"出错告警"）

❌ **不适合使用**：
- 用户没有安装 Bark App
- 不需要持久化的提醒（用本地定时器即可）
- 需要复杂交互的通知（这是单向推送）

## 前置条件检查

### 1. 确认服务地址

**首先询问用户**：服务部署在哪里？可能的场景：

| 场景 | 示例地址 |
|------|----------|
| 本机部署 | `http://localhost:3000` |
| 局域网其他机器 | `http://192.168.1.100:3000` |
| 远程服务器 | `http://1.2.3.4:8080` |
| 域名部署 | `https://bark.example.com` |

设置变量（后面所有示例都使用这个变量）：
```bash
BARK_API_URL="用户提供的地址"
BARK_PASSWORD="用户提供的密码（如果有）"
```

### 2. 检查服务是否可用

```bash
# 测试连通性
curl "${BARK_API_URL}/health"
```

**如果返回 "OK"** → 服务已运行，跳到配置检查  
**如果连接失败** → 需要部署服务，继续阅读下文

### 3. 部署服务（如未部署）

如果用户还没有部署服务，指导用户完成以下步骤：

#### 方式一：快速部署（推荐）

```bash
# 1. 下载预编译二进制（从 GitHub Releases 下载）
# 或者克隆源码编译
git clone https://github.com/Wolido/agent-bark-api.git
cd agent-bark-api
cargo build --release

# 2. 创建配置文件
cp config.toml.example config.toml

# 3. 编辑配置文件，填入设备密钥
nano config.toml
```

配置示例：
```toml
host = "0.0.0.0"
port = 3000  # 可以改为其他端口，如 8080
bark_url = "https://api.day.app"
device_key = "用户的Bark设备密钥"  # 必填
password = "设置一个密码"           # 公网部署时必填
```

然后运行：
```bash
./target/release/agent-bark-api
```

服务默认监听 `0.0.0.0:3000`，可以通过配置修改端口。

#### 方式二：Docker 部署

```bash
docker run -d \
  --name agent-bark-api \
  -p 3000:3000 \
  -e BARK_DEVICE_KEY="用户的设备密钥" \
  -e BARK_PASSWORD="设置密码" \
  ghcr.io/wolido/agent-bark-api:latest
```

`-p 3000:3000` 可以改为 `-p 8080:3000` 使用 8080 端口。

#### 方式三：Systemd 部署（服务器长期运行）

```bash
# 1. 复制二进制到系统目录
sudo cp target/release/agent-bark-api /usr/local/bin/
sudo chmod +x /usr/local/bin/agent-bark-api

# 2. 创建配置目录
sudo mkdir -p /etc/agent-bark-api
sudo nano /etc/agent-bark-api/config.toml

# 3. 创建 systemd 服务
sudo nano /etc/systemd/system/agent-bark-api.service
```

service 文件内容：
```ini
[Unit]
Description=Agent Bark API
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/agent-bark-api
WorkingDirectory=/etc/agent-bark-api
Restart=always

[Install]
WantedBy=multi-user.target
```

```bash
# 启动服务
sudo systemctl daemon-reload
sudo systemctl enable agent-bark-api
sudo systemctl start agent-bark-api

# 查看状态
sudo systemctl status agent-bark-api
```

### 4. 获取 Bark 设备密钥

如果用户没有 device_key，指导用户：

1. 在 iPhone/iPad 上安装 Bark App（App Store 搜索 "Bark"）
2. 打开 App，点击右上角设置图标
3. 复制 "Device Key"（格式类似：`vWs4oJAgD59p8NdyhCDgn`）
4. 将 device_key 填入配置文件

### 5. 验证部署

```bash
# 测试服务运行
curl "${BARK_API_URL}/"
# 应返回：Agent Bark API

curl "${BARK_API_URL}/health"
# 应返回：OK
```

## 配置敏感信息

⚠️ **安全提醒**：device_key 和 password 是敏感信息，不应该：
- 硬编码在代码中
- 提交到 Git 仓库
- 在不安全的聊天中明文传输

### 推荐的配置方式

Agent 应该在内部维护这些配置：

```bash
# 从用户那里获取的地址和密码
BARK_API_URL="http://192.168.1.100:8080"  # 示例：局域网部署
BARK_PASSWORD="用户设置的密码"
```

或者让用户设置环境变量：
```bash
export BARK_API_URL="http://192.168.1.100:8080"
export BARK_API_PASSWORD="密码"
```

## API 调用指南（使用 curl）

### 基础变量设置

所有示例都假设你已经设置了：
```bash
BARK_API_URL="用户提供的地址，如 http://192.168.1.100:8080"
BARK_PASSWORD="密码（如果没有设置密码则留空）"
```

### 1. 即时推送

适用场景：任务完成、错误告警、即时通知

```bash
# 基础调用
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "任务完成",
    "body": "数据处理已完成！"
  }'

# 带可选参数
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "服务器告警",
    "body": "磁盘空间不足，仅剩 10%",
    "sound": "alarm",
    "group": "告警",
    "level": "timeSensitive",
    "badge": 1
  }'
```

**可选参数说明**：
- `sound`: 提示音（如 `"bell"`, `"alarm"`, `"glass"`）
- `group`: 通知分组名称
- `level`: 通知级别（`"active"`, `"timeSensitive"`, `"passive"`）
- `icon`: 图标 URL
- `url`: 点击跳转链接
- `copy`: 复制到剪贴板的内容
- `auto_copy`: 是否自动复制（布尔值）
- `badge`: 角标数字

### 2. 一次性定时推送

适用场景：会议提醒、预约提醒、延迟通知

**⚠️ UTC 时间处理**：

```bash
# 计算30分钟后的 UTC 时间
# macOS
FUTURE_TIME=$(date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ")

# Linux
FUTURE_TIME=$(date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ")

# 发送一次性定时通知
curl -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"会议提醒\",
    \"body\": \"30分钟后与客户开会，请准备\",
    \"at\": \"${FUTURE_TIME}\"
  }"
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

**注意**：返回的 `job_id` 建议保存，用于后续管理任务。

#### 时间格式详细说明

**必须使用 UTC 时间，格式为 ISO 8601**：

```
✅ 正确格式: 2026-02-03T12:30:00Z      (UTC 时间，带 Z 后缀)
❌ 错误格式: 2026-02-03 12:30:00       (没有 T 和 Z)
❌ 错误格式: 2026-02-03T12:30:00+08:00 (带时区偏移，不支持)
```

**不同系统生成 UTC 时间**：

```bash
# macOS - 当前 UTC 时间
date -u +"%Y-%m-%dT%H:%M:%SZ"

# Linux - 当前 UTC 时间  
date -u +"%Y-%m-%dT%H:%M:%SZ"

# macOS - 30分钟后
date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ"

# Linux - 30分钟后
date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ"

# macOS - 明天上午9点 UTC
date -u -v+1d -v9H -v0M -v0S +"%Y-%m-%dT%H:%M:%SZ"

# Linux - 明天上午9点 UTC
date -u -d "tomorrow 09:00" +"%Y-%m-%dT%H:%M:%SZ"
```

**本地时间转 UTC（以北京时间 UTC+8 为例）**：

```bash
# 假设用户说的是北京时间 20:30
LOCAL_TIME="2026-02-03 20:30:00"

# 转换为 UTC（北京时间减8小时）
# 结果为: 2026-02-03T12:30:00Z
```

### 3. 循环定时推送

适用场景：定期提醒、习惯养成、监控告警

```bash
# 每天上午9点提醒喝水，最多提醒5次
curl -X POST "${BARK_API_URL}/schedule/cron" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "喝水提醒",
    "body": "工作再忙也要记得喝水哦",
    "cron": "0 0 9 * * *",
    "max_count": 5
  }'

# 每5分钟提醒一次，最多3次
curl -X POST "${BARK_API_URL}/schedule/cron" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "定时测试",
    "body": "每5分钟一次",
    "cron": "0 */5 * * * *",
    "max_count": 3
  }'
```

#### Cron 表达式格式

格式：`秒 分 时 日 月 星期`

| 位置 | 含义 | 范围 | 示例 |
|------|------|------|------|
| 1 | 秒 | 0-59 | `0` 或 `*/10` |
| 2 | 分 | 0-59 | `0` 或 `*/5` |
| 3 | 时 | 0-23 | `9` 或 `9,18` |
| 4 | 日 | 1-31 | `*` |
| 5 | 月 | 1-12 | `*` |
| 6 | 星期 | 0-6 (0=周日) | `1` (周一) |

**常用表达式**：

```bash
# 每小时整点
"0 0 * * * *"

# 每天上午9点
"0 0 9 * * *"

# 每天上午9点和下午6点
"0 0 9,18 * * *"

# 每周一上午9点
"0 0 9 * * 1"

# 每5分钟
"0 */5 * * * *"

# 每30秒（不建议，太频繁）
"*/30 * * * * *"
```

### 4. 任务管理

```bash
# 查看所有任务
curl -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs"

# 查看单个任务详情（替换为实际的 job_id）
curl -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/xxxxx"

# 取消任务（替换为实际的 job_id）
curl -X DELETE \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/xxxxx"
```

## 完整使用示例

### 场景：为用户设置一天的工作提醒

```bash
#!/bin/bash

# 配置（从用户提供的信息设置）
BARK_API_URL="${BARK_API_URL:-http://localhost:3000}"
BARK_PASSWORD="${BARK_API_PASSWORD:-}"

# 1. 立即通知今天的工作安排
echo "设置即时通知..."
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "今日工作安排",
    "body": "1. 完成代码审查\n2. 参加下午3点的产品评审会\n3. 提交周报",
    "sound": "bell",
    "group": "工作安排"
  }'

# 2. 会议前30分钟提醒
echo "设置会议提醒..."
FUTURE_TIME=$(date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ")
curl -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"会议提醒\",
    \"body\": \"30分钟后有产品评审会，请准备\",
    \"at\": \"${FUTURE_TIME}\"
  }"

# 3. 每天下午提醒喝水，最多提醒5天
echo "设置喝水提醒..."
curl -X POST "${BARK_API_URL}/schedule/cron" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "喝水提醒",
    "body": "工作再忙也要记得喝水哦",
    "cron": "0 0 15 * * *",
    "max_count": 5
  }'

echo "✅ 所有提醒已设置完成！"
```

## 常见问题

### Q: 调用 API 返回 401 Unauthorized？

检查：
1. 是否正确设置了 `Authorization: Bearer <password>` 头
2. 密码是否与服务器配置一致
3. 如果服务器未设置密码，可以不传 Authorization 头

### Q: 连接失败（Connection refused）？

检查：
1. 服务是否已启动：`curl ${BARK_API_URL}/health`
2. 地址和端口是否正确（默认 3000，但可能被修改）
3. 防火墙是否放行端口
4. 如果是远程服务器，确认网络可达：`ping <服务器IP>`

### Q: 一次性任务没有按时发送？

检查：
1. 时间格式必须是 ISO 8601 UTC 格式（带 Z 后缀）
2. 时间必须是未来时间
3. 服务器时区设置是否正确

```bash
# 验证时间格式
echo "当前 UTC 时间: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
```

### Q: 循环任务创建成功但不执行？

检查：
1. Cron 表达式必须是 6 位（秒 分 时 日 月 星期）
2. 服务器是否一直在运行
3. 查看服务器日志：`journalctl -u agent-bark-api -f` 或 `docker logs agent-bark-api`

### Q: 用户说没收到通知？

检查：
1. device_key 是否正确配置
2. 用户手机是否开启了 Bark 通知权限
3. 用户是否处于勿扰模式
4. 查看 API 返回结果中的 `success` 是否为 true

```bash
# 测试即时推送
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "测试",
    "body": "如果能收到这条消息，说明配置正确"
  }'
```

### Q: 如何取消已创建的提醒？

保存创建时返回的 `job_id`，然后调用删除接口：

```bash
# 假设 job_id 是 db253fcc-669e-49b1-a251-ab2e7dbb5357
curl -X DELETE \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/db253fcc-669e-49b1-a251-ab2e7dbb5357"
```

## 最佳实践

1. **先确认服务地址**：调用任何 API 前，先询问用户服务部署在哪里
2. **始终保存 job_id**：创建定时任务后保存 job_id，方便后续管理
3. **合理设置 max_count**：避免无限循环任务堆积
4. **使用分组**：通过 `group` 参数对通知分类，方便用户管理
5. **适度提醒**：避免过于频繁的推送打扰用户
6. **检查响应**：始终检查 API 返回结果中的 `success` 字段

```bash
# 示例：创建任务并保存 job_id
RESPONSE=$(curl -s -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{...}')

JOB_ID=$(echo "$RESPONSE" | grep -o '"job_id":"[^"]*"' | cut -d'"' -f4)
echo "任务已创建: $JOB_ID"
```
