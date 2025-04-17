use std::{
    fmt::{Debug, Display},
    hash::Hash,
    ops::{
        Deref, Index, Range, RangeBounds, RangeFrom, RangeFull,
        RangeInclusive, RangeTo, RangeToInclusive,
    },
    slice::SliceIndex,
    str::Utf8Error,
};


////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait SliceLike<Output: ?Sized = Self>: IndexRangeBounds<Output> {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait IndexRangeBounds<Output: ?Sized> = Index<RangeFrom<usize>, Output = Output>
    + Index<RangeTo<usize>, Output = Output>
    + Index<RangeToInclusive<usize>, Output = Output>
    + Index<Range<usize>, Output = Output>
    + Index<RangeFull, Output = Output>
    + Index<RangeInclusive<usize>, Output = Output>;

////////////////////////////////////////////////////////////////////////////////
//// Structures

pub enum FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    /// `&root[start..end]`
    Borrowed {
        root: &'a B,
        start: usize,
        end: usize,
    },
    Owned(B::Owned),
}

pub struct CowBuf<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    value: FlatCow<'a, B>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T> SliceLike for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl SliceLike for str {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<'a, B> CowBuf<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
    B::Owned: Deref<Target = B>,
{
    pub fn to_cow(self) -> FlatCow<'a, B> {
        self.value
    }

    pub fn start(&mut self, i: usize) {
        match &mut self.value {
            FlatCow::Borrowed { start, end, .. } => {
                assert!(
                    *start == 0,
                    "buf has started at {:?}",
                    start..end
                );
                assert!(i > *end, "start postion overflow {} > {}", i, *end);

                *start = i;
                *end = i;
            }
            FlatCow::Owned(owned) => {
                debug_assert!(owned.is_empty(), "buf has started");
            }
        }
    }
}

impl<'a, T> CowBuf<'a, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    pub fn push(&mut self, b: T) {
        match &mut self.value {
            FlatCow::Borrowed { end, .. } => *end += 1,
            FlatCow::Owned(value) => value.push(b),
        }
    }

    pub fn clone_push(&mut self, b: T) {
        self.value.to_mut().push(b);
    }
}

impl<'a> CowBuf<'a, str> {
    pub fn push(&mut self, b: char) {
        match &mut self.value {
            FlatCow::Borrowed { end, .. } => *end += 1,
            FlatCow::Owned(value) => value.push(b),
        }
    }

    pub fn clone_push(&mut self, b: char) {
        self.value.to_mut().push(b);
    }
}

impl<'a, B> From<&'a B> for CowBuf<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
    B::Owned: Deref<Target = B>,
{
    fn from(value: &'a B) -> Self {
        let cow = FlatCow::Borrowed {
            root: value,
            start: 0,
            end: 0,
        };

        Self { value: cow }
    }
}

impl<'a, T> From<&FlatCow<'a, [T]>> for CowBuf<'a, [T]>
where
    [T]: ToOwned<Owned = Vec<T>>,
{
    fn from(value: &FlatCow<'a, [T]>) -> Self {
        let cow = match value {
            FlatCow::Borrowed { root, start, .. } => FlatCow::Borrowed {
                root: *root,
                start: *start,
                end: *start,
            },
            FlatCow::Owned(..) => FlatCow::Owned(Vec::new()),
        };

        Self { value: cow }
    }
}

impl<'a, B> FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    pub fn own_new(owned: <B as ToOwned>::Owned) -> Self {
        Self::Owned(owned)
    }
}

impl<'a, B> AsRef<B> for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
    B::Owned: Deref<Target = B>,
{
    fn as_ref(&self) -> &B {
        match self {
            Self::Borrowed { root, start, end } => &root[*start..*end],
            Self::Owned(owned) => &owned[..],
        }
    }
}

impl<'a, B> Clone for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
    B::Owned: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed { root, start, end } => Self::Borrowed {
                root,
                start: *start,
                end: *end,
            },
            Self::Owned(owned) => Self::Owned(owned.to_owned()),
        }
    }
}

impl<'a, B> FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
{
    pub fn borrow_new(root: &'a B) -> Self {
        Self::Borrowed {
            root,
            start: 0,
            end: root.len(),
        }
    }

    /// Clone the entire slice if it's not already owned.
    pub fn to_mut(&mut self) -> &mut B::Owned {
        match *self {
            Self::Borrowed { root, start, end } => {
                *self = Self::Owned(root[start..end].to_owned());

                if let Self::Owned(owned) = self {
                    owned
                }
                else {
                    unreachable!()
                }
            }
            Self::Owned(ref mut owned) => owned,
        }
    }

