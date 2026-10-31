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

Output yang benar:

```
 Server starting...
 Starting background services...
 MQTT Client connecting...
 Background services running.
 API Server started on http://127.0.0.1:8080
 WebSocket server started on ws://127.0.0.1:8081
 TCP server started on tcp://127.0.0.1:9001
```

> Saat ini log TCP mencetak `9003` (bug P1-13).

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

## 4. Verifikasi cepat (tanpa broker)

```powershell
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
| `Address already in use (os error 10048)` | Port 8080 dipakai | `Get-NetTCPConnection -LocalPort 8080`; stop prosesnya |
| WS 0 pesan | Broker MQTT mati (P0-01) | Jalankan Mosquitto |
| `MQTT eventloop error: ConnectionRefused` | Broker tidak ada | Install/jalankan Mosquitto |
| `$DBT` tidak berubah | Bug P0-04 | Akan diperbaiki |
| `$VHW` ditolak parser | Bug P0-03 (talker hilang) | Akan diperbaiki |
| Log `tracing` tidak muncul | Subscriber tidak di-init | Bug P1-01 |
| FE GPS kosong | `GET /api/gps` mengembalikan teks | Gunakan `?format=json` (P1-14) |
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
