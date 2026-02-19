use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};

pub struct PlainTextConverter;

impl DocumentConverter for PlainTextConverter {
    fn name(&self) -> &'static str {
        "PlainText"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        if let Some(mime) = &info.mime_type {
            if mime.starts_with("text/") {
                return true;
            }
        }
        matches!(
            info.extension.as_deref(),
            Some(
                "txt"
                    | "md"
                    | "rst"
                    | "log"
                    | "cfg"
                    | "ini"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "json"
                    | "xml"
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
                    | "bash"
                    | "zsh"
                    | "fish"
                    | "ps1"
                    | "bat"
                    | "rb"
                    | "pl"
                    | "lua"
                    | "r"
                    | "swift"
                    | "kt"
                    | "scala"
                    | "hs"
                    | "ex"
                    | "exs"
                    | "clj"
            )
        )
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let text = decode_text(input);
        Ok(ConversionResult::new(text))
    }
}

fn decode_text(input: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(input) {
        return s.to_string();
    }

    // Try BOM detection
    if let Some((encoding, _bom_len)) = encoding_rs::Encoding::for_bom(input) {
        let (decoded, _, _) = encoding.decode(input);
        return decoded.into_owned();
    }

    // Fallback: try common encodings
    let encodings: &[&encoding_rs::Encoding] = &[
        encoding_rs::WINDOWS_1252,
        encoding_rs::UTF_16LE,
        encoding_rs::UTF_16BE,
    ];
    for encoding in encodings {
        let (decoded, _, had_errors) = encoding.decode(input);
        if !had_errors {
            return decoded.into_owned();
        }
    }

    // Last resort: lossy UTF-8
    String::from_utf8_lossy(input).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_text_plain() {
        let c = PlainTextConverter;
        let info = StreamInfo {
            mime_type: Some("text/plain".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_by_extension() {
        let c = PlainTextConverter;
        let info = StreamInfo {
            extension: Some("py".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn converts_utf8() {
        let c = PlainTextConverter;
        let info = StreamInfo::default();
        let result = c.convert(b"hello world", &info).unwrap();
        assert_eq!(result.body, "hello world");
    }

    #[test]
    fn rejects_binary() {
        let c = PlainTextConverter;
        let info = StreamInfo {
            mime_type: Some("application/pdf".into()),
            ..Default::default()
        };
        assert!(!c.accepts(&info));
    }
}
