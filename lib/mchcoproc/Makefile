.PHONY: all
all: clippy debug

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: debug
debug:
	cargo build

.PHONY: release
release:
	#cargo install cargo-auditable cargo-audit
	#cargo auditable build --release
	cargo build --release
