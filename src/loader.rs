use std::{fs::File, os::unix::fs::FileExt};

use elf::{abi::PT_LOAD, endian::LittleEndian};

pub fn load_pie_elf(path: &str) -> (usize, usize) {
    let f = File::open(path).expect("Failed to open file");
    let f2 = File::open(path).expect("Failed to open file");
    let elf = elf::ElfStream::<LittleEndian, _>::open_stream(f).expect("ELF read failed");
    let entry = elf.ehdr.e_entry;
    let mut addr_min = usize::MAX;
    let mut addr_max = usize::MIN;
    for phdr in elf.segments().iter() {
        if phdr.p_type == PT_LOAD {
            addr_min = addr_min.min(phdr.p_vaddr as usize);
            addr_max = addr_max.max((phdr.p_vaddr + phdr.p_memsz) as usize);
        }
    }
    if addr_max < addr_min {
        panic!("No section found");
    }
    // align to 4k
    addr_min = addr_min / 4096 * 4096;
    addr_max = (addr_max + 4095) / 4096 * 4096;

    let size = addr_max - addr_min;

    let start_addr = unsafe {
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
    println!(
        "addr_min = {:#x}, addr_max = {:#x}, start_addr = {:#x}, size = {:#x}",
        addr_min, addr_max, start_addr, size
    );

    let base_addr = start_addr - addr_min;
    let memory_slice = unsafe { std::slice::from_raw_parts_mut(start_addr as *mut u8, size) };
    // load sections
    for phdr in elf.segments().iter() {
        if phdr.p_type == PT_LOAD {
            println!(
                "Loading section at {:#x} with size {:#x} from offset {:#x}",
                phdr.p_vaddr, phdr.p_memsz, phdr.p_offset
            );
            let vaddr = phdr.p_vaddr as usize;
            let memory_slice_offset = vaddr - addr_min;
            println!("memory_slice_offset = {:#x}", memory_slice_offset);
            let target_valid_slice = &mut memory_slice
                [memory_slice_offset..memory_slice_offset + phdr.p_filesz as usize];
            f2.read_exact_at(target_valid_slice, phdr.p_offset as u64)
                .expect("Failed to read file");
            let target_zero_slice = &mut memory_slice[memory_slice_offset + phdr.p_filesz as usize
                ..memory_slice_offset + phdr.p_memsz as usize];
            target_zero_slice.fill(0);
        }
    }
    (entry as usize, base_addr)
}
