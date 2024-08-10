#![feature(allocator_api)]
#![feature(trusted_random_access)]
#![feature(iter_next_chunk)]
#![feature(iter_advance_by)]
#![feature(exact_size_is_empty)]
#![feature(dropck_eyepatch)]
#![feature(inplace_iteration)]
#![feature(trusted_len)]
#![feature(min_specialization)]
#![feature(array_into_iter_constructors)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(strict_provenance)]
#![feature(ptr_sub_ptr)]
#![allow(path_statements)]

pub mod ordered_arr;

use std::{
    alloc::{alloc_zeroed, Layout},
    fmt,
    intrinsics::copy_nonoverlapping,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::{self, null_mut},
    slice::{self, SliceIndex},
    vec::IntoIter,
};


////////////////////////////////////////////////////////////////////////////////
//// Structure

#[repr(C)]
pub struct Array<T> {
    len: usize, // and capacity
    ptr: *mut T,
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

/// Heap Array
impl<T> Array<T> {
    ///////////////////////////////////////
    //// static method

    pub fn empty() -> Self {
        Self::new(0)
    }

    pub fn new(cap: usize) -> Self {
        unsafe {
            let len = cap;

            let ptr = if cap == 0 {
                null_mut()
            } else {
                alloc_zeroed(Self::layout(cap)) as *mut T
            };

            Self { len, ptr }
        }
    }

    pub fn new_with(init: T, cap: usize) -> Self
    where
        T: Copy,
    {
        unsafe {
            let it = Self::new(cap);

            for i in 0..cap {
                (*it.ptr.add(i)) = init;
            }

            it
        }
    }

    pub fn new_with_clone(init: T, cap: usize) -> Self
    where
        T: Clone,
    {
        unsafe {
            let it = Self::new(cap);

            for i in 0..cap {
                (*it.ptr.add(i)) = init.clone();
            }

            it
        }
    }

    pub fn merge(lf: &Self, rh: &Self) -> Self {
        let arr = Array::new(lf.len() + rh.len());

        unsafe {
            copy_nonoverlapping(lf.ptr, arr.ptr, lf.len());
            copy_nonoverlapping(rh.ptr, arr.ptr.add(lf.len()), rh.len());
        }

        arr
    }


    ///////////////////////////////////////
    //// dynamic method

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a raw pointer to the vector’s buffer,
    /// or a dangling raw pointer valid for zero sized reads
    /// if the vector didn’t allocate.
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn layout(cap: usize) -> Layout {
        Layout::array::<T>(cap).unwrap()
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T> {
        let mut i = 0;

        std::iter::from_fn(move || {
            if i == self.len {
                return None;
            }

            let res = Some(&self[i]);
            i += 1;

            res
        })
    }

    /// realloc momory, WARNING: it would invalid the old ptr
    pub fn resize(&mut self, new_cap: usize)
    where
        T: Default,
    {
        unsafe {
            let new_ptr = if new_cap == 0 {
                null_mut()
            } else {
                alloc_zeroed(Self::layout(new_cap)) as *mut T
            };

            let ptr = self.ptr as *mut T;
            let cap = self.len;
            let len = self.len;

            let into_iter = Vec::from_raw_parts(ptr, len, cap).into_iter();

            for (i, v) in into_iter.enumerate() {
                if i >= self.len {
                    break;
                }

                *new_ptr.add(i) = v;
            }

            for i in len..new_cap {
                *new_ptr.add(i) = T::default();
            }

            self.len = new_cap;
            self.ptr = new_ptr;
        }
    }

    fn drop(ptr: *mut T, cap: usize) {
        unsafe { ptr::drop_in_place(ptr::slice_from_raw_parts_mut(ptr, cap)) }
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Standard Traits Implement

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        Self::drop(self.as_mut_ptr(), self.len)
    }
}


impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T, I: SliceIndex<[T]>> Index<I> for Array<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, I: SliceIndex<[T]>> IndexMut<I> for Array<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<T: Clone> Clone for Array<T> {
    fn clone(&self) -> Self {
        let mut cloned = Self::new(self.len);

        cloned[..].clone_from_slice(&self[..]);

        cloned
    }
}

impl<T: Clone> From<&[T]> for Array<T> {
    fn from(src: &[T]) -> Self {
        let mut arr = Array::new(src.len());
        arr[..].clone_from_slice(src);

        arr
    }
}


/// Impl copy from [std::vec::IntoIter]
/// (https://doc.rust-lang.org/src/alloc/vec/mod.rs.html#2654)
impl<T> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        unsafe {
            Vec::from_raw_parts(self.ptr, self.len, self.len).into_iter()
        }
    }
}


impl<T> Default for Array<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T: fmt::Debug> fmt::Debug for Array<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Macros

#[macro_export]
macro_rules! array {
    ( $init:expr; $cap:expr ) => {
        {
            let init = $init;
            let cap = $cap;

            Array::new_with(init, cap)
        }
    };
    ($($item:expr),*) => {
        {
            #[allow(unused_mut)]
            let mut cnt = 0;
            $(
                cnt += 1;

                let _ = $item;
            )*

            #[allow(unused_mut)]
            let mut arr = Array::new(cnt);

            let mut _i = 0;
            $(
                arr[_i] = $item;
                _i += 1;
            )*

            arr
        }
    };

}

#[cfg(test)]
mod tests {
    use super::Array;
    use crate::*;


    #[test]
    fn test_arr() {
        let mut arr = Array::<usize>::new(10);

        arr[2] = 15;
        arr[4] = 20;
        println!("{}", arr[2]);
        println!("{}", arr[1]);

        let arr = [0; 0];

        assert!(arr.is_empty());

        let _arr2 = array![0; 3];
        let arr2 = array!['a', 'b', 'd'];

        for e in arr2.iter() {
            println!("{}", e);
        }

        // test as_ptr/len/from_ptr
        let _ptr = arr2.as_ptr();

        let arr2 = array![1, 2, 3];

        let slice0 = &arr2[..];

        println!("{:?}", slice0);

        println!("{:?}", arr2);

        let mut arr0 = array!['a', 'c'];
        let arr1 = array!['d', 'e'];

        arr0[..].copy_from_slice(&arr1[..]);

        assert_eq!(arr0[..], arr1[..]);

        /* test into_iter */
        let arr = array![0, 1, 2, 3];

        for (i, v) in arr.into_iter().enumerate() {
            assert_eq!(i, v);
            println!("{i} == {v}",);
        }

        let mut arr = array![2, 3, 4];
        arr.resize(4);

        println!("arr: {arr:?}");

        (arr[0], arr[2]) = (arr[2], arr[0]);


        println!("after swap arr: {arr:?}");
    }
}
