#![feature(unsafe_cell_access)]

use std::{
    borrow::{Borrow, BorrowMut},
    cell::{OnceCell, UnsafeCell},
    fmt::Debug,
    hash::Hash,
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
/// # Note
///
/// derive by value T instead of pointer to T
///
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OwnedPtr<T: ?Sized> {
    value: Box<T>,
}

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

unsafe impl<T> Sync for OnceStatic<T> {}

impl<T> Deref for OnceStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.cell.get().expect("uninit")
    }
}

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    pub fn into_inner(owned: Self) -> T {
        // Deref never allows move operations. Box allows it because it's special,
        // and the * operator on box isn't actually using the Deref trait.
        *owned.value
    }
}

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

unsafe impl<T: Sync> Sync for OwnedPtr<T> {}

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

impl<T: Debug> Debug for OwnedPtr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self, f)
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

impl<T: Debug> Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(self, f)
    }
}

impl<T: PartialEq + ?Sized> PartialEq for Ptr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<T: Eq + ?Sized> Eq for Ptr<T> {}

impl<T: PartialOrd + ?Sized> PartialOrd for Ptr<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.deref().partial_cmp(other.deref())
    }
}

impl<T: Ord + ?Sized> Ord for Ptr<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deref().cmp(other.deref())
    }
}

impl<T: Hash + ?Sized> Hash for Ptr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Functions
