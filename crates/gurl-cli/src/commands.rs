use anyhow::{Context, Result};
use clap::Args;
use gurl_core::GurlClient;
use gurl_core::client::{Body, GurlRequest};
use reqwest::Method;
use std::io::{self, IsTerminal, Write};
use url::Url;

#[derive(Args)]
pub struct HttpArgs {
    /// Target URL
    #[arg(default_value = "")]
    pub url: String,

    /// Add request header (repeatable), format: "Key: Value"
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    /// Request body as JSON (sets Content-Type: application/json)
    #[arg(long)]
    pub json: Option<String>,

    /// Request body as raw string
    #[arg(short = 'd', long = "data")]
    pub data: Option<String>,

    /// Form data (repeatable), format: "key=value"
    #[arg(short = 'F', long = "form")]
    pub form: Vec<String>,

    /// Raw output (no envelope, just response body)
    #[arg(long)]
    pub raw: bool,

    /// Quiet output (body content only, no envelope)
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Don't follow redirects
    #[arg(long)]
    pub no_redirect: bool,

    /// Verbose output (include timing and TLS info)
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Request timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Save response body to file
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// Extract a field from the JSON envelope using dot notation (e.g. "content.body", "response.status")
    #[arg(long = "select")]
    pub select: Option<String>,
}

pub async fn execute(method: &str, args: HttpArgs) -> Result<()> {
    let url = Url::parse(&args.url).with_context(|| format!("invalid URL: {}", args.url))?;

    let method: Method = method
        .parse()
        .with_context(|| format!("invalid HTTP method: {method}"))?;

    let mut req = GurlRequest::get(url).with_method(method);

    if args.no_redirect {
        req = req.no_redirects();
    }

    if let Some(secs) = args.timeout {
        req = req.with_timeout(std::time::Duration::from_secs(secs));
    }

    // Parse headers
    let mut header_map = reqwest::header::HeaderMap::new();
    for h in &args.headers {
        let (key, value) = h
            .split_once(':')
            .with_context(|| format!("invalid header format (expected 'Key: Value'): {h}"))?;
        header_map.insert(
            reqwest::header::HeaderName::from_bytes(key.trim().as_bytes())?,
            reqwest::header::HeaderValue::from_str(value.trim())?,
        );
    }
    req = req.with_headers(header_map);

    // Parse body: --json > --data > --form
    if let Some(json_str) = &args.json {
        let value: serde_json::Value =
            serde_json::from_str(json_str).with_context(|| "invalid JSON body")?;
        req = req.with_body(Body::Json(value));
    } else if let Some(data) = &args.data {
        req = req.with_body(Body::Raw(data.as_bytes().to_vec()));
    } else if !args.form.is_empty() {
        let mut pairs = Vec::new();
        for f in &args.form {
            let (key, value) = f
                .split_once('=')
                .with_context(|| format!("invalid form field (expected 'key=value'): {f}"))?;
            pairs.push((key.to_string(), value.to_string()));
        }
        req = req.with_body(Body::Form(pairs));
    }

    let client = GurlClient::new()?;
    let response = client.execute(req).await?;

    // --output: save raw body to file
    if let Some(path) = &args.output {
        std::fs::write(path, &response.content.raw_body)
            .with_context(|| format!("failed to write output to {path}"))?;
        eprintln!("Saved {} bytes to {path}", response.content.raw_body.len());
        return Ok(());
    }

    // --raw: write raw response bytes
    if args.raw {
        io::stdout().write_all(&response.content.raw_body)?;
        return Ok(());
    }

    // --quiet: body content only
    if args.quiet {
        let body_str = match &response.content.body {
            serde_json::Value::String(s) => s.clone(),
            other => serde_json::to_string_pretty(other)?,
        };
        print!("{body_str}");
        return Ok(());
    }

    // --select: extract a field using dot notation
    if let Some(path) = &args.select {
        let envelope = serde_json::to_value(&response)?;
        let selected = select_path(&envelope, path);
        let out = match selected {
            serde_json::Value::String(s) => s,
            serde_json::Value::Null => {
                anyhow::bail!("field '{path}' not found in response envelope");
            }
            other => serde_json::to_string_pretty(&other)?,
        };
        println!("{out}");
        return Ok(());
    }

    // Default: full JSON envelope
    let is_tty = io::stdout().is_terminal();
    let output = if is_tty {
        serde_json::to_string_pretty(&response)?
    } else {
        serde_json::to_string(&response)?
    };
    println!("{output}");

    Ok(())
}

fn select_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut current = value;
    for key in path.split('.') {
        match current {
            serde_json::Value::Object(map) => {
                if let Some(v) = map.get(key) {
                    current = v;
                } else {
                    return serde_json::Value::Null;
                }
            }
            serde_json::Value::Array(arr) => {
                if let Ok(idx) = key.parse::<usize>() {
                    if let Some(v) = arr.get(idx) {
                        current = v;
                    } else {
                        return serde_json::Value::Null;
                    }
                } else {
                    return serde_json::Value::Null;
                }
            }
            _ => return serde_json::Value::Null,
        }
    }
    current.clone()
}
