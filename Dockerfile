FROM rust:latest AS builder
WORKDIR /build
COPY . ./
RUN cargo build --bin rusty-relay-server --release

FROM gcr.io/distroless/cc-debian13
WORKDIR /app
COPY --from=builder /build/target/release/rusty-relay-server ./rusty-relay-server
CMD ["/app/rusty-relay-server"]
