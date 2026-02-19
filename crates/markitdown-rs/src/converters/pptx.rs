use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use crate::utils::table::to_markdown_table;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::io::{Cursor, Read};

pub struct PptxConverter;

impl DocumentConverter for PptxConverter {
    fn name(&self) -> &'static str {
        "PPTX"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("application/vnd.openxmlformats-officedocument.presentationml.presentation")
        ) || matches!(info.extension.as_deref(), Some("pptx"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let cursor = Cursor::new(input);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to open PPTX: {e}")))?;

        // Find slide files in order
        let mut slide_paths: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                let name = file.name().to_string();
                if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                    slide_paths.push(name);
                }
            }
        }
        slide_paths.sort_by_key(|a| extract_slide_number(a));

        let mut md = String::new();
        let mut title: Option<String> = None;

        for (i, path) in slide_paths.iter().enumerate() {
            let xml = match read_zip_text(&mut archive, path) {
                Ok(data) => data,
                Err(_) => continue,
            };

            let slide_content = parse_slide_xml(&xml);

            if !slide_content.is_empty() {
                md.push_str(&format!("## Slide {}\n\n", i + 1));
                md.push_str(&slide_content);
                md.push_str("\n\n");

                // Use first slide's first text as title
                if i == 0 && title.is_none() {
                    let first_line = slide_content.lines().next().unwrap_or("");
                    if !first_line.is_empty() {
                        title = Some(first_line.to_string());
                    }
                }
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        result = result.with_metadata("slide_count", slide_paths.len().to_string());
        if let Some(t) = title {
            result = result.with_title(t);
        }
        Ok(result)
    }
}

fn extract_slide_number(path: &str) -> usize {
    path.strip_prefix("ppt/slides/slide")
        .and_then(|s| s.strip_suffix(".xml"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

fn read_zip_text(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    path: &str,
) -> crate::Result<String> {
    let mut file = archive
        .by_name(path)
        .map_err(|e| crate::Error::ConversionFailed(format!("Entry '{path}' not found: {e}")))?;
    let mut data = String::new();
    file.read_to_string(&mut data)
        .map_err(|e| crate::Error::ConversionFailed(format!("Failed to read '{path}': {e}")))?;
    Ok(data)
}

fn parse_slide_xml(xml: &str) -> String {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut paragraphs: Vec<String> = Vec::new();
    let mut current_paragraph = String::new();
    let mut in_text = false;
    let mut in_table = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let local = local_name_owned(&name_bytes);
                match local.as_str() {
                    "t" => in_text = true,
                    "p" if in_table => {
                        // paragraph inside table cell
                    }
                    "p" => {
                        if !current_paragraph.is_empty() {
                            paragraphs.push(std::mem::take(&mut current_paragraph));
                        }
                    }
                    "tbl" => in_table = true,
                    "tr" => current_row.clear(),
                    "tc" => current_cell.clear(),
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_text {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_table {
                        current_cell.push_str(&text);
                    } else {
                        current_paragraph.push_str(&text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let local = local_name_owned(&name_bytes);
                match local.as_str() {
                    "t" => in_text = false,
                    "tc" => {
                        current_row.push(std::mem::take(&mut current_cell));
                    }
                    "tr" => {
                        if !current_row.is_empty() {
                            table_rows.push(std::mem::take(&mut current_row));
                        }
                    }
                    "tbl" => {
                        in_table = false;
                        if !table_rows.is_empty() {
                            paragraphs.push(to_markdown_table(&table_rows));
                            table_rows.clear();
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }

    paragraphs
        .into_iter()
        .filter(|p| !p.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn local_name_owned(name: &[u8]) -> String {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rsplit_once(':')
        .map(|(_, local)| local)
        .unwrap_or(s)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_pptx() {
        let c = PptxConverter;
        let info = StreamInfo {
            extension: Some("pptx".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn extracts_slide_number() {
        assert_eq!(extract_slide_number("ppt/slides/slide1.xml"), 1);
        assert_eq!(extract_slide_number("ppt/slides/slide12.xml"), 12);
    }
}
