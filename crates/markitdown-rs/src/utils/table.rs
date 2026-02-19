/// Convert a 2D grid of strings into a Markdown table.
/// First row is treated as the header.
pub fn to_markdown_table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }

    // Normalize all rows to same column count
    let rows: Vec<Vec<&str>> = rows
        .iter()
        .map(|r| {
            let mut row: Vec<&str> = r.iter().map(|s| s.as_str()).collect();
            row.resize(cols, "");
            row
        })
        .collect();

    let mut out = String::new();

    // Header
    out.push('|');
    for cell in &rows[0] {
        out.push(' ');
        out.push_str(&cell.replace('|', "\\|"));
        out.push_str(" |");
    }
    out.push('\n');

    // Separator
    out.push('|');
    for _ in 0..cols {
        out.push_str(" --- |");
    }
    out.push('\n');

    // Data rows
    for row in &rows[1..] {
        out.push('|');
        for cell in row {
            out.push(' ');
            out.push_str(&cell.replace('|', "\\|"));
            out.push_str(" |");
        }
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_table() {
        let rows = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];
        let md = to_markdown_table(&rows);
        assert!(md.contains("| Name | Age |"));
        assert!(md.contains("| --- | --- |"));
        assert!(md.contains("| Alice | 30 |"));
    }

    #[test]
    fn test_empty_table() {
        assert_eq!(to_markdown_table(&[]), "");
    }
}
