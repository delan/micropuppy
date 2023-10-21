.POSIX:

CARGO_PROFILE_DIR = debug
CARGO_PROFILE_FLAG =

run-kernel: kernel
	cd qemu && make run-kernel KERNEL=../target/aarch64-unknown-none/$(CARGO_PROFILE_DIR)/kernel

kernel: a53
	cd kernel && make CARGO_PROFILE_FLAG=$(CARGO_PROFILE_FLAG)

a53:
	cd a53 && make

clean:
	cd kernel && make clean
	cd a53 && make clean

.PHONY: kernel a53 clean
