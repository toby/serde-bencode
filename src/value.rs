//! Structures for representing bencoded values with Rust data types.

use serde::de;
use serde::ser::{self, SerializeMap, SerializeSeq};
use serde_bytes::{ByteBuf, Bytes};
use std::collections::HashMap;
use std::fmt;

/// All possible values which may be serialized in bencode.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Value {
    /// A generic list of bytes.
    Bytes(Vec<u8>),

    /// An integer.
    Int(i64),

    /// A list of other bencoded values.
    List(Vec<Value>),

    /// A map of (key, value) pairs.
    Dict(HashMap<Vec<u8>, Value>),
}

impl ser::Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match *self {
            Value::Bytes(ref v) => s.serialize_bytes(v),
            Value::Int(v) => s.serialize_i64(v),
            Value::List(ref v) => {
                let mut seq = s.serialize_seq(Some(v.len()))?;
                for e in v {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
            Value::Dict(ref vs) => {
                let mut map = s.serialize_map(Some(vs.len()))?;
                for (k, v) in vs {
                    map.serialize_entry(&Bytes::new(k), v)?;
                }
                map.end()
            }
        }
    }
}

struct ValueVisitor;

impl<'de> de::Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid BEncode value")
    }

    #[inline]
    fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
        Ok(Value::Int(value))
    }

    #[inline]
    fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
        Ok(Value::Int(value as i64))
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Value, E>
    where
        E: de::Error,
    {
        Ok(Value::Bytes(value.into()))
    }

    #[inline]
    fn visit_string<E>(self, value: String) -> Result<Value, E> {
        Ok(Value::Bytes(value.into()))
    }

    #[inline]
    fn visit_bytes<E>(self, value: &[u8]) -> Result<Value, E> {
        Ok(Value::Bytes(value.into()))
    }

    #[inline]
    fn visit_seq<V>(self, mut access: V) -> Result<Value, V::Error>
    where
        V: de::SeqAccess<'de>,
    {
        let mut seq = Vec::new();
        while let Some(e) = access.next_element()? {
            seq.push(e);
        }
        Ok(Value::List(seq))
    }

    #[inline]
    fn visit_map<V>(self, mut access: V) -> Result<Value, V::Error>
    where
        V: de::MapAccess<'de>,
    {
        let mut map = HashMap::new();
        while let Some((k, v)) = access.next_entry::<ByteBuf, _>()? {
            map.insert(k.into_vec(), v);
        }
        Ok(Value::Dict(map))
    }
}

impl<'de> de::Deserialize<'de> for Value {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Value {
        Value::Int(v)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Bytes(s.into_bytes())
    }
}

impl<'a> From<&'a str> for Value {
    fn from(v: &str) -> Value {
        Value::Bytes(v.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Value {
    fn from(v: Vec<u8>) -> Value {
        Value::Bytes(v)
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Value {
        Value::List(v)
    }
}

impl From<HashMap<Vec<u8>, Value>> for Value {
    fn from(v: HashMap<Vec<u8>, Value>) -> Value {
        Value::Dict(v)
    }
}
