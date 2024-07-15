#![feature(allocator_api, new_uninit)]
use libc::{AT_BASE, AT_PAGESZ};

use crate::{loader::load_pie_elf, stack::Stack};

pub mod loader;
pub mod page;
pub mod stack;

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
        std::ptr::null(),
    ];
    let argv = vec![
        stack.push_str("./hello"),
        stack.push_str("world"),
        stack.push_str("love"),
        stack.push_str("from"),
        stack.push_str("elfloader"),
        std::ptr::null(),
    ];
    // https://man7.org/linux/man-pages/man3/getauxval.3.html
    // https://github.com/rcore-os/rCore/blob/66cb4181ec6d3336d507c7c1ff100127f56fcc0a/kernel/src/process/thread.rs#L153
    // https://github.com/search?q=repo%3Arcore-os%2FrCore+auxv&type=code
    // https://lwn.net/Articles/631631/

    // https://refspecs.linuxbase.org/elf/x86_64-abi-0.99.pdf
    let auxv: Vec<(u64, u64)> = vec![(AT_PAGESZ, 4096), (AT_BASE, base as u64), (0, 0)];
    stack.align_to(8);
    stack.push_slice(&auxv);
    stack.push_slice(&envs);
    stack.push_slice(&argv);
    stack.push_value(argv.len() as u64 - 1);
    println!("sp: {:#x}", stack.sp);
    println!("sp_addr: {:#x}", stack.sp_addr() as usize);

    // pull rsp to the stack and start running.
    unsafe { stack.run(f) };
}
