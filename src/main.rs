#![feature(allocator_api, new_uninit)]
use std::{alloc::{Allocator, Global, GlobalAlloc, Layout}, arch::global_asm, ffi::{c_char, c_int, CString}, fs::File, mem::{align_of, size_of}, os::unix::fs::FileExt, ptr::{read_unaligned, write_unaligned, NonNull}};

use aligned_box::AlignedBox;
use elf::{abi::PT_LOAD, endian::LittleEndian};

fn load_pie_elf(path: &str)->(usize, usize){
    let f = File::open(path).expect("Failed to open file");
    let f2 = File::open(path).expect("Failed to open file");
    let mut elf = elf::ElfStream::<LittleEndian, _>::open_stream(f).expect("ELF read failed");
    let entry = elf.ehdr.e_entry;
    let mut addr_min = usize::MAX;
    let mut addr_max = usize::MIN;
    for phdr in elf.segments().iter() {
        if phdr.p_type == PT_LOAD{
            addr_min = addr_min.min(phdr.p_vaddr as usize);
            addr_max = addr_max.max((phdr.p_vaddr + phdr.p_memsz) as usize);
        }
    }
    if addr_max < addr_min{
        panic!("No section found");
    }
    // align to 4k
    addr_min = addr_min / 4096 * 4096;
    addr_max = (addr_max + 4095) / 4096 * 4096;

    let size = addr_max - addr_min;

    let start_addr = unsafe{
        let addr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
            libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            -1,
            0,
        );
        assert_ne!(addr, libc::MAP_FAILED);
        addr as usize
    };
    println!("addr_min = {:#x}, addr_max = {:#x}, start_addr = {:#x}, size = {:#x}", addr_min, addr_max, start_addr, size);

    let base_addr = start_addr - addr_min;
    let memory_slice = unsafe{std::slice::from_raw_parts_mut(start_addr as *mut u8, size)};
    // load sections
    for phdr in elf.segments().iter() {
        if phdr.p_type == PT_LOAD{
            println!("Loading section at {:#x} with size {:#x} from offset {:#x}", phdr.p_vaddr, phdr.p_memsz, phdr.p_offset);
            let vaddr = phdr.p_vaddr as usize;
            let memory_slice_offset = vaddr - addr_min;
            println!("memory_slice_offset = {:#x}", memory_slice_offset);
            let target_valid_slice = &mut memory_slice[memory_slice_offset..memory_slice_offset + phdr.p_filesz as usize];
            f2.read_exact_at(target_valid_slice, phdr.p_offset as u64).expect("Failed to read file");
            let target_zero_slice = &mut memory_slice[memory_slice_offset + phdr.p_filesz as usize..memory_slice_offset + phdr.p_memsz as usize];
            target_zero_slice.fill(0);
        }
    }
    (entry as usize, base_addr)
}

