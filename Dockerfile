# ---- build stage ----
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ---- runtime stage ----
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/vessel /usr/local/bin/vessel

ENV HTTP_ADDR=0.0.0.0:8080 \
    WS_ADDR=0.0.0.0:8081 \
    TCP_ADDR=0.0.0.0:9001 \
    MQTT_HOST=127.0.0.1 \
    MQTT_PORT=1883 \
    RUST_LOG=info

EXPOSE 8080 8081 9001

CMD ["vessel"]
