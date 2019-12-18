# SPDX-License-Identifier: APACHE-2.0 AND MIT
.PHONY: build check test test-release clean run run-release install update \
	readme fmt lint doc doc-all doc-crate readme fmt lint
all: fmt lint test
build:
	@cd examples/schema && flatc --rust *.fbs
check:
	@cargo check
test:
	@cargo test
test-release:
	@cargo test --release
clean:
	@cargo clean
run: run-tokio
run-release: run-release-tokio
run-%: build
	@cargo run --example rustmq -- --runtime $*
run-release-%: build
	@cargo run --release --example rustmq -- --runtime $*
install: build
	@cargo install --force --path . --example rustmq
update:
	@cargo update
readme:
	@cargo install cargo-readme
	@cargo readme > README.md
fmt:
	@rustfmt --edition 2018 --check src/*.rs
lint:
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
