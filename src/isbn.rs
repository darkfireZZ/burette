use {
    anyhow::bail,
    serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer},
    std::{
        fmt::{self, Display, Formatter},
        str::FromStr,
    },
};

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
        Isbn13::from_str(&s).map_err(Error::custom)
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
                    bail!("ISBN-13 is too long");
                }
                digits[count] = d as u8;
                checksum += if count % 2 == 0 { d } else { d * 3 };
                count += 1;
            } else if c == '-' {
                continue;
            } else {
                bail!("Invalid character in ISBN-13: '{}'", c);
            }
        }

        if count != 13 {
            bail!("ISBN-13 is too short");
        }

        if checksum % 10 != 0 {
            bail!("Invalid ISBN-13 checksum");
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

#[cfg(test)]
mod tests {
    use {super::Isbn13, std::str::FromStr};

    #[test]
    fn valid_isbn13_hyphens_correct() {
        let isbn = Isbn13::from_str("978-3-16-148410-0").expect("valid ISBN-13");
        assert_eq!(isbn.to_string(), "9783161484100");
    }

    #[test]
    fn valid_isbn13_hyphens_weird() {
        let isbn = Isbn13::from_str("--978-1250----066-1-1-4-----").expect("valid ISBN-13");
        assert_eq!(isbn.to_string(), "9781250066114");
    }

    #[test]
    fn valid_isbn13_no_hyphens() {
        let isbn = Isbn13::from_str("9780375826696").expect("valid ISBN-13");
        assert_eq!(isbn.to_string(), "9780375826696");
    }

    #[test]
    fn invalid_isbn_empty() {
        assert!(Isbn13::from_str("").is_err());
    }

    #[test]
    fn invalid_isbn_too_short() {
        assert!(Isbn13::from_str("3-14159-26535-8").is_err());
    }

    #[test]
    fn invalid_isbn_too_long() {
        assert!(Isbn13::from_str("2-71828-18284-590").is_err());
    }

    #[test]
    fn invalid_isbn_invalid_char() {
        assert!(Isbn13::from_str("978-039331604-X").is_err());
    }

    #[test]
    fn invalid_isbn_invalid_checksum() {
        assert!(Isbn13::from_str("9780375826695").is_err());
    }
}
