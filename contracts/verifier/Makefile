default: build

all: test

test: build
	cargo test

build:
	stellar contract build --package zk-verifier
	@ls -lh target/wasm32v1-none/release/*.wasm 2>/dev/null || ls -lh ../../target/wasm32v1-none/release/*.wasm 2>/dev/null

build-unoptimized:
	stellar contract build --package zk-verifier --optimize=false
	@ls -lh target/wasm32v1-none/release/*.wasm 2>/dev/null || ls -lh ../../target/wasm32v1-none/release/*.wasm 2>/dev/null

compare:
	@echo "=== Unoptimized ==="
	stellar contract build --package zk-verifier --optimize=false --print-commands-only
	stellar contract build --package zk-verifier --optimize=false 2>/dev/null
	@wc -c target/wasm32v1-none/release/*.wasm 2>/dev/null || wc -c ../../target/wasm32v1-none/release/*.wasm 2>/dev/null
	@echo ""
	@echo "=== Optimized ==="
	stellar contract build --package zk-verifier --optimize 2>/dev/null
	@wc -c target/wasm32v1-none/release/*.wasm 2>/dev/null || wc -c ../../target/wasm32v1-none/release/*.wasm 2>/dev/null

fmt:
	cargo fmt --all

clean:
	cargo clean
