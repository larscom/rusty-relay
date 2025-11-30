.PHONY: run-server run-client build test clean

run-server:
	RUSTY_RELAY_CONNECT_TOKEN=dev RUST_LOG=info cargo run --bin rusty-relay-server

run-client:
	cargo run --bin rusty-relay-client -- --server localhost:8080 --target http://localhost:5173 --token dev --insecure

build:
	cargo build --bins

test:
	cargo test --bins

clean:
	cargo clean
