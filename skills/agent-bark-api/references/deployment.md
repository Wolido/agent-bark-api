# Agent Bark API 部署指南

## 前置条件

### 获取 Bark 设备密钥

1. iPhone/iPad 安装 Bark App（App Store 搜索 "Bark"）
2. 打开 App，点击右上角设置图标
3. 复制 "Device Key"（格式：`vWs4oJAgD59p8NdyhCDgn`）

## 部署方式

### 方式一：二进制部署（推荐）

```bash
# 1. 克隆并编译
git clone https://github.com/Wolido/agent-bark-api.git
cd agent-bark-api
cargo build --release

# 2. 创建配置
cp config.toml.example config.toml
nano config.toml
```

**配置示例:**
```toml
host = "0.0.0.0"
port = 3000
bark_url = "https://api.day.app"
device_key = "你的Bark设备密钥"  # 必填
password = "设置一个密码"          # 公网部署时必填
```

**启动:**
```bash
./target/release/agent-bark-api
```

### 方式二：Docker 部署

```bash
docker run -d \
  --name agent-bark-api \
  -p 3000:3000 \
  -e BARK_DEVICE_KEY="你的设备密钥" \
  -e BARK_PASSWORD="设置密码" \
  ghcr.io/wolido/agent-bark-api:latest
```

修改端口: `-p 8080:3000`

### 方式三：Systemd 部署（服务器长期运行）

```bash
# 1. 安装二进制
sudo cp target/release/agent-bark-api /usr/local/bin/
sudo chmod +x /usr/local/bin/agent-bark-api

# 2. 创建配置
sudo mkdir -p /etc/agent-bark-api
sudo nano /etc/agent-bark-api/config.toml

# 3. 创建服务文件
sudo nano /etc/systemd/system/agent-bark-api.service
```

**service 文件:**
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

**启动:**
```bash
sudo systemctl daemon-reload
sudo systemctl enable agent-bark-api
sudo systemctl start agent-bark-api
sudo systemctl status agent-bark-api
```

## 验证部署

```bash
curl http://localhost:3000/       # 应返回: Agent Bark API
curl http://localhost:3000/health # 应返回: OK
```

## 配置防火墙

```bash
# 放行端口（如使用 3000）
sudo ufw allow 3000
# 或
sudo firewall-cmd --add-port=3000/tcp --permanent
sudo firewall-cmd --reload
```

## 安全建议

- 公网部署务必设置 `password`
- 使用 HTTPS（配合反向代理如 Nginx）
- 定期更新 Bark App 和服务端