    pub fn into_cow_buf(self) -> CowBuf<'a, B> {
        CowBuf { value: self }
    }

    pub fn into_owned(self) -> B::Owned {
        match self {
            FlatCow::Borrowed { root, start, end } => root[start..end].to_owned(),
            FlatCow::Owned(owned) => owned,
        }
    }
}

impl<'a, T: Clone> FlatCow<'a, [T]> {
    /// ```no_main
    /// Self::Borrowed => Self::Borrowed
    /// Self::Owned => Self::Owned
    /// ``````
    pub fn as_slice_cow<I: RangeBounds<usize> + SliceIndex<[T], Output = [T]>>(
        &self,
        index: I,
    ) -> Self
    {
        match self {
            Self::Borrowed { root, start, end } => {
                let (start, end) = flatcow_union_range(*start..*end, index);

                Self::Borrowed { root, start, end }
            }
            Self::Owned(owned) => Self::Owned(owned[index].to_owned()),
        }
    }
}

impl<'a> FlatCow<'a, str> {
    /// ```no_main
    /// Self::Borrowed => Self::Borrowed
    /// Self::Owned => Self::Owned
    /// ``````
    pub fn as_slice_cow<I: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &self,
        index: I,
    ) -> Self
    {
        match self {
            Self::Borrowed { root, start, end } => {
                let (start, end) = flatcow_union_range(*start..*end, index);

                Self::Borrowed { root, start, end }
            }
            Self::Owned(owned) => Self::Owned(owned[index].to_owned()),
        }
    }
}

impl<'a, B> From<&'a B> for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
    B::Owned: Deref<Target = B>,
{
    fn from(value: &'a B) -> Self {
        Self::borrow_new(value)
    }
}

impl<'a, B> Deref for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike,
    B::Owned: Deref<Target = B>,
{
    type Target = B;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed { root, start, end } => &root[*start..*end],
            Self::Owned(owned) => owned,
        }
    }
}

impl<'a, B> Debug for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike + Debug,
    B::Owned: Deref<Target = B>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self.as_ref())
        }
        else {
            write!(f, "{:?}", self.as_ref())
        }
    }
}

impl<'a, B> Display for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike + Display,
    B::Owned: Deref<Target = B>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a, B, U> PartialEq<U> for FlatCow<'a, B>
where
    U: ?Sized + AsRef<B>,
    B: 'a + ToOwned + ?Sized + SliceLike + Eq,
    B::Owned: Deref<Target = B>,
{
    fn eq(&self, other: &U) -> bool {
        let lf = &self[..];
        let rh = other.as_ref();

        lf == rh
    }
}

impl<'a, B> Eq for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike + Eq,
    B::Owned: Deref<Target = B>,
{
}

impl<'a, B, U> PartialOrd<U> for FlatCow<'a, B>
where
    U: ?Sized + AsRef<B>,
    B: 'a + ToOwned + ?Sized + SliceLike + Ord,
    B::Owned: Deref<Target = B>,
{
    fn partial_cmp(&self, other: &U) -> Option<std::cmp::Ordering> {
        let lf = &self[..];
        let rh = other.as_ref();

        lf.partial_cmp(rh)
    }
}


impl<'a, B> Ord for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike + Ord,
    B::Owned: Deref<Target = B>,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a, B> Hash for FlatCow<'a, B>
where
    B: 'a + ToOwned + ?Sized + SliceLike + Hash,
    B::Owned: Deref<Target = B>,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let it = &self[..];
        it.hash(state);
    }
}

