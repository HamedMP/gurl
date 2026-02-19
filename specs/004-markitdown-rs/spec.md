# 004 — markitdown-rs: A Rust Port of Microsoft MarkItDown

> **Parent spec:** [001-init](../001-init/spec.md) — Section 5.2 (Content Intelligence)
> **Status:** In progress
> **Dependencies:** 002-scaffolding
> **Estimated effort:** 5-7 days
> **Replaces:** Previous 004-html-to-markdown (narrower scope)

---

## Goal

Build `markitdown-rs` — a standalone Rust crate that is a faithful port of Microsoft's [MarkItDown](https://github.com/microsoft/markitdown) Python library. It converts any document format (HTML, PDF, DOCX, XLSX, PPTX, CSV, EPUB, images, audio metadata, Outlook MSG, Jupyter notebooks, RSS/Atom, ZIP archives) to clean LLM-friendly Markdown.

This crate lives inside gurl's workspace but is designed to be publishable as an independent crate on crates.io. gurl uses it as its content intelligence layer, but anyone can `cargo add markitdown-rs` independently.

## Why a full port?

- MarkItDown has 60K+ GitHub stars and proven conversion quality
- No Rust equivalent exists — this fills a real ecosystem gap
- A single-binary converter with no Python dependency is valuable on its own
- gurl gets best-in-class conversion for free by depending on this crate

## Architecture (mirroring Python MarkItDown)

### Python MarkItDown architecture

```
MarkItDown (orchestrator)
  ├── Converter registry (priority-ordered list)
  ├── File type detection (magika / MIME / extension)
  └── Converters:
       ├── PlainTextConverter
       ├── HtmlConverter          (beautifulsoup4 + markdownify)
       ├── PdfConverter           (pdfminer.six + pdfplumber)
       ├── DocxConverter          (mammoth → HTML → MD)
       ├── XlsxConverter          (pandas + openpyxl → HTML → MD)
       ├── XlsConverter           (pandas + xlrd → HTML → MD)
       ├── PptxConverter          (python-pptx)
       ├── CsvConverter           (csv module)
       ├── EpubConverter          (zipfile + HTML → MD)
       ├── RssConverter           (defusedxml + beautifulsoup4)
       ├── ImageConverter         (exiftool metadata)
       ├── AudioConverter         (exiftool metadata)
       ├── OutlookMsgConverter    (olefile)
       ├── IpynbConverter         (JSON parsing)
       ├── WikipediaConverter     (beautifulsoup4, specialized)
       ├── ZipConverter           (recursively converts contents)
       └── DocumentIntelligenceConverter (Azure AI, cloud)
```

### Rust port architecture

```
crates/markitdown-rs/
├── Cargo.toml
└── src/
    ├── lib.rs                    # MarkItDown struct (orchestrator)
    ├── converter.rs              # DocumentConverter trait + Result type
    ├── detection.rs              # MIME type + magic bytes detection
    ├── converters/
    │   ├── mod.rs
    │   ├── plain_text.rs         # PlainTextConverter
    │   ├── html.rs               # HtmlConverter
    │   ├── pdf.rs                # PdfConverter
    │   ├── docx.rs               # DocxConverter
    │   ├── xlsx.rs               # XlsxConverter
    │   ├── pptx.rs               # PptxConverter
    │   ├── csv.rs                # CsvConverter
    │   ├── epub.rs               # EpubConverter
    │   ├── rss.rs                # RssConverter
    │   ├── image.rs              # ImageConverter (EXIF)
    │   ├── audio.rs              # AudioConverter (EXIF)
    │   ├── outlook_msg.rs        # OutlookMsgConverter
    │   ├── ipynb.rs              # IpynbConverter
    │   ├── wikipedia.rs          # WikipediaConverter
    │   └── zip.rs                # ZipConverter
    └── utils/
        ├── mod.rs
        └── table.rs              # HTML table → Markdown table helper
```

## Core trait (mirrors Python's DocumentConverter)

```rust
pub struct StreamInfo {
    pub mime_type: Option<String>,
    pub extension: Option<String>,
    pub charset: Option<String>,
    pub filename: Option<String>,
    pub url: Option<String>,
}

pub struct ConversionResult {
    pub title: Option<String>,
    pub body: String,
    pub metadata: HashMap<String, String>,
}

pub trait DocumentConverter: Send + Sync {
    fn accepts(&self, stream_info: &StreamInfo) -> bool;
    fn convert(&self, input: &[u8], stream_info: &StreamInfo) -> Result<ConversionResult>;
}

pub struct MarkItDown {
    converters: Vec<Box<dyn DocumentConverter>>,
}

impl MarkItDown {
    pub fn new() -> Self { /* registers all built-in converters */ }
    pub fn convert_bytes(&self, input: &[u8], info: &StreamInfo) -> Result<ConversionResult>;
    pub fn convert_file(&self, path: &Path) -> Result<ConversionResult>;
    pub fn convert_url(&self, url: &str) -> Result<ConversionResult>; // uses reqwest
    pub fn register(&mut self, converter: Box<dyn DocumentConverter>);
}
```

