//! SHA-256 hash functions.

use {
    anyhow::anyhow,
    serde::{de::Error, Deserialize, Serialize},
    sha2::{Digest, Sha256},
    std::{
        fmt::{self, Debug, Display, Formatter, Write},
        io::{self, Read},
        str::FromStr,
    },
};

/// [SHA-256 hash](https://en.wikipedia.org/wiki/SHA-2).
#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Hash {
    bytes: [u8; 32],
}

fn hex_digit(byte: u8) -> anyhow::Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(anyhow!("Invalid character in SHA-256 hash")),
    }
}

impl Hash {
    const SHORT_STRING_LENGTH: usize = 12;

    /// Returns a short string representation of the hash.
    ///
    /// This is useful for displaying the hash in a user interface.
    pub fn to_short_string(&self) -> String {
        let mut result = String::with_capacity(Self::SHORT_STRING_LENGTH);
        for byte in &self.bytes[..Self::SHORT_STRING_LENGTH / 2] {
            write!(result, "{:02x}", byte).expect("Writing to string cannot fail");
        }
        result
    }

    /// Creates a new [`struct@Hash`] object from a hex-encoded string.
    ///
    /// # Errors
    ///
    /// This function fails if the input string is not 64 characters long or contains characters
    /// other than hexadecimal digits.
    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        let slice = hex.as_bytes();
        if slice.len() != 64 {
            return Err(anyhow!("Invalid SHA-256 hash length"));
        }
        let mut bytes = [0; 32];

        for (i, byte) in bytes.iter_mut().enumerate() {
            let hi = slice[i * 2];
            let lo = slice[i * 2 + 1];
            *byte = hex_digit(hi)? << 4 | hex_digit(lo)?;
        }

        Ok(Hash { bytes })
    }
}

impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Hash {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for Hash {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s)
    }
}

impl Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_hex(&s).map_err(Error::custom)
    }
}

/// Hashes the contents of the given reader and returns the SHA-256 hash.
///
/// # Errors
///
/// This function fails if it cannot read from the reader.
///
/// # Examples
///
/// ```
/// # use burette::sha256;
/// let source = b"Hello, World!";
/// let hash = sha256::hash_reader(&source[..]).unwrap();
/// assert_eq!(hash.to_string(), "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f");
/// ```
pub fn hash_reader<R: Read>(mut reader: R) -> io::Result<Hash> {
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    while let Ok(count) = reader.read(&mut buffer) {
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    let bytes = hasher.finalize();
    Ok(Hash {
        bytes: bytes.into(),
    })
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        anyhow::anyhow,
        std::{
            io::Read,
            process::{Command, Stdio},
        },
    };

    fn hash_reader_reference_impl<R: Read + Send + 'static>(
        mut reader: R,
    ) -> anyhow::Result<String> {
        let sha_proc = Command::new("sha256sum")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut stdin = sha_proc
            .stdin
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let mut stdout = sha_proc
            .stdout
            .ok_or_else(|| anyhow!("Failed to open stdout"))?;

        std::thread::spawn(move || {
            std::io::copy(&mut reader, &mut stdin).expect("Failed to copy data");
        });

        let mut output = String::new();
        stdout.read_to_string(&mut output)?;

        Ok(output
            .split_whitespace()
            .next()
            .expect("sha256sum output format always contains 2 parts")
            .to_string())
    }

    #[test]
    fn hash_reader_test() {
        let source = "Some test data...";
        let hash = hash_reader(source.as_bytes()).unwrap();
        let hash_str = hash.to_string();

        let hash_ref = hash_reader_reference_impl(source.as_bytes()).unwrap();

        assert_eq!(hash_str, hash_ref);
    }

    #[test]
    fn from_to_string() {
        let hash =
            Hash::from_hex("dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f")
                .expect("Valid hash");
        assert_eq!(
            hash.to_string(),
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }

    #[test]
    fn to_from_string() {
        let source = "hui";
        let original = hash_reader(source.as_bytes()).unwrap();
        let hash_str = original.to_string();
        let hash = Hash::from_str(&hash_str).expect("Valid hash");
        assert_eq!(hash, original);
    }
}
