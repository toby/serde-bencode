# serde-bencode

A [Serde](https://github.com/serde-rs/serde) backed [Bencode](https://en.wikipedia.org/wiki/Bencode) encoding/decoding library for Rust.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
serde = "0.8"
serde_derive = "0.8"
serde_bencode = "0.1.0"
```

Serde works best with Rust nightly, it is highly recommended that you use
it.

## Usage

This is an abbreviated `.torrent` parsing example from
[main.rs](src/main.rs). If you compile this crate as a binary, it will
print metadata for any Torrent sent to stdin.

More examples are available in [test.rs](src/test.rs).

```rust
#![feature(proc_macro)] // Rust nightly

extern crate serde_bencode;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use serde_bencode::decoder;
use std::io::{self, Read};
use serde::bytes::ByteBuf;

#[derive(Debug, Deserialize)]
struct Node(String, i64);

#[derive(Debug, Deserialize)]
struct File {
    path: Vec<String>,
    length: i64,
    #[serde(default)]
    md5sum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Info {
    name: String,
    pieces: ByteBuf,
    #[serde(rename="piece length")]
    piece_length: i64,
    #[serde(default)]
    md5sum: Option<String>,
    #[serde(default)]
    length: Option<i64>,
    #[serde(default)]
    files: Option<Vec<File>>,
    #[serde(default)]
    private: Option<u8>,
    #[serde(default)]
    path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="root hash")]
    root_hash: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Torrent {
    info: Info,
    #[serde(default)]
    announce: Option<String>,
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename="announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename="creation date")]
    creation_date: Option<i64>,
    #[serde(rename="comment")]
    comment: Option<String>,
    #[serde(default)]
    #[serde(rename="created by")]
    created_by: Option<String>,
}

fn main() {
    let stdin = io::stdin();
    let mut buffer = Vec::new();
    let mut handle = stdin.lock();
    match handle.read_to_end(&mut buffer) {
        Ok(_) => {
            match decoder::from_bytes::<Torrent>(&buffer) {
                Ok(t) => println!("{:?}", &t),
                Err(e) => println!("ERROR: {:?}", e)
            }
        },
        Err(e) => println!("ERROR: {:?}", e)

    }
}
```

## Bencode Enum

There is a [Bencode enum](src/bencode_enum.rs) provided when any valid
Bencode value is needed in a single typed container. For example you can
use it to serialize/deserialize type `Vec<Bencode>`:

```rust
let list: Vec<Bencode> = vec!["one".into(), "two".into(), "three".into(), 4i64.into()];
let mut ser = Encoder::new();
list.serialize(&mut ser).unwrap();
let list_serialize: Vec<u8> = ser.into();
assert_eq!(String::from_utf8(list_serialize).unwrap(), "l3:one3:two5:threei4ee");
```

## ByteBuf

In the `main.rs` example you'll notice that the `torrent.info.pieces` is
a `serde::bytes::ByteBuf`. This is a wrapper type provided by Serde that
allows `Vec<u8>` to be decoded as a Bencode ByteString instead of a
List.
