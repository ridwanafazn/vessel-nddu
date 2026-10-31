# Vessel NDDU

Ship sensor simulator & **NMEA 0183 / MQTT / WebSocket / REST** gateway,
written in Rust (Actix-web).

Dirancang untuk menggantikan hardware sensor kapal saat pengujian perangkat
lunak maritim: menghasilkan data navigasi & cuaca yang realistis, lalu
mendistribusikannya lewat REST, WebSocket, MQTT, dan TCP.

![Rust](https://img.shields.io/badge/Rust-1.99-000?logo=rust)
![Actix-web](https://img.shields.io/badge/Actix--web-4-2D5F8A)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Fitur

| Kategori | Sensor | Output NMEA |
|---|---|---|
| Navigasi | GPS | `GPRMC`, `GPGLL`, `GPVTG`, `GPZDA` |
| Navigasi | Gyro / attitude | `HDT`, `HDG`, `ROT`, `XDR` |
| Navigasi | Speed log | `VHW` |
| Navigasi | Depth sounder | `DBT`, `DPT`, `DBK` |
| Navigasi | Kompas magnetik | JSON |
| Cuaca | Suhu | `MTW` |
| Cuaca | Angin | `MWV`, `MWD` |
| Cuaca | Kelembapan & tekanan | JSON |
| Cuaca | Gabungan | JSON |

Kanal distribusi:

- **REST** — kontrol & konfigurasi simulator
- **Swagger UI** — dokumentasi API interaktif
- **WebSocket** — update real-time
- **MQTT** — JSON per sensor + NMEA per sentence
- **TCP** — streaming NMEA ke sistem eksternal (ECDIS/VDR/VMS)

---

## Arsitektur

```
REST / Swagger ──▶ controllers ──▶ shared state (Arc<RwLock>)
                                        │
                 ┌──────────────────────┼──────────────────────┐
                 ▼                      ▼                      ▼
        calculation thread      publication task        TCP client
        (100 ms, fisika)        (update_rate, MQTT+WS)  (NMEA stream)
```

- State & config dipisah (`State` vs `Config`).
- MQTT **fail-safe**: broker mati tidak menghentikan WebSocket/TCP.
- NMEA divalidasi sebelum dikirim.
- Konfigurasi via environment + endpoint runtime.

Detail: [`docs/01-architecture.md`](docs/01-architecture.md)

---

## Quick start

### 1. Prasyarat

| Komponen | Versi |
|---|---|
| Rust | ≥ 1.99 (MSVC toolchain di Windows) |
| Mosquitto | ≥ 2.0 (broker MQTT) |

```powershell
winget install Rustlang.Rustup
winget install EclipseFoundation.Mosquitto
```

### 2. Broker MQTT

```powershell
& "C:\Program Files\mosquitto\mosquitto.exe" -c "C:\Users\<you>\mosquitto\mosquitto.conf" -v
```

Contoh `mosquitto.conf`:

```
listener 1883 127.0.0.1
allow_anonymous true
```

### 3. Jalankan

```powershell
git clone <repo-url>
cd vessel-nddu
cargo run
```

Buka:

- API: <http://127.0.0.1:8080>
- Swagger: <http://127.0.0.1:8080/swagger-ui/>
- OpenAPI JSON: <http://127.0.0.1:8080/api-docs/openapi.json>
- WebSocket: `ws://127.0.0.1:8081`

### 4. Coba cepat

```bash
# login
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin"}'

# konfigurasi GPS
curl -X PATCH http://127.0.0.1:8080/api/gps/config \
  -H 'Content-Type: application/json' \
  -d '{"ip":"127.0.0.1","port":1883,"update_rate":1000,"topics":["vessel/gps"]}'

# buat GPS
curl -X POST http://127.0.0.1:8080/api/gps \
  -H 'Content-Type: application/json' \
  -d '{"latitude":-6.2,"longitude":106.8,"sog":12.5,"cog":45,"is_running":true}'

# baca NMEA / JSON
curl http://127.0.0.1:8080/api/gps
curl 'http://127.0.0.1:8080/api/gps?format=json'
```

---

## Konfigurasi

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

Salin `.env.example` lalu sesuaikan. Gunakan `0.0.0.0` untuk akses LAN.

MQTT bisa diubah saat runtime:

```bash
PATCH /api/mqtt/config   { "host": "broker.local", "port": 1883 }
```

---

## Dokumentasi

| Dokumen | Isi |
|---|---|
| [docs/02-api-contract.md](docs/02-api-contract.md) | Kontrak REST lengkap |
| [docs/03-realtime-contract.md](docs/03-realtime-contract.md) | WS, MQTT, NMEA, TCP |
| [docs/06-tasks.md](docs/06-tasks.md) | Roadmap & progres |
| [docs/07-rust-actix-guide.md](docs/07-rust-actix-guide.md) | Panduan idiomatik |
| [docs/09-runbook.md](docs/09-runbook.md) | Operasional & troubleshooting |
| [docs/](docs/) | Semua dokumentasi |

---

## MQTT topics

| Sensor | JSON | NMEA |
|---|---|---|
| GPS | `vessel/gps` | `vessel/gps/{gprmc,gpgll,gpvtg,gpzda}` |
| Gyro | `vessel/gyro` | `vessel/gyro/{rot,hdt,hdg,xdr}` |
| Speed | `vessel/navigation/speed` | `vessel/navigation/vhw` |
| Depth | `vessel/navigation/depth` | `vessel/navigation/{dbt,dpt,dbk}` |
| Cuaca | `vessel/weather/*` | `vessel/weather/{wimwv,wimwd,wimtw}` |
| Kompas | `vessel/compass/heading` | — |

Prefix mengikuti `topics[0]` pada config sensor.

---

## Testing

```powershell
cargo test          # unit test NMEA & checksum
cargo clippy        # informational; target clippy-clean di Fase 2
```

---

## Struktur proyek

```
src/
├── main.rs          # wiring
├── config.rs        # env config
├── telemetry.rs     # tracing
├── docs/            # OpenAPI
├── data/            # model, DTO, NMEA encoder
├── services/        # simulasi + publikasi
├── controllers/     # handler REST
├── routes/          # route definitions
└── utils/           # MQTT, WS, NMEA, fisika
```

---

## Credits

- **2025 — magang PT LEN Industri**: Ridwan Ahmad Fauzan (backend API, WebSocket, MQTT, simulasi sensor).
- **Lanjutan**: Fijar (intern berikutnya, branch `other-ver`).
- **2026 — refactor & portofolio**: Ridwan Ahmad Fauzan (Swagger, test, CI, Docker, perbaikan bug).

Detail: [CONTRIBUTORS.md](CONTRIBUTORS.md).

---

## Lisensi

MIT — lihat [LICENSE](LICENSE).
