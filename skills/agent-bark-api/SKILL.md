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

### 1. 确认服务已部署

询问用户是否已部署 agent-bark-api 服务：

```bash
# 检查服务是否运行
curl http://localhost:3000/health
```

**如果返回 "OK"** → 服务已运行，跳到配置检查  
**如果连接失败** → 需要部署服务，继续阅读下文

### 2. 部署服务（如未部署）

指导用户完成以下步骤：

#### 方式一：快速部署（推荐）

```bash
# 1. 下载预编译二进制（让用户从 GitHub Releases 下载）
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
port = 3000
bark_url = "https://api.day.app"
device_key = "用户的Bark设备密钥"  # 必填
password = "设置一个密码"           # 公网部署时必填
```

#### 方式二：Docker 部署

```bash
docker run -d \
  --name agent-bark-api \
  -p 3000:3000 \
  -e BARK_DEVICE_KEY="用户的设备密钥" \
  -e BARK_PASSWORD="设置密码" \
  ghcr.io/wolido/agent-bark-api:latest
```

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

### 3. 获取 Bark 设备密钥

如果用户没有 device_key，指导用户：

1. 在 iPhone/iPad 上安装 Bark App（App Store 搜索 "Bark"）
2. 打开 App，点击右上角设置图标
3. 复制 "Device Key"（格式类似：`vWs4oJAgD59p8NdyhCDgn`）
4. 将 device_key 填入配置文件

### 4. 验证部署

```bash
# 测试服务运行
curl http://localhost:3000/
# 应返回：Agent Bark API

curl http://localhost:3000/health
# 应返回：OK
```

## 配置敏感信息

⚠️ **安全提醒**：device_key 和 password 是敏感信息，不应该：
- 硬编码在代码中
- 提交到 Git 仓库
- 在不安全的聊天中明文传输

### 推荐的配置方式

#### 方式一：环境变量（推荐）

```bash
export BARK_API_URL="http://localhost:3000"
export BARK_API_PASSWORD="你的密码"
```

#### 方式二：配置文件（用户本地）

在用户主目录创建 `.agent-bark-api` 文件：

```bash
echo 'BARK_API_URL=http://localhost:3000' > ~/.agent-bark-api
echo 'BARK_API_PASSWORD=你的密码' >> ~/.agent-bark-api
chmod 600 ~/.agent-bark-api  # 设置权限，仅用户可读
```

#### 方式三：Agent 配置

如果 Agent 支持配置系统，引导用户在配置中设置：

```json
{
  "agent_bark_api": {
    "url": "http://localhost:3000",
    "password": "你的密码"
  }
}
```

## API 调用指南

### 基础配置

```python
import os

BARK_API_URL = os.environ.get("BARK_API_URL", "http://localhost:3000")
BARK_PASSWORD = os.environ.get("BARK_API_PASSWORD", "")

headers = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {BARK_PASSWORD}"
}
```

### 1. 即时推送

适用场景：任务完成、错误告警、即时通知

```python
import requests

def send_notification(title: str, body: str, **kwargs):
    """
    发送即时推送通知
    
    Args:
        title: 通知标题
        body: 通知内容
        sound: 提示音 (可选，如 "bell", "alarm")
        group: 分组名称 (可选)
        level: 通知级别 (可选: "active", "timeSensitive", "passive")
        icon: 图标 URL (可选)
        url: 点击跳转链接 (可选)
        copy: 复制到剪贴板的内容 (可选)
        auto_copy: 是否自动复制 (可选，bool)
        badge: 角标数字 (可选，int)
    """
    data = {
        "title": title,
        "body": body,
        **kwargs
    }
    
    response = requests.post(
        f"{BARK_API_URL}/notify",
        headers=headers,
        json=data
    )
    
    result = response.json()
    if result.get("success"):
        print(f"✅ 通知已发送: {title}")
        return True
    else:
        print(f"❌ 发送失败: {result.get('error')}")
        return False

# 示例
send_notification(
    title="任务完成",
    body="数据处理已完成，共处理 1000 条记录",
    sound="bell",
    group="工作通知"
)
```

### 2. 一次性定时推送

适用场景：会议提醒、预约提醒、延迟通知

**⚠️ UTC 时间处理**：

