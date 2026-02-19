---
name: agent-bark-api
description: Use when sending push notifications to user's iPhone - for reminders, alerts, scheduled notifications, or recurring messages.
---

# Agent Bark API

向用户 iPhone 发送推送通知。

## 何时使用

**适合:**
- 设置提醒（"30分钟后开会"）
- 定时推送（"每天9点推送日报"）
- 循环通知（"每小时提醒喝水"）
- 即时通知（"任务完成"、"出错告警"）

**不适合:**
- 用户未安装 Bark App
- 单向推送足够（无需复杂交互）

## 前置条件

### 1. 获取服务地址

询问用户：
```
请提供 Agent Bark API 服务地址（如 http://192.168.1.100:3000）
如有密码也请提供
```

**可能地址:**
- 本机: `http://localhost:3000`
- 局域网: `http://192.168.1.xxx:3000`
- 远程: `http://1.2.3.4:8080`
- 域名: `https://bark.example.com`

### 2. 保存配置

```bash
# 保存配置到 skill 目录
cat > ~/.config/agents/skills/agent-bark-api/config.env << 'EOF'
BARK_API_URL="http://xxx.xxx.xxx.xxx:3000"
BARK_PASSWORD="密码（如有）"
EOF
```

### 3. 检查服务

```bash
# 读取配置
source ~/.config/agents/skills/agent-bark-api/config.env 2>/dev/null || true

# 检查服务
curl "${BARK_API_URL}/health"
# 应返回: OK
```

**服务未部署?** 参见 `references/deployment.md`

## API 调用

### 即时推送

```bash
source ~/.config/agents/skills/agent-bark-api/config.env 2>/dev/null || true

curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "任务完成",
    "body": "数据处理已完成！",
    "sound": "bell",
    "group": "通知分组"
  }'
```

**可选参数:**
- `sound`: 提示音 (`bell`, `alarm`, `glass`)
- `group`: 通知分组
- `level`: 级别 (`active`, `timeSensitive`, `passive`)
- `badge`: 角标数字
- `url`: 点击跳转链接

### 一次性定时推送

```bash
# 计算30分钟后的 UTC 时间（macOS）
FUTURE_TIME=$(date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ")

# Linux
# FUTURE_TIME=$(date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ")

curl -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"会议提醒\",
    \"body\": \"30分钟后开会\",
    \"at\": \"${FUTURE_TIME}\"
  }"
```

**时间格式:** ISO 8601 UTC，如 `2026-02-03T12:30:00Z`

### 循环定时推送

```bash
# 每天上午9点提醒，最多5次
curl -X POST "${BARK_API_URL}/schedule/cron" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "喝水提醒",
    "body": "记得喝水哦",
    "cron": "0 0 9 * * *",
    "max_count": 5
  }'
```

**Cron 格式:** `秒 分 时 日 月 星期`

常用表达式:
- 每天9点: `0 0 9 * * *`
- 每小时: `0 0 * * * *`
- 每5分钟: `0 */5 * * * *`
- 每周一9点: `0 0 9 * * 1`

### 任务管理

```bash
# 查看所有任务
curl -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs"

# 取消任务
curl -X DELETE \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/JOB_ID"
```

## 完整示例

```bash
#!/bin/bash
source ~/.config/agents/skills/agent-bark-api/config.env 2>/dev/null || true

if [ -z "$BARK_API_URL" ]; then
    echo "错误：未配置 BARK_API_URL"
    exit 1
fi

# 即时通知
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "今日安排",
    "body": "1.代码审查 2.产品评审 3.提交周报",
    "group": "工作"
  }'

# 30分钟后会议提醒
FUTURE_TIME=$(date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ")
RESPONSE=$(curl -s -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"会议提醒\",
    \"body\": \"30分钟后有会议\",
    \"at\": \"${FUTURE_TIME}\"
  }")

# 保存 job_id
JOB_ID=$(echo "$RESPONSE" | grep -o '"job_id":"[^"]*"' | cut -d'"' -f4)
echo "提醒已设置，任务ID: $JOB_ID"
```

## 常见问题

```yaml
常见问题:
  - 问题: 401 Unauthorized
    原因: 密码错误或未传
    解决: 检查 Authorization 头
  - 问题: Connection refused
    原因: 服务未启动
    解决: '`curl ${BARK_API_URL}/health`'
  - 问题: 时间格式错误
    原因: 非 UTC 格式
    解决: '`date -u +"%Y-%m-%dT%H:%M:%SZ"`'
  - 问题: 未收到通知
    原因: device_key 错误
    解决: 检查 Bark App 中的 Device Key
```

**详细参考:**
- 部署指南: `references/deployment.md`
- API 完整文档: `references/api-reference.md`
