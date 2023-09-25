extern crate torrust_serde_bencode;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use torrust_serde_bencode::de::{from_bytes, from_str};
use torrust_serde_bencode::error::Result;
use torrust_serde_bencode::ser::{to_bytes, to_string, Serializer};
use torrust_serde_bencode::value::Value;

fn test_value_ser_de<T: Into<Value>>(a: T) {
    let a = a.into();
    let mut ser = Serializer::new();
    a.serialize(&mut ser).unwrap();
    println!("bytes: {:?}", String::from_utf8_lossy(ser.as_ref()));
    let b: Value = from_bytes(ser.as_ref()).unwrap();
    assert_eq!(a, b);
}

fn test_value_de_ser(s: &str) {
    let d: Value = from_str(s).unwrap();
    let e = to_string(&d).unwrap();
    assert_eq!(s, e);
}

fn test_ser_de_eq<T>(a: T)
where
    T: Serialize + DeserializeOwned + Debug + Eq,
{
    let mut ser = Serializer::new();
    a.serialize(&mut ser).unwrap();
    println!("bytes: {:?}", String::from_utf8_lossy(ser.as_ref()));
    let b: T = from_bytes(ser.as_ref()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn ser_de_int() {
    test_value_ser_de(666i64);
}

#[test]
fn ser_de_string() {
    test_value_ser_de("yoda");
}

#[test]
fn ser_de_value_list_mixed() {
    test_value_ser_de(Value::List(vec![
        "one".into(),
        "two".into(),
        "three".into(),
        4i64.into(),
    ]));
}

#[test]
fn ser_de_value_list_nested() {
    let l_grandchild = Value::List(vec!["two".into()]);
    let l_child = Value::List(vec!["one".into(), l_grandchild]);
    test_value_ser_de(vec![
        "one".into(),
        "two".into(),
        "three".into(),
        4i64.into(),
        l_child,
    ]);
}

#[test]
fn ser_de_value_map() {
    let mut m = HashMap::new();
    m.insert("Mc".into(), "Burger".into());
    test_value_ser_de(m);
}

#[test]
fn ser_de_map_value_mixed() {
    let mut ma = HashMap::new();
    ma.insert("M jr.".into(), "nuggets".into());
    let s = Value::List(vec![
        "one".into(),
        "two".into(),
        "three".into(),
        4i64.into(),
    ]);
    let mut m = HashMap::new();
    m.insert("Mc".into(), "Burger".into());
    m.insert("joint".into(), ma.into());
    m.insert("woah".into(), s);
    test_value_ser_de(m);
}

#[test]
fn serialize_i64() {
    assert_eq!(to_string(&666i64).unwrap(), "i666e");
}

#[test]
fn serialize_str() {
    assert_eq!(to_string(&"xxx").unwrap(), "3:xxx");
}

#[test]
fn serialize_ascii_char() {
    assert_eq!(to_string(&'a').unwrap(), "1:a");
}

#[test]
fn serialize_uncode_char() {
    assert_eq!(to_string(&'\u{1F9D0}').unwrap(), "4:\u{01F9D0}");
}

#[test]
fn serialize_bool() {
    assert_eq!(to_string(&false).unwrap(), "i0e");
    assert_eq!(to_string(&true).unwrap(), "i1e");
}

#[test]
fn deserialize_to_string() {
    let r: String = from_str("3:yes").unwrap();
    assert_eq!(r, "yes");
}

#[test]
fn deserialize_to_i64() {
    let r: i64 = from_str("i666e").unwrap();
    assert_eq!(r, 666);
}

#[test]
fn deserialize_to_vec() {
    let r: Vec<i64> = from_str("li666ee").unwrap();
    assert_eq!(r, [666]);
}

#[test]
fn deserialize_to_freestyle() {
    let s = "li666e4:wontd3:onei666e4:yoyoli69ei89e4:yoyoeee";
    test_value_de_ser(s);
}

#[test]
#[ignore]
#[should_panic(expected = "assertion failed")]
fn trailing_chars() {
    let s = "i666ed";
    let r: Result<Value> = from_str(s);
    assert!(r.is_err());
}

#[test]
fn stop_short() {
    let s = "3:we";
    let r: Result<Value> = from_str(s);
    assert!(r.is_err());
}

#[test]
fn multi_digit_string_length() {
    let s: String = from_str("1:o").unwrap();
    assert_eq!(s, "o");

    let s: String = from_str("10:oooooooooo").unwrap();
    assert_eq!(s, "oooooooooo");

    let s: String = from_str("100:oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo").unwrap();
    assert_eq!(s,
               "oooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo");
}

#[test]
fn serialize_struct() {
    #[derive(Debug, Serialize)]
    struct Fake {
        x: i64,
        y: String,
    }
    let f = Fake {
        x: 1111,
        y: "dog".to_string(),
    };
    assert_eq!(to_string(&f).unwrap(), "d1:xi1111e1:y3:doge");
}

#[test]
fn deserialize_to_struct() {
    let b = "d1:xi1111e1:y3:doge";
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
    }
    assert_eq!(
        from_str::<Fake>(b).unwrap(),
        Fake {
            x: 1111,
            y: "dog".to_string(),
        }
    );
}

#[test]
fn deserialize_to_struct_with_option() {
    let b = "d1:xi1111e1:y3:dog1:z2:yoe";
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
        #[serde(default)]
        z: Option<String>,
        #[serde(default)]
        a: Option<String>,
    }
    let r: Fake = from_str(b).unwrap();
    assert_eq!(
        r,
        Fake {
            x: 1111,
            y: "dog".to_string(),
            z: Some("yo".to_string()),
            a: None,
        }
    );
}