```python
from datetime import datetime, timezone, timedelta

def schedule_once(title: str, body: str, minutes_later: int = 30):
    """
    在指定分钟后发送一次性通知
    
    Args:
        title: 通知标题
        body: 通知内容
        minutes_later: 多少分钟后发送（默认30分钟）
    
    Returns:
        job_id: 任务ID，可用于取消
    """
    # 计算未来时间（UTC）
    future_time = datetime.now(timezone.utc) + timedelta(minutes=minutes_later)
    
    # 格式化为 ISO 8601 UTC 格式（带 Z 后缀）
    at_time = future_time.strftime("%Y-%m-%dT%H:%M:%SZ")
    
    data = {
        "title": title,
        "body": body,
        "at": at_time
    }
    
    response = requests.post(
        f"{BARK_API_URL}/schedule/once",
        headers=headers,
        json=data
    )
    
    result = response.json()
    if result.get("success"):
        job_id = result["data"]["job_id"]
        print(f"✅ 定时任务已创建，ID: {job_id}")
        print(f"   将在 {minutes_later} 分钟后发送 ({at_time})")
        return job_id
    else:
        print(f"❌ 创建失败: {result.get('error')}")
        return None

# 示例：30分钟后提醒
job_id = schedule_once(
    title="会议提醒",
    body="15分钟后与客户开会，请准备",
    minutes_later=30
)
```

#### 时间格式详细说明

**必须使用 UTC 时间，格式为 ISO 8601**：

```python
# ✅ 正确格式
"2026-02-03T12:30:00Z"      # UTC 时间，带 Z 后缀

# ❌ 错误格式
"2026-02-03 12:30:00"       # 没有 T 和 Z
"2026-02-03T12:30:00+08:00" # 带时区偏移（不支持）
```

**本地时间转 UTC**：

```python
from datetime import datetime, timezone

# 假设用户说的是北京时间（UTC+8）
local_time_str = "2026-02-03 20:30:00"  # 北京时间

# 解析本地时间（需要知道用户时区）
user_timezone = timezone(timedelta(hours=8))  # 东八区
local_time = datetime.strptime(local_time_str, "%Y-%m-%d %H:%M:%M:%S")
local_time = local_time.replace(tzinfo=user_timezone)

# 转换为 UTC
utc_time = local_time.astimezone(timezone.utc)
utc_time_str = utc_time.strftime("%Y-%m-%dT%H:%M:%SZ")
# 结果: "2026-02-03T12:30:00Z"
```

### 3. 循环定时推送

适用场景：定期提醒、习惯养成、监控告警

```python
def schedule_cron(title: str, body: str, cron: str, max_count: int = None):
    """
    创建循环定时任务
    
    Args:
        title: 通知标题
        body: 通知内容
        cron: cron 表达式（6位：秒 分 时 日 月 星期）
        max_count: 最大执行次数（可选，达到后自动删除）
    
    Returns:
        job_id: 任务ID
    """
    data = {
        "title": title,
        "body": body,
        "cron": cron
    }
    
    if max_count:
        data["max_count"] = max_count
    
    response = requests.post(
        f"{BARK_API_URL}/schedule/cron",
        headers=headers,
        json=data
    )
    
    result = response.json()
    if result.get("success"):
        job_id = result["data"]["job_id"]
        print(f"✅ 循环任务已创建，ID: {job_id}")
        return job_id
    else:
        print(f"❌ 创建失败: {result.get('error')}")
        return None

# 常用 cron 表达式示例：

# 每小时整点
cron = "0 0 * * * *"  # 秒 分 时 日 月 星期

# 每天上午9点
cron = "0 0 9 * * *"

# 每天上午9点和下午6点
cron = "0 0 9,18 * * *"

# 每周一上午9点
cron = "0 0 9 * * 1"

# 每5分钟
cron = "0 */5 * * * *"

# 每30秒（不建议，太频繁）
cron = "*/30 * * * * *"

# 示例：每天提醒喝水，最多提醒3次
job_id = schedule_cron(
    title="健康提醒",
    body="该喝水了，保持身体健康",
    cron="0 0 9,14,17 * * *",  # 每天9点、14点、17点
    max_count=9  # 总共提醒9次（3天）
)
```

#### Cron 表达式格式

| 位置 | 含义 | 范围 | 示例 |
|------|------|------|------|
| 第1位 | 秒 | 0-59 | `0` 或 `*/10` |
| 第2位 | 分 | 0-59 | `0` 或 `*/5` |
| 第3位 | 时 | 0-23 | `9` 或 `9,18` |
| 第4位 | 日 | 1-31 | `*` |
| 第5位 | 月 | 1-12 | `*` |
| 第6位 | 星期 | 0-6 (0=周日) | `1` (周一) |

### 4. 任务管理

