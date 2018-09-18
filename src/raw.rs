//! # Raw PHC
//!
//! Contains functions and structures related to raw, unassociated PHC string
//! data.
//!
//! These structures are mostly used to marshal data between a serialized
//! string format and a hash function specific format that knows how to perform
//! actions.

use super::parser::parse_phc;
use super::salt::Salt;
use base64::{encode_config, STANDARD_NO_PAD};
use std::fmt;
use std::str::FromStr;

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
                hash,
            },
        }
    }

    pub(crate) fn salt(&self) -> Option<&Salt> {
        use self::SaltAndHash::*;
        match self {
            Neither => None,
            Salt(salt) => Some(salt),
            Both { salt, .. } => Some(salt),
        }
    }

    pub(crate) fn hash(&self) -> Option<&[u8]> {
        use self::SaltAndHash::*;
        match self {
            Both { hash, .. } => Some(&hash),
            _ => None,
        }
    }
}

impl fmt::Display for SaltAndHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SaltAndHash::*;
        match self {
            Neither => Ok(()),
            Salt(salt) => write!(f, "${}", salt),
            Both { salt, hash } => write!(f, "${}${}", salt, encode_config(hash, STANDARD_NO_PAD)),
        }
    }
}

/// A parsed PHC string that has not been associated with a hash function.
#[derive(Debug)]
pub struct RawPHC {
    /// The id of the hash function that this PHC string describes.
    id: String,

    /// A list of key-value pairs that describe the parameters that the hash
    /// function should use.
    params: Vec<(String, String)>,

    /// The salt and hash encoded in this PHC.
    salt_and_hash: SaltAndHash,
}

impl fmt::Display for RawPHC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.id)?;
        if !self.params.is_empty() {
            write!(f, "$")?;
            let (k, v) = &self.params[0];
            write!(f, "{}={}", k, v)?;
            for (k, v) in &self.params[1..] {
                write!(f, ",{}={}", k, v)?;
            }
        }
        write!(f, "{}", self.salt_and_hash)
    }
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

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn params(&self) -> Vec<(&str, &str)> {
        self.params
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect()
    }

    pub fn salt(&self) -> Option<&Salt> {
        self.salt_and_hash.salt()
    }

    pub fn hash(&self) -> Option<&[u8]> {
        self.salt_and_hash.hash()
    }
}

impl FromStr for RawPHC {
    // TODO: Have a better error type.
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_phc(s).map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::RawPHC;

    macro_rules! parse_iso_test {
        ($name:ident, $raw:expr) => {
            #[test]
            fn $name() {
                let raw = String::from($raw);
                let phc: RawPHC = raw.parse().unwrap();
                assert_eq!(raw, phc.to_string());
            }
        };
    }

    parse_iso_test!(param_string_no_params_is_iso, "$abc-123");
    parse_iso_test!(param_string_one_param_is_iso, "$abc-123$i=10000");
    parse_iso_test!(param_string_two_params_is_iso, "$abc-123$i=10000,mem=heap");
    parse_iso_test!(salt_string_no_params_is_iso, "$abc-123$abcdefg");
    parse_iso_test!(salt_string_one_param_is_iso, "$abc-123$i=10000$abcdefg");
    parse_iso_test!(
        salt_string_two_params_is_iso,
        "$abc-123$i=10000,mem=heap$abcdefg"
    );
    parse_iso_test!(
        heap_string_no_params_is_iso,
        "$abc-123$abcdefg$c29tZSBzYWx0"
    );
    parse_iso_test!(
        heap_string_one_param_is_iso,
        "$abc-123$i=10000$abcdefg$c29tZSBzYWx0"
    );
    parse_iso_test!(
        heap_string_two_params_is_iso,
        "$abc-123$i=10000,mem=heap$abcdefg$c29tZSBzYWx0"
    );
}
