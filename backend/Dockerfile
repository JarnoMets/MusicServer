# Build stage
FROM rust:1.92 as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libaubio-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Copy build script so the release build inside the image can run the build.rs
COPY build.rs ./

# Migrations are now implemented programmatically in the Rust code (see src/db/schema.rs)
# By default the backend runs programmatic migrations at startup. To opt-out, set
# RUN_MIGRATIONS=false in the container environment.
# and run at runtime during backend startup, so we no longer copy SQL migration files here.

# Copy source code
COPY src ./src

# Build the application
RUN CFLAGS="-D_GNU_SOURCE" cargo build --release

# Runtime stage: use a stable base
## Use the same Rust slim base for runtime to ensure compatible glibc
## (quick fix for containers where the built binary requires a newer glibc
## than the minimal runtime image provides). This increases image size but
## avoids GLIBC mismatches when the builder and runtime differ.
FROM rust:1.92-slim as runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    postgresql-client \
    libssl-dev \
    pkg-config \
    python3 \
    python3-pip \
    ffmpeg \
    libaubio5 \
    aubio-tools \
    bpm-tools \
    && rm -rf /var/lib/apt/lists/* \
    && pip3 install --break-system-packages yt-dlp

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/music-server /app/music-server

EXPOSE 8081

CMD ["/app/music-server"]
