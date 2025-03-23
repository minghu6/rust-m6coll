//!
//! TODO: implement haystack Pattern trait and algorithm
//!

use std::{
    borrow::{Borrow, BorrowMut},
    io::Write,
    ops::{
        Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive
    },
    slice::SliceIndex,
};

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait ConsumeBytesInto: Sized {
    type Err;

    /// (Self, offset)
    fn consume_bytes_into(bytes: &ByteStr)
    -> Result<(Self, usize), Self::Err>;
}

pub trait FromBytesInto: Sized {
    type Err;

    fn from_bytes_into(bytes: &ByteStr) -> Result<Self, Self::Err>;
}

pub trait WriteIntoBytes {
    fn write_into_bytes<W: Write>(&self, bytes: &mut W)
    -> std::io::Result<()>;
}

pub trait Pattern: Sized {
    fn as_bytes(&self) -> &[u8];

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteStr {
    value: [u8],
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteString {
    value: Vec<u8>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T: ToString> WriteIntoBytes for T {
    fn write_into_bytes<W: Write>(
        &self,
        bytes: &mut W,
    ) -> std::io::Result<()> {
        bytes.write_all(self.to_string().as_bytes())
    }
}

impl<T: Borrow<[u8]> + ?Sized> Pattern for &T {
    fn as_bytes(&self) -> &[u8] {
        (*self).borrow()
    }
}

impl Pattern for u8 {
    fn as_bytes(&self) -> &[u8] {
        std::slice::from_ref(self)
    }
}

impl ByteStr {
    pub fn trim(&self) -> &Self {
        self.trim_start().trim_end()
    }

    pub fn trim_start(&self) -> &Self {
        if let Some(pos) = self.iter().position(|c| !c.is_ascii_whitespace()) {
            &self[pos..]
        }
        else {
            &self[self.len()..]
        }
    }

    pub fn trim_end(&self) -> &Self {
        if let Some(pos) = self.iter().rposition(|c| !c.is_ascii_whitespace())
        {
            &self[..=pos]
        }
        else {
            &self[..0]
        }
        .into()
    }

    pub fn new<B: ?Sized + AsRef<[u8]>>(bytes: &B) -> &Self {
        unsafe { &*(bytes.as_ref() as *const [u8] as *const ByteStr) }
    }

    pub fn new_mut<B: ?Sized + AsMut<[u8]>>(bytes: &mut B) -> &mut Self {
        unsafe { &mut *(bytes.as_mut() as *mut [u8] as *mut ByteStr) }
    }

    pub fn parse_into<F: FromBytesInto>(&self) -> Result<F, F::Err> {
        F::from_bytes_into(self)
    }

    /// return leftmost occurance if any
    ///
    /// Python str find "BMHBNFS" algotrithm
    ///
    /// O(1/n) time complexity in most cases
    ///
    /// O(8) space complexity
    ///
    ///
    #[cfg(feature = "bitmap")]
    pub fn find<P: Pattern>(&self, pat: P) -> Option<usize> {
        use m6bitmap::BitMap;

        fn build_searcher(bytes: &[u8]) -> (BitMap, usize) {
            debug_assert!(!bytes.is_empty());

            let mut alphabet = BitMap::new(256);
            let lastpos = bytes.len() - 1;
            let lastc = bytes[lastpos];

            let mut skip = bytes.len();

            for (i, c) in bytes.iter().take(lastpos).cloned().enumerate() {
                alphabet.set(c as usize);

                if c == lastc {
                    skip = lastpos - i;
                }
            }

            alphabet.set(lastc as usize);

            (alphabet, skip)
        }

        let pat = pat.as_bytes();
        let string = self;

        // follow behaviour str::find
        if pat.is_empty() {
            return Some(0);
        }
        else if pat.len() == 1 {
            return self.iter().position(|&c| c == pat.as_bytes()[0]);
        }

        let (patalphabet, skip) = build_searcher(pat.as_bytes());

        let stringlen = string.len();
        let patlen = pat.len();
        let patlastpos = patlen - 1;

        let mut i = patlastpos;

        while i < stringlen {
            if string[i] == pat[patlastpos] {
                if string[i - patlastpos..i] == pat[..patlastpos] {
                    return Some(i - patlastpos);
                }

                if i + 1 == stringlen {
                    break;
                }

                if !patalphabet.test(string[i + 1] as usize) {
                    // sunday
                    i += patlen + 1;
                }
                else {
                    // horspool
                    i += skip;
                }
            }
            else {
                if i + 1 == stringlen {
                    break;
                }

                if !patalphabet.test(string[i + 1] as usize) {
                    // sunday
                    i += patlen + 1;
                }
                else {
                    i += 1;
                }
            }
        }

        None
    }
}

impl ToOwned for ByteStr {
    type Owned = ByteString;

    fn to_owned(&self) -> Self::Owned {
        ByteString {
            value: self.to_vec(),
        }
    }
}

impl Deref for ByteStr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for ByteStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl AsRef<[u8]> for ByteStr {
    fn as_ref(&self) -> &[u8] {
        &self
    }
}

impl AsMut<[u8]> for ByteStr {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.value
    }
}

impl AsRef<Self> for ByteStr {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl AsMut<Self> for ByteStr {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl Borrow<[u8]> for ByteStr {
    fn borrow(&self) -> &[u8] {
        &self
    }
}

impl BorrowMut<[u8]> for ByteStr {
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.value
    }
}

impl Index<usize> for ByteStr {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl IndexMut<usize> for ByteStr {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}

impl Index<RangeFrom<usize>> for ByteStr {
    type Output = Self;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<RangeFrom<usize>> for ByteStr {
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl Index<RangeTo<usize>> for ByteStr {
    type Output = Self;

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<RangeTo<usize>> for ByteStr {
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl Index<RangeToInclusive<usize>> for ByteStr {
    type Output = Self;

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<RangeToInclusive<usize>> for ByteStr {
    fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl Index<Range<usize>> for ByteStr {
    type Output = Self;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<Range<usize>> for ByteStr {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl Index<RangeFull> for ByteStr {
    type Output = Self;

    fn index(&self, index: RangeFull) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<RangeFull> for ByteStr {
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl Index<RangeInclusive<usize>> for ByteStr {
    type Output = Self;

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        Self::new(&self.value[index])
    }
}

impl IndexMut<RangeInclusive<usize>> for ByteStr {
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        Self::new_mut(&mut self.value[index])
    }
}

impl<'a> From<&'a [u8]> for &'a ByteStr {
    fn from(value: &'a [u8]) -> Self {
        ByteStr::new(value)
    }
}

impl<'a> From<&'a mut [u8]> for &'a mut ByteStr {
    fn from(value: &'a mut [u8]) -> Self {
        ByteStr::new_mut(value)
    }
}

impl<'a> Into<&'a [u8]> for &'a ByteStr {
    fn into(self) -> &'a [u8] {
        ByteStr::new(self)
    }
}

impl<'a> Into<&'a mut [u8]> for &'a mut ByteStr {
    fn into(self) -> &'a mut [u8] {
        &mut self.value
    }
}

// impl<I: SliceIndex<Self>> Index<I> for ByteStr {
//     type Output = I::Output;

//     fn index(&self, index: I) -> &Self::Output {
//         &self.value[..].index(**index)
//     }
// }

impl<B: ?Sized + AsRef<[u8]>> PartialEq<B> for ByteStr {
    fn eq(&self, other: &B) -> bool {
        &self.value == other.as_ref()
    }
}


impl ByteString {
    pub fn push(&mut self, value: u8) {
        self.value.push(value)
    }

    pub fn push_str(&mut self, value: &ByteStr) {
        self.value.extend_from_slice(&value[..])
    }

    pub fn new() -> Self {
        Self { value: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            value: Vec::with_capacity(capacity),
        }
    }

    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            value: self.value.split_off(at),
        }
    }
}

impl Deref for ByteString {
    type Target = ByteStr;

    fn deref(&self) -> &Self::Target {
        self.value.as_slice().into()
    }
}

impl DerefMut for ByteString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut_slice().into()
    }
}

impl AsRef<ByteStr> for ByteString {
    fn as_ref(&self) -> &ByteStr {
        self.deref()
    }
}

impl AsMut<ByteStr> for ByteString {
    fn as_mut(&mut self) -> &mut ByteStr {
        self.deref_mut()
    }
}

impl Borrow<ByteStr> for ByteString {
    fn borrow(&self) -> &ByteStr {
        self.deref()
    }
}

impl BorrowMut<ByteStr> for ByteString {
    fn borrow_mut(&mut self) -> &mut ByteStr {
        self.deref_mut()
    }
}

impl<const N: usize> From<&[u8; N]> for ByteString {
    fn from(value: &[u8; N]) -> Self {
        value.to_vec().into()
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

impl Extend<u8> for ByteString {
    fn extend<T: IntoIterator<Item = u8>>(&mut self, iter: T) {
        self.value.extend(iter)
    }
}

impl<'a> Extend<&'a u8> for ByteString {
    fn extend<T: IntoIterator<Item = &'a u8>>(&mut self, iter: T) {
        self.value.extend(iter.into_iter().cloned())
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Modules

#[cfg(feature = "cow")]
mod require_cow {
    use super::*;
    use crate::cow::FlatCow;

    ///////////////////////////////////////////////////////////////////////////
    //// Traits

    pub trait ConsumeBytesAs<'o>: Sized {
        type Err;

        /// (Self, offset)
        fn consume_bytes_as<'i: 'o>(
            bytes: &FlatCow<'i, ByteStr>,
        ) -> Result<(Self, usize), Self::Err>;
    }


    pub trait FromBytesAs<'o>: Sized {
        type Err;

        fn from_bytes_as<'i: 'o>(
            bytes: &FlatCow<'i, ByteStr>,
        ) -> Result<Self, Self::Err>;
    }

    ///////////////////////////////////////////////////////////////////////////
    //// Implementations

    impl ByteStr {}

    impl<'o, T: ConsumeBytesAs<'o>> FromBytesAs<'o> for T {
        type Err = T::Err;

        fn from_bytes_as<'i: 'o>(
            bytes: &FlatCow<'i, ByteStr>,
        ) -> Result<Self, Self::Err> {
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

    use core::slice::SlicePattern;

    use super::*;

    #[test]
    fn test_partial_eq() {
        let a = ByteStr::new(b"abc");
        let b = b"abc";

        assert_eq!(&a, &b);
        assert_eq!(&b, &a.as_slice());
        assert_eq!(b, a.as_slice());
    }

    #[cfg(feature = "bitmap")]
    #[test]
    fn test_find() {
        let haystack = ByteStr::new(b"abcde");
        let needle = ByteStr::new(b"cde");

        assert_eq!(haystack.find(needle), Some(2));

        let needle = ByteStr::new(b"xyz");
        assert_eq!(haystack.find(needle), None);

        let needle = ByteStr::new(b"");
        assert_eq!(haystack.find(needle), Some(0));

        let needle = ByteStr::new(b"a");
        assert_eq!(haystack.find(needle), Some(0));

        let needle = ByteStr::new(b"e");
        assert_eq!(haystack.find(needle), Some(4));
    }
}
