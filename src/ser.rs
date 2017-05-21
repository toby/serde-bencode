use std::str;
use serde::ser::{self, Serialize};
use error::{BencodeError, Result};

type Token = Vec<u8>;
type Context = Vec<Token>;

#[derive(Debug)]
pub struct Serializer {
    stack: Vec<Context>,
}

impl Serializer {
    pub fn new() -> Serializer {
        Serializer { stack: vec![Context::new()] }
    }

    pub fn encoded(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        for c in self.stack.iter() {
            for t in c.iter() {
                result.extend_from_slice(&t)
            }
        }
        result
    }

    fn push(&mut self, token: &[u8]) {
        if let Some(last) = self.stack.last_mut() {
            (*last).push(token.into());
        }
    }

    fn new_context(&mut self) {
        self.stack.push(Context::new());
    }

    fn merge_context(&mut self) {
        if let Some(current) = self.stack.pop() {
            if let Some(last) = self.stack.last_mut() {
                (*last).extend(current);
            }
        }
    }

    fn tokenify_context(&mut self) {
        if let Some(c) = self.stack.pop() {
            let token: Token = c.into_iter().flat_map(|t| t).collect();
            self.push(&token);
        }
    }

    fn sort_current_context(&mut self) {
        if let Some(last) = self.stack.pop() {
            let mut m: Vec<_> = last.chunks(2).collect();
            m.sort();
            self.new_context();
            for c in m.iter() {
                if c[1].len() > 2 {
                    self.push(&c[0]);
                    self.push(&c[1]);
                }
            }
        }
    }

    fn start_dict(&mut self) {
        self.new_context();
        self.push(&"d".as_bytes());
        self.new_context();
    }

    fn end_dict(&mut self) {
        self.sort_current_context();
        self.merge_context();
        self.push("e".as_bytes());
        self.tokenify_context();
    }
}

impl<'a> From<&'a Serializer> for Vec<u8> {
    fn from(encoder: &Serializer) -> Vec<u8> {
        encoder.encoded()
    }
}

impl From<Serializer> for Vec<u8> {
    fn from(encoder: Serializer) -> Vec<u8> {
        encoder.encoded()
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
        self.tokenify_context();
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

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_key<T: ?Sized + ser::Serialize>(&mut self, key: &T) -> Result<()> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: ?Sized + ser::Serialize>(&mut self, value: &T) -> Result<()> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        self.end_dict();
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + ser::Serialize>(&mut self,
                                                   key: &'static str,
                                                   value: &T)
                                                   -> Result<()> {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<()> {
        self.end_dict();
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
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
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

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
        self.new_context();
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
    fn serialize_map(self, _len: Option<usize>) -> Result<Self> {
        self.start_dict();
        Ok(self)
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self> {
        Err(BencodeError::UnknownVariant("Struct variant not supported.".to_string()))
    }
}
