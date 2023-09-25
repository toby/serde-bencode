#![no_main]
use libfuzzer_sys::fuzz_target;

use serde_bencode::value::Value;
use serde_bencode::from_bytes;

fuzz_target!(|data: &[u8]| {
    let _: Result<Value, _> = from_bytes(data);
});
