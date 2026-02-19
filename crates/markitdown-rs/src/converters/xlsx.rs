use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use crate::utils::table::to_markdown_table;
use calamine::{Data, Reader, Sheets, open_workbook_auto_from_rs};
use std::io::Cursor;

pub struct XlsxConverter;

impl DocumentConverter for XlsxConverter {
    fn name(&self) -> &'static str {
        "XLSX"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(
            info.mime_type.as_deref(),
            Some(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    | "application/vnd.ms-excel"
                    | "application/vnd.oasis.opendocument.spreadsheet"
            )
        ) || matches!(
            info.extension.as_deref(),
            Some("xlsx" | "xls" | "xlsb" | "ods")
        )
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let cursor = Cursor::new(input.to_vec());
        let mut workbook: Sheets<_> = open_workbook_auto_from_rs(cursor)
            .map_err(|e| crate::Error::ConversionFailed(format!("Failed to open workbook: {e}")))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let mut md = String::new();

        for (idx, name) in sheet_names.iter().enumerate() {
            if idx > 0 {
                md.push_str("\n\n");
            }
            md.push_str(&format!("## {name}\n\n"));

            match workbook.worksheet_range(name) {
                Ok(range) => {
                    let mut rows: Vec<Vec<String>> = Vec::new();
                    for row in range.rows() {
                        let cells: Vec<String> =
                            row.iter().map(|cell| cell_to_string(cell)).collect();
                        rows.push(cells);
                    }

                    if rows.is_empty() {
                        md.push_str("*Empty sheet*\n");
                    } else {
                        md.push_str(&to_markdown_table(&rows));
                    }
                }
                Err(e) => {
                    md.push_str(&format!("*Error reading sheet: {e}*\n"));
                }
            }
        }

        let mut result = ConversionResult::new(md.trim_end());
        result = result.with_metadata("sheet_count", sheet_names.len().to_string());
        Ok(result)
    }
}

fn cell_to_string(data: &Data) -> String {
    match data {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if *f == (*f as i64) as f64 {
                format!("{}", *f as i64)
            } else {
                format!("{f}")
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => format!("{dt}"),
        Data::Error(e) => format!("#{e:?}"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_xlsx() {
        let c = XlsxConverter;
        let info = StreamInfo {
            mime_type: Some(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
            ),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn accepts_xls_extension() {
        let c = XlsxConverter;
        let info = StreamInfo {
            extension: Some("xls".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }
}
