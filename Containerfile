FROM rust:1.85.1-slim-bookworm as builder

WORKDIR /app

# Install only necessary dependencies with --no-install-recommends
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create an intermediate layer for dependencies (better caching)
COPY Cargo.toml Cargo.lock ./
# Trick to cache dependencies without needing the source code
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/rust_actix_seaorm* target/release/rust-actix-seaorm*

# Now copy the real code and build the final binary
COPY src ./src
COPY .env ./

RUN cargo build --release && \
    # Strip the binary to make it smaller
    strip target/release/rust-actix-seaorm

# Production image based on debian-slim
FROM debian:bookworm-slim as runtime

WORKDIR /app

# Install only absolutely necessary libraries
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev && \
    rm -rf /var/lib/apt/lists/* && \
    # Clean APT
    apt-get clean autoclean && \
    apt-get autoremove -y

# Copy only the binary and the .env file
COPY --from=builder /app/target/release/rust-actix-seaorm /app/
COPY --from=builder /app/.env /app/

# Non-privileged user for security
RUN useradd -m appuser
USER appuser

EXPOSE 8000

CMD ["/app/rust-actix-seaorm"]
