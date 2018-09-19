use std::fmt::Display;
use std::str::FromStr;

/// A type that transforms hash function parameters to and from the raw,
/// serialized version.
pub trait Param {
    /// The type of this parameter.
    type Output;

    /// The parameter's name.
    fn name(&self) -> &str;

    /// The default value for the parameter if it isn't included, or None if
    /// the parameter is required.
    fn default(&self) -> Option<Self::Output>;

    /// Extract the parameter's value from a serialized string, returning the
    /// value or None if parsing failed.
    fn extract(&self, raw: &str) -> Option<Self::Output>;

    /// Serialize a value to a string.
    fn serialize(&self, value: Self::Output) -> Option<String>;
}

/// A generic parameter, suitable for most uses.
///
/// # Examples
/// You can create a GenParam with the `param!` macro:
/// ```
/// # #[macro_use] extern crate phc; fn main() {
/// # use phc::GenParam;
/// let param_i: GenParam<u32> = param!(i);
/// let param_j = param!(j: bool);
/// let param_k = param!(k = false);
/// # }
/// ```
///
/// GenParam implements Param for any type that implements FromStr.
pub struct GenParam<'a, T> {
    /// The name of the param.
    name: &'a str,

    /// The default param value.
    default: Option<T>,
}

impl<'a, T> GenParam<'a, T> {
    /// Create a new param without a default value.
    pub fn new(name: &'a str) -> GenParam<'a, T> {
        GenParam {
            name,
            default: None,
        }
    }

    /// Create a new param with a default value.
    pub fn with_default(name: &'a str, default: T) -> GenParam<'a, T> {
        GenParam {
            name,
            default: Some(default),
        }
    }
}

impl<'a, T> Param for GenParam<'a, T>
where
    T: Clone + Display + FromStr + PartialEq,
{
    type Output = T;

    fn name(&self) -> &str {
        &self.name
    }

    fn default(&self) -> Option<Self::Output> {
        self.default.clone()
    }

    fn extract(&self, raw: &str) -> Option<Self::Output> {
        raw.parse().ok()
    }

    fn serialize(&self, value: Self::Output) -> Option<String> {
        match &self.default {
            None => Some(value.to_string()),
            Some(default) => if value != *default {
                Some(value.to_string())
            } else {
                None
            },
        }
    }
}

/// Macro...
#[macro_export]
macro_rules! param {
    ($name:ident) => {
        $crate::GenParam::new(stringify!($name))
    };
    ($name:ident: $t:ty) => {
        $crate::GenParam::<$t>::new(stringify!($name))
    };
    ($name:ident = $def:expr) => {
        $crate::GenParam::with_default(stringify!($name), $def)
    };
    ($name:ident: $t:ty = $def:expr) => {
        $crate::GenParam::<$t>::with_default(stringify!($name), $def)
    };
}

pub struct RawParamSlice<'a>(&'a [(&'a str, &'a str)]);

impl<'a> RawParamSlice<'a> {
    fn next_if_key(&mut self, key: &str) -> Option<&str> {
        match self.0.split_first() {
            Some(((ref k, ref v), rest)) if *k == key => {
                self.0 = rest;
                Some(v)
            }
            _ => None,
        }
    }

    pub fn extract_single<P>(&mut self, param: &P) -> Option<P::Output>
    where
        P: Param,
    {
        match self.next_if_key(param.name()) {
            None => param.default(),
            Some(val) => param.extract(val),
        }
    }
}

pub trait ParamSet {
    type Params;

    // TODO: Make this return a Result.
    fn extract(&self, raw: RawParamSlice) -> Option<Self::Params>;
}

pub(crate) fn extract_param_set<P>(param_set: &P, raw: &[(&str, &str)]) -> Option<P::Params>
where
    P: ParamSet,
{
    param_set.extract(RawParamSlice(raw))
}

impl<P> ParamSet for P
where
    P: Param,
{
    type Params = P::Output;

    fn extract(&self, mut raw: RawParamSlice) -> Option<Self::Params> {
        raw.extract_single(self)
    }
}

macro_rules! tuple_param_set {
    ($( $name:ident: $t:ident ),+ $(,)*) => {
        impl<$( $t ),+> ParamSet for ($( $t,)+)
        where $(
            $t: Param,
        )* {
            type Params = ($( $t::Output, )+);

            fn extract(&self, mut raw: RawParamSlice) -> Option<Self::Params> {
                let ($( $name, )+) = self;
                Some(($(raw.extract_single($name)?,)+))
            }
        }
    }
}

tuple_param_set!(a: A);
tuple_param_set!(a: A, b: B);
tuple_param_set!(a: A, b: B, c: C);
tuple_param_set!(a: A, b: B, c: C, d: D);
tuple_param_set!(a: A, b: B, c: C, d: D, e: E);
tuple_param_set!(a: A, b: B, c: C, d: D, e: E, f: F);
tuple_param_set!(a: A, b: B, c: C, d: D, e: E, f: F, g: G);
tuple_param_set!(a: A, b: B, c: C, d: D, e: E, f: F, g: G, h: H);

#[macro_export]
macro_rules! param_set {
    (@($name:ident, $( $inp:tt )*) -> ($( $body:tt )*)) => {
        param_set!(@($( $inp )*) -> ($( $body )* param!($name),))
    };
    (@($name:ident: $t:ty, $( $inp:tt )*) -> ($( $body:tt )*)) => {
        param_set!(@($( $inp )*) -> ($( $body )* param!($name: $t),))
    };
    (@($name:ident = $def:expr, $( $inp:tt )*) -> ($( $body:tt )*)) => {
        param_set!(@($( $inp )*) -> ($( $body )* param!($name = $def),))
    };
    (@($name:ident: $t:ty = $def:expr, $( $inp:tt )*) -> ($( $body:tt )*)) => {
        param_set!(@($( $inp )*) -> ($( $body )* param!($name: $t = $def)))
    };
    (@($name:ident) -> ($( $body:tt )*)) => {
        param_set!(@() -> ($( $body )* param!($name)))
    };
    (@($name:ident: $t:ty) -> ($( $body:tt )*)) => {
        param_set!(@() -> ($( $body )* param!($name: $t)))
    };
    (@($name:ident = $def:expr) -> ($( $body:tt )*)) => {
        param_set!(@() -> ($( $body )* param!($name = $def)))
    };
    (@($name:ident: $t:ty = $def:expr) -> ($( $body:tt )*)) => {
        param_set!(@() -> ($( $body )* param!($name: $t = $def)))
    };
    (@() -> $body:expr) => {
        $body
    };
    ($( $inp:tt )*) => {
        param_set!(@($( $inp )*) -> ())
    };
}

#[cfg(test)]
mod tests {
    use super::extract_param_set;

    #[test]
    fn param_set_variants() {
        let params = param_set!(a: u8, b: u32, c = true, y: &str = "Hello!");
    }

    #[test]
    fn two_param_set_two_parses() {
        let params = param_set!(i: u32, mem = true);
        assert_eq!(
            extract_param_set(&params, &[("i", "10000")]),
            Some((10000, true))
        );
    }
}
