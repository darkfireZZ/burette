//! Rust library for `burette`, a document library manager.

#![deny(missing_debug_implementations, missing_docs)]

use {
    anyhow::anyhow,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    std::{
        fmt::{self, Display, Formatter},
        str::FromStr,
    },
};

pub mod cli;

/// Metadata for a document stored in the library.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocMetadata {
    title: String,
    authors: Vec<String>,
    isbn: Vec<Isbn13>,
    file_format: FileFormat,
}

/// A 13-digit International Standard Book Number (ISBN).
///
/// ISBNs are used to uniquely identify books. They are typically printed on the back cover of a
/// book, near the barcode.
///
/// ISBNs are 13 digits long, and the last digit is a check digit that is calculated from the first
/// 12 digits.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Isbn13 {
    digits: [u8; 13],
}

impl<'de> Deserialize<'de> for Isbn13 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Isbn13::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Display for Isbn13 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Technically, ISBN-13s should be formatted with hyphens.
        // However, the size of the groups delimited by hyphens is not fixed, which makes
        // formatting a real pain. So we'll just print the digits without hyphens.
        for digit in &self.digits {
            write!(f, "{}", digit)?;
        }
        Ok(())
    }
}

impl FromStr for Isbn13 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut digits = [0; 13];
        let mut count = 0;
        let mut checksum = 0;
        for c in s.chars() {
            if let Some(d) = c.to_digit(10) {
                if count == 13 {
                    return Err(anyhow!("ISBN-13 is too long"));
                }
                digits[count] = d as u8;
                checksum += if count % 2 == 0 { d } else { d * 3 };
                count += 1;
            } else if c == '-' {
                continue;
            } else {
                return Err(anyhow!("Invalid character in ISBN-13: '{}'", c));
            }
        }

        if count != 13 {
            return Err(anyhow!("ISBN-13 is too short"));
        }

        if checksum % 10 != 0 {
            return Err(anyhow!("Invalid ISBN-13 checksum"));
        }

        Ok(Self { digits })
    }
}

impl Serialize for Isbn13 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
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
