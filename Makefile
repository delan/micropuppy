.POSIX:

CARGO_PROFILE_DIR = debug
CARGO_PROFILE_FLAG =

run-kernel: kernel
	cd qemu && make run-kernel KERNEL=../kernel/target/aarch64-unknown-none/$(CARGO_PROFILE_DIR)/kernel

kernel:
	cd kernel && cargo build $(CARGO_PROFILE_FLAG)

.PHONY: kernel
