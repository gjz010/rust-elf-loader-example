# ELF loader example in Rust

Loading a Linux program compiled with `-static-pie`, say a static one compiled with musl libc.

- Pull ELF segments into memory.
- Setup a stack consisting of aux, envp, argv and argc.
- Run.

## Usage
```bash
$ make
x86_64-unknown-linux-musl-gcc -O3 hello.c -static-pie -fpie -o hello.bin

$ cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/elfloader`
Hello, world!
addr_min = 0x0, addr_max = 0xa000, start_addr = 0x7f5102fb7000, size = 0xa000
Loading section at 0x0 with size 0x440 from offset 0x0
memory_slice_offset = 0x0
Loading section at 0x1000 with size 0x4cf1 from offset 0x1000
memory_slice_offset = 0x1000
Loading section at 0x6000 with size 0x1e3c from offset 0x6000
memory_slice_offset = 0x6000
Loading section at 0x8e00 with size 0xdb0 from offset 0x8e00
memory_slice_offset = 0x8e00
entry: 0x10fc, base: 0x7f5102fb7000
sp: 0x63f38
sp_addr: 0x7f5102fb5f38
Hello, world!
argv[0] = ./hello
argv[1] = world
argv[2] = love
argv[3] = from
argv[4] = elfloader
$ cat test.txt 
Hello, world!
```

## TODO
- Fine-grained `mprotect` for `W^X`.
- Run more complicated examples like musl dynamic linker or even glibc dynamic linker.


## Acknowledgement

[rCore](https://github.com/rcore-os/rCore)
