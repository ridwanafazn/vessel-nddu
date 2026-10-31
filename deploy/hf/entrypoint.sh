#!/bin/sh
set -e

mkdir -p /tmp/nginx/client_body /tmp/nginx/proxy /tmp/nginx/fastcgi \
    /tmp/nginx/uwsgi /tmp/nginx/scgi

# Broker MQTT internal (tidak diekspos ke luar container)
mosquitto -c /etc/mosquitto/mosquitto.conf &

# Backend
vessel &
VESSEL_PID=$!

# Tunggu backend siap sebentar, lalu jalankan nginx di foreground
sleep 1

if ! kill -0 "$VESSEL_PID" 2>/dev/null; then
    echo "vessel failed to start" >&2
    exit 1
fi

exec nginx -c /etc/nginx/nginx.conf -g "daemon off;"
