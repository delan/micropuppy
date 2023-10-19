.POSIX:

CARGO_PROFILE_DIR = debug
CARGO_PROFILE_FLAG =

run-kernel: kernel
	cd qemu && make run-kernel KERNEL=../kernel/target/aarch64-unknown-none/$(CARGO_PROFILE_DIR)/kernel

kernel: qemu/virt-8.0.dtb
	cd kernel && cargo build $(CARGO_PROFILE_FLAG)

qemu/virt-8.0.dtb:
	cd qemu && make virt-8.0.dtb

.PHONY: kernel
