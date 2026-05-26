# Multi-stage Docker build for text-researcher
FROM rust:1.85-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
RUN cargo build --release --workspace

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip \
    && pip3 install --break-system-packages tensorflow numpy ufal.chu-liu-edmonds \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/text-researcher /usr/local/bin/text-researcher

# Copy UDPipe models if available
COPY models/ /app/models/

WORKDIR /data
ENV UDPIPE_MODEL_DIR=/app/models

ENTRYPOINT ["text-researcher"]
