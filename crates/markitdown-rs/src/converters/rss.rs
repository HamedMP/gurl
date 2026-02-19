use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct RssConverter;

impl DocumentConverter for RssConverter {
    fn name(&self) -> &'static str {
        "RSS"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("application/rss+xml" | "application/atom+xml" | "application/xml" | "text/xml")
        ) || matches!(info.extension.as_deref(), Some("rss" | "atom" | "feed"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let xml = String::from_utf8_lossy(input);

        // Detect feed type and parse accordingly
        if xml.contains("<feed") && xml.contains("xmlns=\"http://www.w3.org/2005/Atom\"") {
            parse_atom(&xml)
        } else {
            parse_rss(&xml)
        }
    }
}

struct FeedEntry {
    title: String,
    link: String,
    description: String,
    pub_date: String,
}

fn parse_rss(xml: &str) -> crate::Result<ConversionResult> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut feed_title = String::new();
    let mut entries: Vec<FeedEntry> = Vec::new();

    let mut in_channel = false;
    let mut in_item = false;
    let mut current_tag = String::new();
    let mut current_entry = FeedEntry {
        title: String::new(),
        link: String::new(),
        description: String::new(),
        pub_date: String::new(),
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "channel" => in_channel = true,
                    "item" => {
                        in_item = true;
                        current_entry = FeedEntry {
                            title: String::new(),
                            link: String::new(),
                            description: String::new(),
                            pub_date: String::new(),
                        };
                    }
                    _ => current_tag = name,
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_item {
                    match current_tag.as_str() {
                        "title" => current_entry.title = text,
                        "link" => current_entry.link = text,
                        "description" => current_entry.description = text,
                        "pubDate" => current_entry.pub_date = text,
                        _ => {}
                    }
                } else if in_channel && current_tag == "title" {
                    feed_title = text;
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "item" {
                    entries.push(std::mem::replace(
                        &mut current_entry,
                        FeedEntry {
                            title: String::new(),
                            link: String::new(),
                            description: String::new(),
                            pub_date: String::new(),
                        },
                    ));
                    in_item = false;
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(crate::Error::ConversionFailed(format!(
                    "XML parse error: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    format_feed(&feed_title, &entries)
}

fn parse_atom(xml: &str) -> crate::Result<ConversionResult> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut feed_title = String::new();
    let mut entries: Vec<FeedEntry> = Vec::new();

    let mut in_entry = false;
    let mut in_feed_title = false;
    let mut current_tag = String::new();
    let mut current_entry = FeedEntry {
        title: String::new(),
        link: String::new(),
        description: String::new(),
        pub_date: String::new(),
    };

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "entry" => {
                        in_entry = true;
                        current_entry = FeedEntry {
                            title: String::new(),
                            link: String::new(),
                            description: String::new(),
                            pub_date: String::new(),
                        };
                    }
                    "link" => {
                        if in_entry && current_entry.link.is_empty() {
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"href" {
                                    current_entry.link =
                                        String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                        }
                    }
                    "title" if !in_entry => in_feed_title = true,
                    _ => current_tag = name,
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_feed_title {
                    feed_title = text;
                    in_feed_title = false;
                } else if in_entry {
                    match current_tag.as_str() {
                        "title" => current_entry.title = text,
                        "summary" | "content" => current_entry.description = text,
                        "updated" | "published" => current_entry.pub_date = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "entry" {
                    entries.push(std::mem::replace(
                        &mut current_entry,
                        FeedEntry {
                            title: String::new(),
                            link: String::new(),
                            description: String::new(),
                            pub_date: String::new(),
                        },
                    ));
                    in_entry = false;
                }
                current_tag.clear();
                in_feed_title = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(crate::Error::ConversionFailed(format!(
                    "XML parse error: {e}"
                )));
            }
            _ => {}
        }
        buf.clear();
    }

    format_feed(&feed_title, &entries)
}

fn format_feed(title: &str, entries: &[FeedEntry]) -> crate::Result<ConversionResult> {
    let mut md = String::new();

    if !title.is_empty() {
        md.push_str(&format!("# {title}\n\n"));
    }

    for entry in entries {
        if !entry.title.is_empty() {
            if !entry.link.is_empty() {
                md.push_str(&format!("## [{}]({})\n\n", entry.title, entry.link));
            } else {
                md.push_str(&format!("## {}\n\n", entry.title));
            }
        }

        if !entry.pub_date.is_empty() {
            md.push_str(&format!("*{}*\n\n", entry.pub_date));
        }

        if !entry.description.is_empty() {
            // Strip HTML tags from description (simple approach)
            let clean = strip_html_tags(&entry.description);
            md.push_str(&clean);
            md.push_str("\n\n");
        }

        md.push_str("---\n\n");
    }

    let mut result = ConversionResult::new(md.trim_end());
    if !title.is_empty() {
        result = result.with_title(title);
    }
    Ok(result)
}

fn strip_html_tags(html: &str) -> String {
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
    fn accepts_rss() {
        let c = RssConverter;
        let info = StreamInfo {
            mime_type: Some("application/rss+xml".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn parses_rss_feed() {
        let c = RssConverter;
        let info = StreamInfo {
            extension: Some("rss".into()),
            ..Default::default()
        };
        let rss = r#"<?xml version="1.0"?>
        <rss version="2.0">
        <channel>
            <title>Test Feed</title>
            <item>
                <title>First Post</title>
                <link>https://example.com/1</link>
                <description>Hello world</description>
            </item>
            <item>
                <title>Second Post</title>
                <link>https://example.com/2</link>
                <description>Goodbye world</description>
            </item>
        </channel>
        </rss>"#;
        let result = c.convert(rss.as_bytes(), &info).unwrap();
        assert!(result.body.contains("# Test Feed"));
        assert!(result.body.contains("[First Post](https://example.com/1)"));
        assert!(result.body.contains("Hello world"));
        assert!(result.title.as_deref() == Some("Test Feed"));
    }
}
