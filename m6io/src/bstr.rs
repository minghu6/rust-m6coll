//!
//! TODO: implement haystack Pattern trait and algorithm
//!

use std::{
    borrow::{Borrow, BorrowMut}, fmt::{Debug, Display}, io::Write, ops::{
        Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull,
        RangeInclusive, RangeTo, RangeToInclusive,
    }, slice::SliceIndex
};

////////////////////////////////////////////////////////////////////////////////
//// Macros

#[macro_export]
macro_rules! bstr {
    ($s:literal) => {
        ByteStr::new($s.as_bytes())
    };
}

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait ConsumeByteStr: Sized {
    type Err;

    /// (Self, offset)
    fn consume_bstr(bytes: &ByteStr) -> Result<(Self, usize), Self::Err>;
}

pub trait FromByteStr: Sized {
    type Err;

    fn from_bstr(bytes: &ByteStr) -> Result<Self, Self::Err>;
}

pub trait WriteIntoBytes {
    fn write_into_bytes<W: Write>(&self, w: &mut W) -> std::io::Result<usize>;
}

pub trait ToByteString {
    fn to_bstring(&self) -> ByteString;
}

pub trait Pattern {
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

#[derive(Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteStr {
    value: [u8],
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteString {
    value: Vec<u8>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

// require FromStr as local trait
// impl<T: ConsumeByteStr> std::str::FromStr for T {
//     type Err = T::Err;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let (it, n) = Self::consume_bstr(ByteStr::new(s))?;

//         Ok(it)
//     }
// }

// conflict
// impl<T: ToString> WriteIntoBytes for T {
//     fn write_into_bytes<W: Write>(
//         &self,
//         w: &mut W,
//     ) -> std::io::Result<usize> {
//         let s = self.to_string();
//         let bytes = s.as_bytes();
//         let n = bytes.len();

//         w.write_all(bytes)?;

//         Ok(n)
//     }
// }

impl<T: WriteIntoBytes> ToByteString for T {
    fn to_bstring(&self) -> ByteString {
        let mut cursor = std::io::Cursor::new(Vec::new());

        self.write_into_bytes(&mut cursor).unwrap();

        cursor.into_inner().into()
    }
}

impl<T: Pattern + ?Sized> WriteIntoBytes for T {
    fn write_into_bytes<W: Write>(&self, w: &mut W) -> std::io::Result<usize> {
        let bytes = self.as_bytes();
        let n = bytes.len();

        w.write_all(bytes)?;

        Ok(n)
    }
}

impl<T: AsRef<[u8]> + ?Sized> Pattern for T {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

/// const implementation
impl ByteStr {
    pub const fn from_bytes(slice: &[u8]) -> &Self {
        unsafe { &*(slice as *const [u8] as *const ByteStr) }
    }

    pub const fn from_bytes_mut(slice: &mut [u8]) -> &mut Self {
        unsafe { &mut *(slice as *mut [u8] as *mut ByteStr) }
    }

    pub const unsafe fn from_bytes_permanently<'i, 'o>(slice: &'i [u8]) -> &'o Self {
        unsafe { Self::from_raw_parts(slice.as_ptr(), slice.len()) }
    }

    /// ([..mid), [mid..))
    ///
    /// Panics if mid > len
    pub const fn split_at(&self, mid: usize) -> (&Self, &Self) {
        let (left, right) = self.value.split_at(mid);

        (Self::from_bytes(left), Self::from_bytes(right))
    }

    /// ([..mid), [mid..))
    pub const fn split_at_mut(
        &mut self,
        mid: usize,
    ) -> (&mut Self, &mut Self) {
        let (left, right) = self.value.split_at_mut(mid);

        (Self::from_bytes_mut(left), Self::from_bytes_mut(right))
    }

    pub const fn len(&self) -> usize {
        self.value.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub const unsafe fn from_raw_parts<'a>(
        ptr: *const u8,
        len: usize,
    ) -> &'a Self {
        Self::from_bytes(unsafe { std::slice::from_raw_parts(ptr, len) })
    }

    pub const unsafe fn from_raw_parts_mut<'a>(
        ptr: *mut u8,
        len: usize,
    ) -> &'a mut Self {
        Self::from_bytes_mut(unsafe {
            std::slice::from_raw_parts_mut(ptr, len)
        })
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
        Self::from_bytes(bytes.as_ref())
    }

    pub fn new_mut<B: ?Sized + AsMut<[u8]>>(bytes: &mut B) -> &mut Self {
        Self::from_bytes_mut(bytes.as_mut())
    }

    pub fn parse<F: FromByteStr>(&self) -> Result<F, F::Err> {
        F::from_bstr(self)
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

    pub fn decode_as_utf8<'a>(
        &'a self,
    ) -> Result<&'a str, std::str::Utf8Error> {
        std::str::from_utf8(self)
    }

