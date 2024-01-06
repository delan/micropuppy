.POSIX:

CARGOFLAGS =
CARGOFLAGS_TARGET = -Zbuild-std --target ../aarch64-unknown-none.json

.PHONY: internal
internal:
	@>&2 echo 'use cargo xtask, not make!'
	@exit 1

.PHONY: build
build:
	cargo build $(CARGOFLAGS_TARGET) $(CARGOFLAGS)

.PHONY: test
test:
	# Run tests on the host (for platform-independent packages only).
	cargo test $(CARGOFLAGS)

.PHONY: clean
clean:
	cargo clean
