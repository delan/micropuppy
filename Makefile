.POSIX:

CARGOFLAGS =
CARGOFLAGS_TARGET = -Zbuild-std --target aarch64-unknown-none.json
CARGOFLAGS_LSP = --message-format=json-diagnostic-rendered-ansi

.PHONY: internal
internal:
	@>&2 echo 'use cargo xtask, not make!'
	@exit 1

.PHONY: lsp-check
lsp-check:
	# This yields multiple lines with {"reason":"build-finished"}, but
	# rust-analyzer seems to tolerate that without misbehaving.
	-cargo check $(CARGOFLAGS_LSP) --all-features --all-targets -p xtask $(CARGOFLAGS)
	-cargo check $(CARGOFLAGS_LSP) $(CARGOFLAGS_TARGET) --all-features --workspace --exclude xtask --exclude buddy-alloc $(CARGOFLAGS)
	-cargo check $(CARGOFLAGS_LSP) --all-features -p buddy-alloc $(CARGOFLAGS)