```python
def list_jobs():
    """查看所有定时任务"""
    response = requests.get(
        f"{BARK_API_URL}/jobs",
        headers=headers
    )
    result = response.json()
    
    if result.get("success"):
        jobs = result["data"]
        print(f"当前共有 {len(jobs)} 个任务：")
        for job in jobs:
            job_type = "一次性" if job.get("at") else "循环"
            time_info = job.get("at") or job.get("cron")
            print(f"  - [{job_type}] {job['notify']['title']} ({time_info})")
        return jobs
    return []

def get_job(job_id: str):
    """查看单个任务详情"""
    response = requests.get(
        f"{BARK_API_URL}/jobs/{job_id}",
        headers=headers
    )
    return response.json()

def cancel_job(job_id: str):
    """取消定时任务"""
    response = requests.delete(
        f"{BARK_API_URL}/jobs/{job_id}",
        headers=headers
    )
    result = response.json()
    
    if result.get("success"):
        print(f"✅ 任务已取消: {job_id}")
        return True
    else:
        print(f"❌ 取消失败: {result.get('error')}")
        return False
```

## 完整使用示例

### 场景：为用户设置一天的工作提醒

```python
class AgentBarkClient:
    def __init__(self, base_url: str = "http://localhost:3000", password: str = ""):
        self.base_url = base_url
        self.headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {password}"
        }
    
    def notify(self, title: str, body: str, **kwargs):
        """即时通知"""
        data = {"title": title, "body": body, **kwargs}
        response = requests.post(f"{self.base_url}/notify", headers=self.headers, json=data)
        return response.json()
    
    def remind_in(self, minutes: int, title: str, body: str):
        """在几分钟后提醒"""
        at = (datetime.now(timezone.utc) + timedelta(minutes=minutes)).strftime("%Y-%m-%dT%H:%M:%SZ")
        data = {"title": title, "body": body, "at": at}
        response = requests.post(f"{self.base_url}/schedule/once", headers=self.headers, json=data)
        return response.json()
    
    def daily_reminder(self, hour: int, minute: int, title: str, body: str, max_count: int = None):
        """每天定时提醒"""
        cron = f"0 {minute} {hour} * * *"
        data = {"title": title, "body": body, "cron": cron}
        if max_count:
            data["max_count"] = max_count
        response = requests.post(f"{self.base_url}/schedule/cron", headers=self.headers, json=data)
        return response.json()

# 使用示例
client = AgentBarkClient(
    base_url=os.environ.get("BARK_API_URL", "http://localhost:3000"),
    password=os.environ.get("BARK_API_PASSWORD", "")
)

# 1. 立即通知今天的工作安排
client.notify(
    title="今日工作安排",
    body="1. 完成代码审查\n2. 参加下午3点的产品评审会\n3. 提交周报",
    sound="bell",
    group="工作安排"
)

# 2. 会议前30分钟提醒
client.remind_in(
    minutes=150,  # 现在是9点，会议是11点半，差2.5小时
    title="会议提醒",
    body="30分钟后有产品评审会，请准备"
)

# 3. 每天下午提醒喝水（最多提醒5天）
client.daily_reminder(
    hour=15,
    minute=0,
    title="喝水提醒",
    body="工作再忙也要记得喝水哦",
    max_count=5
)

print("✅ 所有提醒已设置完成！")
```

## 常见问题

### Q: 调用 API 返回 401 Unauthorized？

检查：
1. 是否正确设置了 `Authorization: Bearer <password>` 头
2. 密码是否与服务器配置一致
3. 如果服务器未设置密码，可以不传 Authorization 头

### Q: 一次性任务没有按时发送？

检查：
1. 时间格式必须是 ISO 8601 UTC 格式（带 Z 后缀）
2. 时间必须是未来时间
3. 服务器时区设置是否正确

### Q: 循环任务创建成功但不执行？

检查：
1. Cron 表达式必须是 6 位（秒 分 时 日 月 星期）
2. 服务器是否一直在运行
3. 查看服务器日志：`journalctl -u agent-bark-api -f`

### Q: 用户说没收到通知？

检查：
1. device_key 是否正确配置
2. 用户手机是否开启了 Bark 通知权限
3. 用户是否处于勿扰模式
4. 查看 API 返回结果中的 `code` 是否为 200

### Q: 如何取消已创建的提醒？

保存创建时返回的 `job_id`，调用删除接口：

```python
requests.delete(f"{BARK_API_URL}/jobs/{job_id}", headers=headers)
```

## 最佳实践

1. **始终保存 job_id**：创建定时任务后保存 job_id，方便后续管理
2. **合理设置 max_count**：避免无限循环任务堆积
3. **使用分组**：通过 `group` 参数对通知分类，方便用户管理
4. **适度提醒**：避免过于频繁的推送打扰用户
5. **错误处理**：始终检查 API 返回结果，处理错误情况
