use std::{arch::global_asm, mem::align_of, mem::size_of};

use crate::page::Pages;

pub struct Stack {
    pub stack: Pages,
    pub sp: usize,
}
impl Stack {
    pub fn new(size: usize) -> Self {
        let stack = Pages::new(size);
        Self {
            stack,
            sp: size * 4096,
        }
    }
    pub fn align_to(&mut self, align: usize) {
        self.sp -= self.sp % align;
    }
    pub fn assert_aligned(&self, align: usize) {
        assert_eq!(self.sp % align, 0);
    }
    pub fn assert_aligned_to<T>(&self) {
        self.assert_aligned(align_of::<T>());
    }
    pub fn push_slice<T: Copy>(&mut self, slice: &[T]) {
        let slice_stack = self.alloc_value::<T>(slice.len());
        slice_stack.copy_from_slice(slice);
    }
    pub fn push_value<T: Copy>(&mut self, value: T) {
        self.push_slice(&[value]);
    }
    pub fn alloc_value<T: Copy>(&mut self, size: usize) -> &mut [T] {
        self.assert_aligned_to::<T>();
        let (a, b, c) = unsafe { self.alloc(size_of::<T>() * size).align_to_mut::<T>() };
        assert!(a.is_empty());
        assert!(c.is_empty());
        b
    }
    pub fn alloc(&mut self, len: usize) -> &mut [u8] {
        let old_sp = self.sp;
        self.sp -= len;
        &mut self.stack.as_mut_slice()[self.sp..old_sp]
    }
    pub fn push_bytes(&mut self, data: &[u8]) {
        let slice = self.alloc(data.len());
        slice.copy_from_slice(data);
    }
    pub fn sp_addr(&self) -> *mut u8 {
        unsafe { self.stack.as_ptr().offset(self.sp as isize) as *mut _ }
    }
    pub fn push_str(&mut self, s: &str) -> *mut u8 {
        self.push_bytes(&[0]);
        self.push_bytes(s.as_bytes());
        self.sp_addr()
    }
    pub unsafe fn run(&mut self, f: usize) -> ! {
        // leap of faith
        jump_to_addr(self.sp_addr(), f);
        panic!("Wtf?");
    }
}

extern "C" {
    fn jump_to_addr(sp: *const u8, addr: usize);
}

/*
    a bit register magic.
    rsp: stack pointer where argc stores.
    rbp: 0
    rdx: atexit hook. can be 0 to disable.
    https://github.com/runtimejs/musl-libc/blob/master/crt/x86_64/crt1.s
*/
global_asm!(
    r#"
.globl jump_to_addr
jump_to_addr:
    mov rsp, rdi
    xor rdx, rdx
    # jump to rsi
    jmp rsi
infiloop:
    jmp infiloop
"#
);
