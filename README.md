# Torrust Serde Bencode

A [Serde](https://github.com/serde-rs/serde) backed [Bencode](https://en.wikipedia.org/wiki/Bencode)
encoding/decoding library for Rust.

Forked from: <https://github.com/toby/serde-bencode> due to inactivity in upstream repo.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
torrust-serde-bencode = "^0.2.3"
serde = "^1.0.0"
serde_derive = "^1.0.0"
```

## Usage

This is an abbreviated `.torrent` parsing example from [examples/parse_torrent.rs](examples/parse_torrent.rs). If you compile this crate as a binary, it will print metadata for any Torrent sent to stdin.
