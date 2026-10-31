#!/bin/sh
set -e

# Render/HF memberikan PORT; default 7860 untuk lokal.
PORT="${PORT:-7860}"

mkdir -p /tmp/nginx/client_body /tmp/nginx/proxy /tmp/nginx/fastcgi \
    /tmp/nginx/uwsgi /tmp/nginx/scgi

# Tulis config nginx ke /tmp agar bisa dijalankan sebagai non-root.
sed "s/__PORT__/${PORT}/g" /etc/nginx/nginx.conf > /tmp/nginx/nginx.conf

# Start Mosquitto
mosquitto -c /etc/mosquitto/mosquitto.conf &

# Tunggu Mosquitto ready (max 10 detik)
echo "Waiting for Mosquitto to be ready..."
i=1
while [ $i -le 10 ]; do
    if mosquitto_sub -h 127.0.0.1 -p 1883 -t '$SYS/broker/version' -C 1 -W 1 >/dev/null 2>&1; then
        echo "Mosquitto ready"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "Mosquitto failed to start within 10 seconds" >&2
        exit 1
    fi
    i=$((i + 1))
    sleep 1
done

# Start vessel
vessel &
VESSEL_PID=$!

# Tunggu backend siap sebentar, lalu jalankan nginx di foreground
sleep 1

if ! kill -0 "$VESSEL_PID" 2>/dev/null; then
    echo "vessel failed to start" >&2
    exit 1
fi

exec nginx -c /tmp/nginx/nginx.conf -g "daemon off;"
