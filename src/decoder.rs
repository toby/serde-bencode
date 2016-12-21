use std::io::Read;
use std::result;
use std::str;
use serde::de::{Deserializer, Deserialize, Visitor, VariantVisitor, SeqVisitor, MapVisitor,
                EnumVisitor, Error};
use error::BencodeError;

pub type Result<T> = result::Result<T, BencodeError>;

pub struct BencodeVisitor<'a, R: 'a + Read> {
    de: &'a mut BencodeDecoder<R>,
}

impl <'a, R: 'a + Read> BencodeVisitor<'a, R> {
    fn new(de: &'a mut BencodeDecoder<R>) -> BencodeVisitor<'a, R> {
        BencodeVisitor { de: de }
    }
}

impl<'a, R: 'a + Read> VariantVisitor for BencodeVisitor<'a, R> {
    type Error = BencodeError;

    fn visit_variant<T: Deserialize>(&mut self) -> Result<T> {
        T::deserialize(self.de)
    }

    fn visit_newtype<T: Deserialize>(&mut self) -> Result<T> {
        T::deserialize(self.de)
    }

    fn visit_unit(&mut self) -> Result<()> {
        Err(BencodeError::UnknownVariant("Unit variant not supported.".into()))
    }

    fn visit_tuple<V: Visitor>(&mut self, _: usize, _: V) -> Result<V::Value> {
        Err(BencodeError::UnknownVariant("Tuple variant not supported.".into()))
    }

    fn visit_struct<V: Visitor>(&mut self, _: &'static [&'static str], _: V) -> Result<V::Value> {
        Err(BencodeError::UnknownVariant("Struct variant not supported.".into()))
    }
}

impl<'a, R: 'a + Read> SeqVisitor for BencodeVisitor<'a, R> {
    type Error = BencodeError;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: Deserialize
    {
        self.de.update_state();
        match T::deserialize(self.de) {
            Ok(v) => Ok(Some(v)),
            _ => Ok(None),
        }
    }
    fn end(&mut self) -> Result<()> {
        self.de.state.pop();
        Ok(())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (1, None)
    }
}

