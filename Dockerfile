FROM rust:latest as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libclang-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY config ./config

# Build application
RUN touch src/main.rs
RUN cargo build --release --bin hft_main

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -u 1000 -g root -s /sbin/nologin hft

# Copy binary and config
COPY --from=builder /app/target/release/hft_main /app/hft_main
COPY --chown=hft:root config ./config

# Create directories
RUN mkdir -p logs data && chown -R hft:root /app

# Switch to non-root user
USER hft

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the advanced HFT application
CMD ["./hft_main", "--enable-helius", "--enable-mev", "--enable-jito"]