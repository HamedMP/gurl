use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use serde::Deserialize;

pub struct IpynbConverter;

impl DocumentConverter for IpynbConverter {
    fn name(&self) -> &'static str {
        "Jupyter Notebook"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(info.mime_type.as_deref(), Some("application/x-ipynb+json"))
            || matches!(info.extension.as_deref(), Some("ipynb"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let notebook: Notebook = serde_json::from_slice(input)
            .map_err(|e| crate::Error::ConversionFailed(format!("Invalid notebook JSON: {e}")))?;

        let lang = notebook
            .metadata
            .and_then(|m| m.kernelspec)
            .and_then(|k| k.language)
            .unwrap_or_else(|| "python".to_string());

        let mut md = String::new();
        let mut cell_num = 0;

        for cell in &notebook.cells {
            cell_num += 1;
            let source = cell.source_text();

            match cell.cell_type.as_str() {
                "markdown" => {
                    md.push_str(&source);
                    md.push_str("\n\n");
                }
                "code" => {
                    md.push_str(&format!("```{lang}\n"));
                    md.push_str(&source);
                    if !source.ends_with('\n') {
                        md.push('\n');
                    }
                    md.push_str("```\n\n");

                    // Include text outputs
                    if let Some(outputs) = &cell.outputs {
                        for output in outputs {
                            if let Some(text) = output.text_content() {
                                md.push_str("**Output:**\n\n```\n");
                                md.push_str(&text);
                                if !text.ends_with('\n') {
                                    md.push('\n');
                                }
                                md.push_str("```\n\n");
                            }
                        }
                    }
                }
                "raw" => {
                    md.push_str(&source);
                    md.push_str("\n\n");
                }
                _ => {}
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        result = result.with_metadata("cell_count", cell_num.to_string());
        result = result.with_metadata("language", lang);
        Ok(result)
    }
}

#[derive(Deserialize)]
struct Notebook {
    cells: Vec<Cell>,
    metadata: Option<NotebookMetadata>,
}

#[derive(Deserialize)]
struct NotebookMetadata {
    kernelspec: Option<Kernelspec>,
}

#[derive(Deserialize)]
struct Kernelspec {
    language: Option<String>,
}

#[derive(Deserialize)]
struct Cell {
    cell_type: String,
    source: CellSource,
    outputs: Option<Vec<CellOutput>>,
}

// Jupyter source can be a single string or array of strings
#[derive(Deserialize)]
#[serde(untagged)]
enum CellSource {
    Single(String),
    Lines(Vec<String>),
}

impl Cell {
    fn source_text(&self) -> String {
        match &self.source {
            CellSource::Single(s) => s.clone(),
            CellSource::Lines(lines) => lines.join(""),
        }
    }
}

#[derive(Deserialize)]
struct CellOutput {
    text: Option<CellSource>,
    data: Option<OutputData>,
}

#[derive(Deserialize)]
struct OutputData {
    #[serde(rename = "text/plain")]
    text_plain: Option<CellSource>,
}

impl CellOutput {
    fn text_content(&self) -> Option<String> {
        // Try direct text field first (stream outputs)
        if let Some(text) = &self.text {
            return Some(match text {
                CellSource::Single(s) => s.clone(),
                CellSource::Lines(lines) => lines.join(""),
            });
        }

        // Try data.text/plain (execute_result, display_data)
        if let Some(data) = &self.data {
            if let Some(text) = &data.text_plain {
                return Some(match text {
                    CellSource::Single(s) => s.clone(),
                    CellSource::Lines(lines) => lines.join(""),
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ipynb() {
        let c = IpynbConverter;
        let info = StreamInfo {
            extension: Some("ipynb".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn converts_notebook() {
        let c = IpynbConverter;
        let info = StreamInfo {
            extension: Some("ipynb".into()),
            ..Default::default()
        };
        let notebook = r##"{
            "cells": [
                {
                    "cell_type": "markdown",
                    "source": ["# Hello\n", "This is markdown"],
                    "metadata": {}
                },
                {
                    "cell_type": "code",
                    "source": "print('hello')",
                    "metadata": {},
                    "outputs": [
                        {
                            "output_type": "stream",
                            "text": ["hello\n"]
                        }
                    ]
                }
            ],
            "metadata": {
                "kernelspec": {
                    "language": "python"
                }
            },
            "nbformat": 4,
            "nbformat_minor": 5
        }"##;
        let result = c.convert(notebook.as_bytes(), &info).unwrap();
        assert!(result.body.contains("# Hello"));
        assert!(result.body.contains("```python"));
        assert!(result.body.contains("print('hello')"));
        assert!(result.body.contains("**Output:**"));
        assert!(result.body.contains("hello"));
    }
}
