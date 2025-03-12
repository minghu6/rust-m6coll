//!
//! TODO: implement haystack Pattern trait and algorithm
//!

use std::{borrow::Borrow, ops::{Deref, Index}, slice::SliceIndex};


////////////////////////////////////////////////////////////////////////////////
//// Structures


#[derive(Debug, Eq, PartialOrd, Ord, Hash)]
pub struct ByteStr {
    value: [u8]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteString {
    value: Vec<u8>
}


////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl ByteStr {
    pub fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    pub fn trim_start(&self) -> &Self {
        if let Some(pos) = self.iter().position(|c| !c.is_ascii_whitespace()) {
            &self[pos..]
        } else {
            &self[self.len()..]
        }.into()
    }

    pub fn trim_end(&self) -> &Self {
        if let Some(pos) = self.iter().rposition(|c| !c.is_ascii_whitespace()) {
            &self[..=pos]
        } else {
            &self[..0]
        }.into()
    }

    pub fn new<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> &Self {
        unsafe { &*(bytes.as_ref() as *const [u8] as *const ByteStr) }
    }
}

impl ToOwned for ByteStr {

    type Owned = ByteString;

    fn to_owned(&self) -> Self::Owned {
        ByteString {
            value: self.to_vec()
        }
    }
}

impl Deref for ByteStr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl AsRef<[u8]> for ByteStr {
    fn as_ref(&self) -> &[u8] {
        &self
    }
}

impl<'a> From<&'a [u8]> for &'a ByteStr
{
    fn from(value: &'a [u8]) -> Self {
        ByteStr::new(value)
    }
}

impl<'a> Into<&'a [u8]> for &'a ByteStr {
    fn into(self) -> &'a [u8] {
        unsafe { &*(self as *const ByteStr as *const [u8]) }
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for ByteStr {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.value.index(index)
    }
}

impl<B: ?Sized + AsRef<[u8]>> PartialEq<B> for ByteStr {
    fn eq(&self, other: &B) -> bool {
        &self.value == other.as_ref()
    }
}

impl ByteString {
    pub fn push(&mut self, value: u8) {
        self.value.push(value)
    }
}

impl Deref for ByteString {
    type Target = ByteStr;

    fn deref(&self) -> &Self::Target {
        self.value.as_slice().into()
    }
}

impl Borrow<ByteStr> for ByteString {
    fn borrow(&self) -> &ByteStr {
        self.deref()
    }
}

impl Borrow<[u8]> for ByteString {
    fn borrow(&self) -> &[u8] {
        self.deref()
    }
}

impl From<&[u8]> for ByteString {
    fn from(value: &[u8]) -> Self {
        value.to_vec().into()
    }
}

impl From<Vec<u8>> for ByteString {
    fn from(value: Vec<u8>) -> Self {
        Self { value }
    }
}

impl Into<Vec<u8>> for ByteString {
    fn into(self) -> Vec<u8> {
        self.value
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for ByteString {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.value.index(index)
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Modules

#[cfg(feature = "cow")]
mod require_cow {
    use crate::cow::FlatCow;

    use super::*;

    ///////////////////////////////////////////////////////////////////////////
    //// Traits

    pub trait ConsumeBytesAs<'a>: Sized {
        type Err;

        /// (Self, offset)
        fn consume_bytes_as(bytes: &FlatCow<ByteStr>) -> Result<(Self, usize), Self::Err>;
    }

    pub trait ConsumeBytesInto: Sized {
        type Err;

        /// (Self, offset)
        fn consume_bytes_into(bytes: &ByteStr) -> Result<(Self, usize), Self::Err>;
    }

    pub trait FromBytesAs<'a>: Sized {
        type Err;

        fn from_bytes_as(bytes: &FlatCow<ByteStr>) -> Result<Self, Self::Err>;
    }

    pub trait FromBytesInto: Sized {
        type Err;

        fn from_bytes_into(bytes: &ByteStr) -> Result<Self, Self::Err>;
    }

    ///////////////////////////////////////////////////////////////////////////
    //// Implementations

    impl<'a, T: ConsumeBytesAs<'a>> FromBytesAs<'a> for T {
        type Err = T::Err;

        fn from_bytes_as(bytes: &FlatCow<ByteStr>) -> Result<Self, Self::Err> {
            let (it, _offset) = T::consume_bytes_as(bytes)?;
            Ok(it)
        }
    }

    impl<T: ConsumeBytesInto> FromBytesInto for T {
        type Err = T::Err;

        fn from_bytes_into(bytes: &ByteStr) -> Result<Self, Self::Err> {
            let (it, _offset) = T::consume_bytes_into(bytes)?;
            Ok(it)
        }
    }
}

#[cfg(feature = "cow")]
pub use require_cow::*;


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_partial_eq() {
        let a = ByteStr::new(b"abc");
        let b = b"abc";

        assert_eq!(&a, &b);
        assert_eq!(&b, &a.as_ref());
        assert_eq!(b, &a[..]);
    }
}
