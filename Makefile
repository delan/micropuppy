.POSIX:

run-kernel: kernel
	cd qemu && make run-kernel

kernel:
	cd kernel && cargo build

.PHONY: kernel
