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
        let title = extract_title(&html);

        // Step 1: Next.js RSC payload extraction.
        // Modern Next.js App Router sites embed content as RSC data, not in the DOM.
        // Try this first — it gives the cleanest article content without sidebar noise.
        if let Some(rsc_md) = extract_nextjs_rsc(&html) {
            if rsc_md.len() > 200 {
                let mut result = ConversionResult::new(&rsc_md);
                if let Some(t) = title.clone() {
                    result = result.with_title(t);
                }
                return Ok(result);
            }
        }

        // Step 2: Direct element extraction (article, main, etc.)
        if let Some(main_html) = extract_main_element(&html) {
            let md = htmd::convert(&main_html)
                .map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;
            let trimmed = md.trim();
            if !trimmed.is_empty() && trimmed.len() > 100 && !is_mostly_nav(trimmed) {
                let mut result = ConversionResult::new(trimmed);
                if let Some(t) = title.clone() {
                    result = result.with_title(t);
                }
                return Ok(result);
            }
        }

        // Step 3: Readability on noise-stripped HTML
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

    // Strip inline scripts/styles from extracted element
    best.filter(|_| best_len > 200)
        .map(|h| strip_inline_noise(&h))
}

fn strip_inline_noise(html: &str) -> String {
    use scraper::{Html, Selector};
    let doc = Html::parse_fragment(html);
    let tags = ["script", "style", "noscript", "svg"];
    let mut fragments = Vec::new();
    for tag in &tags {
        if let Ok(sel) = Selector::parse(tag) {
            for el in doc.select(&sel) {
                fragments.push(el.html());
            }
        }
    }
    fragments.sort_by(|a, b| b.len().cmp(&a.len()));
    let mut cleaned = html.to_string();
    for frag in &fragments {
        if let Some(pos) = cleaned.find(frag.as_str()) {
            cleaned.replace_range(pos..pos + frag.len(), "");
        }
    }
    cleaned
}

/// Check if markdown is predominantly navigation links rather than content.
/// Uses two signals: line-level link ratio and character-level prose ratio.
fn is_mostly_nav(md: &str) -> bool {
    let mut link_lines = 0;
    let mut nav_text_lines = 0;
    let mut total_lines = 0;
    let mut longest_prose = 0;

    for line in md.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        total_lines += 1;

        let content = trimmed.trim_start_matches(['*', '-', '+', ' ']).trim();

        // Link-only lines: "* [text](url)" etc
        if content.starts_with('[') && content.ends_with(')') && content.contains("](") {
            link_lines += 1;
            continue;
        }

        // Navigation filler text
        let lower = content.to_lowercase();
        if lower == "expand menu"
            || lower == "collapse menu"
            || lower == "toggle"
            || lower == "menu"
        {
            nav_text_lines += 1;
            continue;
        }

        // Track longest prose line (real content has long paragraphs)
        if content.len() > longest_prose {
            longest_prose = content.len();
        }
    }

    if total_lines < 10 {
        return false;
    }

    let nav_total = link_lines + nav_text_lines;
    let nav_ratio = nav_total * 100 / total_lines;

    // Navigation-heavy if: >45% nav lines AND no substantial prose (longest line < 120 chars)
    nav_ratio > 45 && longest_prose < 120
}

/// Extract content from Next.js React Server Components payload.
/// Modern Next.js sites (App Router) embed page content as RSC data in script tags,
/// not in the visible DOM. This function recovers that content.
fn extract_nextjs_rsc(html: &str) -> Option<String> {
    const RSC_MARKER: &str = "self.__next_f.push([1,\"";

    if !html.contains(RSC_MARKER) {
        return None;
    }

    // Extract and concatenate all RSC chunks
    let mut payload = String::new();
    let mut search_from = 0;
    while let Some(start) = html[search_from..].find(RSC_MARKER) {
        let abs_start = search_from + start + RSC_MARKER.len();
        // Find the closing "])
        if let Some(end) = find_rsc_chunk_end(&html[abs_start..]) {
            let chunk = &html[abs_start..abs_start + end];
            payload.push_str(chunk);
            search_from = abs_start + end;
        } else {
            break;
        }
    }

    if payload.is_empty() {
        return None;
    }

    // Unescape the JS string encoding
    let unescaped = unescape_rsc(&payload);

    // Find the page-content section
    let content_start = unescaped
        .find("\"id\":\"page-content\"")
        .or_else(|| unescaped.find("\"id\": \"page-content\""))
        .or_else(|| unescaped.find("\"id\":\"content\""))
        .or_else(|| unescaped.find("\"phase\":\"content\""))?;

    let content_section = &unescaped[content_start..];

    // Extract text fragments from children values
    let fragments = extract_rsc_text_fragments(content_section);
    if fragments.is_empty() {
        return None;
    }

    // Reconstruct as simple markdown
    let mut md = String::new();
    for frag in &fragments {
        let trimmed = frag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !md.is_empty() {
            // Add spacing between fragments
            if trimmed.starts_with('#') {
                md.push_str("\n\n");
            } else if md.ends_with('\n') {
                // already has newline
            } else {
                md.push(' ');
            }
        }
        md.push_str(trimmed);
    }

    if md.len() > 100 { Some(md) } else { None }
}

