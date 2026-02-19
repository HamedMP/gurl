use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use std::io::{Cursor, Read};

pub struct OutlookMsgConverter;

impl DocumentConverter for OutlookMsgConverter {
    fn name(&self) -> &'static str {
        "Outlook MSG"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("application/vnd.ms-outlook" | "application/x-msg")
        ) || matches!(info.extension.as_deref(), Some("msg"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let cursor = Cursor::new(input);
        let mut comp = cfb::CompoundFile::open(cursor)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to open MSG: {e}")))?;

        let subject = read_msg_property(&mut comp, "0037", "Subject");
        let from = read_msg_property(&mut comp, "0C1A", "From");
        let to = read_msg_property(&mut comp, "0E04", "To");
        let date = read_msg_property(&mut comp, "0039", "Date");
        let body = read_msg_property(&mut comp, "1000", "Body");

        let mut md = String::new();

        // Header
        if let Some(subj) = &subject {
            md.push_str(&format!("# {subj}\n\n"));
        }

        md.push_str("## Email Details\n\n");
        if let Some(f) = &from {
            md.push_str(&format!("- **From**: {f}\n"));
        }
        if let Some(t) = &to {
            md.push_str(&format!("- **To**: {t}\n"));
        }
        if let Some(d) = &date {
            md.push_str(&format!("- **Date**: {d}\n"));
        }

        // Body
        if let Some(b) = &body {
            md.push_str(&format!("\n## Body\n\n{b}\n"));
        }

        // List attachments
        let attachments = list_attachments(&mut comp);
        if !attachments.is_empty() {
            md.push_str("\n## Attachments\n\n");
            for att in &attachments {
                md.push_str(&format!("- {att}\n"));
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        if let Some(subj) = subject {
            result = result.with_title(subj);
        }
        Ok(result)
    }
}

fn read_msg_property(
    comp: &mut cfb::CompoundFile<Cursor<&[u8]>>,
    prop_tag: &str,
    _label: &str,
) -> Option<String> {
    // MSG properties are stored as streams with names like:
    // __substg1.0_PPPPTTTT where PPPP is property tag and TTTT is type
    // 001F = Unicode string, 001E = ANSI string, 0102 = binary

    // Try Unicode first (001F)
    let unicode_path = format!("/__substg1.0_{prop_tag}001F");
    if let Some(text) = read_stream_as_string(comp, &unicode_path, true) {
        return Some(text);
    }

    // Try ANSI (001E)
    let ansi_path = format!("/__substg1.0_{prop_tag}001E");
    if let Some(text) = read_stream_as_string(comp, &ansi_path, false) {
        return Some(text);
    }

    None
}

fn read_stream_as_string(
    comp: &mut cfb::CompoundFile<Cursor<&[u8]>>,
    path: &str,
    is_unicode: bool,
) -> Option<String> {
    let mut stream = comp.open_stream(path).ok()?;
    let mut data = Vec::new();
    stream.read_to_end(&mut data).ok()?;

    if data.is_empty() {
        return None;
    }

    let text = if is_unicode {
        // UTF-16LE decoding
        let (decoded, _, _) = encoding_rs::UTF_16LE.decode(&data);
        decoded.into_owned()
    } else {
        String::from_utf8_lossy(&data).into_owned()
    };

    let trimmed = text.trim_end_matches('\0').trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn list_attachments(comp: &mut cfb::CompoundFile<Cursor<&[u8]>>) -> Vec<String> {
    let mut attachments = Vec::new();

    // Attachment storages are named __attach_version1.0_#XXXXXXXX
    let entries: Vec<_> = comp
        .walk()
        .filter(|e| e.is_storage() && e.name().starts_with("__attach"))
        .map(|e| e.path().to_path_buf())
        .collect();

    for entry_path in entries {
        let path_str = entry_path.to_string_lossy();
        // Try to read attachment filename
        let filename_path = format!("{path_str}/__substg1.0_3707001F");
        if let Some(name) = read_stream_as_string(comp, &filename_path, true) {
            attachments.push(name);
        } else {
            let filename_path = format!("{path_str}/__substg1.0_3704001F");
            if let Some(name) = read_stream_as_string(comp, &filename_path, true) {
                attachments.push(name);
            }
        }
    }

    attachments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_msg() {
        let c = OutlookMsgConverter;
        let info = StreamInfo {
            extension: Some("msg".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_outlook_mime() {
        let c = OutlookMsgConverter;
        let info = StreamInfo {
            mime_type: Some("application/vnd.ms-outlook".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }
}
