# SPDX-License-Identifier: APACHE-2.0 AND MIT
.PHONY: build check test clean run install update readme fmt lint
.PHONY: doc doc-all doc-crate readme fmt lint
all: fmt lint test
build:
	@cd schema && flatc --rust *.fbs
check: build
	@cargo check
test: build
	@cargo test
clean:
	@cargo clean
run: build
	@cargo run
install: build
	@cargo install --force --path .
update:
	@cargo update
readme:
	@cargo install cargo-readme
	@cargo readme > README.md
fmt: build
	@rustfmt --edition 2018 --check src/*.rs
lint: build
	@cargo clippy -- -D warnings
doc: doc-crate
doc-all: doc-crate doc-book doc-std
doc-crate:
	@cargo doc --all --open &
doc-%:
	@rustup doc --$* &
# CI targets.
.PHONY: arch64 ubuntu64
arch64: arch64-image
	docker run -v $(PWD):/home/build rustbox/$@ make all clean
ubuntu64: ubuntu64-image
	docker run -v $(PWD):/home/build rustbox/$@ make all clean
%-arch64: arch64-image
	docker run -v $(PWD):/home/build rustbox/arch64 make $* clean
%-ubuntu64: ubuntu64-image
	docker run -v $(PWD):/home/build rustbox/ubuntu64 make $* clean
%-image:
	docker build -t rustbox/$* -f Dockerfile.$* .
