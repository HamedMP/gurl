use crate::converter::{ConversionResult, DocumentConverter, StreamInfo};
use crate::utils::table::to_markdown_table;

pub struct CsvConverter;

impl DocumentConverter for CsvConverter {
    fn name(&self) -> &'static str {
        "CSV"
    }

    fn accepts(&self, info: &StreamInfo) -> bool {
        matches!(info.mime_type.as_deref(), Some("text/csv"))
            || matches!(info.extension.as_deref(), Some("csv"))
    }

    fn convert(&self, input: &[u8], _info: &StreamInfo) -> crate::Result<ConversionResult> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_reader(input);

        let mut rows: Vec<Vec<String>> = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| crate::Error::ConversionFailed(e.to_string()))?;
            rows.push(record.iter().map(|s| s.to_string()).collect());
        }

        if rows.is_empty() {
            return Ok(ConversionResult::new(""));
        }

        let md = to_markdown_table(&rows);
        Ok(ConversionResult::new(md))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_csv() {
        let c = CsvConverter;
        let info = StreamInfo {
            mime_type: Some("text/csv".into()),
            ..Default::default()
        };
        assert!(c.accepts(&info));
    }

    #[test]
    fn converts_csv_to_table() {
        let c = CsvConverter;
        let info = StreamInfo {
            extension: Some("csv".into()),
            ..Default::default()
        };
        let csv_data = b"Name,Age,City\nAlice,30,NYC\nBob,25,LA\n";
        let result = c.convert(csv_data, &info).unwrap();
        assert!(result.body.contains("| Name | Age | City |"));
        assert!(result.body.contains("| --- |"));
        assert!(result.body.contains("| Alice | 30 | NYC |"));
        assert!(result.body.contains("| Bob | 25 | LA |"));
    }

    #[test]
    fn handles_empty_csv() {
        let c = CsvConverter;
        let info = StreamInfo::default();
        let result = c.convert(b"", &info).unwrap();
        assert!(result.body.is_empty());
    }
}
