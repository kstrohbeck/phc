//! # Salt
//!
//! Enum containing salt values (ASCII or binary.)

use base64::{decode_config, encode_config, STANDARD_NO_PAD};
use std::fmt;

/// Salt value used in hashing.
#[derive(Clone, Debug, PartialEq)]
pub enum Salt {
    /// A string with characters in the range [a-zA-Z0-9/+.-].
    Ascii(String),

    /// A binary value encoded as Base64.
    Binary(Vec<u8>),
}

impl Salt {
    /// Returns a version of this salt interpreted as binary.
    ///
    /// # Examples
    ///
    /// ```
    /// use phc::Salt;
    /// let salt = Salt::from("c29tZSBzYWx0");
    /// assert_eq!(salt.as_binary(), Salt::from(b"some salt"));
    /// ```
    /// If the salt is invalid ascii, it's returned without change.
    /// ```
    /// use phc::Salt;
    /// let salt = Salt::from("Not salt!");
    /// assert_eq!(salt.clone().as_binary(), salt);
    /// ```
    pub fn as_binary(self) -> Salt {
        use self::Salt::*;
        match self {
            Ascii(ascii) => match decode_config(&ascii, STANDARD_NO_PAD) {
                Ok(binary) => Binary(binary),
                _ => Ascii(ascii),
            },
            Binary(_) => self,
        }
    }
}

impl From<String> for Salt {
    fn from(ascii: String) -> Salt {
        Salt::Ascii(ascii)
    }
}

impl From<&str> for Salt {
    fn from(ascii: &str) -> Salt {
        Salt::Ascii(ascii.into())
    }
}

impl From<Vec<u8>> for Salt {
    fn from(binary: Vec<u8>) -> Salt {
        Salt::Binary(binary)
    }
}

impl From<&[u8]> for Salt {
    fn from(binary: &[u8]) -> Salt {
        Salt::Binary(binary.into())
    }
}

macro_rules! salt_from_array_ref {
    ($( $n:expr ),* $(,)*) => {
        $(
            impl From<&[u8; $n]> for Salt {
                fn from(binary: &[u8; $n]) -> Salt {
                    Salt::Binary(binary.to_vec())
                }
            }

            impl From<[u8; $n]> for Salt {
                fn from(binary: [u8; $n]) -> Salt {
                    Salt::Binary(binary.to_vec())
                }
            }
        )*
    }
}

salt_from_array_ref!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

impl fmt::Display for Salt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Salt::Ascii(ascii) => f.write_str(&ascii),
            Salt::Binary(binary) => write!(f, "{}", encode_config(binary, STANDARD_NO_PAD)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Salt;

    #[test]
    fn ascii_salt_serializes() {
        let salt = Salt::from("abcdefg");
        assert_eq!(salt.to_string(), "abcdefg");
    }

    #[test]
    fn binary_salt_serializes_to_base_64() {
        let salt = Salt::from(b"some salt");
        assert_eq!(salt.to_string(), "c29tZSBzYWx0");
    }

    #[test]
    fn base_64_ascii_salt_converts_to_binary() {
        let salt = Salt::from("c29tZSBzYWx0");
        assert_eq!(salt.as_binary(), Salt::from(b"some salt"));
    }

    #[test]
    fn binary_ascii_salt_to_binary_is_same() {
        let salt = Salt::from(b"some salt");
        assert_eq!(salt.clone().as_binary(), salt);
    }

    #[test]
    fn non_base_64_ascii_doesnt_convert_to_binary() {
        let salt = Salt::from("Not base64!!!");
        assert_eq!(salt.clone().as_binary(), salt);
    }
}
