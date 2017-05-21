use std::str;
use serde::ser::{Serializer, Serialize, SerializeSeq, SerializeTuple, SerializeTupleStruct,
                 SerializeTupleVariant, SerializeMap, SerializeStruct, SerializeStructVariant};
use error::BencodeError;

type Token = Vec<u8>;
type Context = Vec<Token>;

#[derive(Debug)]
pub struct Encoder {
    stack: Vec<Context>,
}

impl Encoder {
    pub fn new() -> Encoder {
        Encoder { stack: vec![Context::new()] }
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

impl<'a> From<&'a Encoder> for Vec<u8> {
    fn from(encoder: &Encoder) -> Vec<u8> {
        encoder.encoded()
    }
}

impl From<Encoder> for Vec<u8> {
    fn from(encoder: Encoder) -> Vec<u8> {
        encoder.encoded()
    }
}

impl<'a> SerializeSeq for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.push("e".as_bytes());
        self.tokenify_context();
        Ok(())
    }
}

impl<'a> SerializeTuple for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleStruct for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeTupleVariant for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl<'a> SerializeMap for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut **self)
    }
    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_dict();
        Ok(())
    }
}

impl<'a> SerializeStruct for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              key: &'static str,
                                              value: &T)
                                              -> Result<(), Self::Error> {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.end_dict();
        Ok(())
    }
}

impl<'a> SerializeStructVariant for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    fn serialize_field<T: ?Sized + Serialize>(&mut self,
                                              key: &'static str,
                                              value: &T)
                                              -> Result<(), Self::Error> {
        SerializeStruct::serialize_field(self, key, value)
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeStruct::end(self)
    }
}

impl<'a> Serializer for &'a mut Encoder {
    type Ok = ();
    type Error = BencodeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i8(self, value: i8) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i16(self, value: i16) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i32(self, value: i32) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i64(self, value: i64) -> Result<(), Self::Error> {
        let mut token = Vec::new();
        token.extend_from_slice(&"i".as_bytes());
        token.extend_from_slice(&value.to_string().as_bytes());
        token.extend_from_slice(&"e".as_bytes());
        self.push(&token);
        Ok(())
    }
    fn serialize_u8(self, value: u8) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u16(self, value: u16) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u32(self, value: u32) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u64(self, value: u64) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_f32(self, _value: f32) -> Result<(), Self::Error> {
        Err(BencodeError::InvalidValue("Cannot serialize f32".to_string()))
    }
    fn serialize_f64(self, _value: f64) -> Result<(), Self::Error> {
        Err(BencodeError::InvalidValue("Cannot serialize f64".to_string()))
    }
    fn serialize_char(self, value: char) -> Result<(), Self::Error> {
        self.serialize_bytes(&[value as u8])
    }
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        self.serialize_bytes(value.as_bytes())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<(), Self::Error> {
        let mut token = Vec::new();
        token.extend_from_slice(&value.len().to_string().as_bytes());
        token.extend_from_slice(&":".as_bytes());
        token.extend_from_slice(value);
        self.push(&token);
        Ok(())
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(self,
                              _name: &'static str,
                              _variant_index: u32,
                              _variant: &'static str)
                              -> Result<(), Self::Error> {
        Err(BencodeError::UnknownVariant("Unit variant not supported.".to_string()))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(self,
                                                       _name: &'static str,
                                                       value: &T)
                                                       -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(self,
                                                        _name: &'static str,
                                                        _variant_index: u32,
                                                        _variant: &'static str,
                                                        value: &T)
                                                        -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.push("".as_bytes());
        Ok(())
    }
    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self, Self::Error> {
        self.new_context();
        self.push("l".as_bytes());
        Ok(self)
    }
    // fn serialize_seq_fixed_size(self, size: usize) -> Result<Self, Self::Error> {
    //     self.serialize_seq(Some(size))
    // }
    fn serialize_tuple(self, size: usize) -> Result<Self, Self::Error> {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self, Self::Error> {
        self.serialize_seq(Some(len))
    }
    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               _variant_index: u32,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self, Self::Error> {
        Err(BencodeError::UnknownVariant("Tuple variant not supported.".to_string()))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self, Self::Error> {
        self.start_dict();
        Ok(self)
    }
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self, Self::Error> {
        self.serialize_map(Some(len))
    }
    fn serialize_struct_variant(self,
                                _name: &'static str,
                                _variant_index: u32,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self, Self::Error> {
        Err(BencodeError::UnknownVariant("Struct variant not supported.".to_string()))
    }
}
