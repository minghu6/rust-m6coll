use std::{
    alloc::Layout,
    ptr::NonNull,
};

////////////////////////////////////////////////////////////////////////////////
//// Structures

pub struct AlignedRawBuf {
    align: usize,
    rawbuf: RawBuf,
}

pub struct AlignedRawBufRef {
    align: usize,
    rawbuf: RawBufRef,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct AlignedRefSpec<T> {
    /// aligned value
    ref_spec: RefSpec<T>,
}

/// Owned data, Buffer + Cursor
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawBuf {
    cur: usize,
    cap: usize,
    data: NonNull<u8>,
}

/// Reference data
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct RawBufRef {
    cur: usize,
    cap: usize,
    /// maybe unaligned
    data: NonNull<u8>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct RefSpec<T> {
    /// maybe unaligned
    data: NonNull<T>,
}

////////////////////////////////////////////////////////////////////////////////
//// Implementations

impl AlignedRawBuf {
    /// exp: align parameter 8 for u64
    ///
    /// unaligned capcity would cause waste of memory
    pub fn with_capacity_align_to(cap: usize, align: usize) -> Option<Self> {
        // let Some(align) = Alignment::new(align) else { return None };

        let Ok(layout) = Layout::from_size_align(cap, align)
        else {
            return None;
        };

        let align = layout.align();
        let cap = layout.size();

        let data = unsafe {
            NonNull::new_unchecked(std::alloc::alloc_zeroed(layout))
        };

        Some(Self {
            align,
            rawbuf: RawBuf { cur: 0, cap, data },
        })
    }

    pub const fn cur_ptr(&self) -> *const u8 {
        self.rawbuf.cur_ptr()
    }

    /// reset cur to head
    pub const fn reset(&mut self) {
        self.rawbuf.reset()
    }

    pub const fn as_slice(&self) -> &[u8] {
        self.rawbuf.as_slice()
    }

    pub const fn as_mut_slice(&self) -> &mut [u8] {
        self.rawbuf.as_mut_slice()
    }

    pub const fn to_ref(&self) -> AlignedRawBufRef {
        AlignedRawBufRef { align: self.align, rawbuf: self.rawbuf.to_ref() }
    }

    pub const fn rem_len(&self) -> usize {
        self.rawbuf.rem_len()
    }

    pub const fn forward_bytes(&mut self, count: usize) {
        self.rawbuf.forward_bytes(align_size(self.align, count))
    }

    pub const fn consume_bytes(&mut self, count: usize) -> AlignedRawBufRef {
        AlignedRawBufRef {
            align: self.align,
            rawbuf: self.rawbuf.consume_bytes(align_size(self.align, count)),
        }
    }
}

impl AlignedRawBufRef {
    ///
    /// **Panic**:
    ///
    /// align ceil overflow its capacity
    pub fn from_slice(src: &[u8], align: usize) -> Self {
        let p = src.as_ptr();
        let cap = src.len();

        let count = ptr_align_ceil_count(p, align);

        if count > cap {
            panic!("Align ceil {count} overflow its capacity {cap}");
        }

        Self {
            align,
            rawbuf: RawBufRef {
                cur: 0,
                cap: cap - count,
                data: unsafe {
                    NonNull::new(p.byte_add(count) as *mut _).unwrap()
                },
            },
        }
    }

    pub const fn rem_len(&self) -> usize {
        self.rawbuf.rem_len()
    }

    pub const fn head_ptr(&self) -> *const u8 {
        self.rawbuf.head_ptr()
    }

    pub const fn cur_ptr(&self) -> *const u8 {
        self.rawbuf.cur_ptr()
    }

    /// reset cur to head
    pub const fn reset(&mut self) {
        self.rawbuf.reset()
    }

    pub const fn head_slice(&self) -> &[u8] {
        self.rawbuf.head_slice()
    }

    pub const fn head_mut_slice(&self) -> &mut [u8] {
        self.rawbuf.head_mut_slice()
    }

    pub const fn cur_slice(&self) -> &[u8] {
        self.rawbuf.cur_slice()
    }

    pub const fn consumed_slice(&self) -> &[u8] {
        self.rawbuf.consumed_slice()
    }

    pub const fn forward_bytes(&mut self, count: usize) {
        self.rawbuf.forward_bytes(align_size(self.align, count));
    }

    /// aligned forward
    pub const fn forward<T>(&mut self) {
        self.forward_bytes(size_of::<T>())
    }

    pub const fn cast<T>(&self) -> AlignedRefSpec<T> {
        AlignedRefSpec {
            ref_spec: self.rawbuf.cast(),
        }
    }

    pub const fn cast_ref<T>(&self) -> &T {
        unsafe {
            self.rawbuf
                .data
                .byte_add(self.rawbuf.cur)
                .cast::<T>()
                .as_ref()
        }
    }

    pub const fn consume<T>(&mut self) -> AlignedRefSpec<T> {
        let spec = self.rawbuf.consume::<T>();

        AlignedRefSpec { ref_spec: spec }
    }

    pub const fn consume_bytes(&mut self, count: usize) -> Self {
        Self {
            align: self.align,
            rawbuf: self.rawbuf.consume_bytes(align_size(self.align, count)),
        }
    }

    pub const fn to_rawbufref(&self) -> RawBufRef {
        self.rawbuf
    }
}

impl Into<RawBufRef> for AlignedRawBufRef {
    fn into(self) -> RawBufRef {
        self.to_rawbufref()
    }
}

impl<T> AlignedRefSpec<T> {
    pub const fn read(&self) -> T {
        unsafe { self.ref_spec.data.read() }
    }

    pub const fn write(&mut self, val: T) {
        unsafe { self.ref_spec.data.write(val) }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ref_spec.data.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ref_spec.data.as_mut() }
    }
}

impl<T> RefSpec<T> {
    pub const fn from_raw(data: NonNull<T>) -> Self {
        Self { data }
    }

