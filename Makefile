hello.bin: hello.c
	x86_64-unknown-linux-musl-gcc -O3 hello.c -static-pie -fpie -o hello.bin
.phony: clean
clean:
	rm -f hello.bin