#[test]
fn deserialize_to_value() {
    let b = "d1:xi1111e1:y3:doge";
    let r: Value = from_str(b).unwrap();
    let mut d = HashMap::new();
    d.insert("x".into(), 1111.into());
    d.insert("y".into(), "dog".into());
    assert_eq!(r, Value::Dict(d));
}

#[test]
fn deserialize_to_value_struct_mix() {
    let b = "d1:xi1111e1:y3:dog1:zi66e1:qli666eee";
    #[derive(PartialEq, Debug, Deserialize)]
    struct Fake {
        y: String,
        x: i64,
        z: Value,
        q: Vec<i64>,
    }
    let r: Fake = from_str(b).unwrap();
    assert_eq!(
        r,
        Fake {
            x: 1111,
            y: "dog".to_string(),
            z: Value::Int(66),
            q: vec![666],
        }
    );
}

#[test]
fn serialize_lexical_sorted_keys() {
    #[derive(Serialize)]
    struct Fake {
        aaa: i32,
        bb: i32,
        z: i32,
        c: i32,
    }
    let f = Fake {
        aaa: 1,
        bb: 2,
        z: 3,
        c: 4,
    };
    assert_eq!(to_string(&f).unwrap(), "d3:aaai1e2:bbi2e1:ci4e1:zi3ee");
}

#[test]
fn serialize_newtype_struct() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Fake(i32);
    let f = Fake(66);
    assert_eq!(to_string(&f).unwrap(), "i66e");
    test_ser_de_eq(f);
}

#[test]
fn serialize_some() {
    let f = Some(1);
    assert_eq!(to_string(&f).unwrap(), "i1e");
}

#[test]
fn serialize_none() {
    let f: Option<Value> = None;
    assert_eq!(to_string(&f).unwrap(), "");
}

#[test]
fn serialize_tuple() {
    let f = (1, 2, 3, "one");
    assert_eq!(to_string(&f).unwrap(), "li1ei2ei3e3:onee");
}

#[test]
fn serialize_tuple_struct() {
    #[derive(Serialize)]
    struct Fake(i32, i32);
    let f = Fake(66, 66);
    assert_eq!(to_string(&f).unwrap(), "li66ei66ee");
}

