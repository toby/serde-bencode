# serde-bencode

A [Serde](https://github.com/serde-rs/serde) backed [Bencode](https://en.wikipedia.org/wiki/Bencode)
encoding/decoding library for Rust.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
serde_bencode = "^0.1.0"
serde = "^1.0.0"
serde_derive = "^1.0.0"
```

Serde works best with Rust nightly, it is highly recommended that you use
it.

## Usage

This is an abbreviated `.torrent` parsing example from
[examples/parse_torrent.rs](examples/parse_torrent.rs). If you compile this crate as a binary, it
will print metadata for any Torrent sent to stdin.

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

In the `parse_torrent.rs` example you'll notice that the `torrent.info.pieces` is
a `serde::bytes::ByteBuf`. This is a wrapper type provided by Serde that
allows `Vec<u8>` to be decoded as a Bencode ByteString instead of a
List.
