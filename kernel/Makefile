.POSIX:

CARGOFLAGS =
CARGO_TEST_PACKAGES = buddy-alloc

build:
	cargo build -Zbuild-std --target aarch64-unknown-none.json $(CARGOFLAGS)

test:
	cargo test $(CARGOFLAGS) -p $(CARGO_TEST_PACKAGES)

clean:
	cargo clean

.PHONY: kernel clean