pub struct Pages{
    pages: NonNull<[u8]>,
}
impl Pages{
    pub fn new(num_pages: usize) -> Self{
        let pages = Global.allocate_zeroed( Layout::from_size_align(num_pages*4096, 4096).unwrap()).unwrap();
        Self{
            pages
        }
    }
    pub fn as_slice(&self) -> &[u8]{
        unsafe{self.pages.as_ref()}
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8]{
        unsafe{self.pages.as_mut()}
    }
    pub fn num_bytes(&self) -> usize{
        self.pages.len()
    }
    pub fn num_pages(&self) -> usize{
        self.pages.len() / 4096
    }
     pub fn as_ptr(&self) -> *const u8{
        self.pages.as_ptr() as _
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8{
        self.pages.as_ptr() as _
    }
}

pub struct Stack{
    pub stack: Pages,
    pub sp: usize
}
impl Stack{
    pub fn new(size: usize) -> Self{
        let stack = Pages::new(size);
        Self{
            stack,
            sp: size * 4096
        }
    }
    pub fn align_to(&mut self, align: usize){
        self.sp -= self.sp % align;
    }
    pub fn assert_aligned(&self, align: usize){
        assert_eq!(self.sp % align, 0);
    }
    pub fn assert_aligned_to<T>(&self){
        self.assert_aligned(align_of::<T>());
    }
    pub fn push_slice<T: Copy>(&mut self, slice: &[T]){
        let slice_stack = self.alloc_value::<T>(slice.len());
        slice_stack.copy_from_slice(slice);
    }
    pub fn push_value<T: Copy>(&mut self, value: T){
        self.push_slice(&[value]);
    }
    pub fn alloc_value<T: Copy>(&mut self, size: usize) -> &mut [T]{
        self.assert_aligned_to::<T>();
        let (a, b, c) = unsafe {self.alloc(size_of::<T>() * size).align_to_mut::<T>() };
        assert!(a.is_empty());
        assert!(c.is_empty());
        b
    }
    pub fn alloc(&mut self, len: usize)->&mut [u8]{
        let old_sp = self.sp;
        self.sp -= len;
        &mut self.stack.as_mut_slice()[self.sp..old_sp]
    }
    pub fn push_bytes(&mut self, data: &[u8]){
        let slice = self.alloc(data.len());
        slice.copy_from_slice(data);
    }
    pub fn sp_addr(&self) -> *mut u8{
        unsafe {self.stack.as_ptr().offset(self.sp as isize) as *mut _}
    }
    pub fn push_str(&mut self, s: &str)->*mut u8 {
        self.push_bytes(&[0]);
        self.push_bytes(s.as_bytes());
        self.sp_addr()
    }
    pub unsafe fn run(&mut self, f: usize)->!{
        // leap of faith
        jump_to_addr(self.sp_addr(), f);
        panic!("Wtf?");
    }
}


pub const AT_PHDR: u64 = 3;
pub const AT_PHENT: u64 = 4;
pub const AT_PHNUM: u64 = 5;
pub const AT_PAGESZ: u64 = 6;
pub const AT_BASE: u64 = 7;
pub const AT_ENTRY: u64 = 9;

#[no_mangle]
extern "C"{
    fn jump_to_addr(sp: *const u8, addr: usize);
}
global_asm!(r#"
.globl jump_to_addr
jump_to_addr:
    mov rsp, rdi
    xor rdx, rdx
    # jump to rsi
    jmp rsi
infiloop:
    jmp infiloop
"#);




fn main() {
    println!("Hello, world!");
    let (entry, base) = load_pie_elf("./hello.bin");
    println!("entry: {:#x}, base: {:#x}", entry, base);
    let f = entry + base;
    let mut stack = Stack::new(100);
    stack.push_value(0u64);
    // These strings don't care about alignment.
    let envs = vec![
        stack.push_str("FOO=1"),
        stack.push_str("BAR=2"),
        stack.push_str("BAZ=3"),
        std::ptr::null()
    ];
    let argv = vec![
        stack.push_str("./hello"),
        stack.push_str("world"),
        stack.push_str("love"),
        stack.push_str("from"),
        stack.push_str("elfloader"),
        std::ptr::null()
    ];
    // https://man7.org/linux/man-pages/man3/getauxval.3.html
    // https://github.com/rcore-os/rCore/blob/66cb4181ec6d3336d507c7c1ff100127f56fcc0a/kernel/src/process/thread.rs#L153
    // https://github.com/search?q=repo%3Arcore-os%2FrCore+auxv&type=code
    // https://lwn.net/Articles/631631/

    // https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf
    let auxv: Vec<(u64, u64)> = vec![
        (AT_PAGESZ, 4096),
        (AT_BASE, base as u64),

        (0, 0)
    ];
    stack.align_to(8);
    stack.push_slice(&auxv);
    stack.push_slice(&envs);
    stack.push_slice(&argv);
    stack.push_value(argv.len() as u64 -1);
    println!("sp: {:#x}", stack.sp);
    println!("sp_addr: {:#x}", stack.sp_addr() as usize);
    unsafe {stack.run(f)};
    
}
