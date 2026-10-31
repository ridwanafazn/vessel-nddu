# Hugging Face Spaces image: satu port (7860) untuk REST + Swagger + WebSocket.
# Mosquitto dijalankan di dalam container agar MQTT tetap hidup tanpa broker eksternal.

FROM rust:1-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential pkg-config curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates nginx mosquitto \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/vessel /usr/local/bin/vessel
COPY deploy/hf/nginx.conf /etc/nginx/nginx.conf
COPY deploy/hf/mosquitto.conf /etc/mosquitto/mosquitto.conf
COPY deploy/hf/entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh \
    && mkdir -p /tmp/nginx/client_body /tmp/nginx/proxy /tmp/nginx/fastcgi /tmp/nginx/uwsgi /tmp/nginx/scgi

ENV HTTP_ADDR=127.0.0.1:8080 \
    WS_ADDR=127.0.0.1:8081 \
    TCP_ADDR=127.0.0.1:9001 \
    MQTT_HOST=127.0.0.1 \
    MQTT_PORT=1883 \
    MQTT_CLIENT_ID=vessel-hf \
    RUST_LOG=info

EXPOSE 7860

CMD ["/entrypoint.sh"]
