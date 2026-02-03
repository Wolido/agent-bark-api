use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRequest {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_copy: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarkResponse {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Notifier {
    client: Client,
    pub base_url: String,
    pub device_key: String,
}

impl Notifier {
    pub fn new(base_url: String, device_key: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            device_key,
        }
    }

    pub async fn send(&self, req: &NotifyRequest) -> anyhow::Result<BarkResponse> {
        let url = format!("{}/{}", self.base_url, self.device_key);
        
        info!("Sending notification to {}: title={}", url, req.title);
        
        let response = self.client
            .post(&url)
            .json(req)
            .send()
            .await?;
        
        let status = response.status();
        let body = response.text().await?;
        
        if !status.is_success() {
            error!("Bark API error: status={}, body={}", status, body);
            return Err(anyhow::anyhow!("Bark API error: {}", body));
        }
        
        let bark_resp: BarkResponse = serde_json::from_str(&body)?;
        
        if bark_resp.code != 200 {
            error!("Bark returned error: {:?}", bark_resp);
            return Err(anyhow::anyhow!("Bark error: {}", bark_resp.message));
        }
        
        info!("Notification sent successfully: {:?}", bark_resp);
        Ok(bark_resp)
    }


}
