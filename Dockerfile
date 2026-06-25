# ── Build stage ───────────────────────────────────────────────────────────────
FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libopencv-dev \
    libclang-dev \
    clang \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Pre-fetch crates so this layer is cached unless Cargo.lock changes
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

COPY src ./src
RUN cargo build --release

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

# libopencv-dev brings the shared libraries needed at runtime.
# (Headers are unused here but avoid chasing individual .so package names.)
RUN apt-get update && apt-get install -y --no-install-recommends \
    libopencv-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/film-dust-cleaner .

EXPOSE 3000
CMD ["./film-dust-cleaner", "serve"]
