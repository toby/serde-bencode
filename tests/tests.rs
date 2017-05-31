extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_bencode;

use serde_bencode::value::Value;
use serde_bencode::de::{self, from_bytes};
use serde_bencode::ser::Serializer;
use serde_bencode::error::Result;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::str::FromStr;
use std::collections::HashMap;
use std::fmt::Debug;

fn encode<T: Serialize>(b: &T) -> Vec<u8> {
    let mut ser = Serializer::new();
    b.serialize(&mut ser).unwrap();
    ser.into_vec()
}

fn decode_enum(s: &String) -> Option<Value> {
    match de::from_str(&s.as_str()) {
        Ok(r) => Some(r),
        _ => None,
    }
}

fn test_enum_enc_dec<T: Into<Value>>(a: T) {
    let a = a.into();
    let a_bytes = encode(&a);
    let b: Value = from_bytes(a_bytes.as_ref()).unwrap();
    assert_eq!(a, b);
}

fn test_enum_dec_enc(s: &String) {
    let d = decode_enum(s);
    let e = encode(&d.unwrap());
    assert_eq!(&s.as_bytes().to_vec(), &e);
}

fn test_ser_de_eq<T>(a: T)
    where T: Serialize + DeserializeOwned + Debug + Eq
{
    let mut ser = Serializer::new();
    a.serialize(&mut ser).unwrap();
    println!("bytes: {:?}", String::from_utf8_lossy(ser.as_ref()));
    let b: T = from_bytes(ser.as_ref()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn enc_dec_int() {
    test_enum_enc_dec(666i64);
}

#[test]
fn enc_dec_string() {
    test_enum_enc_dec(String::from_str("yoda").unwrap());
}

#[test]
fn enc_dec_enum_list_mixed() {
    test_enum_enc_dec(Value::List(vec!["one".into(), "two".into(), "three".into(), 4i64.into()]));
}

#[test]
fn enc_dec_enum_list_nested() {
    let l_grandchild = Value::List(vec!["two".into()]);
    let l_child = Value::List(vec!["one".into(), l_grandchild]);
    test_enum_enc_dec(vec!["one".into(),
                           "two".into(),
                           "three".into(),
                           4i64.into(),
                           l_child]);
}

#[test]
fn enc_dec_map() {
    let mut m = HashMap::new();
    m.insert("Mc".into(), "Burger".into());
    test_enum_enc_dec(m);
}

#[test]
fn enc_dec_map_enum_mixed() {
    let mut ma = HashMap::new();
    ma.insert("M jr.".into(), "nuggets".into());
    let s = Value::List(vec!["one".into(), "two".into(), "three".into(), 4i64.into()]);
    let mut m = HashMap::new();
    m.insert("Mc".into(), "Burger".into());
    m.insert("joint".into(), ma.into());
    m.insert("woah".into(), s);
    test_enum_enc_dec(m);
}

#[test]
fn serialize_i64() {
    let x: i64 = 666;
    let mut ser = Serializer::new();
    x.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(r, b"i666e");
}

#[test]
fn serialize_str() {
    let x = "xxx";
    let mut ser = Serializer::new();
    x.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(r, b"3:xxx");
}

#[test]
fn serialize_bool() {
    let x = false;
    let mut ser = Serializer::new();
    x.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(r, b"i0e");

    let x = true;
    let mut ser = Serializer::new();
    x.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(r, b"i1e");
}

#[test]
fn deserialize_to_string() {
    let s = "3:yes";
    let r: Result<String> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, "yes"),
        _ => panic!(),
    }
}

#[test]
fn deserialize_to_i64() {
    let s = "i666e";
    let r: Result<i64> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, 666),
        _ => panic!(),
    }
}

#[test]
fn deserialize_to_vec() {
    let s = "li666ee";
    let r: Result<Vec<i64>> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, [666]),
        _ => panic!(),
    }
}

#[test]
fn deserialize_to_freestyle() {
    let s = "li666e4:wontd3:onei666e4:yoyoli69ei89e4:yoyoeee";
    test_enum_dec_enc(&String::from_str(s).unwrap());
}

#[test]
#[ignore]
#[should_panic(expected = "assertion failed")]
fn trailing_chars() {
    let s = "i666ed";
    let r = decode_enum(&String::from_str(s).unwrap());
    assert!(r != None);
}

#[test]
fn stop_short() {
    let s = "3:we";
    let r = decode_enum(&String::from_str(s).unwrap());
    assert_eq!(r, None);
}

#[test]
fn multi_digit_string_length() {
    assert_eq!(de::from_str::<String>(&String::from_str("1:o").unwrap()).unwrap(),
               "o".to_owned());
    assert_eq!(de::from_str::<String>(&String::from_str("10:oooooooooo").unwrap()).unwrap(),
               "oooooooooo".to_owned());
    assert_eq!(de::from_str::<String>(&String::from_str("100:oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo").unwrap()).unwrap(), "oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo".to_owned());
}

