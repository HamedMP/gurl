use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use crate::utils::table::to_markdown_table;

pub struct DocxConverter;

impl DocumentConverter for DocxConverter {
    fn name(&self) -> &'static str {
        "DOCX"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document")
        ) || matches!(info.extension.as_deref(), Some("docx"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let docx = docx_rs::read_docx(input)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to parse DOCX: {e}")))?;

        let mut md = String::new();
        extract_document(&docx.document, &mut md);

        Ok(ConversionResult::new(md.trim_end()))
    }
}

fn extract_document(doc: &docx_rs::Document, out: &mut String) {
    for child in &doc.children {
        extract_document_child(child, out);
    }
}

fn extract_document_child(child: &docx_rs::DocumentChild, out: &mut String) {
    match child {
        docx_rs::DocumentChild::Paragraph(p) => {
            let text = extract_paragraph_text(p);
            if !text.trim().is_empty() {
                // Check for heading style
                let level = detect_heading_level(p);
                if let Some(lvl) = level {
                    let prefix = "#".repeat(lvl);
                    out.push_str(&format!("{prefix} {}\n\n", text.trim()));
                } else {
                    out.push_str(text.trim());
                    out.push_str("\n\n");
                }
            }
        }
        docx_rs::DocumentChild::Table(table) => {
            extract_table(table, out);
            out.push('\n');
        }
        _ => {}
    }
}

fn extract_paragraph_text(p: &docx_rs::Paragraph) -> String {
    let mut text = String::new();
    for child in &p.children {
        extract_paragraph_child(child, &mut text);
    }
    text
}

fn extract_paragraph_child(child: &docx_rs::ParagraphChild, out: &mut String) {
    match child {
        docx_rs::ParagraphChild::Run(run) => {
            let is_bold = run.run_property.bold.is_some();
            let is_italic = run.run_property.italic.is_some();

            let mut run_text = String::new();
            for rc in &run.children {
                match rc {
                    docx_rs::RunChild::Text(t) => run_text.push_str(&t.text),
                    docx_rs::RunChild::Tab(_) => run_text.push('\t'),
                    docx_rs::RunChild::Break(_) => run_text.push('\n'),
                    _ => {}
                }
            }

            if !run_text.is_empty() {
                if is_bold && is_italic {
                    out.push_str(&format!("***{run_text}***"));
                } else if is_bold {
                    out.push_str(&format!("**{run_text}**"));
                } else if is_italic {
                    out.push_str(&format!("*{run_text}*"));
                } else {
                    out.push_str(&run_text);
                }
            }
        }
        docx_rs::ParagraphChild::Hyperlink(link) => {
            for run in &link.children {
                extract_paragraph_child(run, out);
            }
        }
        _ => {}
    }
}

fn detect_heading_level(p: &docx_rs::Paragraph) -> Option<usize> {
    if let Some(style) = &p.property.style {
        let id = &style.val;
        // Common DOCX heading style IDs
        if id.starts_with("Heading") || id.starts_with("heading") {
            if let Some(level_str) = id
                .strip_prefix("Heading")
                .or_else(|| id.strip_prefix("heading"))
            {
                if let Ok(level) = level_str.parse::<usize>() {
                    return Some(level.min(6));
                }
            }
        }
        match id.as_str() {
            "Title" | "title" => return Some(1),
            "Subtitle" | "subtitle" => return Some(2),
            _ => {}
        }
    }
    None
}

fn extract_table(table: &docx_rs::Table, out: &mut String) {
    let mut rows: Vec<Vec<String>> = Vec::new();

    for row in &table.rows {
        let docx_rs::TableChild::TableRow(tr) = row;
        let mut cells: Vec<String> = Vec::new();
        for cell in &tr.cells {
            let docx_rs::TableRowChild::TableCell(tc) = cell;
            let mut cell_text = String::new();
            for content in &tc.children {
                if let docx_rs::TableCellContent::Paragraph(p) = content {
                    let text = extract_paragraph_text(p);
                    if !cell_text.is_empty() && !text.trim().is_empty() {
                        cell_text.push(' ');
                    }
                    cell_text.push_str(text.trim());
                }
            }
            cells.push(cell_text);
        }
        if !cells.is_empty() {
            rows.push(cells);
        }
    }

    if !rows.is_empty() {
        out.push_str(&to_markdown_table(&rows));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_docx() {
        let c = DocxConverter;
        let info = StreamInfo {
            mime_type: Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".into(),
            ),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_by_extension() {
        let c = DocxConverter;
        let info = StreamInfo {
            extension: Some("docx".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }
}
