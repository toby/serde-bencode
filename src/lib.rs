#[macro_use]
extern crate serde;
extern crate serde_bytes;
extern crate smallvec;

pub mod error;
pub mod ser;
pub mod de;
pub mod value;

pub use error::{Error, Result};
pub use ser::{to_bytes, to_string, Serializer};
pub use de::{from_str, from_bytes, Deserializer};
pub use value::Value;
