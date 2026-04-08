# == Rust Build Stage ==
FROM rust:1.92-alpine AS builder

COPY ./ ./

RUN  \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/target \
    cargo build --release --bin wg-api && \
    cp ./target/release/wg-api ./wg-api.exe

# == WG App Image ==
FROM alpine
WORKDIR /wg
COPY --from=builder /wg-api.exe /wg/wg-api.exe

RUN ls

EXPOSE 3100

# Run the binary
CMD ["/wg/wg-api.exe"]

