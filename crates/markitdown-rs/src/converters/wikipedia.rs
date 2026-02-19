use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use scraper::{Html, Selector};

pub struct WikipediaConverter;

impl DocumentConverter for WikipediaConverter {
    fn name(&self) -> &'static str {
        "Wikipedia"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        if let Some(url) = &info.url {
            return url.contains("wikipedia.org");
        }
        false
    }

    fn convert(&self, input: &[u8], info: &StreamInfo) -> crate::Result<ConversionResult> {
        let html_str = String::from_utf8_lossy(input);
        let document = Html::parse_document(&html_str);

        // Extract article title
        let title = extract_title(&document);

        // Extract article content, stripping Wikipedia-specific elements
        let cleaned = clean_wikipedia_html(&html_str);

        // Convert cleaned HTML to markdown
        #[cfg(feature = "html")]
        let md = {
            htmd::convert(&cleaned).map_err(|e| crate::Error::ConversionFailed(e.to_string()))?
        };

        #[cfg(not(feature = "html"))]
        let md = strip_tags(&cleaned);

        let mut result = ConversionResult::new(md.trim());
        if let Some(t) = title {
            result = result.with_title(t);
        }
        if let Some(url) = &info.url {
            result = result.with_metadata("source_url", url.clone());
        }
        Ok(result)
    }
}

fn extract_title(document: &Html) -> Option<String> {
    let sel = Selector::parse("h1#firstHeading, h1.firstHeading").ok()?;
    document
        .select(&sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
}

fn clean_wikipedia_html(html: &str) -> String {
    let document = Html::parse_document(html);

    // Try to get the main article content
    let content_sel = Selector::parse("#mw-content-text .mw-parser-output").ok();

    let content_html = if let Some(sel) = &content_sel {
        if let Some(el) = document.select(sel).next() {
            el.html()
        } else {
            html.to_string()
        }
    } else {
        html.to_string()
    };

    // Remove elements that add noise
    let remove_selectors = [
        ".mw-editsection", // [edit] links
        ".reference",      // footnote references
        "#toc",            // table of contents
        ".toc",            // table of contents (alt)
        ".navbox",         // navigation boxes
        ".sistersitebox",  // sister site boxes
        ".sidebar",        // sidebars
        ".infobox",        // infoboxes (keep? remove for cleaner text)
        ".metadata",       // metadata banners
        ".hatnote",        // disambiguation notices
        ".mbox-small",     // small message boxes
        "sup.reference",   // reference superscripts
        ".reflist",        // reference lists
        ".refbegin",       // reference begin
        ".external",       // external links section
        "style",           // inline styles
        "script",          // scripts
    ];

    let mut cleaned = content_html;
    for selector_str in &remove_selectors {
        if let Ok(sel) = Selector::parse(selector_str) {
            let doc = Html::parse_fragment(&cleaned);
            let to_remove: Vec<String> = doc.select(&sel).map(|el| el.html()).collect();
            for fragment in &to_remove {
                cleaned = cleaned.replace(fragment, "");
            }
        }
    }

    cleaned
}

#[cfg(not(feature = "html"))]
fn strip_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_wikipedia_url() {
        let c = WikipediaConverter;
        let info = StreamInfo {
            url: Some("https://en.wikipedia.org/wiki/Rust_(programming_language)".into()),
            mime_type: Some("text/html".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn rejects_non_wikipedia() {
        let c = WikipediaConverter;
        let info = StreamInfo {
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        assert!(!c.accepts(&info));
    }
}
