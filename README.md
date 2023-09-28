# serde-bencode

**Archived:** I no longer maintain this repo as I've moved to [Go full-time](https://charm.sh). Please see [torrust/torrust-serde-bencode](https://github.com/torrust/torrust-serde-bencode) for a maintained fork.

A [Serde](https://github.com/serde-rs/serde) backed [Bencode](https://en.wikipedia.org/wiki/Bencode)
encoding/decoding library for Rust.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
serde_bencode = "^0.2.3"
serde = "^1.0.0"
serde_derive = "^1.0.0"
```

## Usage

This is an abbreviated `.torrent` parsing example from
[examples/parse_torrent.rs](examples/parse_torrent.rs). If you compile this crate as a binary, it
will print metadata for any Torrent sent to stdin.
