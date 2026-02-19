use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::Cursor;

pub struct EpubConverter;

impl DocumentConverter for EpubConverter {
    fn name(&self) -> &'static str {
        "EPUB"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(info.mime_type.as_deref(), Some("application/epub+zip"))
            || matches!(info.extension.as_deref(), Some("epub"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let cursor = Cursor::new(input);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to open EPUB: {e}")))?;

        // 1. Find the OPF file path from META-INF/container.xml
        let opf_path = find_opf_path(&mut archive)?;

        // 2. Parse the OPF to get reading order (spine)
        let spine = parse_opf(&mut archive, &opf_path)?;

        // 3. Extract and convert each chapter
        let mut md = String::new();
        let mut title: Option<String> = None;

        for (i, chapter_path) in spine.iter().enumerate() {
            let full_path = resolve_path(&opf_path, chapter_path);
            let html = match read_zip_entry(&mut archive, &full_path) {
                Ok(data) => data,
                Err(_) => continue,
            };

            let html_str = String::from_utf8_lossy(&html);

            // Extract title from first chapter if possible
            if i == 0 && title.is_none() {
                title = extract_html_title(&html_str);
            }

            // Convert HTML to markdown using htmd
            #[cfg(feature = "html")]
            {
                if let Ok(chapter_md) = htmd::convert(&html_str) {
                    let trimmed = chapter_md.trim();
                    if !trimmed.is_empty() {
                        if !md.is_empty() {
                            md.push_str("\n\n---\n\n");
                        }
                        md.push_str(trimmed);
                    }
                }
            }

            #[cfg(not(feature = "html"))]
            {
                // Without HTML feature, just strip tags
                let text = strip_html(&html_str);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    if !md.is_empty() {
                        md.push_str("\n\n---\n\n");
                    }
                    md.push_str(trimmed);
                }
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        result = result.with_metadata("chapter_count", spine.len().to_string());
        if let Some(t) = title {
            result = result.with_title(t);
        }
        Ok(result)
    }
}

fn find_opf_path(archive: &mut zip::ZipArchive<Cursor<&[u8]>>) -> crate::Result<String> {
    let container = read_zip_entry(archive, "META-INF/container.xml")?;
    let xml = String::from_utf8_lossy(&container);

    let mut reader = Reader::from_str(&xml);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "rootfile" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"full-path" {
                            return Ok(String::from_utf8_lossy(&attr.value).to_string());
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(crate::Error::ConversionFailed(format!(
                    "container.xml parse error: {e}"
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    Err(crate::Error::ConversionFailed(
        "No rootfile found in container.xml".to_string(),
    ))
}

struct SpineItem {
    id: String,
    href: String,
}

fn parse_opf(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    opf_path: &str,
) -> crate::Result<Vec<String>> {
    let opf_data = read_zip_entry(archive, opf_path)?;
    let xml = String::from_utf8_lossy(&opf_data);

    let mut reader = Reader::from_str(&xml);
    let mut buf = Vec::new();

    let mut manifest: Vec<SpineItem> = Vec::new();
    let mut spine_order: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "item" => {
                        let mut id = String::new();
                        let mut href = String::new();
                        let mut media_type = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => {
                                    id = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"href" => {
                                    href = String::from_utf8_lossy(&attr.value).to_string()
                                }
                                b"media-type" => {
                                    media_type =
                                        String::from_utf8_lossy(&attr.value).to_string()
                                }
                                _ => {}
                            }
                        }
                        if media_type.contains("html") || media_type.contains("xml") {
                            manifest.push(SpineItem { id, href });
                        }
                    }
                    "itemref" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref" {
                                spine_order
                                    .push(String::from_utf8_lossy(&attr.value).to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(crate::Error::ConversionFailed(format!(
                    "OPF parse error: {e}"
                )))
            }
            _ => {}
        }
        buf.clear();
    }

    // Map spine idrefs to hrefs
    let mut result = Vec::new();
    for idref in &spine_order {
        if let Some(item) = manifest.iter().find(|m| m.id == *idref) {
            result.push(item.href.clone());
        }
    }

    // If spine is empty, use manifest order
    if result.is_empty() {
        result = manifest.into_iter().map(|m| m.href).collect();
    }

    Ok(result)
}

fn resolve_path(opf_path: &str, relative: &str) -> String {
    if let Some(dir) = opf_path.rsplit_once('/') {
        format!("{}/{relative}", dir.0)
    } else {
        relative.to_string()
    }
}

fn read_zip_entry(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    path: &str,
) -> crate::Result<Vec<u8>> {
    use std::io::Read;

    let mut file = archive.by_name(path).map_err(|e| {
        crate::Error::ConversionFailed(format!("Entry '{path}' not found in archive: {e}"))
    })?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| crate::Error::ConversionFailed(format!("Failed to read '{path}': {e}")))?;
    Ok(data)
}

fn extract_html_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title>")?;
    let after = start + 7;
    let end = lower[after..].find("</title>")?;
    let title = html[after..after + end].trim().to_string();
    if title.is_empty() {
        None
    } else {
        Some(title)
    }
}

#[cfg(not(feature = "html"))]
fn strip_html(html: &str) -> String {
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
    fn accepts_epub() {
        let c = EpubConverter;
        let info = StreamInfo {
            mime_type: Some("application/epub+zip".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_by_extension() {
        let c = EpubConverter;
        let info = StreamInfo {
            extension: Some("epub".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }
}
