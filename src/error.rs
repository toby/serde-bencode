use std::fmt;
use std::fmt::Display;
use std::error::Error as StdError;
use std::io::Error as IoError;
use std::result::Result as StdResult;
use serde::ser::Error as SerError;
use serde::de::Error as DeError;
use serde::de::{Unexpected, Expected};

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    InvalidType(String),
    InvalidValue(String),
    InvalidLength(String),
    UnknownVariant(String),
    UnknownField(String),
    MissingField(String),
    DuplicateField(String),
    Custom(String),
    EndOfStream,
}

impl SerError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl DeError for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }

    fn invalid_type(unexp: Unexpected, exp: &Expected) -> Self {
        Error::InvalidType(format!("Invalid Type: {} (expected: `{}`)", unexp, exp))
    }

    fn invalid_value(unexp: Unexpected, exp: &Expected) -> Self {
        Error::InvalidValue(format!("Invalid Value: {} (expected: `{}`)", unexp, exp))
    }

    fn invalid_length(len: usize, exp: &Expected) -> Self {
        Error::InvalidLength(format!("Invalid Length: {} (expected: {})", len, exp))
    }

    fn unknown_variant(field: &str, expected: &'static [&'static str]) -> Self {
        Error::UnknownVariant(format!("Unknown Variant: `{}` (expected one of: {:?})",
                                             field,
                                             expected))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Error::UnknownField(format!("Unknown Field: `{}` (expected one of: {:?})",
                                           field,
                                           expected))
    }

    fn missing_field(field: &'static str) -> Self {
        Error::MissingField(format!("Missing Field: `{}`", field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        Error::DuplicateField(format!("Duplicat Field: `{}`", field))
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::IoError(ref error) => StdError::description(error),
            Error::InvalidType(ref s) => s,
            Error::InvalidValue(ref s) => s,
            Error::InvalidLength(ref s) => s,
            Error::UnknownVariant(ref s) => s,
            Error::UnknownField(ref s) => s,
            Error::MissingField(ref s) => s,
            Error::DuplicateField(ref s) => s,
            Error::Custom(ref s) => s,
            Error::EndOfStream => "End of stream",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::IoError(ref error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
