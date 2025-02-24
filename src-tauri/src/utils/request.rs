/*
RTLauncher, a third-party Minecraft launcher built with the newest
technology and provides innovative funtionalities
Copyright (C) 2025 lutouna

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

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
