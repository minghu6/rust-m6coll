#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(slice_as_array)]

pub use proc_macros::*;

////////////////////////////////////////////////////////////////////////////////

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

// pub trait ToBits: Sized {
//     type Int where size_of::<Self> == size_of::<Self::Int>();

//     fn to_bits(self) -> Self::Int;
// }

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl FromBytes for u8 {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        bytes[0]
    }

    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        bytes[0]
    }
}

impl ToBytes for u8 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        [self]
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        [self]
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

impl ToBytes for u16 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_le_bytes()
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_be_bytes()
    }
}

impl FromBytes for u32 {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_le_bytes(bytes)
    }

    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl ToBytes for u32 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_le_bytes()
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_be_bytes()
    }
}

impl FromBytes for u64 {
    fn from_le_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_le_bytes(bytes)
    }

    fn from_be_bytes(bytes: [u8; size_of::<Self>()]) -> Self {
        Self::from_be_bytes(bytes)
    }
}

impl ToBytes for u64 {
    fn to_le_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_le_bytes()
    }

    fn to_be_bytes(self) -> [u8; size_of::<Self>()] {
        self.to_be_bytes()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_array_copy() {
        let mut arr = [1, 2, size_of::<u8>()
        ];

        arr[2..3].copy_from_slice(&[4]);
    }
}