## Converter mapping: Python → Rust

### Tier 1 — Core converters (build first)

| Converter | Python deps | Rust crate(s) | Approach |
|-----------|-------------|---------------|----------|
| **PlainTextConverter** | charset-normalizer | `encoding_rs` | Detect charset, decode to UTF-8, pass through |
| **HtmlConverter** | beautifulsoup4, markdownify | `readabilityrs` + `htmd` | Readability extraction → HTML-to-MD |
| **CsvConverter** | csv (stdlib) | `csv` crate | Parse rows → markdown table |
| **RssConverter** | defusedxml, beautifulsoup4 | `quick-xml` + `scraper` | Parse feed → extract entries → MD |
| **IpynbConverter** | json (stdlib) | `serde_json` | Parse notebook JSON → extract cells |
| **WikipediaConverter** | beautifulsoup4 | `scraper` | Specialized HTML cleanup for Wikipedia |

### Tier 2 — Document converters

| Converter | Python deps | Rust crate(s) | Approach |
|-----------|-------------|---------------|----------|
| **PdfConverter** | pdfminer.six, pdfplumber | `pdf-extract` or `lopdf` | Extract text, basic structure |
| **DocxConverter** | mammoth → HTML → MD | `docx-rs` or `ooxmlsdk` | Parse DOCX XML → build HTML → HtmlConverter |
| **XlsxConverter** | pandas, openpyxl | `calamine` | Read sheets → build HTML tables → HtmlConverter |
| **XlsConverter** | pandas, xlrd | `calamine` (handles both) | Same as XLSX |
| **PptxConverter** | python-pptx | `ooxmlsdk` or custom ZIP+XML | Parse slides → extract text/tables/images |
| **EpubConverter** | zipfile, xml | `zip` + `quick-xml` + HtmlConverter | Unzip → parse OPF → convert HTML chapters |

### Tier 3 — Specialized converters

| Converter | Python deps | Rust crate(s) | Approach |
|-----------|-------------|---------------|----------|
| **ImageConverter** | exiftool (external) | `kamadak-exif` (or `rexiv2`) | Read EXIF metadata → format as MD |
| **AudioConverter** | exiftool (external) | `symphonia` (metadata) or `id3`/`metaflac` | Read audio metadata → format as MD |
| **OutlookMsgConverter** | olefile | Custom OLE2 parser or `cfb` crate | Parse OLE2 → extract email fields |
| **ZipConverter** | zipfile (stdlib) | `zip` crate | Unzip → recursively convert each file |

### Not porting (cloud-only)

| Converter | Reason |
|-----------|--------|
| **DocumentIntelligenceConverter** | Azure cloud service — not local. Could add as optional feature later |
| **YouTubeConverter** | Requires YouTube API/transcript service — cloud dependency |
| **BingSerpConverter** | Requires Bing API — cloud dependency |

## Feature flags

Use Cargo feature flags so users can opt into only the converters they need:

```toml
[features]
default = ["html", "csv", "plain-text", "rss", "ipynb"]
html = ["readabilityrs", "htmd", "scraper"]
pdf = ["pdf-extract"]
docx = ["docx-rs"]
xlsx = ["calamine"]
pptx = []  # custom parser, no extra dep
csv = ["csv"]
epub = ["zip", "quick-xml"]
image = ["kamadak-exif"]
audio = ["symphonia"]
outlook = ["cfb"]
all = ["html", "pdf", "docx", "xlsx", "pptx", "csv", "epub", "image", "audio", "outlook"]
```

This keeps the default binary small (HTML + plaintext + CSV) while allowing `cargo add markitdown-rs --features all` for everything.

## Implementation order

### Phase A — Foundation + HTML (days 1-2)

1. Crate scaffolding: `MarkItDown` struct, `DocumentConverter` trait, `StreamInfo`, `ConversionResult`
2. `detection.rs` — MIME type detection from magic bytes + extension
3. `PlainTextConverter` — charset detection + passthrough
4. `HtmlConverter` — `readabilityrs` (content extraction) + `htmd` (HTML→MD)
5. `CsvConverter` — CSV → markdown table
6. Wire into gurl-core's content pipeline
7. Quality benchmark HTML conversion against Jina Reader

