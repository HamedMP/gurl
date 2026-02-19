#[cfg(feature = "html")]
pub mod html;

#[cfg(feature = "csv-convert")]
pub mod csv;

pub mod plain_text;

#[cfg(feature = "rss")]
pub mod rss;

pub mod ipynb;

#[cfg(feature = "pdf")]
pub mod pdf;

#[cfg(feature = "docx")]
pub mod docx;

#[cfg(feature = "xlsx")]
pub mod xlsx;

#[cfg(feature = "epub")]
pub mod epub;

#[cfg(feature = "image")]
pub mod image;

#[cfg(feature = "outlook")]
pub mod outlook_msg;

#[cfg(feature = "pptx")]
pub mod pptx;

#[cfg(feature = "zip-convert")]
pub mod zip;

#[cfg(feature = "wikipedia")]
pub mod wikipedia;
