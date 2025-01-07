#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub use proc_macros::*;

////////////////////////////////////////////////////////////////////////////////
//// Traits

pub trait ToBytes: Sized {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()];
    fn to_be_bytes(self) -> [u8; size_of::<Self>()];
    fn to_ne_bytes(self) -> [u8; size_of::<Self>()] {
        if cfg!(target_endian="little") {
            self.to_le_bytes()
        }
        else {
            self.to_be_bytes()
        }
    }
}

pub trait FromBytes: Sized {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self;
    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self;
    fn from_ne_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        if cfg!(target_endian="little") {
            Self::from_le_bytes(bytes)
        }
        else {
            Self::from_be_bytes(bytes)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl ToBytes for u8 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        [self]
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        [self]
    }
}

impl FromBytes for u8 {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        bytes[0]
    }

    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        bytes[0]
    }
}

impl ToBytes for u16 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_le_bytes()
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_be_bytes()
    }
}

impl FromBytes for u16 {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_le_bytes(bytes)
    }

    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_be_bytes(bytes)
    }
}




#[cfg(test)]
mod tests {

}

