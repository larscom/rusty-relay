.PHONY: run-server run-client build clean

run-server:
	RUSTY_RELAY_CONNECT_TOKEN=dev RUST_LOG=debug cargo run --bin rusty-relay-server

run-client:
	cargo run --bin rusty-relay-client -- --server localhost:8080 --target http://localhost:3000 --token dev --insecure

build:
	cargo build --bins

clean:
	cargo clean
