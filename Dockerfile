# 🛡️ CHAINGUARD HARDENED BUILD - ENTERPRISE SECURITY
FROM cgr.dev/chainguard/rust:latest-dev as builder

WORKDIR /app

# Chainguard rust:latest-dev comes with essential build tools pre-installed
# Including OpenSSL development libraries and pkg-config
# No additional packages needed - security hardened by default

# Copy manifests
COPY Cargo.toml ./

# Build dependencies (cached layer)
RUN mkdir -p src/bin && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src
COPY config ./config

# Build application with static linking for distroless compatibility
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --bin hft-ninja --target x86_64-unknown-linux-gnu

# 🚀 PRODUCTION RUNTIME - CHAINGUARD STATIC (DISTROLESS)
FROM cgr.dev/chainguard/static:latest

WORKDIR /app

# Chainguard static image:
# - Zero vulnerabilities by design
# - No shell, no package manager, no attack surface
# - Built-in non-root user (nonroot:65532)
# - CA certificates included
# - Ultra-minimal: ~2MB total size

# Copy binary and config (static binary, no dependencies needed)
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/hft-ninja /usr/local/bin/hft-ninja
COPY --from=builder /app/config /app/config

# Use built-in non-root user from Chainguard
USER nonroot:nonroot

# Expose port
EXPOSE 8080

# Note: No health check in distroless - handled by Kubernetes/orchestrator
# Environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the hardened HFT application
ENTRYPOINT ["/usr/local/bin/hft-ninja"]
CMD ["--enable-helius", "--enable-mev", "--enable-jito"]