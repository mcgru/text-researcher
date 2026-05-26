# text-researcher Makefile

.PHONY: build test lint clean docker deps

build:
	cargo build --release --workspace

test:
	cargo test --workspace

lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --check

clean:
	cargo clean

deps:
	pip3 install --user tensorflow numpy ufal.chu-liu-edmonds

docker:
	docker build -t text-researcher .