    pub const fn read_unaligned(&self) -> T {
        unsafe { self.data.read_unaligned() }
    }

    pub const fn write_unaligned(&mut self, val: T) {
        unsafe { self.data.write_unaligned(val) }
    }
}

impl RawBuf {
    pub fn with_capacity(cap: usize) -> Self {
        let alloc =
            unsafe { Box::<[u8]>::new_uninit_slice(cap).assume_init() };

        Self {
            cur: 0,
            cap,
            data: unsafe { NonNull::new_unchecked(Box::into_raw(alloc) as _) },
        }
    }

    pub fn new_from_slice(src: &[u8]) -> Self {
        let mut alloc =
            unsafe { Box::<[u8]>::new_uninit_slice(src.len()).assume_init() };

        alloc.copy_from_slice(src);

        Self {
            cur: 0,
            cap: src.len(),
            data: unsafe { NonNull::new_unchecked(Box::into_raw(alloc) as _) },
        }
    }

    pub const fn head_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub const fn cur_ptr(&self) -> *const u8 {
        unsafe { self.data.byte_add(self.cur).as_ptr() }
    }

    pub const fn cur_mut_ptr(&self) -> *mut u8 {
        unsafe { self.data.byte_add(self.cur).as_ptr() }
    }

    /// reset cur to head
    pub const fn reset(&mut self) {
        self.cur = 0;
    }

    pub const fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.cur_ptr(), self.rem_len()) }
    }

    pub const fn as_mut_slice(&self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.cur_mut_ptr(), self.rem_len())
        }
    }

    pub const fn to_ref(&self) -> RawBufRef {
        RawBufRef { cur: self.cur, cap: self.cap, data: self.data }
    }

    pub const fn rem_len(&self) -> usize {
        self.cap - self.cur
    }

    pub const fn forward<T>(&mut self) {
        self.forward_bytes(size_of::<T>())
    }

    pub const fn forward_bytes(&mut self, count: usize) {
        assert!(count <= self.rem_len(), "exceeded remains buffer");

        self.cur += count;
    }

    pub const fn cast<T>(&self) -> RefSpec<T> {
        assert!(size_of::<T>() <= self.rem_len(), "exceeded remains buffer");

        RefSpec::from_raw(self.data.cast())
    }

    pub const fn consume<T>(&mut self) -> RefSpec<T> {
        let spec = self.cast::<T>();

        self.forward::<T>();

        spec
    }

    pub const fn consume_bytes(&mut self, count: usize) -> RawBufRef {
        let p = self.cur_mut_ptr();

        self.forward_bytes(count);

        RawBufRef { cur: 0, cap: count, data: unsafe { NonNull::new_unchecked(p) } }
    }
}

impl Drop for RawBuf {
    fn drop(&mut self) {
        let slice =
            core::ptr::slice_from_raw_parts_mut(self.data.as_ptr(), self.cap);
        let _ = unsafe { Box::<[u8]>::from_raw(slice) };
    }
}

impl RawBufRef {
    pub fn from_slice(src: &[u8]) -> Self {
        Self {
            cur: 0,
            cap: src.len(),
            data: NonNull::new(src.as_ptr() as *mut _).unwrap(),
        }
    }

    pub const fn rem_len(&self) -> usize {
        self.cap - self.cur
    }

    pub const fn head_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub const fn head_mut_ptr(&self) -> *mut u8 {
        self.data.as_ptr()
    }

    pub const fn cur_ptr(&self) -> *const u8 {
        unsafe { self.data.byte_add(self.cur).as_ptr() }
    }

    pub const fn cur_mut_ptr(&self) -> *mut u8 {
        unsafe { self.data.byte_add(self.cur).as_ptr() }
    }

