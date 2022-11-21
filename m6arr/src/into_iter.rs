//! Simplified from [std::vec::IntoIter](https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#2654)

use std::alloc::{Allocator, Global};
use core::array;
use core::fmt;
use core::intrinsics::arith_offset;
use core::iter::{
    FusedIterator, InPlaceIterable, SourceIter, TrustedLen, TrustedRandomAccessNoCoerce,
};
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr;
use core::slice;



pub struct IntoIter<
    T,
    A: Allocator = Global,
> {
    pub(super) phantom: PhantomData<T>,
    pub(super) cap: usize,
    // the drop impl reconstructs a RawVec from buf, cap and alloc
    // to avoid dropping the allocator twice we need to wrap it into ManuallyDrop
    pub(super) alloc: ManuallyDrop<A>,
    pub(super) ptr: *const T,
    pub(super) end: *const T,
}

impl<T: fmt::Debug, A: Allocator> fmt::Debug for IntoIter<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<T, A: Allocator> IntoIter<T, A> {
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len()) }
    }


    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { &mut *self.as_raw_mut_slice() }
    }

    /// Returns a reference to the underlying allocator.
    #[inline]
    pub fn allocator(&self) -> &A {
        &self.alloc
    }

    fn as_raw_mut_slice(&mut self) -> *mut [T] {
        ptr::slice_from_raw_parts_mut(self.ptr as *mut T, self.len())
    }

    /// Forgets to Drop the remaining elements while still allowing the backing allocation to be freed.
    pub(crate) fn forget_remaining_elements(&mut self) {
        self.ptr = self.end;
    }
}

impl<T, A: Allocator> AsRef<[T]> for IntoIter<T, A> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

unsafe impl<T: Send, A: Allocator + Send> Send for IntoIter<T, A> {}
unsafe impl<T: Sync, A: Allocator + Sync> Sync for IntoIter<T, A> {}

impl<T, A: Allocator> Iterator for IntoIter<T, A> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.ptr as *const _ == self.end {
            None
        } else if mem::size_of::<T>() == 0 {
            // purposefully don't use 'ptr.offset' because for
            // vectors with 0-size elements this would return the
            // same pointer.
            self.ptr = unsafe { arith_offset(self.ptr as *const i8, 1) as *mut T };

            // Make up a value of this ZST.
            Some(unsafe { mem::zeroed() })
        } else {
            let old = self.ptr;
            self.ptr = unsafe { self.ptr.offset(1) };

            Some(unsafe { ptr::read(old) })
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if mem::size_of::<T>() == 0 {
            self.end.addr().wrapping_sub(self.ptr.addr())
        } else {
            unsafe { self.end.sub_ptr(self.ptr) }
        };
        (exact, Some(exact))
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        let step_size = self.len().min(n);
        let to_drop = ptr::slice_from_raw_parts_mut(self.ptr as *mut T, step_size);
        if mem::size_of::<T>() == 0 {
            // SAFETY: due to unchecked casts of unsigned amounts to signed offsets the wraparound
            // effectively results in unsigned pointers representing positions 0..usize::MAX,
            // which is valid for ZSTs.
            self.ptr = unsafe { arith_offset(self.ptr as *const i8, step_size as isize) as *mut T }
        } else {
            // SAFETY: the min() above ensures that step_size is in bounds
            self.ptr = unsafe { self.ptr.add(step_size) };
        }
        // SAFETY: the min() above ensures that step_size is in bounds
        unsafe {
            ptr::drop_in_place(to_drop);
        }
        if step_size < n {
            return Err(step_size);
        }
        Ok(())
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }

    #[inline]
    fn next_chunk<const N: usize>(&mut self) -> Result<[T; N], core::array::IntoIter<T, N>> {
        let mut raw_ary = MaybeUninit::uninit_array();

        let len = self.len();

        if mem::size_of::<T>() == 0 {
            if len < N {
                self.forget_remaining_elements();
                // Safety: ZSTs can be conjured ex nihilo, only the amount has to be correct
                return Err(unsafe { array::IntoIter::new_unchecked(raw_ary, 0..len) });
            }

            self.ptr = unsafe { arith_offset(self.ptr as *const i8, N as isize) as *mut T };
            // Safety: ditto
            return Ok(unsafe { MaybeUninit::array_assume_init(raw_ary) });
        }

        if len < N {
            // Safety: `len` indicates that this many elements are available and we just checked that
            // it fits into the array.
            unsafe {
                ptr::copy_nonoverlapping(self.ptr, raw_ary.as_mut_ptr() as *mut T, len);
                self.forget_remaining_elements();
                return Err(array::IntoIter::new_unchecked(raw_ary, 0..len));
            }
        }

        // Safety: `len` is larger than the array size. Copy a fixed amount here to fully initialize
        // the array.
        return unsafe {
            ptr::copy_nonoverlapping(self.ptr, raw_ary.as_mut_ptr() as *mut T, N);
            self.ptr = self.ptr.add(N);
            Ok(MaybeUninit::array_assume_init(raw_ary))
        };
    }

    unsafe fn __iterator_get_unchecked(&mut self, i: usize) -> Self::Item
    where
        Self: TrustedRandomAccessNoCoerce,
    {
        // SAFETY: the caller must guarantee that `i` is in bounds of the
        // `Vec<T>`, so `i` cannot overflow an `isize`, and the `self.ptr.add(i)`
        // is guaranteed to pointer to an element of the `Vec<T>` and
        // thus guaranteed to be valid to dereference.
        //
        // Also note the implementation of `Self: TrustedRandomAccess` requires
        // that `T: Copy` so reading elements from the buffer doesn't invalidate
        // them for `Drop`.
        if mem::size_of::<T>() == 0 { mem::zeroed() } else { ptr::read(self.ptr.add(i)) }

    }
}

