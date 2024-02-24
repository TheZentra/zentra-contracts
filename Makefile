default: build

test: build
	cargo test --all --tests

build:
	cargo rustc --manifest-path=stream/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	mkdir -p target/wasm32-unknown-unknown/optimized
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/stream.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/stream.wasm
	cd target/wasm32-unknown-unknown/optimized/ && \
		for i in *.wasm ; do \
			ls -l "$$i"; \
		done

fmt:
	cargo fmt --all

clean:
	cargo clean

generate-js:
	soroban contract bindings typescript --overwrite \
		--contract-id CAATXU6OCY3BHIIY44NN73LM3PRAQSEU2ASNZ43XNAFECAENPVSRJEBJ \
		--wasm ./target/wasm32-unknown-unknown/optimized/stream.wasm --output-dir ./js/js-stream/
