use anyhow::Result;
use reqwest::{Client, Method};
use serde_json::Value;
use serde_json::json;

#[derive(Debug)]
pub struct HTTPClient {
    url: String,
    client: Client,
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

    pub async fn fetch(&self, url: &str) -> Result<String> {
        self.call(
            Method::POST,
            "/fetch",
            Some(json!({
                "url": url
            })),
        )
        .await
    }
}
