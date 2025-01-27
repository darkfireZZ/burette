//! Rust library for `burette`, a document library manager.

#![warn(
    clippy::dbg_macro,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    missing_debug_implementations,
    missing_docs
)]

use {
    anyhow::{anyhow, bail, Context},
    serde::{Deserialize, Serialize},
    std::{
        env,
        fmt::{self, Debug, Display, Formatter},
        path::{Path, PathBuf},
        str::FromStr,
    },
};

pub mod cli;
pub mod sha256;

mod library;
pub use library::*;

mod isbn;
pub use isbn::Isbn13;

/// Format a string into a format suitable for use as a file name.
pub fn format_as_file_name(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_whitespace = true;
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {
                result.push(c.to_ascii_lowercase());
                prev_was_whitespace = false;
            }
            c if c == '-' || c == '_' || c.is_whitespace() => {
                if !prev_was_whitespace {
                    result.push('_');
                }
                prev_was_whitespace = true;
            }
            _ => {}
        }
    }
    result
}

/// Return the home directory of the current user.
fn home_dir() -> anyhow::Result<PathBuf> {
    let home_dir =
        env::var_os("HOME").ok_or_else(|| anyhow!("Failed to read HOME environment variable"))?;
    Ok(PathBuf::from(home_dir))
}

/// Return the location of the default library directory.
///
/// The default library directory is `$HOME/.book-store`.
pub fn default_library_dir() -> anyhow::Result<PathBuf> {
    Ok(home_dir()
        .context("Failed to determine library directory")?
        .join(".book-store"))
}

/// File formats.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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

    /// Determines the file format from a file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file format cannot be determined due to IO errors or if the file
    /// format is not recognized.
    pub fn from_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let format = file_format::FileFormat::from_file(&path).with_context(|| {
            format!(
                "Failed to determine file format for {}",
                path.as_ref().display()
            )
        })?;
        Ok(match format {
            file_format::FileFormat::ElectronicPublication => Self::Epub,
            file_format::FileFormat::PortableDocumentFormat => Self::Pdf,
            _ => bail!(
                "Unsupported file format: {}",
                format.short_name().unwrap_or(format.name())
            ),
        })
    }
}

impl Display for FileFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self.mime_type(), f)
    }
}

impl FromStr for FileFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_mime_type(s)
    }
}

impl Serialize for FileFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.mime_type())
    }
}

impl<'de> Deserialize<'de> for FileFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_mime_type(&s).map_err(serde::de::Error::custom)
    }
}
