use std::mem::size_of;

pub use m6arr::*;
pub use m6entry::Entry;
pub use m6bitmap::*;
pub use m6stack::*;

////////////////////////////////////////////////////////////////////////////////
//// Common Traits

pub trait ToLeBytes {
    fn to_le_bytes(&self) -> Array<u8>;
}


////////////////////////////////////////////////////////////////////////////////
//// Impl Traits

///////////////////////////////////////
//// Impl ToLeBytes

impl ToLeBytes for u32 {
    fn to_le_bytes(&self) -> Array<u8> {
        Array::copy_from_slice(&(*self as u32).to_le_bytes()[..])
    }
}

impl ToLeBytes for u64 {
    fn to_le_bytes(&self) -> Array<u8> {
        Array::copy_from_slice(&(*self as u32).to_le_bytes()[..])
    }
}

impl ToLeBytes for usize {
    fn to_le_bytes(&self) -> Array<u8> {
        Array::copy_from_slice(&(*self as u32).to_le_bytes()[..])
    }
}

impl<T: ToLeBytes> ToLeBytes for Array<T> {
    /// WARNING: untable across compilations for non-primitive value
    fn to_le_bytes(&self) -> Array<u8> {
        let unit = size_of::<T>();
        let cap = unit * self.len();
        let mut arr = Array::new(cap);

        for i in 0..self.len() {
            arr[i * unit..(i + 1) * unit].copy_from_slice(&self[i].to_le_bytes())
        }

        arr
    }
}

impl<K: ToLeBytes, V: ToLeBytes> ToLeBytes for Entry<K, V> {
    /// WARNING: untable across compilations for non-primitive value
    fn to_le_bytes(&self) -> Array<u8> {
        let size = size_of::<K>() + size_of::<V>();
        let mut arr = Array::new(size);

        arr[0..size_of::<K>()].copy_from_slice(&self.0.to_le_bytes());
        arr[size_of::<K>()..].copy_from_slice(&self.1.to_le_bytes());

        arr
    }
}

