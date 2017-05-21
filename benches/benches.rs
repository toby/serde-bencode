#![feature(test)]

extern crate test;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_bencode;

use test::Bencher;
use serde::Serialize;
use serde_bencode::ser::Serializer;
use serde_bencode::de::from_bytes;


#[bench]
fn ser_de_simple(b: &mut Bencher) {
    #[derive(Serialize, Deserialize)]
    struct Fake {
        a: i64,
        b: i64,
    }

    b.iter(|| {
        let a = Fake {a: 2, b: 7};
        let mut ser = Serializer::new();
        a.serialize(&mut ser).unwrap();
        let a_bytes: Vec<u8> = ser.into();
        let b: Fake = from_bytes(a_bytes.as_ref()).unwrap();
        b
    });
}

#[bench]
fn ser_de_nested(b: &mut Bencher) {
    #[derive(Serialize, Deserialize)]
    struct FakeA {
        a: i64,
        b: i64,
    }

    #[derive(Serialize, Deserialize)]
    struct FakeB {
        a: i64,
        b: FakeA,
    }

    b.iter(|| {
        let a = FakeB {a: 2, b: FakeA {a: 7, b: 9}};
        let mut ser = Serializer::new();
        a.serialize(&mut ser).unwrap();
        let a_bytes: Vec<u8> = ser.into();
        let b: FakeB = from_bytes(a_bytes.as_ref()).unwrap();
        b
    });
}