    pub fn eq_ignore_ascii_case(&self, other: &Self) -> bool {
        let self_slice: &[u8] = self.as_ref();
        let other_slice: &[u8] = other.as_ref();

        self_slice.eq_ignore_ascii_case(other_slice)
    }

}

impl Display for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in &self.value {
            pretty_debug_u8(f, *b)?;
        }

        Ok(())
    }
}

impl Debug for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // if f.alternate() {
        //     write!(f, "b\"{self}\" ")
        // }
        // else {
        //     f.debug_struct("ByteStr").field("value", &&self.value).finish()
        // }
        write!(f, "b\"{self}\" ")
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
    fn index_mut(
        &mut self,
        index: RangeToInclusive<usize>,
    ) -> &mut Self::Output {
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
    fn index_mut(
        &mut self,
        index: RangeInclusive<usize>,
    ) -> &mut Self::Output {
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

    ///
    /// Returns a newly allocated vector containing the elements in the range [at, len).
    /// After the call, the original vector will be left containing the elements [0, at)
    /// with its previous capacity unchanged.
    pub fn split_off(&mut self, at: usize) -> Self {
        Self {
            value: self.value.split_off(at),
        }
    }

    pub fn decode_into_utf8(
        self,
    ) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.value)
    }

    pub fn as_bstr(&self) -> &ByteStr {
        self.deref()
    }
}

impl Display for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_bstr())
    }
}

