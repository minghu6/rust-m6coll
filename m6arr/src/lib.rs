#![allow(path_statements)]

use core::slice;
use std::{
    alloc::{alloc_zeroed, Layout},
    fmt,
    ops::{Deref, DerefMut, Index, IndexMut},
    ptr::{null_mut, self},
    slice::SliceIndex, intrinsics::copy_nonoverlapping,
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

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn layout(cap: usize) -> Layout {
        Layout::array::<T>(cap).unwrap()
    }

    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &T> + 'a> {
        let mut i = 0;

        Box::new(std::iter::from_fn(move || {
            if i == self.len {
                return None;
            }

            let res = Some(&self[i]);
            i += 1;

            res
        }))
    }

    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn copy_from_slice(src: &[T]) -> Self where T: Copy {
        let mut arr = Array::new(src.len());
        arr[..].copy_from_slice(src);

        arr
    }


    pub fn clone_from_slice(src: &[T]) -> Self where T: Clone {
        let mut arr = Array::new(src.len());
        arr[..].clone_from_slice(src);

        arr
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Standard Traits Implement

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {
            // if self.len > 0 {
            //     dealloc(self.ptr as *mut u8, Self::layout(self.len));
            // }

            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.as_mut_ptr(), self.len))
        }
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

    // test from_raw_mut
    fn f() -> Array<i32> {
        let a = [10, 2, 4, 1];

        Array::copy_from_slice(&a[..])
    }

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

        println!("{:?}", f())
    }
}

