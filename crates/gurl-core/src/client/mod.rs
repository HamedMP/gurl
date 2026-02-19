mod request;
pub mod response;
mod timing;

pub use request::{Body, GurlRequest};
pub use response::{ResponseMeta, Timing, TlsInfo};

use crate::output::envelope::{Content, GurlResponse, RequestMeta};
use reqwest::Client;
use std::time::Instant;

pub struct GurlClient {
    http: Client,
    http_no_redirect: Client,
}

impl GurlClient {
    pub fn new() -> Result<Self, reqwest::Error> {
        let ua = format!("gurl/{}", env!("CARGO_PKG_VERSION"));
        let http = Client::builder().user_agent(&ua).build()?;
        let http_no_redirect = Client::builder()
            .user_agent(&ua)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            http_no_redirect,
        })
    }

    pub async fn execute(&self, req: GurlRequest) -> Result<GurlResponse, crate::Error> {
        let method = req.method.clone();
        let url = req.url.clone();
        let timestamp = chrono::Utc::now();

        let client = if req.follow_redirects {
            &self.http
        } else {
            &self.http_no_redirect
        };
        let mut builder = client.request(method.clone(), url.clone());
        builder = builder.headers(req.headers.clone());
        if let Some(body) = &req.body {
            match body {
                Body::Raw(data) => builder = builder.body(data.clone()),
                Body::Json(value) => builder = builder.json(value),
                Body::Form(params) => builder = builder.form(params),
            }
        }

        let start = Instant::now();
        let response = builder
            .send()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;
        let total_ms = start.elapsed().as_millis() as u64;

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("")
            .to_string();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let content_type = headers.get("content-type").cloned().unwrap_or_default();

        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| crate::Error::Request(e.to_string()))?;

        let body_str = String::from_utf8_lossy(&body_bytes).to_string();

        Ok(GurlResponse {
            gurl: env!("CARGO_PKG_VERSION").to_string(),
            request: RequestMeta {
                method: method.to_string(),
                url: url.to_string(),
                timestamp,
            },
            response: ResponseMeta {
                status,
                status_text,
                headers,
                timing: Timing {
                    dns_ms: None,
                    connect_ms: None,
                    tls_ms: None,
                    first_byte_ms: None,
                    total_ms,
                },
                tls: None,
            },
            content: Content {
                content_type: "raw".to_string(),
                original_type: content_type,
                title: None,
                body: serde_json::Value::String(body_str),
                metadata: None,
                links: None,
                images: None,
            },
        })
    }
}

impl Default for GurlClient {
    fn default() -> Self {
        Self::new().expect("failed to create HTTP client")
    }
}
