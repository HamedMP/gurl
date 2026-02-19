pub mod client;
pub mod output;

pub use client::GurlClient;
pub use output::envelope::GurlResponse;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("request failed: {0}")]
    Request(String),

    #[error("invalid URL: {0}")]
    InvalidUrl(String),
}
