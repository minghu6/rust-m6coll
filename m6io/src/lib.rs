#![feature(trait_alias)]
#![feature(slice_pattern)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(str_from_raw_parts)]

#[cfg(feature = "cow")]
mod cow;
#[cfg(feature = "bstr")]
mod bstr;

#[cfg(feature = "nom")]
pub mod nom;

#[cfg(feature = "bstr")]
pub use bstr::*;
#[cfg(feature = "cow")]
pub use cow::*;
