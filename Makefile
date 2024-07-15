hello.bin: hello.c
	x86_64-unknown-linux-musl-gcc -O3 hello.c -static-pie -fpie -o hello.bin
objdump: hello.bin
	objdump -D hello.bin > hello.bin.dump
.phony: clean objdump
clean:
	rm -f hello.bin test.txt