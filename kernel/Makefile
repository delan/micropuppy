.POSIX:

CARGOFLAGS =

build:
	cargo build -Zbuild-std --target aarch64-unknown-none.json $(CARGOFLAGS)

test:
	# Run tests on the host (for platform-independent packages only).
	cargo test $(CARGOFLAGS)

clean:
	cargo clean

.PHONY: kernel clean
