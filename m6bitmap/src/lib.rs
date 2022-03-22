use std::{
    alloc::{alloc_zeroed, Layout},
    ptr::{drop_in_place, null_mut, slice_from_raw_parts_mut},
};



#[repr(C)]
pub struct BitMap {
    cap_byte: usize, // capacity by bytes
    ptr: *mut u8,
}

const fn bit_mask(i: u8) -> u8 {
    match i {
        0 => 0b_0000_0001,
        1 => 0b_0000_0010,
        2 => 0b_0000_0100,
        3 => 0b_0000_1000,
        4 => 0b_0001_0000,
        5 => 0b_0010_0000,
        6 => 0b_0100_0000,
        7 => 0b_1000_0000,
        _ => unreachable!(),
    }
}

pub const fn bit_set(val: u8, i: u8) -> u8 {
    val | bit_mask(i)
}

pub const fn bit_get(val: u8, i: u8) -> bool {
    val & bit_mask(i) != 0
}


////////////////////////////////////////////////////////////////////////////////
//// Implement

impl BitMap {
    /// Bits Len (Cap)
    pub fn new(cap: u128) -> Self {
        debug_assert!(cap % 8 == 0);
        debug_assert!((cap / 8) <= usize::MAX as u128);

        let cap_byte = (cap / 8) as usize;

        unsafe {
            let ptr = if cap == 0 {
                null_mut()
            } else {
                alloc_zeroed(Layout::array::<u8>(cap_byte).unwrap()) as *mut u8
            };

            Self { cap_byte, ptr }
        }
    }

    pub fn len(&self) -> usize {
        self.cap_byte
    }

    pub fn is_empty(&self) -> bool {
        self.cap_byte == 0
    }

    pub fn test(&self, i: usize) -> bool {
        debug_assert!(i < self.cap_byte * 8);

        let off_byte = i / 8;
        let off_bits = (i % 8) as u8;

        unsafe { bit_get(*(self.ptr.add(off_byte)), off_bits) }
    }

    pub fn htest(&self, _i: u128) -> bool {
        todo!()
    }

    pub fn set(&mut self, i: usize) {
        debug_assert!(i < self.cap_byte * 8);

        let off_byte = i / 8;
        let off_bits = (i % 8) as u8;

        unsafe {
            let p = self.ptr.add(off_byte);
            *p = bit_set(*p, off_bits)
        }
    }

    /// Display Head
    pub fn display_head(&self, n: usize) {
        debug_assert!(n < self.cap_byte);

        unsafe {
            for i in 0..n {
                print!("{:08b} ", *self.ptr.add(i))
            }
            println!()
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Standard Traits Implement

impl Drop for BitMap {
    fn drop(&mut self) {
        unsafe {
            drop_in_place(slice_from_raw_parts_mut(self.ptr, self.cap_byte))
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::BitMap;

    #[test]
    fn test_bitop() {
        use super::*;

        assert_eq!(bit_set(0b_0000_1110, 4), 0b_0001_1110);
        assert_eq!(bit_get(0b_0000_1110, 4), false);
    }


    #[test]
    fn test_bitmap() {
        let max = 256;
        let mut map = BitMap::new(max as u128);

        for i in 0..max {
            for j in 0..i {
                assert!(map.test(j as usize))
            }
            for j2 in i..max {
                assert!(!map.test(j2 as usize))
            }
            map.set(i as usize);
        }
    }

    #[test]
    fn view_bit() {
        println!("{:08b}", 4);
    }
}