    pub const fn cur_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.cur_ptr(), self.rem_len()) }
    }

    /// [0..cur)
    pub const fn consumed_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.head_ptr(), self.cur) }
    }

    pub const fn head_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.head_ptr(), self.rem_len()) }
    }

    pub const fn head_mut_slice(&self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.head_mut_ptr(),
                self.rem_len(),
            )
        }
    }

    /// reset cur to head
    pub const fn reset(&mut self) {
        self.cur = 0;
    }

    pub const fn forward<T>(&mut self) {
        self.forward_bytes(size_of::<T>())
    }

    pub const fn forward_bytes(&mut self, count: usize) {
        assert!(count <= self.rem_len(), "exceeded remains buffer");

        self.cur += count;
    }

    pub const fn cast<T>(&self) -> RefSpec<T> {
        assert!(size_of::<T>() <= self.rem_len(), "exceeded remains buffer");

        RefSpec::from_raw(unsafe { self.data.byte_add(self.cur).cast() })
    }

    pub const fn consume<T>(&mut self) -> RefSpec<T> {
        let spec = self.cast::<T>();

        self.forward::<T>();

        spec
    }

    pub const fn consume_bytes(&mut self, count: usize) -> Self {
        let p = self.cur_mut_ptr();

        self.forward_bytes(count);

        Self { cur: 0, cap: count, data: unsafe { NonNull::new_unchecked(p) } }
    }
}

////////////////////////////////////////////////////////////////////////////////
//// Functions

const fn align_size(align: usize, size: usize) -> usize {
    (size + (align - 1)) & !(align - 1)
}

/// return ALIGN_TO bytes
///
/// 0 for null
pub fn ptr_align_to<T>(p: *const T) -> usize {
    let addr = p.addr();

    if addr == 0 {
        0
    }
    else {
        let aligned_bits: u32 = 1 << addr.trailing_zeros();

        (aligned_bits / 8).max(1) as _
    }
}

pub fn ptr_align_ceil_count<T>(p: *const T, align: usize) -> usize {
    assert!(align.is_power_of_two());

    let addr = p.addr();

    (addr + (align - 1) & !(align - 1)) - addr
}



#[cfg(test)]
mod tests {
    use std::{
        alloc::Layout,
        ptr::{Alignment, NonNull},
    };

    use super::*;


    #[test]
    fn verify_ref_align() {
        println!("{}", 0usize.trailing_zeros());

        #[derive(Debug, Default, Clone, Copy)]
        #[repr(C, align(8))]
        struct A {
            a1: u8,
            a2: u16,
        }

        // #[derive(Debug)]
        // #[repr(C)]
        // struct B {
        //     b1: u8,
        //     b2: u16,
        // }

        unsafe {
            let mut slice = [0u8; 61];
            let heap = Box::<[u8]>::new_zeroed_slice(11).assume_init();
            let l0 = Layout::from_size_align(71, 32).unwrap();

            let mut arr_ref = AlignedRawBufRef::from_slice(&mut slice, 8);

            let a = arr_ref.cast::<A>().read();

            println!("a: {a:?}");
            println!("l0: {}", l0.size());
            println!("u64 align: {}", core::mem::align_of::<u64>());
            println!("u64 align: {:?}", Alignment::of::<u64>());
            println!("u64 align: {:?}", Alignment::new(8).unwrap());

            let layout = std::alloc::alloc_zeroed(l0);

            let p = slice.as_ptr();
            let p2 = heap.as_ptr();

            println!("p align to {}", ptr_align_to(p));
            println!("p2 align to {}", ptr_align_to(p2));
            println!("layput align to {}", ptr_align_to(layout));

            let _a = NonNull::new(p.byte_add(12) as *mut u8)
                .unwrap()
                .cast::<A>()
                .as_ref();
            let _a = (p.add(12) as *const A).read();
            // let _a = &* core::mem::transmute::<*const u8, *mut A>(p.byte_add(12));

            println!(
                "{:?}/{:?}",
                NonNull::new(p.byte_add(12) as *mut u8)
                    .unwrap()
                    .cast::<A>()
                    .as_ptr(),
                p.byte_add(12)
            );

            let _a = p.byte_add(12).cast::<A>().as_ref().unwrap();
            // let _a = p.byte_add(12).try_cast_aligned::<A>().as_ref().unwrap();

            assert!(p.byte_add(12).try_cast_aligned::<A>().as_ref().is_none());

            // let _a = &* p.byte_add(12).cast::<A>();

            // let b = (*_a).a1 + 2;
            // println!("b {b}: {}", ptr_align_to(_a as *const A));

            arr_ref.forward_bytes(12);
            // arr_ref.cast::<A>();
            arr_ref.cur_ptr().try_cast_aligned::<A>().unwrap();

            arr_ref.cast_ref::<A>();

            assert_eq!(p, arr_ref.head_ptr());
            // assert_eq!(p.add(12), arr_ref.cur_ptr());

            println!("{:?}", size_of::<&A>());
        }
    }
}
