use crate::converter::StreamInfo;

pub fn detect(input: &[u8], info: &StreamInfo) -> StreamInfo {
    let mut result = info.clone();

    // If no MIME type provided, try detection
    if result.mime_type.is_none() {
        // 1. Try magic bytes
        if let Some(kind) = infer::get(input) {
            result.mime_type = Some(kind.mime_type().to_string());
        }

        // 2. Fallback: guess from extension
        if result.mime_type.is_none() {
            if let Some(ext) = &result.extension {
                if let Some(mime) = mime_guess::from_ext(ext).first() {
                    result.mime_type = Some(mime.to_string());
                }
            }
        }

        // 3. Fallback: guess from filename
        if result.mime_type.is_none() {
            if let Some(filename) = &result.filename {
                if let Some(mime) = mime_guess::from_path(filename).first() {
                    result.mime_type = Some(mime.to_string());
                }
            }
        }

        // 4. Fallback: guess from URL
        if result.mime_type.is_none() {
            if let Some(url) = &result.url {
                if let Some(mime) = mime_guess::from_path(url).first() {
                    result.mime_type = Some(mime.to_string());
                }
            }
        }

        // 5. Heuristic: check if it looks like HTML
        if result.mime_type.is_none() && looks_like_html(input) {
            result.mime_type = Some("text/html".to_string());
        }

        // 6. Heuristic: check if it looks like JSON
        if result.mime_type.is_none() && looks_like_json(input) {
            result.mime_type = Some("application/json".to_string());
        }

        // 7. Heuristic: check if it's valid UTF-8 text
        if result.mime_type.is_none() && std::str::from_utf8(input).is_ok() {
            result.mime_type = Some("text/plain".to_string());
        }
    }

    // Extract extension from filename/URL if not set
    if result.extension.is_none() {
        let source = result.filename.as_deref().or(result.url.as_deref());
        if let Some(s) = source {
            if let Some(ext) = s.rsplit('.').next() {
                let ext = ext.split(['?', '#']).next().unwrap_or(ext);
                if ext.len() <= 10 {
                    result.extension = Some(ext.to_lowercase());
                }
            }
        }
    }

    result
}

fn looks_like_html(input: &[u8]) -> bool {
    let prefix = &input[..input.len().min(512)];
    let s = String::from_utf8_lossy(prefix).to_lowercase();
    s.contains("<!doctype html") || s.contains("<html") || s.contains("<head") || s.contains("<body")
}

fn looks_like_json(input: &[u8]) -> bool {
    let s = std::str::from_utf8(input).unwrap_or("");
    let trimmed = s.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}
