extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_bencode;

use serde_bencode::bencode_enum::Bencode;
use serde_bencode::de;
use serde_bencode::ser::Serializer;
use serde_bencode::error::BencodeError;
use serde::ser::Serialize;
use serde::de::Deserialize;
use std::str::FromStr;
use std::collections::BTreeMap;
use std::fmt::Debug;

fn encode<T: Serialize>(b: &T) -> Vec<u8> {
    let mut ser = Serializer::new();
    b.serialize(&mut ser).unwrap();
    ser.into_vec()
}

fn decode_bytes<'de, T: Deserialize<'de> + Debug>(b: &'de [u8]) -> Option<T> {
    match de::from_bytes(b) {
        Ok(r) => Some(r),
        Err(e) => {
            println!("Error: {:?}", e);
            None
        }
    }
}

fn decode_enum(s: &String) -> Option<Bencode> {
    match de::from_str(&s.as_str()) {
        Ok(r) => Some(r),
        _ => None,
    }
}

fn test_enum_enc_dec<T: Into<Bencode> + std::fmt::Debug>(x: T) {
    let b = &x.into();
    let s = encode(b);
    match decode_bytes::<Bencode>(&s) {
        Some(d) => assert_eq!(&d, b),
        _ => panic!("encode and decode don't match"),
    }
}

fn test_enum_dec_enc(s: &String) {
    let d = decode_enum(s);
    let e = encode(&d.unwrap());
    assert_eq!(&s.as_bytes().to_vec(), &e);
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
    test_enum_enc_dec(Bencode::List(vec!["one".into(), "two".into(), "three".into(), 4i64.into()]));
}

#[test]
fn enc_dec_enum_list_nested() {
    let l_grandchild = Bencode::List(vec!["two".into()]);
    let l_child = Bencode::List(vec!["one".into(), l_grandchild]);
    test_enum_enc_dec(vec!["one".into(),
                           "two".into(),
                           "three".into(),
                           4i64.into(),
                           l_child]);
}

#[test]
#[ignore]
fn enc_dec_map() {
    let mut m = BTreeMap::new();
    m.insert("Mc".into(), "Burger".into());
    test_enum_enc_dec(m);
}

#[test]
#[ignore]
fn enc_dec_map_enum_mixed() {
    let mut ma = BTreeMap::new();
    ma.insert("M jr.".into(), "nuggets".into());
    let s = Bencode::List(vec!["one".into(), "two".into(), "three".into(), 4i64.into()]);
    let mut m = BTreeMap::new();
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
    let r: Result<String, BencodeError> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, "yes"),
        _ => panic!(),
    }
}

#[test]
fn deserialize_to_i64() {
    let s = "i666e";
    let r: Result<i64, BencodeError> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, 666),
        _ => panic!(),
    }
}

#[test]
fn deserialize_to_vec() {
    let s = "li666ee";
    let r: Result<Vec<i64>, BencodeError> = de::from_str(&s);
    match r {
        Ok(v) => assert_eq!(v, [666]),
        _ => panic!(),
    }
}

#[test]
#[ignore]
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
#[ignore]
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
    let r: Result<Fake, BencodeError>;
    r = de::from_bytes(b);
    match r {
        Ok(r) => {
            assert_eq!(r,
                       Fake {
                           x: 1111,
                           y: String::from_str("dog").unwrap(),
                       })
        }
        Err(e) => panic!("Error: {:?}", e),
    }
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
    let r: Result<Fake, BencodeError>;
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
fn deserialize_to_bencode() {
    let b = "d1:xi1111e1:y3:doge".as_bytes();
    #[derive(Debug, Deserialize)]
    let r: Result<Bencode, BencodeError>;
    r = de::from_bytes(b);
    let mut d: BTreeMap<Bencode, Bencode> = BTreeMap::new();
    d.insert("x".into(), 1111.into());
    d.insert("y".into(), "dog".into());
    match r {
        Ok(r) => assert_eq!(r, Bencode::Dict(d)),
        Err(e) => panic!("Error: {:?}", e),
    }
}

#[test]
fn deserialize_to_bencode_struct_mix() {
    let b = "d1:xi1111e1:y3:dog1:zi66e1:qli666eee".as_bytes();
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
        z: Bencode,
        q: Vec<i64>,
    };
    let r: Result<Fake, BencodeError>;
    r = de::from_bytes(b);
    match r {
        Ok(r) => {
            assert_eq!(r,
                       Fake {
                           x: 1111,
                           y: String::from_str("dog").unwrap(),
                           z: Bencode::Integer(66),
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
fn serialize_newtype_variant() {
    #[derive(Serialize)]
    enum Fake {
        Test(i32),
    };
    let f = Fake::Test(66);
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
    let f: Option<Bencode> = None;
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
fn readme_bencode_example() {
    let list: Vec<Bencode> = vec!["one".into(), "two".into(), "three".into(), 4i64.into()];
    let mut ser = Serializer::new();
    list.serialize(&mut ser).unwrap();
    let list_serialize: Vec<u8> = ser.into_vec();
    assert_eq!(String::from_utf8(list_serialize).unwrap(),
               "l3:one3:two5:threei4ee");
}

#[test]
#[ignore]
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
