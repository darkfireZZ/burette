//! Rust library for `burette`, a document library manager.

#![deny(missing_debug_implementations, missing_docs)]

use {
    anyhow::anyhow,
    serde::{Deserialize, Serialize},
    std::{
        fmt::{self, Display, Formatter},
        str::FromStr,
    },
};

pub mod cli;
mod isbn;

pub use isbn::Isbn13;

/// Metadata for a document stored in the library.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocMetadata {
    title: String,
    authors: Vec<String>,
    isbn: Vec<Isbn13>,
    file_format: FileFormat,
}

/// File formats.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum FileFormat {
    /// EPUB file format.
    Epub,
    /// PDF file format.
    Pdf,
}

impl FileFormat {
    /// Get the file extension for this file format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Epub => "epub",
            Self::Pdf => "pdf",
        }
    }

    /// Parse a MIME type string into a `FileFormat`.
    ///
    /// # Errors
    ///
    /// Returns an error if the MIME type is not recognized.
    pub fn from_mime_type(mime_type: &str) -> Result<Self, anyhow::Error> {
        match mime_type {
            "application/epub+zip" => Ok(Self::Epub),
            "application/pdf" => Ok(Self::Pdf),
            _ => Err(anyhow!("Unknown file format: {}", mime_type)),
        }
    }

    /// Get the MIME type for this file format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
            Self::Pdf => "application/pdf",
        }
    }
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.mime_type().fmt(f)
    }
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_mime_type(s)
    }
}
