use serde_bytes::ByteBuf;
use std::collections::BTreeMap;

#[derive(PartialOrd, Eq, Ord, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Bencode {
    ByteString(ByteBuf),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<Bencode, Bencode>)
}

impl From<i64> for Bencode {
    fn from(i: i64) -> Bencode {
        Bencode::Integer(i)
    }
}

impl From<String> for Bencode {
    fn from(s: String) -> Bencode {
        Bencode::ByteString(s.into_bytes().into())
    }
}

impl<'a> From<&'a str> for Bencode {
    fn from(s: &str) -> Bencode {
        Bencode::ByteString(s.as_bytes().to_vec().into())
    }
}

impl From<Vec<u8>> for Bencode {
    fn from(bs: Vec<u8>) -> Bencode {
        Bencode::ByteString(bs.into())
    }
}

impl From<Vec<Bencode>> for Bencode {
    fn from(l: Vec<Bencode>) -> Bencode {
        Bencode::List(l.clone())
    }
}

impl From<BTreeMap<Vec<u8>, Bencode>> for Bencode {
    fn from(m: BTreeMap<Vec<u8>, Bencode>) -> Bencode {
        let mut n: BTreeMap<Bencode, Bencode> = BTreeMap::new();
        for (key, value) in m.iter() {
            n.insert(key.clone().into(), value.clone());
        }
        Bencode::Dict(n)
    }
}
