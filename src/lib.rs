#![feature(proc_macro)]

#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod encoder;
pub mod decoder;
pub mod bencode_enum;

#[cfg(test)]
mod test;
