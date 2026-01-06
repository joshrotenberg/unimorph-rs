# Build stage
FROM rust:latest AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release binary
RUN cargo build --release -p unimorph

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 unimorph

# Copy binary from builder
COPY --from=builder /build/target/release/unimorph /usr/local/bin/unimorph

# Create data directory
RUN mkdir -p /data && chown unimorph:unimorph /data

USER unimorph
WORKDIR /home/unimorph

# Default data directory
ENV UNIMORPH_DATA=/data

ENTRYPOINT ["unimorph"]
CMD ["--help"]
