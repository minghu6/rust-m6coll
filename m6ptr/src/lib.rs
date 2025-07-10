#![feature(box_as_ptr)]
#![feature(slice_ptr_get)]
#![feature(box_vec_non_null)]
#![feature(unsafe_cell_access)]

use std::{
    borrow::{Borrow, BorrowMut},
    cell::{OnceCell, UnsafeCell},
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::{LazyLock, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard},
};


////////////////////////////////////////////////////////////////////////////////
//// Structures

////////////////////////////////////////
//// Intrusive Structure Model

///
///
/// Owned pointer =derive=> Ptr
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OwnedPtr<T: ?Sized> {
    value: Box<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ptr<T: ?Sized> {
    value: NonNull<T>,
}

////////////////////////////////////////
//// Self-referential Structure Model


////////////////////////////////////////
//// Static Variables

/// Init (once) on main thread, shared bettwen threads
///
/// should be defined static variable not const
#[derive(Debug)]
#[repr(transparent)]
pub struct OnceStatic<T> {
    cell: OnceCell<T>,
}

#[repr(transparent)]
pub struct LazyStatic<T, F = fn() -> T> {
    value: UnsafeCell<RwLock<LazyLock<T, F>>>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl<T, F: FnOnce() -> T> LazyStatic<T, F> {
    pub const fn new(f: F) -> Self {
        Self {
            value: UnsafeCell::new(RwLock::new(LazyLock::new(f))),
        }
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, LazyLock<T, F>>> {
        unsafe { self.value.as_ref_unchecked().read() }
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, LazyLock<T, F>>> {
        unsafe { self.value.as_ref_unchecked().write() }
    }
}

unsafe impl<T, F> Sync for LazyStatic<T, F> {}

impl<T> OnceStatic<T> {
    pub const fn new() -> Self {
        Self {
            cell: OnceCell::new(),
        }
    }

    ///
    /// # Errors
    ///
    /// This method returns `Ok(())` if the cell was empty and `Err(value)` if it was full.
    ///
    pub fn init(&self, value: T) -> Result<(), T> {
        self.cell.set(value)
    }
}

impl<T> Deref for OnceStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.cell.get().expect("uninit")
    }
}

unsafe impl<T> Sync for OnceStatic<T> {}

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }
}

unsafe impl<T: Sync> Sync for OwnedPtr<T> {}

impl<T: ?Sized> OwnedPtr<T> {
    pub fn from_box(value: Box<T>) -> Self {
        Self { value }
    }

    pub fn ptr(&self) -> Ptr<T> {
        Ptr {
            value: unsafe {
                NonNull::new_unchecked(Box::as_ptr(&self.value) as _)
            },
        }
    }
}

// impl<T: ?Sized> Drop for OwnedPtr<T> {
//     fn drop(&mut self) {
//         unsafe {
//             let _ = Box::from_raw(self.value.as_ptr());
//         }
//     }
// }

impl<T: ?Sized> Deref for OwnedPtr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: ?Sized> DerefMut for OwnedPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut()
    }
}

impl<T: ?Sized> Borrow<T> for OwnedPtr<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> BorrowMut<T> for OwnedPtr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T> Ptr<T> {
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        std::ptr::eq(this.value.as_ptr(), other.value.as_ptr())
    }
}

unsafe impl<T: Sync + ?Sized> Sync for Ptr<T> {}

unsafe impl<T: ?Sized> Send for Ptr<T> {}

impl<T: ?Sized> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

impl<T: ?Sized> Copy for Ptr<T> {}

impl<T: ?Sized> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.value.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.value.as_mut() }
    }
}

impl<T: ?Sized> Borrow<T> for Ptr<T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> BorrowMut<T> for Ptr<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions
