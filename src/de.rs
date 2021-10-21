//! Deserialize bencode data to a Rust data structure

use crate::error::{Error, Result};
use serde::{
    de::{self, Error as _, Unexpected},
    forward_to_deserialize_any,
};
use std::io::Read;
use std::str;

#[doc(hidden)]
// TODO: This should be pub(crate).
pub struct BencodeAccess<'a, R: 'a + Read> {
    de: &'a mut Deserializer<R>,
    len: Option<usize>,
}

impl<'a, R: 'a + Read> BencodeAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: Option<usize>) -> BencodeAccess<'a, R> {
        BencodeAccess { de, len }
    }
}

impl<'de, 'a, R: 'a + Read> de::SeqAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>> {
        let res = match self.de.parse()? {
            ParseResult::End => Ok(None),
            r => {
                self.de.next = Some(r);
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
        };
        if let Some(l) = self.len {
            let l = l - 1;
            self.len = Some(l);
            if l == 0 && ParseResult::End != self.de.parse()? {
                return Err(Error::InvalidType("expected `e`".to_string()));
            }
        }
        res
    }
}

impl<'de, 'a, R: 'a + Read> de::MapAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.de.parse()? {
            ParseResult::End => Ok(None),
            r => {
                self.de.next = Some(r);
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

impl<'de, 'a, R: 'a + Read> de::VariantAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T: de::DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        let res = seed.deserialize(&mut *self.de)?;
        if ParseResult::End != self.de.parse()? {
            return Err(Error::InvalidType("expected `e`".to_string()));
        }
        Ok(res)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value> {
        let res = match self.de.parse()? {
            ParseResult::List => visitor.visit_seq(BencodeAccess::new(&mut *self.de, Some(len)))?,
            _ => return Err(Error::InvalidType("expected list".to_string())),
        };
        if ParseResult::End != self.de.parse()? {
            return Err(Error::InvalidType("expected `e`".to_string()));
        }
        Ok(res)
    }

    fn struct_variant<V: de::Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        let res = de::Deserializer::deserialize_any(&mut *self.de, visitor)?;
        if ParseResult::End != self.de.parse()? {
            return Err(Error::InvalidType("expected `e`".to_string()));
        }
        Ok(res)
    }
}

impl<'de, 'a, R: 'a + Read> de::EnumAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        match self.de.parse()? {
            t @ ParseResult::Bytes(_) => {
                self.de.next = Some(t);
                Ok((seed.deserialize(&mut *self.de)?, self))
            }
            ParseResult::Map => Ok((seed.deserialize(&mut *self.de)?, self)),
            t => Err(Error::InvalidValue(format!(
                "Expected bytes or map; got `{:?}`",
                t
            ))),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ParseResult {
    Int(i64),
    Bytes(Vec<u8>),
    /// list start
    List,
    /// map start
    Map,
    /// list or map end
    End,
}

/// A structure for deserializing bencode into Rust values.
#[derive(Debug)]
pub struct Deserializer<R: Read> {
    reader: R,
    next: Option<ParseResult>,
}

