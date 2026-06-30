#![feature(trait_alias)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(str_from_raw_parts)]


#[cfg(feature = "cow")]
pub mod cow;
#[cfg(feature = "bstr")]
pub mod bstr;
#[cfg(feature = "rawbuf")]
pub mod rawbuf;
#[cfg(feature = "nom")]
pub mod nom;
