FROM rust:slim-trixie AS builder

ARG VERSION=""

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY build.rs ./
COPY migrations/ ./migrations/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir -p src && touch src/lib.rs && \
    cargo build --locked --release

RUN if [ -n "$VERSION" ]; then \
      sed -i 's;version\s*=\s*"0.0.0";version = "'"${VERSION}"'";' Cargo.toml && \
      cargo update gjallarbot; \
    fi

COPY src/ ./src/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    rm src/lib.rs && cargo build --locked --release && \
    cp "target/release/gjallarbot" /app/gjallarbot

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false gjallarbot

WORKDIR /workdir
RUN mkdir -p data && chown gjallarbot:gjallarbot data

VOLUME /workdir/data

COPY --from=builder /app/gjallarbot /usr/local/bin/

USER gjallarbot

ENTRYPOINT ["gjallarbot"]
