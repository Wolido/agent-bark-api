# Agent Bark API 完整参考

## 基础变量

```bash
BARK_API_URL="http://xxx.xxx.xxx.xxx:3000"
BARK_PASSWORD="密码（如有）"
```

## 即时推送 /notify

```bash
curl -X POST "${BARK_API_URL}/notify" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "标题",
    "body": "正文内容",
    "sound": "bell",
    "group": "分组名",
    "level": "active",
    "icon": "图标URL",
    "url": "点击跳转URL",
    "copy": "复制内容",
    "auto_copy": false,
    "badge": 1
  }'
```

**参数说明:**
- `title` (必填): 通知标题
- `body` (必填): 通知正文
- `sound`: 提示音 (`bell`, `alarm`, `glass`, `jump` 等)
- `group`: 通知分组名称
- `level`: 通知级别 (`active`, `timeSensitive`, `passive`)
- `icon`: 图标 URL
- `url`: 点击跳转链接
- `copy`: 复制到剪贴板的内容
- `auto_copy`: 是否自动复制 (布尔值)
- `badge`: 角标数字

## 一次性定时推送 /schedule/once

```bash
# macOS - 30分钟后
FUTURE_TIME=$(date -u -v+30M +"%Y-%m-%dT%H:%M:%SZ")

# Linux - 30分钟后
FUTURE_TIME=$(date -u -d "+30 minutes" +"%Y-%m-%dT%H:%M:%SZ")

curl -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d "{
    \"title\": \"会议提醒\",
    \"body\": \"30分钟后开会\",
    \"at\": \"${FUTURE_TIME}\"
  }"
```

**时间格式:**
- ✅ 正确: `2026-02-03T12:30:00Z` (UTC，带 Z 后缀)
- ❌ 错误: `2026-02-03 12:30:00` (没有 T 和 Z)
- ❌ 错误: `2026-02-03T20:30:00+08:00` (不支持时区偏移)

**更多时间计算:**
```bash
# macOS
# 明天上午9点 UTC
date -u -v+1d -v9H -v0M -v0S +"%Y-%m-%dT%H:%M:%SZ"

# Linux
# 明天上午9点 UTC
date -u -d "tomorrow 09:00" +"%Y-%m-%dT%H:%M:%SZ"
```

**响应:**
```json
{
  "success": true,
  "data": {
    "job_id": "db253fcc-669e-49b1-a251-ab2e7dbb5357"
  }
}
```

## 循环定时推送 /schedule/cron

```bash
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

| 位置 | 含义 | 范围 | 示例 |
|------|------|------|------|
| 1 | 秒 | 0-59 | `0` 或 `*/10` |
| 2 | 分 | 0-59 | `0` 或 `*/5` |
| 3 | 时 | 0-23 | `9` 或 `9,18` |
| 4 | 日 | 1-31 | `*` |
| 5 | 月 | 1-12 | `*` |
| 6 | 星期 | 0-6 (0=周日) | `1` (周一) |

**常用表达式:**
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

# 每30秒
"*/30 * * * * *"
```

## 任务管理

### 查看所有任务

```bash
curl -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs"
```

### 查看单个任务

```bash
curl -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/JOB_ID"
```

### 取消任务

```bash
curl -X DELETE \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  "${BARK_API_URL}/jobs/JOB_ID"
```

## 错误处理

| 状态码 | 含义 | 解决 |
|--------|------|------|
| 200 | 成功 | - |
| 400 | 请求参数错误 | 检查 JSON 格式和必填字段 |
| 401 | 未授权 | 检查 Authorization 头 |
| 404 | 任务不存在 | 检查 job_id |
| 500 | 服务器错误 | 查看服务端日志 |

## 最佳实践

1. **始终保存 job_id**: 创建定时任务后保存 job_id
2. **合理设置 max_count**: 避免无限循环任务堆积
3. **使用分组**: 通过 `group` 参数对通知分类
4. **检查响应**: 检查 `success` 字段确认成功
5. **UTC 时间**: 始终使用 UTC 时间格式

```bash
# 创建任务并保存 job_id
RESPONSE=$(curl -s -X POST "${BARK_API_URL}/schedule/once" \
  -H "Authorization: Bearer ${BARK_PASSWORD}" \
  -H "Content-Type: application/json" \
  -d '{...}')

JOB_ID=$(echo "$RESPONSE" | grep -o '"job_id":"[^"]*"' | cut -d'"' -f4)
echo "任务ID: $JOB_ID"
```
