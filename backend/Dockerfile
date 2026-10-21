# ---- build stage ----
FROM rust:1-bookworm AS builder
WORKDIR /app

# Cache dependencies first: build a throwaway crate with the real manifest.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    echo "" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Now build the real sources. `touch` forces cargo to see fresh mtimes;
# without it the stub artifacts built above are considered up to date.
COPY src ./src
COPY migrations ./migrations
RUN find src -name '*.rs' -exec touch {} + && \
    cargo build --release --bin public-authority-api

# ---- runtime stage ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/* && \
    useradd --system --create-home --uid 10001 appuser

COPY --from=builder /app/target/release/public-authority-api /usr/local/bin/public-authority-api

USER appuser
EXPOSE 8080

ENV BIND_ADDR=0.0.0.0:8080 \
    RUST_LOG=info,public_authority_api=debug

HEALTHCHECK --interval=15s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://localhost:8080/health || exit 1

CMD ["public-authority-api"]
