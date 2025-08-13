# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx

# Build the actual application
# SQLx offline mode - uses cached query data
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        ca-certificates \
        libssl3 \
        libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/raworc /usr/local/bin/raworc

# Copy migrations
COPY migrations ./migrations

# Create necessary directories
RUN mkdir -p /var/lib/raworc/volumes /app/logs

# Note: Running as root to access Docker socket
# In production, consider using Docker socket proxy for security

EXPOSE 9000

CMD ["raworc", "start"]