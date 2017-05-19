use std::fmt;
use std::fmt::Display;
use std::error::Error as StdError;
use std::io::Error as IoError;
use serde::ser::Error as SerError;
use serde::de::Error as DeError;
use serde::de::{Unexpected, Expected};

#[derive(Debug)]
pub enum BencodeError {
    IoError(IoError),
    InvalidType(String),
    InvalidValue(String),
    InvalidLength(String),
    UnknownVariant(String),
    UnknownField(String),
    MissingField(String),
    DuplicateField(String),
    Custom(String),
    EndOfStream
}

impl SerError for BencodeError {
    fn custom<T: Display>(msg: T) -> Self {
        BencodeError::Custom(msg.to_string())
    }
}

impl DeError for BencodeError {
    fn custom<T: Display>(msg: T) -> Self {
        BencodeError::Custom(msg.to_string())
    }

    fn invalid_type(unexp: Unexpected, exp: &Expected) -> Self {
        BencodeError::InvalidType(format!("Invalid Type: {} (expected: `{}`)", unexp, exp))
    }

    fn invalid_value(unexp: Unexpected, exp: &Expected) -> Self {
        BencodeError::InvalidValue(format!("Invalid Value: {} (expected: `{}`)", unexp, exp))
    }

    fn invalid_length(len: usize, exp: &Expected) -> Self {
        BencodeError::InvalidLength(format!("Invalid Length: {} (expected: {})", len, exp))
    }

    fn unknown_variant(field: &str, expected: &'static [&'static str]) -> Self {
        BencodeError::UnknownVariant(format!("Unknown Variant: `{}` (expected one of: {:?})", field, expected))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        BencodeError::UnknownField(format!("Unknown Field: `{}` (expected one of: {:?})", field, expected))
    }

    fn missing_field(field: &'static str) -> Self {
        BencodeError::MissingField(format!("Missing Field: `{}`", field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        BencodeError::DuplicateField(format!("Duplicat Field: `{}`", field))
    }
}

impl StdError for BencodeError {
    fn description(&self) -> &str {
        match *self {
            BencodeError::IoError(ref error) => StdError::description(error),
            BencodeError::InvalidType(ref s) => s,
            BencodeError::InvalidValue(ref s) => s,
            BencodeError::InvalidLength(ref s) => s,
            BencodeError::UnknownVariant(ref s) => s,
            BencodeError::UnknownField(ref s) => s,
            BencodeError::MissingField(ref s) => s,
            BencodeError::DuplicateField(ref s) => s,
            BencodeError::Custom(ref s) => s,
            BencodeError::EndOfStream => "End of stream",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            BencodeError::IoError(ref error) => Some(error),
            _ => None,
        }
    }
}

impl fmt::Display for BencodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}
