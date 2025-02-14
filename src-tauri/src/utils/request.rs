// ***
// 请求工具类
// ***

use reqwest::Client;
use std::error::Error;


#[derive(Clone)]
pub struct Request {
    url: String,
    client: Client,
}

impl Request {
    pub fn new(url: String) -> Self {
        Self {
            url,
            client: Client::new(),
        }
    }

    // 发送get请求
    pub async fn fetch_get(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let response = reqwest::get(&self.url).await?;
        let body = response.text().await?;
        Ok(body)
    }

    // 获取url
    pub fn get_url(&self) -> &str {
        &self.url
    }
}
