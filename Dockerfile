# Multi-stage build for unimorph CLI
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release --package unimorph

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/unimorph /usr/local/bin/unimorph

# Create data directory
RUN mkdir -p /data
ENV UNIMORPH_DATA=/data

ENTRYPOINT ["unimorph"]
CMD ["--help"]
