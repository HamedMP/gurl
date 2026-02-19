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

        // Step 1: Try to extract the main content element directly.
        // Many doc sites use <article>, <main>, or [role="main"].
        // If we find one with substantial content, use it instead of full-page readability.
        let title = extract_title(&html);
        if let Some(main_html) = extract_main_element(&html) {
            let md = htmd::convert(&main_html)
                .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;
            let trimmed = md.trim();
            if !trimmed.is_empty() && trimmed.len() > 100 {
                let mut result = ConversionResult::new(trimmed);
                if let Some(t) = title.clone() {
                    result = result.with_title(t);
                }
                return Ok(result);
            }
        }

        // Step 2: Fallback to readability on noise-stripped HTML
        let cleaned = strip_noise(&html);
        let readability = readabilityrs::Readability::new(&cleaned, url, None)
            .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;

        let content_html = match readability.parse() {
            Some(article) => article.content.unwrap_or_else(|| cleaned.to_string()),
            None => cleaned.to_string(),
        };

        let md = htmd::convert(&content_html)
            .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;

        let trimmed = md.trim();
        if trimmed.is_empty() {
            return Err(crate::Error::ConversionFailed(
                "no content extracted".to_string(),
            ));
        }

        let mut result = ConversionResult::new(trimmed);
        if let Some(t) = title {
            result = result.with_title(t);
        }
        Ok(result)
    }
}

fn extract_title(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);

    // Try <title> tag
    if let Ok(sel) = Selector::parse("title") {
        if let Some(el) = doc.select(&sel).next() {
            let t = el.text().collect::<String>();
            let t = t.trim().to_string();
            if !t.is_empty() {
                return Some(t);
            }
        }
    }
    None
}

fn extract_main_element(html: &str) -> Option<String> {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);

    let selectors = [
        "article#content-container",
        "article[role='main']",
        "main article",
        "main",
        "article",
        "[role='main']",
    ];

    let mut best: Option<String> = None;
    let mut best_len = 0;

    for sel_str in &selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            if let Some(el) = doc.select(&selector).next() {
                let inner = el.inner_html();
                if inner.len() > best_len {
                    best_len = inner.len();
                    best = Some(inner);
                }
            }
        }
    }

    // Only return if we found substantial content
    if best_len > 200 { best } else { None }
}

fn strip_noise(html: &str) -> String {
    use scraper::{Html, Selector};

    // Regex-based removal is simpler and avoids tree-walking complexity.
    // We find noise elements via scraper, collect their outer HTML, then remove them.
    let doc = Html::parse_document(html);

    let noise_selectors = [
        // Cookie/consent banners
        "[class*='cookie']",
        "[id*='cookie']",
        "[class*='consent']",
        "[id*='consent']",
        "[class*='gdpr']",
        "[id*='gdpr']",
        // Navigation and chrome
        "nav",
        "header",
        "footer",
        "[role='navigation']",
        "[role='banner']",
        "[role='contentinfo']",
        // Sidebars
        "[class*='sidebar']",
        "[id*='sidebar']",
        "aside",
        // Overlays and modals
        "[class*='overlay']",
        "[class*='modal']",
        "[class*='popup']",
        // Script and style
        "script",
        "style",
        "noscript",
        "svg",
    ];

    let mut fragments_to_remove = Vec::new();

    for sel_str in &noise_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for el in doc.select(&selector) {
                fragments_to_remove.push(el.html());
            }
        }
    }

    // Sort longest first so we remove outer elements before inner ones get confused
    fragments_to_remove.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut cleaned = html.to_string();
    for fragment in &fragments_to_remove {
        if let Some(pos) = cleaned.find(fragment.as_str()) {
            cleaned.replace_range(pos..pos + fragment.len(), "");
        }
    }

    cleaned
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
    fn strips_cookie_banner() {
        let c = HtmlConverter;
        let info = StreamInfo {
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        let html = r#"
        <html>
        <body>
            <div class="cookie-consent">
                <p>We use cookies to improve your experience.</p>
                <button>Accept</button>
            </div>
            <article>
                <h1>Real Article</h1>
                <p>This is the actual content that matters to agents.</p>
            </article>
        </body>
        </html>
        "#;
        let result = c.convert(html.as_bytes(), &info).unwrap();
        assert!(result.body.contains("Real Article"));
        assert!(!result.body.contains("cookie"));
    }

    #[test]
    fn strips_nav_and_footer() {
        let c = HtmlConverter;
        let info = StreamInfo {
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        let html = r#"
        <html>
        <body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <article>
                <h1>Main Content</h1>
                <p>This is what the agent should see.</p>
            </article>
            <footer>Copyright 2025</footer>
        </body>
        </html>
        "#;
        let result = c.convert(html.as_bytes(), &info).unwrap();
        assert!(result.body.contains("Main Content"));
        assert!(!result.body.contains("Copyright"));
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
