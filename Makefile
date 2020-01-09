# SPDX-License-Identifier: Apache-2.0 AND MIT
TARGET	:= mqctl
CRATE 	:= async-mq
.PHONY: build check test clean run release release-test release-run install update \
	readme fmt lint doc doc-all doc-crate readme fmt lint
all: fmt lint test
build:
	@cd examples/schema && flatc --rust *.fbs
check:
	@cargo check
test: build
	@cargo test
clean:
	@-rm -f examples/$(CRATE)/schema/*_generated.rs
	@cargo clean
run: run-tokio
run-%: build
	@cargo run --example $(TARGET) -- --runtime $*
release:
	@cargo build --release
release-test: build
	@cargo test --release
release-run: release-run-tokio
release-run-%: build
	@cargo run --release --example $(CRATE) -- --runtime $*
install: build
	@cargo install --force --path . --example $(TARGET)
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
	docker run -v $(PWD):/home/build $(CRATE)/$@ make all clean
ubuntu64: ubuntu64-image
	docker run -v $(PWD):/home/build $(CRATE)/$@ make all clean
%-arch64: arch64-image
	docker run -v $(PWD):/home/build $(CRATE)/arch64 make $* clean
%-ubuntu64: ubuntu64-image
	docker run -v $(PWD):/home/build $(CRATE)/ubuntu64 make $* clean
%-image:
	docker build -t $(CRATE)/$* -f Dockerfile.$* .
