use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};

pub struct PdfConverter;

impl DocumentConverter for PdfConverter {
    fn name(&self) -> &'static str {
        "PDF"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(info.mime_type.as_deref(), Some("application/pdf"))
            || matches!(info.extension.as_deref(), Some("pdf"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let pages = pdf_extract::extract_text_from_mem_by_pages(input)
            .map_err(|e| crate::Error::ConversionFailed(format!("PDF extraction failed: {e}")))?;

        let mut md = String::new();
        for (i, page_text) in pages.iter().enumerate() {
            let trimmed = page_text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if i > 0 {
                md.push_str("\n\n---\n\n");
            }
            md.push_str(trimmed);
        }

        let mut result = ConversionResult::new(md);
        result = result.with_metadata("page_count", pages.len().to_string());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_pdf() {
        let c = PdfConverter;
        let info = StreamInfo {
            mime_type: Some("application/pdf".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn rejects_non_pdf() {
        let c = PdfConverter;
        let info = StreamInfo {
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        assert!(!c.accepts(&info));
    }
}
