use anyhow::{Context, Result};
use clap::Args;
use gurl_core::GurlClient;
use gurl_core::client::{Body, GurlRequest};
use reqwest::Method;
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

    /// Raw output (no envelope, just response body)
    #[arg(long)]
    pub raw: bool,

    /// Don't follow redirects
    #[arg(long)]
    pub no_redirect: bool,

    /// Verbose output (include timing and TLS info)
    #[arg(short = 'v', long)]
    pub verbose: bool,
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

    // Parse body
    if let Some(json_str) = &args.json {
        let value: serde_json::Value =
            serde_json::from_str(json_str).with_context(|| "invalid JSON body")?;
        req = req.with_body(Body::Json(value));
    } else if let Some(data) = &args.data {
        req = req.with_body(Body::Raw(data.as_bytes().to_vec()));
    }

    let client = GurlClient::new()?;
    let response = client.execute(req).await?;

    if args.raw {
        use std::io::Write;
        std::io::stdout().write_all(&response.content.raw_body)?;
    } else {
        let output = serde_json::to_string_pretty(&response)?;
        println!("{output}");
    }

    Ok(())
}
