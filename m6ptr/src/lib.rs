use std::{
    borrow::{Borrow, BorrowMut},
    hash::Hash,
    ops::{Deref, DerefMut, Range, RangeBounds},
    ptr::NonNull,
    slice::SliceIndex,
    str::Utf8Error,
    sync::Arc,
};

////////////////////////////////////////////////////////////////////////////////
//// Structures

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ptr<T: ?Sized> {
    value: NonNull<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OwnedPtr<T: ?Sized> {
    value: NonNull<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[repr(transparent)]
pub struct ArcPtr<T: ?Sized> {
    value: Arc<OwnedPtr<T>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct RoPtr<T: ?Sized> {
    value: Ptr<T>,
}

#[derive(Debug)]
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

impl<'a, T> CowBuf<'a, [T]>
where
    T: Clone,
{
    pub fn to_cow(self) -> FlatCow<'a, [T]> {
        self.value
    }

    pub fn start(&mut self, i: usize) {
        match &mut self.value {
            FlatCow::Borrowed { start, end, .. } => {
                assert!(
                    *start != 0 || *end != 0,
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

impl<'a, T: Clone> From<&'a [T]> for CowBuf<'a, [T]> {
    fn from(value: &'a [T]) -> Self {
        let cow = FlatCow::Borrowed {
            root: value,
            start: 0,
            end: 0,
        };

        Self { value: cow }
    }
}

impl<'a, T: Clone> From<&FlatCow<'a, [T]>> for CowBuf<'a, [T]> {
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
    B::Owned: Clone,
{
    pub fn own_new(owned: <B as ToOwned>::Owned) -> Self {
        Self::Owned(owned)
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

impl<'a, T: Clone> FlatCow<'a, [T]> {
    pub fn borrow_new(root: &'a [T]) -> Self {
        Self::Borrowed {
            root,
            start: 0,
            end: root.len(),
        }
    }

    /// Clone the entire slice if it's not already owned.
    pub fn to_mut(&mut self) -> &mut <[T] as ToOwned>::Owned {
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

    /// ```no_main
    /// Self::Borrowed => Self::Borrowed
    /// Self::Owned => Self::Owned
    /// ``````
    pub fn as_slice_cow<
        I: RangeBounds<usize> + SliceIndex<[T], Output = [T]>,
    >(
        &self,
        index: I,
    ) -> Self {
        match self {
            Self::Borrowed { root, start, end } => {
                let (start, end) = flatcow_union_range(*start..*end, index);

                Self::Borrowed { root, start, end }
            }
            Self::Owned(owned) => Self::Owned(owned[index].to_owned()),
        }
    }

    pub fn to_cow_buf(self) -> CowBuf<'a, [T]> {
        CowBuf { value: self }
    }
}

impl<'a, T: Clone> Deref for FlatCow<'a, [T]> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed { root, start, end } => &root[*start..*end],
            Self::Owned(owned) => owned,
        }
    }
}

impl<'a, T: Clone + Eq> PartialEq for FlatCow<'a, [T]> {
    fn eq(&self, other: &Self) -> bool {
        let lf = &self[..];
        let rh = &other[..];

        lf == rh
    }
}

impl<'a, T: Clone + Eq> Eq for FlatCow<'a, [T]> {}

impl<'a, T: Clone + Ord> PartialOrd for FlatCow<'a, [T]> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let lf = &self[..];
        let rh = &other[..];

        lf.partial_cmp(rh)
    }
}

impl<'a, T: Clone + Ord> Ord for FlatCow<'a, [T]> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a, T: Clone + Hash> Hash for FlatCow<'a, [T]> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let it = &self[..];
        it.hash(state);
    }
}

impl<'a> FlatCow<'a, str> {
    pub fn borrow_new(root: &'a str) -> Self {
        Self::Borrowed {
            root,
            start: 0,
            end: root.len(),
        }
    }

    pub fn to_mut(&mut self) -> &mut <str as ToOwned>::Owned {
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

    /// ```no_main
    /// Self::Borrowed => Self::Borrowed
    /// Self::Owned => Self::Owned
    /// ``````
    pub fn as_slice_cow<
        I: RangeBounds<usize> + SliceIndex<str, Output = str>,
    >(
        &self,
        index: I,
    ) -> Self {
        match self {
            Self::Borrowed { root, start, end } => {
                let (start, end) = flatcow_union_range(*start..*end, index);

                Self::Borrowed { root, start, end }
            }
            Self::Owned(owned) => Self::Owned(owned[index].to_owned()),
        }
    }
}

impl<'a> Deref for FlatCow<'a, str> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed {
                root, start, end, ..
            } => &root[*start..*end],
            Self::Owned(owned) => owned,
        }
    }
}

impl<'a> PartialEq for FlatCow<'a, str> {
    fn eq(&self, other: &Self) -> bool {
        let lf = &self[..];
        let rh = &other[..];

        lf == rh
    }
}

impl<'a> Eq for FlatCow<'a, str> {}

impl<'a> PartialOrd for FlatCow<'a, str> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let lf = &self[..];
        let rh = &other[..];

        lf.partial_cmp(rh)
    }
}

impl<'a> Ord for FlatCow<'a, str> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<'a> Hash for FlatCow<'a, str> {
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

impl<T: ?Sized> RoPtr<T> {
    pub fn as_ref(&self) -> &T {
        self.value.as_ref()
    }
}

impl<T: ?Sized> Deref for RoPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}

impl<T> ArcPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(OwnedPtr::new(value)),
        }
    }
}

impl<T: ?Sized> ArcPtr<T> {
    pub fn from_box(value: Box<T>) -> Self {
        Self {
            value: Arc::new(OwnedPtr::from_box(value)),
        }
    }

    pub fn as_ref(&self) -> &T {
        self.value.as_ref()
    }

    pub fn ptr(&self) -> RoPtr<T> {
        RoPtr {
            value: self.value.ptr(),
        }
    }
}

impl<T: ?Sized> Deref for ArcPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref()
    }
}

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: NonNull::new(Box::into_raw(Box::new(value))).unwrap(),
        }
    }
}

impl<T: ?Sized> OwnedPtr<T> {
    pub fn from_box(value: Box<T>) -> Self {
        Self {
            value: NonNull::new(Box::into_raw(value)).unwrap(),
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { &*self.value.as_ptr() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.value.as_ptr() }
    }

    pub fn ptr(&self) -> Ptr<T> {
        Ptr { value: self.value }
    }
}

impl<T: ?Sized> Drop for OwnedPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.value.as_ptr());
        }
    }
}

impl<T: ?Sized> Deref for OwnedPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> DerefMut for OwnedPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized> Borrow<T> for OwnedPtr<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> BorrowMut<T> for OwnedPtr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

impl<T: ?Sized> Ptr<T> {
    pub fn as_ref(&self) -> &T {
        unsafe { &*self.value.as_ptr() }
    }

    pub fn as_mut(&self) -> &mut T {
        unsafe { &mut *self.value.as_ptr() }
    }

    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        std::ptr::eq(this.value.as_ptr(), other.value.as_ptr())
    }
}

impl<T: ?Sized> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T: ?Sized> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T: ?Sized> Borrow<T> for Ptr<T> {
    fn borrow(&self) -> &T {
        self.as_ref()
    }
}

impl<T: ?Sized> BorrowMut<T> for Ptr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

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
        _ => unimplemented!(),
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



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_cow() {
        let v = ('a'..'j').into_iter().collect::<String>();
        let cow = FlatCow::<str>::borrow_new(&v[..]);

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
