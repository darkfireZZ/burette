use {
    anyhow::{anyhow, bail, Context},
    serde::{de::Error, Deserialize, Serialize},
    std::{
        fmt::{self, Display, Formatter},
        path::Path,
        str::FromStr,
    },
};

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
    #[must_use]
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
    #[must_use]
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
        Self::from_mime_type(&s).map_err(Error::custom)
    }
}
