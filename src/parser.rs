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
