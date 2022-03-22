
pub(crate) mod entry;

use std::mem::size_of;

pub use m6arr::*;
pub use entry::Entry;


////////////////////////////////////////////////////////////////////////////////
//// Common Traits


pub trait ToLeBytes {
    fn to_le_bytes(&self) -> Array<u8>;
}



////////////////////////////////////////////////////////////////////////////////
//// Impl

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
    fn to_le_bytes(&self) -> Array<u8> {
        let unit = size_of::<T>();
        let cap = unit * self.len();
        let mut arr = Array::new(cap);

        for i in 0..self.len() {
            arr[i*unit..(i+1)*unit].copy_from_slice(&self[i].to_le_bytes())
        }

        arr
    }
}
