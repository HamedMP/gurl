use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub status: u16,
    pub status_text: String,
    pub headers: std::collections::HashMap<String, String>,
    pub timing: Timing,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<TlsInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Timing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_byte_ms: Option<u64>,
    pub total_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TlsInfo {
    pub version: String,
    pub cipher: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cert_issuer: Option<String>,
}
