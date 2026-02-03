use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    // Server settings
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    
    // Bark settings
    #[serde(default = "default_bark_url")]
    pub bark_url: String,
    #[serde(default)]
    pub device_key: String,
    
    // Auth settings
    #[serde(default)]
    pub password: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            bark_url: default_bark_url(),
            device_key: String::new(),
            password: String::new(),
        }
    }
}

impl AppConfig {
    /// 验证配置是否有效
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.device_key.is_empty() {
            return Err(anyhow::anyhow!(
                "设备密钥 device_key 不能为空。\
                请在 config.toml 中设置，或通过环境变量 BARK_DEVICE_KEY 传入。"
            ));
        }
        Ok(())
    }
}

fn default_bark_url() -> String {
    "https://api.day.app".to_string()
}



fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            // 环境变量前缀 BARK，比如 BARK_DEVICE_KEY
            .add_source(config::Environment::with_prefix("BARK"))
            .build()?;
        
        Ok(settings.try_deserialize().unwrap_or_default())
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .unwrap_or_else(|_| "0.0.0.0:3000".parse().unwrap())
    }
}
