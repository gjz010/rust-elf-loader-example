use std::{
    alloc::{Allocator, Global, Layout},
    ptr::NonNull,
};

pub struct Pages {
    pages: NonNull<[u8]>,
}
impl Pages {
    pub fn new(num_pages: usize) -> Self {
        let pages = Global
            .allocate_zeroed(Layout::from_size_align(num_pages * 4096, 4096).unwrap())
            .unwrap();
        Self { pages }
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe { self.pages.as_ref() }
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { self.pages.as_mut() }
    }
    pub fn num_bytes(&self) -> usize {
        self.pages.len()
    }
    pub fn num_pages(&self) -> usize {
        self.pages.len() / 4096
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.pages.as_ptr() as _
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.pages.as_ptr() as _
    }
}
