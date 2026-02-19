use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};

pub struct HtmlConverter;

impl DocumentConverter for HtmlConverter {
    fn name(&self) -> &'static str {
        "HTML"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("text/html" | "application/xhtml+xml")
        ) || matches!(info.extension.as_deref(), Some("html" | "htm" | "xhtml"))
    }

    fn convert(&self, input: &[u8], info: &StreamInfo) -> crate::Result<ConversionResult> {
        let html = String::from_utf8_lossy(input);
        let url = info.url.as_deref();

        // Step 1: Extract main content with readability
        let readability = readabilityrs::Readability::new(&html, url, None)
            .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;

        let (title, content_html) = match readability.parse() {
            Some(article) => {
                let title = article.title.filter(|t| !t.is_empty());
                let content = article.content.unwrap_or_else(|| html.into_owned());
                (title, content)
            }
            None => {
                // Readability couldn't extract — fall back to full HTML
                (None, html.into_owned())
            }
        };

        // Step 2: Convert HTML to Markdown
        let md = htmd::convert(&content_html)
            .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;

        let mut result = ConversionResult::new(md.trim());
        if let Some(t) = title {
            result = result.with_title(t);
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_html() {
        let c = HtmlConverter;
        let info = StreamInfo {
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_by_extension() {
        let c = HtmlConverter;
        let info = StreamInfo {
            extension: Some("html".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn converts_simple_html() {
        let c = HtmlConverter;
        let info = StreamInfo {
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <article>
                <h1>Hello World</h1>
                <p>This is a test paragraph with <strong>bold</strong> text.</p>
            </article>
        </body>
        </html>
        "#;
        let result = c.convert(html.as_bytes(), &info).unwrap();
        assert!(result.body.contains("Hello World"));
        assert!(result.body.contains("bold"));
    }

    #[test]
    fn rejects_non_html() {
        let c = HtmlConverter;
        let info = StreamInfo {
            mime_type: Some("application/json".into()),
            ..Default::default()
        };
        assert!(!c.accepts(&info));
    }
}
