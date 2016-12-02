use std::str;
use serde::ser::{Serializer, Serialize, Error};
use error::BencodeError;

type Token = Vec<u8>;
type Context = Vec<Token>;

#[derive(Debug)]
pub struct Encoder {
    stack: Vec<Context>
}

impl Encoder {
    pub fn new() -> Encoder {
        let mut stack = Vec::new();
        stack.push(Context::new());
        Encoder {
            stack: stack
        }
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
                self.push(&c[0]);
                self.push(&c[1]);
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

impl Serializer for Encoder {

    type Error = BencodeError;
    type SeqState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_bool(&mut self, value: bool) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_isize(&mut self, value: isize) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i8(&mut self, value: i8) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i16(&mut self, value: i16) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i32(&mut self, value: i32) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_i64(&mut self, value: i64) -> Result<(), Self::Error> {
        let mut token = Vec::new();
        token.extend_from_slice(&"i".as_bytes());
        token.extend_from_slice(&value.to_string().as_bytes());
        token.extend_from_slice(&"e".as_bytes());
        self.push(&token);
        Ok(())
    }
    fn serialize_usize(&mut self, value: usize) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u8(&mut self, value: u8) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u16(&mut self, value: u16) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u32(&mut self, value: u32) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_u64(&mut self, value: u64) -> Result<(), Self::Error> {
        self.serialize_i64(value as i64)
    }
    fn serialize_f32(&mut self, _value: f32) -> Result<(), Self::Error> {
        Err(BencodeError::invalid_value("Cannot serialize f32"))
    }
    fn serialize_f64(&mut self, _value: f64) -> Result<(), Self::Error> {
        Err(BencodeError::invalid_value("Cannot serialize f64"))
    }
    fn serialize_char(&mut self, value: char) -> Result<(), Self::Error> {
        self.serialize_bytes(&[value as u8])
    }
    fn serialize_str(&mut self, value: &str) -> Result<(), Self::Error> {
        self.serialize_bytes(value.as_bytes())
    }
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Self::Error> {
        let mut token = Vec::new();
        token.extend_from_slice(&value.len().to_string().as_bytes());
        token.extend_from_slice(&":".as_bytes());
        token.extend_from_slice(value);
        self.push(&token);
        Ok(())
    }
    fn serialize_unit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_unit_variant(&mut self, _name: &'static str, _variant_index: usize, _variant: &'static str) -> Result<(), Self::Error> {
        self.serialize_unit()
    }
    fn serialize_newtype_struct<T: Serialize>(&mut self, _name: &'static str, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize>(&mut self, _name: &'static str, _variant_index: usize, _variant: &'static str, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_none(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn serialize_some<T: Serialize>(&mut self, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_seq(&mut self, _len: Option<usize>) -> Result<Self::SeqState, Self::Error> {
        self.new_context();
        self.push("l".as_bytes());
        Ok(())
    }
    fn serialize_seq_elt<T: Serialize>(&mut self, _state: &mut Self::SeqState, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_seq_end(&mut self, _state: Self::SeqState) -> Result<(), Self::Error> {
        self.push("e".as_bytes());
        self.tokenify_context();
        Ok(())
    }
    fn serialize_seq_fixed_size(&mut self, size: usize) -> Result<Self::SeqState, Self::Error> {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple(&mut self, size: usize) -> Result<Self::TupleState, Self::Error> {
        self.serialize_seq(Some(size))
    }
    fn serialize_tuple_elt<T: Serialize>(&mut self, _state: &mut Self::TupleState, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_tuple_end(&mut self, _state: Self::TupleState) -> Result<(), Self::Error> {
        self.serialize_seq_end(())
    }
    fn serialize_tuple_struct(&mut self, _name: &'static str, len: usize) -> Result<Self::TupleStructState, Self::Error> {
        self.serialize_tuple(len)
    }
    fn serialize_tuple_struct_elt<T: Serialize>(&mut self, _state: &mut Self::TupleStructState, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_tuple_struct_end(&mut self, _state: Self::TupleStructState) -> Result<(), Self::Error> {
        self.serialize_tuple_end(())
    }
    fn serialize_tuple_variant(&mut self, name: &'static str, _variant_index: usize, _variant: &'static str, len: usize) -> Result<Self::TupleVariantState, Self::Error> {
        self.serialize_tuple_struct(name, len)
    }
    fn serialize_tuple_variant_elt<T: Serialize>(&mut self, _state: &mut Self::TupleVariantState, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_tuple_variant_end(&mut self, _state: Self::TupleVariantState) -> Result<(), Self::Error> {
        self.serialize_tuple_struct_end(())
    }
    fn serialize_map(&mut self, _len: Option<usize>) -> Result<Self::MapState, Self::Error> {
        self.start_dict();
        Ok(())
    }
    fn serialize_map_key<T: Serialize>(&mut self, _state: &mut Self::MapState, key: T) -> Result<(), Self::Error> {
        key.serialize(self)
    }
    fn serialize_map_value<T: Serialize>(&mut self, _state: &mut Self::MapState, value: T) -> Result<(), Self::Error> {
        value.serialize(self)
    }
    fn serialize_map_end(&mut self, _state: Self::MapState) -> Result<(), Self::Error> {
        self.end_dict();
        Ok(())
    }
    fn serialize_struct(&mut self, _name: &'static str, _len: usize) -> Result<Self::StructState, Self::Error> {
        self.start_dict();
        Ok(())
    }
    fn serialize_struct_elt<V: Serialize>(&mut self, _state: &mut Self::StructState, key: &'static str, value: V) -> Result<(), Self::Error> {
        key.serialize(self).unwrap();
        value.serialize(self).unwrap();
        Ok(())
    }
    fn serialize_struct_end(&mut self, _state: Self::StructState) -> Result<(), Self::Error> {
        self.end_dict();
        Ok(())
    }
    fn serialize_struct_variant(&mut self, name: &'static str, _variant_index: usize, _variant: &'static str, len: usize) -> Result<Self::StructVariantState, Self::Error> {
        self.serialize_struct(name, len)
    }
    fn serialize_struct_variant_elt<V: Serialize>(&mut self, _state: &mut Self::StructVariantState, key: &'static str, value: V) -> Result<(), Self::Error> {
        key.serialize(self).unwrap();
        value.serialize(self).unwrap();
        Ok(())
    }
    fn serialize_struct_variant_end(&mut self, _state: Self::StructVariantState) -> Result<(), Self::Error> {
        self.serialize_struct_end(())
    }
}
