use super::raw::{RawPHC, SaltAndHash};
use base64::{decode_config, STANDARD_NO_PAD};
use nom::types::CompleteStr;
use nom::*;

fn is_name_char(c: char) -> bool {
    match c {
        'a'..='z' | '0'..='9' | '-' => true,
        _ => false,
    }
}

fn is_value_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '+' | '.' | '-' => true,
        _ => false,
    }
}

fn is_base_64_char(c: char) -> bool {
    match c {
        'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '+' => true,
        _ => false,
    }
}

macro_rules! as_ref {
    ($i:expr, $submac:ident!( $( $args:tt )* )) => {
        map!($i, $submac!($( $args )*), |x| x.0)
    }
}

macro_rules! or_else {
    ($i:expr, $submac:ident!( $( $args:tt )*), $f:expr) => {
        {
            match $submac!($i, $( $args )*) {
                Ok((i, o)) => Ok((i, o)),
                Err(nom::Err::Error(_)) => Ok(($i, $f())),
                Err(e) => Err(e),
            }
        }
    }
}

named!(name<CompleteStr, &str>, as_ref!(take_while1!(is_name_char)));

named!(value<CompleteStr, &str>, as_ref!(take_while1!(is_value_char)));

named!(base_64<CompleteStr, Vec<u8>>,
    map_res!(
        take_while1!(is_base_64_char),
        |enc: CompleteStr| decode_config(enc.as_bytes(), STANDARD_NO_PAD)
    )
);

named!(id<CompleteStr, &str>, preceded!(char!('$'), name));

named!(params<CompleteStr, Vec<(String, String)>>,
    or_else!(
        preceded!(
            char!('$'),
            separated_nonempty_list!(
                char!(','),
                map!(
                    separated_pair!(name, char!('='), value),
                    |(k, v)| (k.to_string(), v.to_string())
                )
            )
        ),
        Vec::new
    )
);

named!(salt_and_hash<CompleteStr, SaltAndHash>,
    map!(
        opt!(
            tuple!(
                preceded!(char!('$'), value),
                opt!(preceded!(char!('$'), base_64))
            )
        ),
        SaltAndHash::from_option
    )
);

named!(phc<CompleteStr, RawPHC>,
    do_parse!(
        i: id >>
        p: params >>
        sh: salt_and_hash >>
        (RawPHC::from_parts(i, p, sh))
    )
);

pub(crate) fn parse_phc(raw: &str) -> Result<RawPHC, Err<CompleteStr>> {
    phc(CompleteStr(raw)).map(|x| x.1)
}

#[cfg(test)]
mod tests {
    use super::{parse_phc, RawPHC};

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
                let phc: RawPHC = parse_phc(&raw).unwrap();
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
