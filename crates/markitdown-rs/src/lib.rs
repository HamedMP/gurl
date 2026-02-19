pub mod converter;
pub mod converters;
pub mod detection;
pub mod utils;

use converter::{ConversionResult, DocumentConverter, StreamInfo};
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no converter found for input")]
    NoConverterFound,
    #[error("conversion failed: {0}")]
    ConversionFailed(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MarkItDown {
    converters: Vec<Box<dyn DocumentConverter>>,
}

impl Default for MarkItDown {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkItDown {
    pub fn new() -> Self {
        let mut m = Self {
            converters: Vec::new(),
        };
        m.register_defaults();
        m
    }

    fn register_defaults(&mut self) {
        // Order matters: more specific converters first, generic last.

        // Binary document formats
        #[cfg(feature = "pdf")]
        self.register(Box::new(converters::pdf::PdfConverter));

        #[cfg(feature = "docx")]
        self.register(Box::new(converters::docx::DocxConverter));

        #[cfg(feature = "xlsx")]
        self.register(Box::new(converters::xlsx::XlsxConverter));

        #[cfg(feature = "pptx")]
        self.register(Box::new(converters::pptx::PptxConverter));

        #[cfg(feature = "epub")]
        self.register(Box::new(converters::epub::EpubConverter));

        #[cfg(feature = "outlook")]
        self.register(Box::new(converters::outlook_msg::OutlookMsgConverter));

        #[cfg(feature = "image")]
        self.register(Box::new(converters::image::ImageConverter));

        #[cfg(feature = "zip-convert")]
        self.register(Box::new(converters::zip::ZipConverter));

        // Specific text-based formats
        self.register(Box::new(converters::ipynb::IpynbConverter));

        #[cfg(feature = "csv-convert")]
        self.register(Box::new(converters::csv::CsvConverter));

        #[cfg(feature = "rss")]
        self.register(Box::new(converters::rss::RssConverter));

        // Wikipedia before generic HTML (more specific URL-based match)
        #[cfg(feature = "wikipedia")]
        self.register(Box::new(converters::wikipedia::WikipediaConverter));

        // General HTML
        #[cfg(feature = "html")]
        self.register(Box::new(converters::html::HtmlConverter));

        // Catch-all for text
        self.register(Box::new(converters::plain_text::PlainTextConverter));
    }

    pub fn register(&mut self, converter: Box<dyn DocumentConverter>) {
        self.converters.push(converter);
    }

    pub fn convert_bytes(
        &self,
        input: &[u8],
        info: &StreamInfo,
    ) -> Result<ConversionResult> {
        let info = detection::detect(input, info);

        for converter in &self.converters {
            if converter.accepts(&info) {
                return converter.convert(input, &info);
            }
        }

        Err(Error::NoConverterFound)
    }

    pub fn convert_file(&self, path: &Path) -> Result<ConversionResult> {
        let data = std::fs::read(path)?;
        let info = StreamInfo {
            filename: path.file_name().map(|n| n.to_string_lossy().into_owned()),
            extension: path.extension().map(|e| e.to_string_lossy().into_owned()),
            ..Default::default()
        };
        self.convert_bytes(&data, &info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_roundtrip() {
        let m = MarkItDown::new();
        let info = StreamInfo {
            mime_type: Some("text/plain".into()),
            ..Default::default()
        };
        let result = m.convert_bytes(b"Hello, world!", &info).unwrap();
        assert_eq!(result.body, "Hello, world!");
    }

    #[test]
    fn unknown_binary_returns_error() {
        let m = MarkItDown::new();
        let info = StreamInfo {
            mime_type: Some("application/octet-stream".into()),
            ..Default::default()
        };
        assert!(m.convert_bytes(&[0xFF, 0xFE, 0x00], &info).is_err());
    }

    #[test]
    fn convert_file_not_found() {
        let m = MarkItDown::new();
        assert!(m.convert_file(Path::new("/nonexistent/file.txt")).is_err());
    }
}
