use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use std::io::{Cursor, Read};

pub struct ZipConverter;

impl DocumentConverter for ZipConverter {
    fn name(&self) -> &'static str {
        "ZIP"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("application/zip" | "application/x-zip-compressed")
        ) || matches!(info.extension.as_deref(), Some("zip"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let cursor = Cursor::new(input);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to open ZIP: {e}")))?;

        let mut md = String::new();
        md.push_str("# Archive Contents\n\n");

        let mut text_contents: Vec<(String, String)> = Vec::new();
        let mut file_list: Vec<String> = Vec::new();

        for i in 0..archive.len() {
            let mut file = match archive.by_index(i) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let name = file.name().to_string();
            let size = file.size();

            if file.is_dir() {
                continue;
            }

            file_list.push(format!("- `{name}` ({} bytes)", size));

            // Extract text from small text-like files
            if size < 1_000_000 {
                let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
                if matches!(
                    ext.as_str(),
                    "txt"
                        | "md"
                        | "csv"
                        | "json"
                        | "yaml"
                        | "yml"
                        | "toml"
                        | "xml"
                        | "html"
                        | "css"
                        | "js"
                        | "ts"
                        | "py"
                        | "rs"
                        | "go"
                        | "java"
                        | "c"
                        | "cpp"
                        | "h"
                        | "sh"
                        | "cfg"
                        | "ini"
                        | "log"
                        | "rst"
                ) {
                    let mut content = String::new();
                    if file.read_to_string(&mut content).is_ok() && !content.is_empty() {
                        text_contents.push((name.clone(), content));
                    }
                }
            }
        }

        // File listing
        md.push_str(&format!("**{} files**\n\n", file_list.len()));
        for entry in &file_list {
            md.push_str(entry);
            md.push('\n');
        }

        // Extracted text content
        if !text_contents.is_empty() {
            md.push_str("\n---\n\n");
            for (name, content) in &text_contents {
                let ext = name.rsplit('.').next().unwrap_or("txt");
                md.push_str(&format!("## {name}\n\n"));
                md.push_str(&format!("```{ext}\n"));
                md.push_str(content);
                if !content.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        result = result.with_metadata("file_count", file_list.len().to_string());
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_zip() {
        let c = ZipConverter;
        let info = StreamInfo {
            mime_type: Some("application/zip".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_by_extension() {
        let c = ZipConverter;
        let info = StreamInfo {
            extension: Some("zip".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }
}