impl<'de, R: Read> Deserializer<R> {
    /// Create a new deserializer.
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer { reader, next: None }
    }

    fn parse_int(&mut self) -> Result<i64> {
        let mut buf = [0; 1];
        let mut result = Vec::new();
        loop {
            if 1 != self.reader.read(&mut buf).map_err(Error::IoError)? {
                return Err(Error::EndOfStream);
            }
            match buf[0] {
                b'e' => {
                    let len_str = String::from_utf8(result).map_err(|_| {
                        Error::InvalidValue("Non UTF-8 integer encoding".to_string())
                    })?;
                    let len_int = len_str.parse().map_err(|_| {
                        Error::InvalidValue(format!("Can't parse `{}` as integer", len_str))
                    })?;
                    return Ok(len_int);
                }
                n => result.push(n),
            }
        }
    }

    fn parse_bytes_len(&mut self, len_char: u8) -> Result<usize> {
        let mut buf = [0; 1];
        let mut len = Vec::new();
        len.push(len_char);
        loop {
            if 1 != self.reader.read(&mut buf).map_err(Error::IoError)? {
                return Err(Error::EndOfStream);
            }
            match buf[0] {
                b':' => {
                    let len_str = String::from_utf8(len).map_err(|_| {
                        Error::InvalidValue("Non UTF-8 integer encoding".to_string())
                    })?;
                    let len_int = len_str.parse().map_err(|_| {
                        Error::InvalidValue(format!("Can't parse `{}` as string length", len_str))
                    })?;
                    return Ok(len_int);
                }
                n => len.push(n),
            }
        }
    }

    fn parse_bytes(&mut self, len_char: u8) -> Result<Vec<u8>> {
        let len = self.parse_bytes_len(len_char)?;
        let mut buf = vec![0u8; len];
        let actual_len = self
            .reader
            .read(buf.as_mut_slice())
            .map_err(Error::IoError)?;
        if len != actual_len {
            return Err(Error::EndOfStream);
        }
        Ok(buf)
    }

    fn parse(&mut self) -> Result<ParseResult> {
        if let Some(t) = self.next.take() {
            return Ok(t);
        }
        let mut buf = [0; 1];
        if 1 != self.reader.read(&mut buf).map_err(Error::IoError)? {
            return Err(Error::EndOfStream);
        }
        match buf[0] {
            b'i' => Ok(ParseResult::Int(self.parse_int()?)),
            n @ b'0'..=b'9' => Ok(ParseResult::Bytes(self.parse_bytes(n)?)),
            b'l' => Ok(ParseResult::List),
            b'd' => Ok(ParseResult::Map),
            b'e' => Ok(ParseResult::End),
            c => Err(Error::InvalidValue(format!(
                "Invalid character `{}`",
                c as char
            ))),
        }
    }

    fn parse_only_bytes(&mut self) -> Result<Vec<u8>> {
        match self.parse()? {
            ParseResult::Bytes(bytes) => Ok(bytes),
            ParseResult::Int(i) => Err(Error::invalid_type(Unexpected::Signed(i), &"Bytes")),
            ParseResult::List => Err(Error::invalid_type(Unexpected::Seq, &"Bytes")),
            ParseResult::Map => Err(Error::invalid_type(Unexpected::Map, &"Bytes")),
            ParseResult::End => Err(Error::EndOfStream),
        }
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        match self.parse()? {
            ParseResult::Int(i) => visitor.visit_i64(i),
            ParseResult::Bytes(s) => visitor.visit_bytes(s.as_ref()),
            ParseResult::List => visitor.visit_seq(BencodeAccess::new(&mut self, None)),
            ParseResult::Map => visitor.visit_map(BencodeAccess::new(&mut self, None)),
            ParseResult::End => Err(Error::EndOfStream),
        }
    }

    forward_to_deserialize_any! {
        i64 seq bool i8 i16 i32 u8 u16 u32
        u64 f32 f64 char unit bytes byte_buf map unit_struct tuple_struct tuple
        ignored_any identifier struct
    }

    #[inline]
    fn deserialize_newtype_struct<V: de::Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(BencodeAccess::new(self, None))
    }

    // Do not delegate this to `deserialize_any` because we want to call `visit_str` instead of
    // `visit_bytes` on the visitor, to correctly support adjacently tagged enums (the tag is
    // parsed as str, not bytes).
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let bytes = self.parse_only_bytes()?;
        let s = str::from_utf8(&bytes)
            .map_err(|_| Error::invalid_value(Unexpected::Bytes(&bytes), &"utf-8 string"))?;
        visitor.visit_str(s)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let bytes = self.parse_only_bytes()?;
        let s = String::from_utf8(bytes).map_err(|error| {
            Error::invalid_value(Unexpected::Bytes(error.as_bytes()), &"utf-8 string")
        })?;
        visitor.visit_string(s)
    }
}

/// Deserialize an instance of type `T` from a string of bencode.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), serde_bencode::Error> {
/// use serde_derive::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
/// struct Address {
///     street: String,
///     city: String,
/// }
///
/// let encoded = "d4:city18:Duckburg, Calisota6:street17:1313 Webfoot Walke".to_string();
/// let decoded: Address = serde_bencode::from_str(&encoded)?;
///
/// assert_eq!(
///     decoded,
///     Address {
///         street: "1313 Webfoot Walk".to_string(),
///         city: "Duckburg, Calisota".to_string(),
///     }
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if the input bencode is improperly formatted or if the structure of
/// the input does not match the structure expected by `T`. It can also fail if `T`'s
/// implementation of `Deserialize` decides to fail.
pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    from_bytes(s.as_bytes())
}

/// Deserialize an instance of type `T` from a bencode byte vector.
///
/// # Examples
/// ```
/// # fn main() -> Result<(), serde_bencode::Error> {
/// use serde_derive::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
/// struct Address {
///     street: String,
///     city: String,
/// }
///
/// let encoded = "d4:city18:Duckburg, Calisota6:street17:1313 Webfoot Walke".as_bytes();
/// let decoded: Address = serde_bencode::from_bytes(&encoded)?;
///
/// assert_eq!(
///     decoded,
///     Address {
///         street: "1313 Webfoot Walk".to_string(),
///         city: "Duckburg, Calisota".to_string(),
///     }
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if the input bencode is improperly formatted or if the structure of
/// the input does not match the structure expected by `T`. It can also fail if `T`'s
/// implementation of `Deserialize` decides to fail.
pub fn from_bytes<'de, T>(b: &'de [u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    de::Deserialize::deserialize(&mut Deserializer::new(b))
}
