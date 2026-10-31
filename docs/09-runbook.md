# 09 — Runbook

## 1. Prasyarat (Windows)

| Komponen | Status di mesin Anda | Cara install |
|---|---|---|
| Rust 1.99.1 | ✅ sudah | `winget install Rustlang.Rustup` |
| MSVC Build Tools | ✅ sudah (VS Community 18) | — |
| Mosquitto | ❌ belum | `winget install EclipseFoundation.Mosquitto` |
| Port 8080 | ✅ bebas (smile-fest-api dimatikan) | — |
| Docker | tidak dipakai | — |

> Docker **tidak diperlukan**. Mosquitto native lebih ringan.

## 2. Menjalankan aplikasi

```powershell
cd C:\Users\Ridwan\Git-v2\vessel-nddu

# terminal baru agar PATH cargo terbaca
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

cargo run              # debug
cargo run --release    # produksi
```

Output yang benar (contoh):

```
INFO vessel: vessel backend starting version="0.1.0" http=127.0.0.1:8080 ws=127.0.0.1:8081 tcp=127.0.0.1:9001 mqtt=127.0.0.1:1883
INFO vessel: starting background services
INFO vessel: background services running
INFO vessel: API server started addr=127.0.0.1:8080
INFO vessel: WebSocket server started addr=127.0.0.1:8081
INFO vessel: TCP server started addr=127.0.0.1:9001
INFO vessel::utils::mqtt_manager: MQTT connected
```

### Konfigurasi via environment

| Variabel | Default | Fungsi |
|---|---|---|
| `HTTP_ADDR` | `127.0.0.1:8080` | REST |
| `WS_ADDR` | `127.0.0.1:8081` | WebSocket |
| `TCP_ADDR` | `127.0.0.1:9001` | TCP server |
| `MQTT_HOST` | `127.0.0.1` | Broker |
| `MQTT_PORT` | `1883` | Broker |
| `MQTT_CLIENT_ID` | `vessel-client` | Client ID |
| `MQTT_USERNAME` | — | Opsional |
| `MQTT_PASSWORD` | — | Opsional |
| `MQTT_KEEP_ALIVE_SECS` | `5` | Keepalive |
| `RUST_LOG` | `info` | Level log |

Contoh akses dari LAN:

```powershell
$env:HTTP_ADDR="0.0.0.0:8080"
$env:WS_ADDR="0.0.0.0:8081"
cargo run
```

## 3. Mosquitto — kenapa dan bagaimana

### Kenapa perlu

Aplikasi adalah **publisher**, bukan broker. Tanpa broker:
`publish().await` memenuhi channel (kapasitas 10) → **thread macet** →
WebSocket & NMEA berhenti.

### Install

```powershell
winget install EclipseFoundation.Mosquitto
```

Lokasi: `C:\Program Files\mosquitto\`.

### Konfigurasi (sudah dibuat)

File: `C:\Users\Ridwan\mosquitto\mosquitto.conf`

```
listener 1883 127.0.0.1
allow_anonymous true
persistence false
log_dest stdout
log_type all
```

> ⚠️ Service `mosquitto` bawaan installer memakai config default
> (`C:\Program Files\mosquitto\mosquitto.conf`) yang bind ke `::1` dan
> menolak anonymous. Untuk pengembangan, jalankan manual dengan `-c` ke file
> di atas, atau ubah config service (butuh admin).

### Jalankan

```powershell
# foreground (mudah debug)
& "C:\Program Files\mosquitto\mosquitto.exe" -c "C:\Users\Ridwan\mosquitto\mosquitto.conf" -v
```

Jika service Windows aktif dan bentrok, hentikan dulu:

```powershell
Stop-Service mosquitto -Force   # butuh admin
```

### Verifikasi

```powershell
# subscribe di terminal 1
& "C:\Program Files\mosquitto\mosquitto_sub.exe" -t "vessel/#" -v

