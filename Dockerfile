# ── Stage 1: builder ─────────────────────────────────────────────────────────
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl \
    && strip /build/target/x86_64-unknown-linux-musl/release/dossy

# ── Stage 2: minimal scratch image ───────────────────────────────────────────
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/dossy /dossy

ENTRYPOINT ["/dossy"]
