use crate::utils::request;
use log::info;
use open;
use request::Request;
use std::error::Error;
use std::io::{BufRead, Write};
use url;

// Tauri共享
#[tauri::command]
pub async fn get_code() -> Result<String, String> {
    let login = Login::new();
    login.get_code().await.map_err(|e| e.to_string())
}

pub struct Login {
    request: request::Request,
}

impl Login {
    pub fn new() -> Self {
        Self {
            request: Request::new(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize?client_id=1662e9cb-e526-4bea-8237-11526075b7f3&response_type=code&redirect_uri=http://localhost:40323&response_mode=query&scope=XboxLive.signin offline_access"
                    .to_string(),
            )
        }
    }

    // 获取授权码
    pub async fn get_code(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        // 监听40323端口
        let listener = match std::net::TcpListener::bind("localhost:40323") {
            Ok(l) => {
                log::info!("成功在 localhost:40323 启动监听服务器");
                l
            }
            Err(e) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("无法启动监听服务器: {}", e),
                )));
            }
        };

        // 服务器就绪后再打开浏览器
        if let Err(e) = open::that(self.request.get_url()) {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("无法打开浏览器: {}", e),
            )));
        }

        info!("等待浏览器回调...");
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("收到来自 {} 的连接", addr);
                let buf_reader = std::io::BufReader::new(&stream);
                let request_line = buf_reader.lines().next().ok_or("无法读取请求")??;
                info!("收到请求: {}", request_line);

                // 解析 URL 中的 code 参数
                if let Some(code) = request_line
                    .split(' ')
                    .nth(1)
                    .and_then(|path| {
                        url::Url::parse(&format!("http://localhost:40323{}", path)).ok()
                    })
                    .and_then(|url| {
                        url.query_pairs()
                            .find(|(key, _)| key == "code")
                            .map(|(_, value)| value.to_string())
                    })
                {
                    // html结构
                    let response = "HTTP/1.1 200 OK\r\n\
                        Content-Type: text/html; charset=utf-8\r\n\
                        \r\n\
                        <!DOCTYPE html>\
                        <html lang=\"zh-CN\">\
                        <head>\
                            <meta charset=\"utf-8\">\
                            <title>授权成功</title>\
                            <style>\
                                body { font-family: system-ui, -apple-system, sans-serif; \
                                       display: flex; justify-content: center; \
                                       align-items: center; height: 100vh; margin: 0; \
                                       background-color: #f0f2f5; }\
                                .container { text-align: center; padding: 2rem; \
                                           background: white; border-radius: 8px; \
                                           box-shadow: 0 2px 4px rgba(0,0,0,0.1); }\
                            </style>\
                        </head>\
                        <body>\
                            <div class=\"container\">\
                                <h1>授权成功</h1>\
                                <p>请返回Real Time Launcher继续操作</p>\
                            </div>\
                        </body>\
                        </html>";
                    let mut writer = std::io::BufWriter::new(&stream);
                    writer.write_all(response.as_bytes())?;
                    writer.flush()?;

                    // 构建json
                    let json_response = serde_json::json!({
                        "code": code,
                        "status": "success",
                        "message": "授权成功"
                    });
                    return Ok(json_response.to_string());
                }
            }
            Err(e) => {
                let json_response = serde_json::json!({
                    "code": "",
                    "status": "error",
                    "message": format!("接受连接失败: {}", e)
                });
                return Ok(json_response.to_string());
            }
        }

        let json_response = serde_json::json!({
            "code": "",
            "status": "error",
            "message": "未能获取授权码"
        });
        Ok(json_response.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_code() {
        let login = Login::new();
        let code = login.get_code().await.unwrap();
        println!("获取到的授权码: {}", code);
    }
}
