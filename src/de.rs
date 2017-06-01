use std::io::Read;
use std::str;
use serde::de;
use error::{Error, Result};

pub struct BencodeAccess<'a, R: 'a + Read> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a + Read> BencodeAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> BencodeAccess<'a, R> {
        BencodeAccess { de: de }
    }
}

impl<'de, 'a, R: 'a + Read> de::SeqAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(&mut self,
                                                      seed: T)
                                                      -> Result<Option<T::Value>> {
        match self.de.parse()? {
            ParseResult::End => Ok(None),
            r @ _ => {
                self.de.next = Some(r);
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
        }
    }
}

impl<'de, 'a, R: 'a + Read> de::MapAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: de::DeserializeSeed<'de>
    {
        match self.de.parse()? {
            ParseResult::End => Ok(None),
            r @ _ => {
                self.de.next = Some(r);
                Ok(Some(seed.deserialize(&mut *self.de)?))
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: de::DeserializeSeed<'de>
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
        self.de.expect(&[ParseToken::End])?;
        Ok(res)
    }

    fn tuple_variant<V: de::Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value> {
        let res = match self.de.expect(&[ParseToken::List])? {
            ParseResult::List => {
                let res = visitor.visit_seq(BencodeAccess::new(&mut *self.de))?;
                self.de.expect(&[ParseToken::End])?;
                res
            }
            _ => unreachable!(),
        };
        self.de.expect(&[ParseToken::End])?;
        Ok(res)
    }

    fn struct_variant<V: de::Visitor<'de>>(self,
                                           _: &'static [&'static str],
                                           visitor: V)
                                           -> Result<V::Value> {
        let res = de::Deserializer::deserialize_any(&mut *self.de, visitor)?;
        self.de.expect(&[ParseToken::End])?;
        Ok(res)
    }
}

impl<'de, 'a, R: 'a + Read> de::EnumAccess<'de> for BencodeAccess<'a, R> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: de::DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        match self.de.expect(&[ParseToken::Bytes, ParseToken::Map])? {
            t @ ParseResult::Bytes(_) => {
                self.de.next = Some(t);
                Ok((seed.deserialize(&mut *self.de)?, self))
            }
            ParseResult::Map => Ok((seed.deserialize(&mut *self.de)?, self)),
            _ => unreachable!(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ParseToken {
    Int,
    Bytes,
    /// list start
    List,
    /// map start
    Map,
    /// list or map end
    End,
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

#[derive(Debug)]
pub struct Deserializer<R: Read> {
    reader: R,
    next: Option<ParseResult>,
}

impl<'de, R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Deserializer<R> {
        Deserializer {
            reader: reader,
            next: None,
        }
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
                    let len_str =
                        String::from_utf8(result)
                            .map_err(|_| {
                                         Error::InvalidValue("Non UTF-8 integer encoding"
                                                                 .to_string())
                                     })?;
                    let len_int = len_str
                        .parse()
                        .map_err(|_| {
                                     Error::InvalidValue(format!("Can't parse `{}` as integer",
                                                                 len_str))
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
                    let len_str =
                        String::from_utf8(len)
                            .map_err(|_| {
                                         Error::InvalidValue("Non UTF-8 integer encoding"
                                                                 .to_string())
                                     })?;
                    let len_int = len_str.parse()
                        .map_err(|_| Error::InvalidValue(format!("Can't parse `{}` as string length", len_str)))?;
                    return Ok(len_int);
                }
                n => len.push(n),
            }
        }
    }

    fn parse_bytes(&mut self, len_char: u8) -> Result<Vec<u8>> {
        let len = self.parse_bytes_len(len_char)?;
        let mut buf = vec![0u8; len];
        let actual_len = self.reader
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
            n @ b'0'...b'9' => Ok(ParseResult::Bytes(self.parse_bytes(n)?)),
            b'l' => Ok(ParseResult::List),
            b'd' => Ok(ParseResult::Map),
            b'e' => Ok(ParseResult::End),
            c @ _ => Err(Error::InvalidValue(format!("Invalid character `{}`", c as char))),
        }
    }

    fn expect(&mut self, expected: &[ParseToken]) -> Result<ParseResult> {
        let cur = self.parse()?;
        for e in expected {
            match (*e, &cur) {
                (ParseToken::Int, &ParseResult::Int(_)) |
                (ParseToken::Bytes, &ParseResult::Bytes(_)) |
                (ParseToken::List, &ParseResult::List) |
                (ParseToken::Map, &ParseResult::Map) |
                (ParseToken::End, &ParseResult::End) => return Ok(cur),
                _ => (),
            }
        }
        Err(Error::InvalidValue(format!("Expected one of {:?}; got `{:?}`", expected, cur)))
    }
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V: de::Visitor<'de>>(mut self, visitor: V) -> Result<V::Value> {
        match self.parse()? {
            ParseResult::Int(i) => visitor.visit_i64(i),
            ParseResult::Bytes(s) => visitor.visit_bytes(s.as_ref()),
            ParseResult::List => visitor.visit_seq(BencodeAccess::new(&mut self)),
            ParseResult::Map => visitor.visit_map(BencodeAccess::new(&mut self)),
            ParseResult::End => Err(Error::EndOfStream),
        }
    }

    forward_to_deserialize_any! {
        i64 string seq bool i8 i16 i32 u8 u16 u32
        u64 f32 f64 char str unit bytes byte_buf map unit_struct tuple_struct tuple
        newtype_struct ignored_any identifier struct
    }

    #[inline]
    fn deserialize_option<V: de::Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_enum<V>(self,
                           _name: &str,
                           _variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value>
        where V: de::Visitor<'de>
    {
        visitor.visit_enum(BencodeAccess::new(self))
    }
}

pub fn from_str<'de, T>(s: &'de str) -> Result<T>
    where T: de::Deserialize<'de>
{
    from_bytes(s.as_bytes())
}

pub fn from_bytes<'de, T>(b: &'de [u8]) -> Result<T>
    where T: de::Deserialize<'de>
{
    de::Deserialize::deserialize(&mut Deserializer::new(b))
}