impl Debug for ByteString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.as_bstr())
        }
        else {
            write!(f, "{:?}", self.as_bstr())
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

impl From<&ByteStr> for ByteString {
    fn from(value: &ByteStr) -> Self {
        value.to_vec().into()
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

impl From<Box<[u8]>> for ByteString {
    fn from(value: Box<[u8]>) -> Self {
        Self { value: value.to_vec() }
    }
}

impl Into<Vec<u8>> for ByteString {
    fn into(self) -> Vec<u8> {
        self.value
    }
}

impl Into<Box<[u8]>> for ByteString {
    fn into(self) -> Box<[u8]> {
        self.value.into_boxed_slice()
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

impl<B: ?Sized + AsRef<[u8]>> PartialEq<B> for ByteString {
    fn eq(&self, other: &B) -> bool {
        self.as_ref() == other.as_ref()
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

fn pretty_debug_u8(f: &mut std::fmt::Formatter<'_>, b: u8) -> std::fmt::Result {
    match b {
        0 => write!(f, "\\NUL"),
        1 => write!(f, "\\SOH"),
        2 => write!(f, "\\STX"),
        3 => write!(f, "\\ETX"),
        4 => write!(f, "\\EOT"),
        5 => write!(f, "\\ENQ"),
        6 => write!(f, "\\ACK"),
        7 => write!(f, "\\BEL"),
        8 => write!(f, "\\BS"),
        9 => write!(f, "\\HT"),
        10 => write!(f, "\\LF"),
        11 => write!(f, "\\VT"),
        12 => write!(f, "\\FF"),
        13 => write!(f, "\\CR"),
        14 => write!(f, "\\SO"),
        15 => write!(f, "\\SI"),
        16 => write!(f, "\\DLE"),
        17 => write!(f, "\\DC1"),
        18 => write!(f, "\\DC2"),
        19 => write!(f, "\\DC3"),
        20 => write!(f, "\\DC4"),
        21 => write!(f, "\\NAK"),
        22 => write!(f, "\\SYN"),
        23 => write!(f, "\\ETB"),
        24 => write!(f, "\\CAN"),
        25 => write!(f, "\\EM"),
        26 => write!(f, "\\SUB"),
        27 => write!(f, "\\ESC"),
        28 => write!(f, "\\FS"),
        29 => write!(f, "\\GS"),
        30 => write!(f, "\\RS"),
        31 => write!(f, "\\US"),
        32..=126 => write!(f, "{}", b as char),
        127 => write!(f, "DEL"),
        128.. => write!(f, "\\x{b:X}"),
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

    impl<T: ConsumeByteStr> FromByteStr for T {
        type Err = T::Err;

        fn from_bstr(bytes: &ByteStr) -> Result<Self, Self::Err> {
            let (it, _offset) = T::consume_bstr(bytes)?;
            Ok(it)
        }
    }
}

#[cfg(feature = "cow")]
pub use require_cow::*;

#[cfg(feature = "nom")]
mod support_nom {
    use std::{
        iter::{Copied, Enumerate},
        slice::Iter,
    };

    use nom::{AsBytes, Compare, Input, Offset};

    use super::*;

    // impl Offset for ByteStr {
    //     fn offset(&self, second: &Self) -> usize {
    //         self.value.offset(&second)
    //     }
    // }

    impl<'a> Offset for &'a ByteStr {
        fn offset(&self, second: &Self) -> usize {
            self.value.offset(&second.value)
        }
    }

    impl<'a> Input for &'a ByteStr {
        type Item = u8;
        type Iter = Copied<Iter<'a, u8>>;
        type IterIndices = Enumerate<Self::Iter>;

        fn input_len(&self) -> usize {
            (&self.value).input_len()
        }

        fn take(&self, index: usize) -> Self {
            ByteStr::from_bytes(Input::take(&&self.value, index))
        }

        fn take_from(&self, index: usize) -> Self {
            ByteStr::from_bytes((&self.value).take_from(index))
        }

        fn take_split(&self, index: usize) -> (Self, Self) {
            let (left, right) = (&self.value).take_split(index);

            (ByteStr::from_bytes(left), ByteStr::from_bytes(right))
        }

        fn position<P>(&self, predicate: P) -> Option<usize>
        where
            P: Fn(Self::Item) -> bool,
        {
            (&self.value).position(predicate)
        }

        fn iter_elements(&self) -> Self::Iter {
            (&self.value).iter_elements()
        }

        fn iter_indices(&self) -> Self::IterIndices {
            (&self.value).iter_indices()
        }

        fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
            (&self.value).slice_index(count)
        }
    }

    impl<'a> AsBytes for &'a ByteStr {
        fn as_bytes(&self) -> &[u8] {
            &self.value
        }
    }

    impl<'a, 'b> Compare<&'b str> for &'a ByteStr {
        fn compare(&self, t: &'b str) -> nom::CompareResult {
            (&self.value).compare(t.as_bytes())
        }

        fn compare_no_case(&self, t: &'b str) -> nom::CompareResult {
            (&self.value).compare_no_case(t.as_bytes())
        }
    }
}



#[cfg(test)]
mod tests {

    use core::slice::SlicePattern;

    use super::*;

    #[test]
    fn test_pretty_print_u8() {
        println!("{:#?}", ByteStr::new("a\u{AF}\u{0}"))
    }

    #[test]
    fn test_partial_eq() {
        let a = ByteStr::new(b"abc");
        let b = b"abc";

        assert_eq!(&a, &b);
        assert_eq!(&b, &a.as_slice());
        assert_eq!(b, a.as_slice());

        assert_eq!(ByteStr::new("\r\n"), b"\r\n");
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