impl<T, A: Allocator> DoubleEndedIterator for IntoIter<T, A> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        if self.end == self.ptr {
            None
        } else if mem::size_of::<T>() == 0 {
            // See above for why 'ptr.offset' isn't used
            self.end = unsafe { arith_offset(self.end as *const i8, -1) as *mut T };

            // Make up a value of this ZST.
            Some(unsafe { mem::zeroed() })
        } else {
            self.end = unsafe { self.end.offset(-1) };

            Some(unsafe { ptr::read(self.end) })
        }
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), usize> {
        let step_size = self.len().min(n);
        if mem::size_of::<T>() == 0 {
            // SAFETY: same as for advance_by()
            self.end = unsafe {
                arith_offset(self.end as *const i8, step_size.wrapping_neg() as isize) as *mut T
            }
        } else {
            // SAFETY: same as for advance_by()
            self.end = unsafe { self.end.offset(step_size.wrapping_neg() as isize) };
        }
        let to_drop = ptr::slice_from_raw_parts_mut(self.end as *mut T, step_size);
        // SAFETY: same as for advance_by()
        unsafe {
            ptr::drop_in_place(to_drop);
        }
        if step_size < n {
            return Err(step_size);
        }
        Ok(())
    }
}

impl<T, A: Allocator> ExactSizeIterator for IntoIter<T, A> {
    fn is_empty(&self) -> bool {
        self.ptr == self.end
    }
}

impl<T, A: Allocator> FusedIterator for IntoIter<T, A> {}

unsafe impl<T, A: Allocator> TrustedLen for IntoIter<T, A> {}



// TrustedRandomAccess (without NoCoerce) must not be implemented because



unsafe impl<#[may_dangle] T, A: Allocator> Drop for IntoIter<T, A> {
    fn drop(&mut self) {
        // destroy the remaining elements
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(
                self.ptr as *mut T,
                self.cap,
            ))
        }

    }
}

// In addition to the SAFETY invariants of the following three unsafe traits
// also refer to the vec::in_place_collect module documentation to get an overview
#[doc(hidden)]
unsafe impl<T, A: Allocator> InPlaceIterable for IntoIter<T, A> {}

#[doc(hidden)]
unsafe impl<T, A: Allocator> SourceIter for IntoIter<T, A> {
    type Source = Self;

    #[inline]
    unsafe fn as_inner(&mut self) -> &mut Self::Source {
        self
    }
}
