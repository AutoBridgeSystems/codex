# syntax=docker/dockerfile:1.7

############################################################
# Build stage: compile the Adom CLI binary with Cargo
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
RUN cd adom-rs \
    && cargo build --release --bin adom

############################################################
# Runtime stage: minimal image that only contains the binary
############################################################
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

COPY --from=builder /workspace/adom-rs/target/release/adom /usr/local/bin/adom

# Non-root user for better safety
RUN useradd -m -u 1000 adom
USER adom

ENTRYPOINT ["/usr/bin/tini", "--", "adom"]
