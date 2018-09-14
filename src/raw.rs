//! # Raw PHC
//!
//! Contains functions and structures related to raw, unassociated PHC string
//! data.
//!
//! These structures are mostly used to marshal data between a serialized
//! string format and a hash function specific format that knows how to perform
//! actions.

use std::fmt;

/// Salt value used in hashing.
#[derive(Debug, PartialEq)]
pub enum Salt {
    /// A string with characters in the range [a-zA-Z0-9/+.-].
    Ascii(String),

    /// A binary value encoded as Base64.
    Binary(Vec<u8>),
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

impl fmt::Display for Salt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Salt::Ascii(ascii) => write!(f, "${}", ascii),
            // TODO: Base64 encode this.
            Salt::Binary(_binary) => write!(f, "$???",),
        }
    }
}

/// Salt and hash information stored in the PHC string.
#[derive(Debug, PartialEq)]
pub enum SaltAndHash {
    /// A string that has neither salt nor hash values.
    Neither,

    /// A string that has a salt value.
    Salt(Salt),

    /// A string that has both a salt and a hash value.
    Both { salt: Salt, hash: Vec<u8> },
}

impl SaltAndHash {
    /// Create a SaltAndHash from a nested pair of salt and hash.
    ///
    /// To enforce the requirements that a hash can never exist in a PHC string
    /// without a hash, the input pair can only have a hash when it has a salt.
    pub(crate) fn from_option<T: Into<Salt>>(
        salt_and_hash: Option<(T, Option<Vec<u8>>)>,
    ) -> SaltAndHash {
        use self::SaltAndHash::*;
        match salt_and_hash {
            None => Neither,
            Some((salt, None)) => Salt(salt.into()),
            Some((salt, Some(hash))) => Both {
                salt: salt.into(),
                hash: hash.into(),
            },
        }
    }
}

/// A parsed PHC string that has not been associated with a hash function.
#[derive(Debug)]
pub struct RawPHC {
    /// The id of the hash function that this PHC string describes.
    pub id: String,

    /// A list of key-value pairs that describe the parameters that the hash
    /// function should use.
    pub params: Vec<(String, String)>,

    /// The salt and hash encoded in this PHC.
    pub salt_and_hash: SaltAndHash,
}

impl RawPHC {
    /// Create a PHC struct from parts, usually parsed from a string.
    pub(crate) fn from_parts<T: Into<String>, U: Into<Vec<(String, String)>>>(
        id: T,
        params: U,
        salt_and_hash: SaltAndHash,
    ) -> RawPHC {
        RawPHC {
            id: id.into(),
            params: params.into(),
            salt_and_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Salt;

    #[test]
    fn ascii_salt_serializes() {
        let salt = Salt::from("abcdefg");
        assert_eq!(format!("{}", salt), "$abcdefg");
    }
}