#[test]
fn readme_value_example() {
    let list: Vec<Value> = vec!["one".into(), "two".into(), "three".into(), 4i64.into()];
    assert_eq!(to_string(&list).unwrap(), "l3:one3:two5:threei4ee");
}

#[test]
fn struct_none_vals() {
    #[derive(Serialize)]
    struct Fake {
        a: Option<i32>,
        b: Option<i32>,
    }
    let f = Fake {
        a: None,
        b: Some(1),
    };
    assert_eq!(to_string(&f).unwrap(), "d1:bi1ee");
}

#[test]
fn ser_de_variant_unit() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A,
        B,
    }
    test_ser_de_eq(("pre".to_string(), Mock::A, "post".to_string()));
    test_ser_de_eq(("pre".to_string(), Mock::B, "post".to_string()));
}

#[test]
fn ser_de_variant_newtype() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A(i64),
        B(i64),
    }
    test_ser_de_eq(("pre".to_string(), Mock::A(123), "post".to_string()));
    test_ser_de_eq(("pre".to_string(), Mock::B(321), "post".to_string()));
}

#[test]
fn ser_de_variant_tuple() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A(i64, i64),
        B(i64, i64),
    }
    test_ser_de_eq(("pre".to_string(), Mock::A(123, 321), "post".to_string()));
    test_ser_de_eq(("pre".to_string(), Mock::B(321, 123), "post".to_string()));
}

#[test]
fn ser_de_variant_struct() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    enum Mock {
        A { a: i64, b: i64 },
        B { c: i64, d: i64 },
    }
    test_ser_de_eq((
        "pre".to_string(),
        Mock::A { a: 123, b: 321 },
        "post".to_string(),
    ));
    test_ser_de_eq((
        "pre".to_string(),
        Mock::B { c: 321, d: 123 },
        "post".to_string(),
    ));
}

#[test]
fn test_to_bytes() {
    assert_eq!(to_bytes(&"test").unwrap(), b"4:test");
}

#[test]
fn ser_de_adjacently_tagged_enum() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    #[serde(tag = "t", content = "c")]
    enum Mock {
        A,
        B,
    }

    test_ser_de_eq(Mock::A);
    test_ser_de_eq(Mock::B);
}

#[test]
fn ser_de_flattened_adjacently_tagged_enum() {
    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    struct Message {
        id: u64,
        #[serde(flatten)]
        body: Body,
    }

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
    #[serde(tag = "type", content = "content")]
    enum Body {
        Request { token: u64 },
        Response,
    }

    test_ser_de_eq(Message {
        id: 123,
        body: Body::Request { token: 456 },
    });
}

// https://github.com/toby/serde-bencode/issues/16 (simplified)
#[test]
fn ser_de_vec_of_tuples() {
    test_ser_de_eq(vec![(1, 2), (3, 4)]);
}

// https://github.com/toby/serde-bencode/issues/17
#[test]
fn ser_de_field_vec_tuple() {
    #[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
    struct Foo {
        bar: Vec<(u16,)>,
    }

    let foo = Foo {
        bar: vec![(1,), (3,)],
    };

    test_ser_de_eq(foo);
}

#[test]
fn ser_de_flattened_enum() {
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct KrpcMessage {
        message_type: MessageType,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    enum MessageType {
        Query,
        Response,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct KrpcResponse {
        #[serde(flatten)]
        krpc: KrpcMessage,
    }

    // Passes
    test_ser_de_eq(KrpcMessage {
        message_type: MessageType::Response,
    });

    // Fails
    test_ser_de_eq(KrpcResponse {
        krpc: KrpcMessage {
            message_type: MessageType::Response,
        },
    });
}

#[test]
fn deserialize_too_long_byte_string() {
    let _: Result<Value> = from_str("123456789123:1");
}
