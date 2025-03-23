#![feature(trait_alias)]
#![feature(slice_pattern)]

#[cfg(feature = "cow")]
mod cow;
#[cfg(feature = "bytestr")]
mod bytestr;

#[cfg(feature = "bytestr")]
pub use bytestr::*;
#[cfg(feature = "cow")]
pub use cow::*;