#[test]
fn serialize_struct() {
    #[derive(Debug, Serialize)]
    struct Fake {
        x: i64,
        y: String,
    };
    let f = Fake {
        x: 1111,
        y: String::from("dog"),
    };
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "d1:xi1111e1:y3:doge");
}

#[test]
fn deserialize_to_struct() {
    let b = "d1:xi1111e1:y3:doge".as_bytes();
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
    };
    assert_eq!(de::from_bytes::<Fake>(b).unwrap(),
               Fake {
                   x: 1111,
                   y: String::from_str("dog").unwrap(),
               });
}

#[test]
fn deserialize_to_struct_with_option() {
    let b = "d1:xi1111e1:y3:dog1:z2:yoe".as_bytes();
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
        #[serde(default)]
        z: Option<String>,
        #[serde(default)]
        a: Option<String>,
    };
    let r: Result<Fake>;
    r = de::from_bytes(b);
    match r {
        Ok(r) => {
            assert_eq!(r,
                       Fake {
                           x: 1111,
                           y: String::from_str("dog").unwrap(),
                           z: Some(String::from_str("yo").unwrap()),
                           a: None,
                       })
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn deserialize_to_value() {
    let b = "d1:xi1111e1:y3:doge".as_bytes();
    let r: Value = de::from_bytes(b).unwrap();
    let mut d = HashMap::new();
    d.insert("x".into(), 1111.into());
    d.insert("y".into(), "dog".into());
    assert_eq!(r, Value::Dict(d));
}

#[test]
fn deserialize_to_value_struct_mix() {
    let b = "d1:xi1111e1:y3:dog1:zi66e1:qli666eee".as_bytes();
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
        z: Value,
        q: Vec<i64>,
    };
    let r: Result<Fake>;
    r = de::from_bytes(b);
    match r {
        Ok(r) => {
            assert_eq!(r,
                       Fake {
                           x: 1111,
                           y: String::from_str("dog").unwrap(),
                           z: Value::Int(66),
                           q: vec![666],
                       })
        }
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn serialize_lexical_sorted_keys() {
    #[derive(Serialize)]
    struct Fake {
        aaa: i32,
        bb: i32,
        z: i32,
        c: i32,
    };
    let f = Fake {
        aaa: 1,
        bb: 2,
        z: 3,
        c: 4,
    };
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(),
               "d3:aaai1e2:bbi2e1:ci4e1:zi3ee");
}

#[test]
fn serialize_newtype_struct() {
    #[derive(Serialize)]
    struct Fake(i32);
    let f = Fake(66);
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "i66e");
}

#[test]
fn serialize_some() {
    let f = Some(1);
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "i1e");
}

#[test]
fn serialize_none() {
    let f: Option<Value> = None;
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "");
}

#[test]
fn serialize_tuple() {
    let f = (1, 2, 3, "one");
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "li1ei2ei3e3:onee");
}

#[test]
fn serialize_tuple_struct() {
    #[derive(Serialize)]
    struct Fake(i32, i32);
    let f = Fake(66, 66);
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "li66ei66ee");
}

#[test]
fn readme_value_example() {
    let list: Vec<Value> = vec!["one".into(), "two".into(), "three".into(), 4i64.into()];
    let mut ser = Serializer::new();
    list.serialize(&mut ser).unwrap();
    let list_serialize: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(list_serialize).unwrap(),
               "l3:one3:two5:threei4ee");
}

#[test]
fn struct_none_vals() {
    #[derive(Serialize)]
    struct Fake {
        a: Option<i32>,
        b: Option<i32>,
    };
    let f = Fake {
        a: None,
        b: Some(1),
    };
    let mut ser = Serializer::new();
    f.serialize(&mut ser).unwrap();
    let r: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(r).unwrap(), "d1:bi1ee");
}

#[test]
fn ser_de_variant_unit() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A,
        B,
    };
    test_ser_de_eq(Mock::A);
    test_ser_de_eq(Mock::B);
}

#[test]
fn ser_de_variant_newtype() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A(i64),
        B(i64),
    };
    test_ser_de_eq(Mock::A(123));
    test_ser_de_eq(Mock::B(321));
}

#[test]
fn ser_de_variant_tuple() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A(i64, i64),
        B(i64, i64),
    };
    test_ser_de_eq(Mock::A(123, 321));
    test_ser_de_eq(Mock::B(321, 123));
}

#[test]
fn ser_de_variant_struct() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A { a: i64, b: i64 },
        B { c: i64, d: i64 },
    };
    test_ser_de_eq(Mock::A { a: 123, b: 321 });
    test_ser_de_eq(Mock::B { c: 321, d: 123 });
}
