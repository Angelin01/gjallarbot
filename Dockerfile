FROM --platform=$BUILDPLATFORM rust:slim-trixie AS builder

ARG VERSION=""
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /app

RUN case "$TARGETPLATFORM" in \
      "linux/amd64") TARGET=x86_64-unknown-linux-gnu ;; \
      "linux/arm64") TARGET=aarch64-unknown-linux-gnu ;; \
    *) echo "Unsupported platform: $TARGETPLATFORM" >&2 && exit 1 ;; \
    esac && \
    echo "$TARGET" > /tmp/target

RUN rustup target add "$(cat /tmp/target)"

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY build.rs ./
COPY migrations/ ./migrations/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    mkdir -p src && touch src/lib.rs && \
    cargo build --locked --release --target $(cat /tmp/target)

RUN if [ -n "$VERSION" ]; then \
      sed -i 's;version\s*=\s*"0.0.0";version = "'"${VERSION}"'";' Cargo.toml && \
      cargo update gjallarbot \
    fi

COPY src/ ./src/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    rm src/lib.rs && cargo build --locked --release --target $(cat /tmp/target) && \
    cp "target/$(cat /tmp/target)/release/gjallarbot" /app/gjallarbot

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