### Phase B — Documents (days 3-4)

8. `PdfConverter` — text extraction from PDFs
9. `DocxConverter` — DOCX → HTML → MD pipeline
10. `XlsxConverter` / `XlsConverter` — spreadsheet sheets → tables
11. `EpubConverter` — EPUB chapters → MD
12. `RssConverter` — RSS/Atom feed → MD

### Phase C — Media + Specialty (days 5-6)

13. `ImageConverter` — EXIF metadata extraction
14. `AudioConverter` — audio file metadata
15. `OutlookMsgConverter` — OLE2 email parsing
16. `IpynbConverter` — Jupyter notebook cells
17. `PptxConverter` — PowerPoint slides
18. `ZipConverter` — recursive archive conversion
19. `WikipediaConverter` — specialized Wikipedia HTML cleanup

### Phase D — Polish (day 7)

20. CLI integration: `gurl get <url>` auto-converts based on content type
21. Standalone CLI: `markitdown-rs convert <file>` (like Python's `markitdown` CLI)
22. Integration tests against MarkItDown's own test fixtures
23. Benchmark suite: speed comparison vs Python MarkItDown

## Acceptance criteria

### Core
- [ ] `MarkItDown::new()` registers all built-in converters
- [ ] `convert_file("test.html")` produces clean markdown
- [ ] `convert_file("test.pdf")` extracts text
- [ ] `convert_file("test.docx")` converts Word docs
- [ ] `convert_file("test.xlsx")` converts spreadsheet tables
- [ ] `convert_file("test.csv")` produces markdown table
- [ ] `convert_file("test.epub")` converts ebook chapters
- [ ] `convert_file("test.pptx")` extracts slide content
- [ ] `convert_file("test.ipynb")` extracts notebook cells
- [ ] `convert_file("test.rss")` parses feed entries
- [ ] `convert_file("test.jpg")` extracts EXIF metadata
- [ ] `convert_file("test.msg")` extracts email fields
- [ ] `convert_file("test.zip")` recursively converts contents
- [ ] Unknown formats return an appropriate error

### Quality
- [ ] HTML conversion quality competitive with Jina Reader on 5 test pages
- [ ] PDF text extraction handles multi-page documents
- [ ] DOCX preserves headings, lists, tables, bold/italic
- [ ] XLSX preserves multi-sheet structure
- [ ] Tables render as proper markdown tables

### Integration
- [ ] `gurl get https://example.com` uses HtmlConverter automatically
- [ ] `gurl get https://example.com/report.pdf` uses PdfConverter
- [ ] Feature flags work: `--features html` compiles without pdf/docx deps
- [ ] Standalone `markitdown-rs` crate is publishable independently

## Rust crate dependencies

```toml
[dependencies]
# Core
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
encoding_rs = "0.8"

# HTML (default)
readabilityrs = { version = "0.1", optional = true }
htmd = { version = "0.5", optional = true }
scraper = { version = "0.22", optional = true }

# PDF
pdf-extract = { version = "0.7", optional = true }

# Office documents
calamine = { version = "0.26", optional = true }    # XLSX/XLS
docx-rs = { version = "0.4", optional = true }      # DOCX

# Structured formats
csv = { version = "1", optional = true }
quick-xml = { version = "0.37", optional = true }   # RSS, EPUB metadata
zip = { version = "2", optional = true }             # EPUB, ZIP

# Media metadata
kamadak-exif = { version = "0.6", optional = true }  # Image EXIF
symphonia = { version = "0.5", optional = true, features = ["all-codecs"] }  # Audio metadata

# Email
cfb = { version = "0.10", optional = true }          # OLE2 (Outlook MSG)

# Detection
infer = "0.16"                                        # Magic bytes detection
mime_guess = "2"                                      # Extension → MIME
```

## Notes

- The `WikipediaConverter` in Python is essentially HtmlConverter with specialized cleanup for Wikipedia's DOM structure. In Rust, implement as a thin wrapper around HtmlConverter that strips Wikipedia-specific elements (sidebar, references, edit links).
- The `PptxConverter` is the hardest converter — PPTX is a ZIP of XML files with complex relationships. Consider using the `zip` + `quick-xml` approach rather than looking for a PPTX-specific crate.
- Python's `mammoth` (DOCX converter) works by converting DOCX → HTML → MD. Our Rust port should follow the same two-stage approach: parse DOCX XML → generate HTML → run through HtmlConverter.
- The `DocumentIntelligenceConverter` is an Azure cloud service. We skip it for the local tool but could add it as an optional feature behind a feature flag + API key config.
