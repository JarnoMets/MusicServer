# Build stage
FROM rust:1.86-slim as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Migrations are now implemented programmatically in the Rust code (see src/db/schema.rs)
# By default the backend runs programmatic migrations at startup. To opt-out, set
# RUN_MIGRATIONS=false in the container environment.
# and run at runtime during backend startup, so we no longer copy SQL migration files here.

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    libssl-dev \
    pkg-config \
    python3 \
    python3-pip \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/* \
    && pip3 install --break-system-packages yt-dlp

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/music-server /app/music-server

EXPOSE 8081

CMD ["/app/music-server"]
