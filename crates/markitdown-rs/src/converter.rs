use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct StreamInfo {
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub charset: Option<String>,
    pub filename: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub title: Option<String>,
    pub body: String,
    pub metadata: HashMap<String, String>,
}

impl ConversionResult {
    pub fn new(body: impl Into<String>) -> Self {
        Self {
            title: None,
            body: body.into(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

pub trait DocumentConverter: Send + Sync {
    fn name(&self) -> &'static str;
    fn accepts(&self, stream_info: &StreamInfo) -> bool;
    fn convert(&self, input: &[u8], stream_info: &StreamInfo) -> crate::Result<ConversionResult>;
}
