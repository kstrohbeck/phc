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
                hash: hash.into(),
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
            Salt(salt) => write!(f, "{}", salt),
            Both { salt, hash } => write!(f, "{}${}", salt, encode_config(hash, STANDARD_NO_PAD)),
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
    params: Vec<(String, String)>,

    /// The salt and hash encoded in this PHC.
    salt_and_hash: SaltAndHash,
}

impl fmt::Display for RawPHC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}", self.id)?;
        if self.params.len() > 0 {
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

    pub fn params(&self) -> Vec<(&str, &str)> {
        self.params.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect()
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

    mod params {
        type Entry<'a> = (&'a str, &'a [(&'a str, &'a str)]);
        pub(crate) const NONE: Entry = ("", &[]);
        pub(crate) const ONE: Entry = ("$i=10000", &[("i", "10000")]);
        pub(crate) const TWO: Entry = ("$i=10000,mem=heap", &[("i", "10000"), ("mem", "heap")]);
    }

    mod salt_and_hash {
        type Entry<'a> = (&'a str, Option<&'a str>, Option<&'a [u8]>);
        pub(crate) const NEITHER: Entry = ("", None, None);
        pub(crate) const SALT: Entry = ("$abcdefg", Some("abcdefg"), None);
        pub(crate) const BOTH: Entry = ("$abcdefg$aGVsbG8", Some("abcdefg"), Some(b"hello"));
    }

    macro_rules! test_phc {
        ($name:ident, $id:expr, $params:expr, $salt_and_hash:expr $(,)*) => {
            #[test]
            fn $name() {
                let raw = format!("${}{}{}", $id, $params.0, $salt_and_hash.0);
                println!("{}", raw);
                let phc: RawPHC = raw.parse().unwrap();
                assert_eq!(phc.id, $id);
                assert_eq!(phc.params(), $params.1);
                // TODO: This feels like it could be written more eloquently.
                match phc.salt() {
                    None => assert_eq!(None, $salt_and_hash.1),
                    Some(salt) => {
                        let salt_str = salt.to_string();
                        assert_eq!(Some(salt_str.as_str()), $salt_and_hash.1);
                    }
                }
                assert_eq!(phc.hash(), $salt_and_hash.2);
            }
        };
    }

    test_phc!(
        param_string_no_params,
        "abc-123",
        params::NONE,
        salt_and_hash::NEITHER,
    );

    test_phc!(
        param_string_one_param,
        "abc-123",
        params::ONE,
        salt_and_hash::NEITHER,
    );

    test_phc!(
        param_string_two_params,
        "abc-123",
        params::TWO,
        salt_and_hash::NEITHER,
    );

    test_phc!(
        salt_string_no_params,
        "abc-123",
        params::NONE,
        salt_and_hash::SALT,
    );

    test_phc!(
        salt_string_one_param,
        "abc-123",
        params::ONE,
        salt_and_hash::SALT,
    );

    test_phc!(
        salt_string_two_params,
        "abc-123",
        params::TWO,
        salt_and_hash::SALT,
    );

    test_phc!(
        hash_string_no_params,
        "abc-123",
        params::NONE,
        salt_and_hash::BOTH,
    );

    test_phc!(
        hash_string_one_param,
        "abc-123",
        params::ONE,
        salt_and_hash::BOTH,
    );

    test_phc!(
        hash_string_two_params,
        "abc-123",
        params::TWO,
        salt_and_hash::BOTH,
    );
}