impl<'a> TryFrom<FlatCow<'a, [u8]>> for FlatCow<'a, str> {
    type Error = Utf8Error;

    fn try_from(value: FlatCow<'a, [u8]>) -> Result<Self, Self::Error> {
        Ok(match value {
            FlatCow::Borrowed { root, start, end } => {
                let root = std::str::from_utf8(&root[start..end])?;

                Self::Borrowed {
                    root,
                    start: 0,
                    end: root.len(),
                }
            }
            FlatCow::Owned(owned) => {
                let owned = String::from_utf8(owned)
                    .map_err(|err| err.utf8_error())?;

                FlatCow::Owned(owned)
            }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions


/// offset + limit
fn flatcow_union_range<I: RangeBounds<usize>>(
    r1: Range<usize>,
    r2: I,
) -> (usize, usize) {
    use std::ops::Bound::*;

    let Range {
        start: offset,
        end: limit,
    } = r1;

    let start2 = match r2.start_bound().cloned() {
        Included(start2) => start2,
        Excluded(..) => unreachable!(),
        Unbounded => 0,
    };

    let end2 = match r2.end_bound().cloned() {
        Included(end2) => end2 + 1,
        Excluded(end2) => end2,
        Unbounded => limit - offset,
    };

    let start = (start2 + offset).min(limit);
    let end = (end2 + offset).min(limit);

    (start, end)
}

////////////////////////////////////////////////////////////////////////////////
//// Modules

#[cfg(feature = "bstr")]
mod support_bytestr {
    use core::slice::SlicePattern;

    use super::*;
    use crate::bstr::{ByteStr, ByteString};

    impl SliceLike for ByteStr {
        fn len(&self) -> usize {
            self.as_slice().len()
        }
    }

    impl<'a> CowBuf<'a, ByteStr> {
        pub fn push(&mut self, b: u8) {
            match &mut self.value {
                FlatCow::Borrowed { end, .. } => *end += 1,
                FlatCow::Owned(value) => value.push(b),
            }
        }

        pub fn clone_push(&mut self, b: u8) {
            self.value.to_mut().push(b);
        }
    }

    impl<'a> From<&FlatCow<'a, ByteStr>> for CowBuf<'a, ByteStr> {
        fn from(value: &FlatCow<'a, ByteStr>) -> Self {
            let cow = match value {
                FlatCow::Borrowed { root, start, .. } => FlatCow::Borrowed {
                    root: *root,
                    start: *start,
                    end: *start,
                },
                FlatCow::Owned(..) => FlatCow::Owned(ByteString::new()),
            };

            Self { value: cow }
        }
    }

    impl<'a> FlatCow<'a, ByteStr> {
        /// ```no_main
        /// Self::Borrowed => Self::Borrowed
        /// Self::Owned => Self::Owned
        /// ``````
        pub fn as_slice_cow<I: RangeBounds<usize> + SliceIndex<[u8], Output = [u8]>>(
            &self,
            index: I,
        ) -> Self
        {
            match self {
                Self::Borrowed { root, start, end } => {
                    let (start, end) = flatcow_union_range(*start..*end, index);

                    Self::Borrowed { root, start, end }
                }
                Self::Owned(owned) => Self::Owned(owned[index].to_owned().into()),
            }
        }
    }

    impl<'a> From<FlatCow<'a, ByteStr>> for FlatCow<'a, [u8]> {
        fn from(value: FlatCow<'a, ByteStr>) -> Self {
            use FlatCow::*;

            match value {
                Borrowed { root, start, end } => Borrowed {
                    root: root.into(),
                    start,
                    end,
                },
                Owned(owned) => Owned(owned.into()),
            }
        }
    }

    impl<'a> TryFrom<FlatCow<'a, ByteStr>> for FlatCow<'a, str> {
        type Error = Utf8Error;

        fn try_from(value: FlatCow<'a, ByteStr>) -> Result<Self, Self::Error> {
            let oth: FlatCow<[u8]> = value.into();

            oth.try_into()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_cow() {
        let v = ('a'..'j').into_iter().collect::<String>();
        let cow = FlatCow::<str>::borrow_new(&v[..]);

        println!("{cow}");

        assert_eq!(&cow[1..8], "bcdefgh");

        let cow1 = cow.as_slice_cow(1..8);

        assert!(matches!(cow1, FlatCow::Borrowed { .. }));
        assert_eq!(&cow1[3..6], "efg");

        /* test Owned */

        let cow = FlatCow::<[usize]>::own_new(
            (0..10).into_iter().collect::<Vec<_>>(),
        );

        assert_eq!(&cow[1..8], &[1, 2, 3, 4, 5, 6, 7]);

        let cow1 = cow.as_slice_cow(1..8);

        assert_eq!(&cow1[..], &[1, 2, 3, 4, 5, 6, 7]);

        /* test Borrowed */

        let v = (0..10).into_iter().collect::<Vec<_>>();
        let cow = FlatCow::<[usize]>::borrow_new(&v[..]);

        assert_eq!(&cow[1..8], &[1, 2, 3, 4, 5, 6, 7]);

        let cow1 = cow.as_slice_cow(1..8);

        assert!(matches!(cow1, FlatCow::Borrowed { .. }));
        assert_eq!(&cow1[3..6], &[4, 5, 6]);
    }
}
