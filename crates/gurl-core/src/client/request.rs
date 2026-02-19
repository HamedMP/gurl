use reqwest::Method;
use reqwest::header::HeaderMap;
use std::time::Duration;
use url::Url;

pub struct GurlRequest {
    pub method: Method,
    pub url: Url,
    pub headers: HeaderMap,
    pub body: Option<Body>,
    pub follow_redirects: bool,
    pub timeout: Option<Duration>,
}

pub enum Body {
    Raw(Vec<u8>),
    Json(serde_json::Value),
    Form(Vec<(String, String)>),
}

impl GurlRequest {
    pub fn get(url: Url) -> Self {
        Self {
            method: Method::GET,
            url,
            headers: HeaderMap::new(),
            body: None,
            follow_redirects: true,
            timeout: None,
        }
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }

    pub fn no_redirects(mut self) -> Self {
        self.follow_redirects = false;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}