# terminal 2: jalankan vessel + buat GPS
# harus muncul pesan vessel/gps, vessel/gps/gprmc, dst.
```

### Matikan Docker

Karena tidak dipakai:

```powershell
# tutup Docker Desktop dari tray, atau
Stop-Process -Name "Docker Desktop" -Force -ErrorAction SilentlyContinue
```

## 4. Verifikasi cepat

```powershell
# Swagger
curl.exe -s -o NUL -w "%{http_code}`n" http://127.0.0.1:8080/swagger-ui/
curl.exe -s -o NUL -w "%{http_code}`n" http://127.0.0.1:8080/api-docs/openapi.json

# login
curl.exe -s -X POST http://127.0.0.1:8080/api/auth/login `
  -H "Content-Type: application/json" `
  -d '{\"username\":\"admin\",\"password\":\"admin\"}'

# set config GPS
curl.exe -s -X PATCH http://127.0.0.1:8080/api/gps/config `
  -H "Content-Type: application/json" `
  -d '{\"ip\":\"127.0.0.1\",\"port\":1883,\"update_rate\":1000,\"topics\":[\"vessel/gps\"]}'

# buat GPS
curl.exe -s -X POST http://127.0.0.1:8080/api/gps `
  -H "Content-Type: application/json" `
  -d '{\"latitude\":-6.2,\"longitude\":106.8,\"sog\":12.5,\"cog\":45,\"is_running\":true}'

# baca NMEA
curl.exe -s http://127.0.0.1:8080/api/gps
```

## 5. Uji WebSocket

```javascript
// ws_test.mjs — Node 22+
const ws = new WebSocket("ws://127.0.0.1:8081");
let n = 0;
ws.onmessage = () => n++;
setTimeout(() => { console.log("messages:", n); process.exit(0); }, 5000);
```

```powershell
node ws_test.mjs
```

- Broker hidup → ±15 pesan/5 detik.
- Broker mati → **0 pesan** (bug P0-01, akan diperbaiki).

## 6. Uji TCP client

1. Jalankan listener lokal di port 9100.
2. `PATCH /api/gps/config` dengan `tcp_enabled=true, ip=127.0.0.1, port=9100`.
3. Amati stream NMEA.

Contoh PowerShell:

```powershell
$l = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 9100)
$l.Start()
$t = $l.AcceptTcpClientAsync()
# ... set config lewat curl ...
$c = $t.Result; $s = $c.GetStream()
$buf = New-Object byte[] 4096
$s.Read($buf, 0, $buf.Length)
[System.Text.Encoding]::ASCII.GetString($buf)
```

## 7. Troubleshooting

| Gejala | Penyebab | Solusi |
|---|---|---|
| `Address already in use (os error 10048)` | Port dipakai | `Get-NetTCPConnection -LocalPort 8080`; stop prosesnya |
| WS tetap jalan walau broker mati | **Perilaku benar** (fail-safe) | Cek `dropped` di `/api/mqtt/config` |
| `MQTT connection error` | Broker tidak ada | Jalankan Mosquitto |
| `dropped` naik terus | Broker mati/lambat | Normal; pesan dibuang, pipeline tetap jalan |
| Log tidak muncul | `RUST_LOG` salah | `$env:RUST_LOG="debug"` |
| FE GPS kosong | Memakai URL lama | Gunakan `/api/gps?format=json` |
| `cargo` tidak dikenali | PATH belum refresh | Buka terminal baru |
| Build lambat | Debug profile | `cargo run --release` |

## 8. Menghentikan

```powershell
Get-Process vessel -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process mosquitto -ErrorAction SilentlyContinue | Stop-Process -Force
```

## 9. Struktur port

| Port | Protokol | Bind |
|---|---|---|
| 8080 | HTTP REST | `127.0.0.1` |
| 8081 | WebSocket | `127.0.0.1` |
| 9001 | TCP (stub) | `127.0.0.1` |
| 1883 | MQTT broker | `127.0.0.1` |

Untuk akses LAN: ubah bind ke `0.0.0.0` (P1-02).
