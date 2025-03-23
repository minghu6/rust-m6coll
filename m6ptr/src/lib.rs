use std::{
    borrow::{Borrow, BorrowMut}, cell::OnceCell, ops::{Deref, DerefMut}, ptr::NonNull, sync::Arc
};


////////////////////////////////////////////////////////////////////////////////
//// Structures

/// Owned pointer =derive=> Ptr
///
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct OwnedPtr<T: ?Sized> {
    value: NonNull<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Ptr<T: ?Sized> {
    value: NonNull<T>,
}

/// Arc pointer =derive=> RoPtr
///
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

/// Init (once) on main thread, shared bettwen threads
#[derive(Debug)]
#[repr(transparent)]
pub struct OnceStatic<T> {
    cell: OnceCell<T>
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

unsafe impl<T> Sync for OnceStatic<T> {}

impl<T> OnceStatic<T> {
    pub const fn new() -> Self {
        Self { cell: OnceCell::new() }
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

impl <T> Deref for OnceStatic<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.cell.get().expect("uninit")
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

////////////////////////////////////////////////////////////////////////////////
//// Functions


#[cfg(test)]
mod tests {

}
