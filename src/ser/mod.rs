mod string;

use std::str;
use serde::ser;
use error::{BencodeError, Result};

#[derive(Debug)]
pub struct Serializer {
    buf: Vec<u8>,
}

impl Serializer {
    pub fn new() -> Serializer {
        Serializer { buf: Vec::new() }
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    fn push<T: AsRef<[u8]>>(&mut self, token: T) {
        self.buf.extend_from_slice(token.as_ref());
    }
}

impl AsRef<[u8]> for Serializer {
    fn as_ref(&self) -> &[u8] {
        self.buf.as_ref()
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        self.push("e".as_bytes());
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_element<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

pub struct SerializeMap<'a> {
    ser: &'a mut Serializer,
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    cur_key: Option<Vec<u8>>,
}

impl<'a> SerializeMap<'a> {
    pub fn new(ser: &'a mut Serializer) -> SerializeMap {
        SerializeMap {
            ser: ser,
            entries: Vec::new(),
            cur_key: None,
        }
    }
}

impl<'a> ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_key<T: ?Sized + ser::Serialize>(&mut self, key: &T) -> Result<()> {
        if self.cur_key.is_some() {
            return Err(BencodeError::InvalidValue("`serialize_key` called multiple times without calling  `serialize_value`".to_string()));
        }
        self.cur_key = Some(key.serialize(&mut string::StringSerializer)?);
        Ok(())
    }
    fn serialize_value<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        let key = self.cur_key.take().ok_or(BencodeError::InvalidValue("`serialize_value` called without calling `serialize_key`".to_string()))?;
        let mut ser = Serializer::new();
        value.serialize(&mut ser)?;
        self.entries.push((key, ser.into_vec()));
        Ok(())
    }
    fn end(self) -> Result<()> {
        if self.cur_key.is_some() {
            return Err(BencodeError::InvalidValue("`serialize_key` called without calling  `serialize_value`".to_string()));
        }
        let SerializeMap {
            mut ser,
            mut entries,
            ..
        } = self;
        entries.sort_by(|&(ref a, _), &(ref b, _)| a.cmp(b));
        ser.push("d");
        for (k, v) in entries {
            ser::Serializer::serialize_bytes(&mut *ser, k.as_ref())?;
            ser.push(v);
        }
        ser.push("e");
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for SerializeMap<'a> {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   key: &'static str,
                                                   value: &T)
                                                   -> Result<()> {
        ser::SerializeMap::serialize_key(self, key)?;
        ser::SerializeMap::serialize_value(self, value)
    }
    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a> ser::SerializeStructVariant for SerializeMap<'a> {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   key: &'static str,
                                                   value: &T)
                                                   -> Result<()> {
        ser::SerializeStruct::serialize_field(self, key, value)
    }
    fn end(self) -> Result<()> {
        ser::SerializeStruct::end(self)
    }
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeMap<'a>;
    type SerializeStructVariant = SerializeMap<'a>;

    fn serialize_bool(self, value: bool) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i8(self, value: i8) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i16(self, value: i16) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i32(self, value: i32) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i64(self, value: i64) -> Result<()> {
        let mut token = Vec::new();
        token.extend_from_slice(&"i".as_bytes());
        token.extend_from_slice(&value.to_string().as_bytes());
        token.extend_from_slice(&"e".as_bytes());
        self.push(&token);
        Ok(())
    }
    fn serialize_u8(self, value: u8) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u16(self, value: u16) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u32(self, value: u32) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u64(self, value: u64) -> Result<()> {
        self.serialize_i64(value as i64)
    }
    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(BencodeError::InvalidValue("Cannot serialize f32".to_string()))
    }
    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(BencodeError::InvalidValue("Cannot serialize f64".to_string()))
    }
    fn serialize_char(self, value: char) -> Result<()> {
        self.serialize_bytes(&[value as u8])
    }
    fn serialize_str(self, value: &str) -> Result<()> {
        self.serialize_bytes(value.as_bytes())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        let mut token = Vec::new();
        token.extend_from_slice(&value.len().to_string().as_bytes());
        token.extend_from_slice(&":".as_bytes());
        token.extend_from_slice(value);
        self.push(&token);
        Ok(())
    }
    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                              -> Result<()> {
        Err(BencodeError::UnknownVariant("Unit variant not supported.".to_string()))
    }
    fn serialize_newtype_struct<T: ?Sized + ser::Serialize>(self,
                                                            _name: &'static str,
                                                            value: &T)
                                                            -> Result<()> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + ser::Serialize>(self,
                                                             _name: &'static str,
                                                             _variant_index: u32,
                                                             _variant: &'static str,
                                                             value: &T)
                                                             -> Result<()> {
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<()> {
        self.push("".as_bytes());
        Ok(())
    }
    fn serialize_some<T: ?Sized + ser::Serialize>(self, value: &T) -> Result<()> {
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self> {
        self.push("l".as_bytes());
        Ok(self)
    }
    // fn serialize_seq_fixed_size(self, size: usize) -> Result<Self> {
    //     self.serialize_seq(Some(size))
    // }
    fn serialize_tuple(self, size: usize) -> Result<Self> {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self> {
        Err(BencodeError::UnknownVariant("Tuple variant not supported.".to_string()))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap::new(self))
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        Err(BencodeError::UnknownVariant("Struct variant not supported.".to_string()))
    }
}
