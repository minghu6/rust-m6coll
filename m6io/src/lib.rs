#![feature(trait_alias)]
#![feature(slice_pattern)]
#![feature(ascii_char)]
#![feature(ascii_char_variants)]
#![feature(str_from_raw_parts)]
#![feature(new_zeroed_alloc)]
#![feature(allocator_api)]
#![feature(ptr_alignment_type)]
#![feature(pointer_try_cast_aligned)]

#[cfg(feature = "cow")]
pub mod cow;
#[cfg(feature = "bstr")]
pub mod bstr;
#[cfg(feature = "rawbuf")]
pub mod rawbuf;
#[cfg(feature = "nom")]
pub mod nom;
