default: build

test: build
	cargo test --all --tests

build:
	cargo rustc --manifest-path=contracts/direct-stream/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	cargo rustc --manifest-path=contracts/mocks/mock-token/Cargo.toml --crate-type=cdylib --target=wasm32-unknown-unknown --release
	mkdir -p target/wasm32-unknown-unknown/optimized
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/zentra_direct_stream.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/zentra_direct_stream.wasm
	soroban contract optimize \
		--wasm target/wasm32-unknown-unknown/release/mock_token.wasm \
		--wasm-out target/wasm32-unknown-unknown/optimized/mock_token.wasm
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
		--rpc-url http://localhost:26657 \
		--network-passphrase "Test SDF Network ; September 2015" \
		--network testnet \
		--wasm ./target/wasm32-unknown-unknown/optimized/stream.wasm --output-dir ./js/js-stream/
