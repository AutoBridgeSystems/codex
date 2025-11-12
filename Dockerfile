# syntax=docker/dockerfile:1.7

############################################################
# Build stage: compile the Codex CLI binary with Cargo
############################################################
FROM rust:1.84-bullseye AS builder

WORKDIR /workspace

# Install build prerequisites required by some crates (e.g., openssl)
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire repo (faster incremental builds by respecting Docker cache)
COPY . .

# Build the release binary
RUN cd codex-rs \
    && cargo build --release --bin codex

############################################################
# Runtime stage: minimal image that only contains the binary
############################################################
FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY --from=builder /workspace/codex-rs/target/release/codex /usr/local/bin/codex

# Non-root user for better safety
RUN useradd -m -u 1000 codex
USER codex

ENTRYPOINT ["/usr/bin/tini", "--", "codex"]
