use std::fmt;
use std::error::Error as StdError;
use std::io::Error as IoError;
use serde::ser::Error as SerError;
use serde::de::Error as DeError;
use serde::de::Type;

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
    fn custom<T: Into<String>>(msg: T) -> Self {
        BencodeError::Custom(msg.into())
    }

    fn invalid_value(msg: &str) -> Self {
        BencodeError::InvalidValue(msg.into())
    }
}

impl DeError for BencodeError {
    fn custom<T: Into<String>>(msg: T) -> Self {
        BencodeError::Custom(msg.into())
    }

    fn end_of_stream() -> Self {
        BencodeError::EndOfStream
    }

    fn invalid_type(ty: Type) -> Self {
        BencodeError::InvalidType(format!("Invalid Type: {:?}", ty))
    }

    fn invalid_value(msg: &str) -> Self {
        BencodeError::InvalidValue(format!("Invalid Value: {}", msg))
    }

    fn invalid_length(len: usize) -> Self {
        BencodeError::InvalidLength(format!("Invalid Length: {}", len))
    }

    fn unknown_variant(field: &str) -> Self {
        BencodeError::UnknownVariant(format!("Unknown Variant: {}", field))
    }

    fn unknown_field(field: &str) -> Self {
        BencodeError::UnknownField(format!("Unknown Field: {}", field))
    }

    fn missing_field(field: &'static str) -> Self {
        BencodeError::MissingField(format!("Missing Field: {}", field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        BencodeError::DuplicateField(format!("Duplicat Field: {}", field))
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
