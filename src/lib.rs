#![allow(clippy::float_cmp)]

#[macro_use]
extern crate more_asserts;

#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
/// The generated bindings for the C library. This provides a raw (and memory un-safe) access to the
/// API. The rest of the public API defines helper classes that wrap the C functionality for ease of
/// use and memory safety. See [filter](rust_liquid_dsp::filter).
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[doc(hidden)]
use quick_error::quick_error;

pub mod filter;

#[cfg(test)]
mod test;

quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum Error {
        CapacityError(needed: usize, found: usize) {
            display(
                "Capacity for output buffer is too small. Need at least {} bytes. Only have {} bytes.",
                needed, found
            )
        }
    }
}