/// Find the end of an RSC chunk, handling escaped quotes
fn find_rsc_chunk_end(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // skip escaped character
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn unescape_rsc(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('/') => result.push('/'),
                Some('u') => {
                    // Unicode escape: \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(cp) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(cp) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => break,
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Extract text fragments from RSC payload content section.
/// Looks for "children" values that contain text strings.
fn extract_rsc_text_fragments(content: &str) -> Vec<String> {
    let mut fragments = Vec::new();
    let mut seen_text = std::collections::HashSet::new();

    // Track element context for markdown formatting
    let bytes = content.as_bytes();
    let len = bytes.len();
    // Limit scan to reasonable size
    let scan_limit = len.min(200_000);

    let mut i = 0;
    while i < scan_limit {
        // Look for element patterns: "$","tagName"
        if i + 5 < scan_limit && &bytes[i..i + 4] == b"\"$\"," {
            // Found a React element marker, extract the tag name
            if let Some((tag, after_tag)) = extract_quoted_string(&content[i + 4..]) {
                let tag_lower = tag.to_lowercase();
                let is_heading = tag_lower.starts_with('h')
                    && tag_lower.len() == 2
                    && tag_lower.as_bytes()[1].is_ascii_digit();
                // Skip non-content elements entirely
                if matches!(
                    tag_lower.as_str(),
                    "svg" | "path" | "script" | "style" | "noscript" | "img"
                ) {
                    i += 4;
                    continue;
                }

                let is_content_tag = matches!(
                    tag_lower.as_str(),
                    "p" | "li" | "span" | "strong" | "em" | "code" | "pre" | "td" | "th"
                ) || is_heading;

                if is_content_tag {
                    // Find the children of this element
                    let search_area = &content[i + 4 + after_tag..];
                    if let Some(children_start) = search_area.find("\"children\":") {
                        let children_area = &search_area[children_start + 11..];
                        let extracted = extract_children_text(children_area);
                        if !extracted.is_empty() {
                            let text = extracted.trim().to_string();
                            if text.len() > 2
                                && !is_rsc_noise(&text)
                                && !text.starts_with('$')
                                && !text.contains(":props:")
                                && seen_text.insert(text.clone())
                            {
                                if is_heading {
                                    let level = tag_lower.as_bytes()[1] - b'0';
                                    let prefix = "#".repeat(level as usize);
                                    fragments.push(format!("\n\n{prefix} {text}\n"));
                                } else if tag_lower == "li" {
                                    fragments.push(format!("\n- {text}"));
                                } else if tag_lower == "code" || tag_lower == "pre" {
                                    fragments.push(format!("`{text}`"));
                                } else {
                                    fragments.push(text);
                                }
                            }
                        }
                    }
                }
            }
            i += 4;
        } else {
            i += 1;
        }
    }

    fragments
}

fn extract_quoted_string(s: &str) -> Option<(String, usize)> {
    if !s.starts_with('"') {
        return None;
    }
    let mut end = 1;
    let bytes = s.as_bytes();
    while end < bytes.len() {
        if bytes[end] == b'\\' {
            end += 2;
            continue;
        }
        if bytes[end] == b'"' {
            return Some((s[1..end].to_string(), end + 1));
        }
        end += 1;
    }
    None
}

fn extract_children_text(s: &str) -> String {
    let trimmed = s.trim_start();

    // Direct string: "children":"some text"
    if trimmed.starts_with('"') {
        if let Some((text, _)) = extract_quoted_string(trimmed) {
            return text;
        }
    }

    // Array of mixed content: "children":["text", element, "more text"]
    if trimmed.starts_with('[') {
        let mut result = String::new();
        let mut depth = 0;
        let mut i = 1; // skip opening [
        let bytes = trimmed.as_bytes();
        let limit = bytes.len().min(5000);

        while i < limit {
            match bytes[i] {
                b'[' => {
                    depth += 1;
                    i += 1;
                }
                b']' => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    i += 1;
                }
                b'"' if depth == 0 => {
                    if let Some((text, advance)) = extract_quoted_string(&trimmed[i..]) {
                        if !text.starts_with('$')
                            && !text.starts_with("geist")
                            && !is_rsc_noise(&text)
                        {
                            result.push_str(&text);
                        }
                        i += advance;
                    } else {
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }
        return result;
    }

    String::new()
}

fn is_rsc_noise(s: &str) -> bool {
    // CSS class names and modules
    s.contains("module__")
        || s.contains("className")
        || s.starts_with("geist-")
        || s.starts_with("linked-")
        || (s.contains("__") && s.contains("_") && !s.contains(' '))
        // React internals and prop names
        || s.starts_with("$L")
        || s.starts_with("$undefined")
        || s.contains("SetInnerHTML")
        || s.contains("data-testid")
        // SVG noise
        || s.starts_with("<path")
        || s.starts_with("M ")
        || s.contains("viewBox")
        || s.contains("fill-rule")
        || s.contains("clip-rule")
        || s.contains("currentColor")
        || s.contains("strokeLine")
        // CSS/style noise
        || s.starts_with("var(--")
        || s.contains("colorvar(")
        || s.contains("text-[")
        || s.starts_with("codegrid ")
        || s.contains("font-mono")
        || s.contains("hyphens-none")
        || s.contains("fontFeature")
        // Single-word prop names
        || matches!(s, "children" | "style" | "width" | "height" | "stroke" | "color"
            | "props" | "fallback" | "round" | "src" | "alt" | "href" | "type")
}

fn strip_noise(html: &str) -> String {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);

    let noise_selectors = [
        "[class*='cookie']",
        "[id*='cookie']",
        "[class*='consent']",
        "[id*='consent']",
        "[class*='gdpr']",
        "[id*='gdpr']",
        "nav",
        "header",
        "footer",
        "[role='navigation']",
        "[role='banner']",
        "[role='contentinfo']",
        "[class*='sidebar']",
        "[id*='sidebar']",
        "aside",
        "[class*='overlay']",
        "[class*='modal']",
        "[class*='popup']",
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
    fn rejects_nav_heavy_content() {
        let md = "* [Home](/)\n* [About](/about)\n* [Docs](/docs)\n* [API](/api)\n* [Blog](/blog)\n* [Contact](/contact)\n* [FAQ](/faq)\n* [Support](/support)\n* [Login](/login)\n* [Signup](/signup)\n* [Dashboard](/dashboard)\n";
        assert!(is_mostly_nav(md));
    }

    #[test]
    fn accepts_real_content() {
        let md = "# Getting Started\n\nThis guide walks you through setting up the project.\n\n## Installation\n\nRun the following command:\n\n```\nnpm install my-package\n```\n\nSee [the docs](/docs) for more info.\n";
        assert!(!is_mostly_nav(md));
    }

    #[test]
    fn extracts_nextjs_rsc() {
        let html = r#"<html><body>
        <script>self.__next_f.push([1,"4e:[[\"$\",\"div\",null,{\"id\":\"page-content\",\"children\":[[\"$\",\"h1\",null,{\"children\":\"Getting Started with the Framework\"}],[\"$\",\"p\",null,{\"children\":\"Welcome to the documentation. This guide will walk you through setting up your development environment and creating your first application with all the tools you need.\"}],[\"$\",\"p\",null,{\"children\":\"Follow the steps below to get everything configured correctly.\"}]]}]]"])</script>
        </body></html>"#;

        let result = extract_nextjs_rsc(html);
        assert!(result.is_some(), "RSC extraction returned None");
        let md = result.unwrap();
        assert!(md.contains("Getting Started"), "Missing heading: {md}");
        assert!(
            md.contains("Welcome to the documentation"),
            "Missing paragraph: {md}"
        );
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