impl<'a, R: 'a + Read> MapVisitor for BencodeVisitor<'a, R> {
    type Error = BencodeError;
    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: Deserialize
    {
        if self.de.state.last() != Some(&State::E) {
            self.de.update_state();
            match K::deserialize(self.de) {
                Ok(v) => Ok(Some(v)),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: Deserialize
    {
        self.de.update_state();
        V::deserialize(self.de)
    }

    fn end(&mut self) -> Result<()> {
        self.de.state.pop();
        Ok(())
    }
}

#[derive(PartialEq, Debug)]
enum State {
    S(Vec<u8>),
    I(i64),
    L,
    D,
    E,
}

#[derive(Debug)]
pub struct BencodeDecoder<R: Read> {
    reader: R,
    state: Vec<State>,
    is_struct: bool,
    is_option: bool
}

impl<R: Read> BencodeDecoder<R> {
    pub fn new(reader: R) -> BencodeDecoder<R> {
        BencodeDecoder {
            reader: reader,
            state: vec![],
            is_struct: false,
            is_option: false
        }
    }

    fn parse_int(&mut self) -> Result<State> {
        let mut buf = [0; 1];
        let mut result = String::new();
        while self.reader.read(&mut buf).unwrap() != 0 {
            match str::from_utf8(&buf) {
                Ok("e") => {
                    return match result.parse::<i64>() {
                        Ok(i) => Ok(State::I(i)),
                        _ => Err(Error::invalid_value(&result)),
                    }
                }
                Ok(c) => result.push_str(&c),
                _ => return Err(Error::invalid_value("Non UTF-8 integer encoding")),
            }
        }
        Err(Error::end_of_stream())
    }

    fn parse_byte_string_body(&mut self, len: i64) -> Result<Vec<u8>> {
        let mut buf = [0; 1];
        let mut result = Vec::new();
        for _ in 0..len {
            assert!(self.reader.read(&mut buf).unwrap() != 0);
            result.push(buf[0]);
        }
        Ok(result)
    }

    fn parse_byte_string_len(&mut self, len_char: char) -> Result<i64> {
        let mut buf = [0; 1];
        let mut len = String::new();
        len.push(len_char);
        loop {
            match self.reader.read(&mut buf) {
                Ok(1) => {
                    match String::from_utf8(buf.to_vec()) {
                        Ok(c) => {
                            match c.as_str() {
                                ":" => {
                                    match len.parse::<i64>() {
                                        Ok(len) => return Ok(len),
                                        _ => return Err(Error::invalid_value(&len)),
                                    }
                                }
                                n => len.push_str(n),
                            }
                        }
                        Err(_) => return Err(Error::invalid_value("Non UTF-8 string length encoding")),
                    }
                }
                _ => return Err(Error::end_of_stream()),
            }
        }
    }

    fn parse_byte_string(&mut self, len_char: char) -> Result<State> {
        match self.parse_byte_string_len(len_char) {
            Ok(len) => match self.parse_byte_string_body(len) {
                Ok(b) => Ok(State::S(b)),
                Err(e) => Err(e)
            },
            Err(e) => Err(e)
        }
    }

    fn parse_state(&mut self) -> Result<State> {
        let mut buf = [0; 1];
        if 1 == self.reader.read(&mut buf).unwrap() {
            match buf[0].into() {
                'l' => Ok(State::L),
                'd' => Ok(State::D),
                'e' => Ok(State::E),
                'i' => self.parse_int(),
                n @ '0' ... '9' => self.parse_byte_string(n),
                _ => Err(Error::end_of_stream()),
            }
        } else {
            Err(Error::end_of_stream())
        }
    }

    fn update_state(&mut self) {
        match self.parse_state() {
            Ok(s) => self.state.push(s),
            _ => (),
        }
    }
}

impl<R: Read> Deserializer for BencodeDecoder<R> {
    type Error = BencodeError;

    #[inline]
    fn deserialize<V: Visitor>(&mut self, mut visitor: V) -> Result<V::Value> {
        if self.state.last() == None {
            self.update_state();
        }
        if self.is_option {
            self.is_option = false;
            visitor.visit_some(self)
        } else {
            match self.state.pop() {
                Some(State::I(i)) => visitor.visit_i64(i),
                Some(State::S(s)) => visitor.visit_byte_buf(s),
                Some(State::L) => visitor.visit_seq(BencodeVisitor::new(self)),
                Some(State::D) => visitor.visit_map(BencodeVisitor::new(self)),
                _ => Err(Error::end_of_stream()),
            }
        }
    }

    forward_to_deserialize! {
        i64 string seq seq_fixed_size bool isize i8 i16 i32 usize u8 u16 u32
        u64 f32 f64 char str unit bytes map unit_struct tuple_struct tuple
        newtype_struct ignored_any
    }

    #[inline]
    fn deserialize_option<V:Visitor>(&mut self, visitor: V) -> Result<V::Value> {
        self.is_option = true;
        if self.state.last() == None {
            self.update_state();
        }
        self.deserialize(visitor)
    }

    #[inline]
    fn deserialize_struct<V: Visitor>(&mut self, _name: &'static str, _fields: &'static [&'static str], mut visitor: V) -> Result<V::Value> {
        self.is_struct = true;
        if self.state.last() == None {
            self.update_state();
        }
        visitor.visit_map(BencodeVisitor::new(self))
    }

    #[inline]
    fn deserialize_struct_field<V: Visitor>(&mut self, mut visitor: V) -> Result<V::Value> {
        if self.is_struct {
            match self.state.last() {
                Some(&State::S(ref b)) => visitor.visit_bytes(b),
                _ => Err(Error::end_of_stream()),
            }
        } else {
            self.is_struct = true;
            match self.state.last() {
                Some(&State::I(_)) => visitor.visit_str("Integer"),
                Some(&State::S(_)) => visitor.visit_str("ByteString"),
                Some(&State::D) => visitor.visit_str("Dict"),
                Some(&State::L) => visitor.visit_str("List"),
                _ => Err(Error::end_of_stream()),
            }
        }
    }

    #[inline]
    fn deserialize_enum<V: EnumVisitor>(&mut self,
                                        _enum: &'static str,
                                        _variants: &'static [&'static str],
                                        mut visitor: V)
                                        -> Result<V::Value> {
        self.is_struct = false;
        if self.state.last() == None {
            self.update_state();
        }
        visitor.visit(BencodeVisitor::new(self))
    }
}

pub fn from_str<T>(s: &str) -> Result<T>
    where T: Deserialize
{
    self::from_bytes(s.as_bytes())
}

pub fn from_bytes<T>(b: &[u8]) -> Result<T>
    where T: Deserialize
{
    let mut d = BencodeDecoder::new(b);
    Deserialize::deserialize(&mut d)
}
