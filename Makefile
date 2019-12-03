# SPDX-License-Identifier: GPL-2.0
.PHONY: gen check test clean run install doc doc-crate fmt lint
all: fmt lint test
gen:
	@cd flatbuff/src && flatc --rust *.fbs && rustfmt --edition 2018 *.rs
check:
	@cargo check
test:
	@cargo test
clean:
	@cargo clean
	@rm flatbuff/src/*.rs
run:
	@cargo run
install:
	@cargo install --force --path .
doc: doc-crate doc-book doc-std
doc-crate:
	@cargo doc --all --open &
doc-%:
	@rustup doc --$* &
fmt:
	@rustfmt --edition 2018 --check mq/src/*.rs
lint:
	@cd mq && cargo clippy -- -D warnings
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
