use anyhow::Result;
use reqwest::{Client, Method};
use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FetchResponse {
    pub success: bool,

    #[allow(dead_code)]
    pub title: Option<String>,

    pub html: Option<String>,
    pub txtnav_text: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct HTTPClient {
    url: String,
    client: Client,
}

pub enum TxtNavMode {
    Include,
    Exclude,
}

impl HTTPClient {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::new();
        return Ok(Self {
            url: url.to_string(),
            client,
        });
    }

    pub async fn call(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<String> {
        let request = self
            .client
            .request(method, format!("{}{}", self.url, endpoint));

        let request = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        };

        let response = request.send().await?;

        let text = response.text().await?;

        return Ok(text);
    }

    pub async fn fetch(&self, url: &str, txtnav_mode: TxtNavMode) -> Result<String> {
        let include_txtnav = match txtnav_mode {
            TxtNavMode::Include => true,
            TxtNavMode::Exclude => false,
        };

        self.call(
            Method::POST,
            "/fetch",
            Some(json!({
                "url": url,
                "txtnav": include_txtnav
            })),
        )
        .await
    }
}
