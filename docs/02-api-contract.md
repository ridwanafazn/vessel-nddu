# 02 — Kontrak REST API

> **Sumber kebenaran** untuk backend dan FE.
> Setiap perubahan wajib update dokumen ini **sebelum** kode.

## 1. Aturan umum

| Item | Nilai |
|---|---|
| Base URL (lokal) | `http://127.0.0.1:8080` |
| Format | `application/json` (kecuali `GET /api/gps`) |
| Envelope sukses | `{ "message": string, "data": object }` |
| Envelope error | `{ "message": string }` |
| Auth | Belum ada middleware; hanya `POST /api/auth/login` |
| CORS | `allow_any_origin` (akan diperketat) |
| Versioning | Belum ada. Usulan: `/api/v2/...` untuk breaking change |

### Status code yang dipakai

| Code | Arti di proyek ini |
|---|---|
| 200 | OK |
| 201 | Resource dibuat |
| 400 | Config belum lengkap (compass) |
| 401 | Login gagal |
| 404 | State belum dibuat |
| 409 | Config belum lengkap / instance sudah ada |
| 500 | Panic / error internal (belum terstruktur) |

## 2. Matriks endpoint

| # | Method | Path | Body | Response | FE pakai |
|---|---|---|---|---|---|
| 1 | POST | `/api/auth/login` | `{username,password}` | `{success,message,username?}` | ✅ |
| 2 | GET | `/api/gps` | — | **`text/plain` NMEA GPRMC** | ⚠️ |
| 3 | POST | `/api/gps` | `CreateGpsRequest` | `{message,data}` | ✅ |
| 4 | PATCH | `/api/gps` | `UpdateGpsRequest` | `{message,data}` | ✅ |
| 5 | DELETE | `/api/gps` | — | `{message}` | ✅ |
| 6 | GET | `/api/gps/config` | — | `{message,data}` | ✅ |
| 7 | POST/PATCH | `/api/gps/config` | `UpdateGpsConfigRequest` | `{message,data}` | ✅ |
| 8 | DELETE | `/api/gps/config` | — | `{message}` | ✅ |
| 9 | GET | `/api/gyro` | — | `{message,data}` | ✅ |
| 10 | POST | `/api/gyro` | `CreateGyroRequest` | `{message,data}` | ✅ |
| 11 | PATCH | `/api/gyro` | `UpdateGyroRequest` | `{message,data}` | ✅ |
| 12 | DELETE | `/api/gyro` | — | `{message}` | ✅ |
| 13 | GET/POST/PATCH/DELETE | `/api/gyro/config` | `UpdateGyroConfigRequest` | `{message,data}` | ✅ |
| 14 | GET | `/api/weather` | — | `{message,data}` | ✅ |
| 15 | POST | `/api/weather` | `CreateWeatherRequest` | `{message,data}` | ✅ |
| 16 | PATCH | `/api/weather` | `UpdateWeatherRequest` | `{message,data}` | ✅ |
| 17 | DELETE | `/api/weather` | — | `{message}` | ✅ |
| 18 | GET/POST/PATCH/DELETE | `/api/weather/config` | `UpdateWeatherConfigRequest` | `{message,data}` | ✅ |
| 19 | GET | `/api/compass` | — | `{message,data}` | ✅ |
| 20 | POST | `/api/compass` | `{heading}` | `{message,data}` | ✅ |
| 21 | PATCH | `/api/compass` | `{heading?,is_running?}` | `{message,data}` | ✅ |
| 22 | DELETE | `/api/compass` | — | `{message}` | ✅ |
| 23 | GET/POST/PATCH/DELETE | `/api/compass/config` | `{ip,port,username,password,topics,update_rate}` | `{message,data}` | ✅ |
| 24 | GET | `/api/speed` | — | `{message,data}` | ✅ |
| 25 | POST | `/api/speed` | `{speed_through_water,is_running}` | `{message,data}` | ✅ |
| 26 | PATCH | `/api/speed` | `{speed_through_water?,is_running?}` | `{message,data}` | ✅ |
| 27 | DELETE | `/api/speed` | — | `{message}` | ✅ |
| 28 | GET | `/api/depth` | — | `{message,data}` | ✅ |
| 29 | POST | `/api/depth` | `{ocean_water_depth,constant_clearance,is_running}` | `{message,data}` | ✅ |
| 30 | PATCH | `/api/depth` | `{ocean_water_depth?,constant_clearance?,is_running?}` | `{message,data}` | ✅ |
| 31 | DELETE | `/api/depth` | — | `{message}` | ✅ |
| 32 | GET/POST/PATCH/DELETE | `/api/navigation/config` | `UpdateNavigationConfigRequest` | `{message,data}` | ✅ |
| 33 | GET/POST/PATCH/DELETE | `/api/temperature` | `{air_temperature,is_running}` | `{message,data}` | ❌ |
| 34 | GET/POST/PATCH/DELETE | `/api/humidity` | `{air_humidity,is_running}` | `{message,data}` | ❌ |
| 35 | GET/POST/PATCH/DELETE | `/api/wind` | `{wind_direction_relative,wind_velocity,is_running}` | `{message,data}` | ❌ |
| 36 | GET/POST/PATCH/DELETE | `/api/pressure` | `{air_pressure,is_running}` | `{message,data}` | ❌ |

## 3. Skema data

### 3.1 GPS

```jsonc
// GpsState (response)
{
  "latitude": -6.2,
  "longitude": 106.8,
  "sog": 12.5,          // knots
  "cog": 45.0,          // derajat
  "variation": 0.6484,  // derajat (WMM, + = East)
  "is_running": true,
  "last_update": "2026-10-31T12:23:31.327Z"
}
```

```jsonc
// CreateGpsRequest
{ "latitude": -6.2, "longitude": 106.8, "sog": 12.5, "cog": 45.0, "is_running": true }
// UpdateGpsRequest: semua field opsional
```

```jsonc
// GpsConfig
{
  "ip": "127.0.0.1",
  "port": 1883,
  "username": "user",
  "password": "pass",
  "update_rate": 1000,
  "topics": ["vessel/gps"],
  "enabled_sentences": ["GPRMC", "GPGLL", "GPVTG", "GPZDA"],
  "tcp_enabled": false
}
```

> ⚠️ `ip/port/username/password` **belum dipakai** untuk koneksi MQTT (hardcoded
> `127.0.0.1:1883`). Lihat P0-02.

### 3.2 Gyro

```jsonc
{ "yaw": 90.0, "pitch": 1.2, "roll": -2.0, "yaw_rate": 0.5,
  "is_running": true, "last_update": "..." }
```

### 3.3 Weather (gabungan)

```jsonc
{ "air_temperature": 28.5, "air_humidity": 75.0,
  "wind_direction_relative": 45.0, "wind_velocity": 12.0,
  "air_pressure": 1013.2, "is_running": true, "last_update": "..." }
```

### 3.4 Compass

```jsonc
// response menambahkan deviation & variation dari konstanta
{ "heading": 90.0, "deviation": 1.0, "variation": 3.0,
  "is_running": false, "last_update": "31 Oct 2026, 19:23:34" }
```

> ⚠️ Format `last_update` **berbeda** dari sensor lain (string lokal, bukan RFC3339).

### 3.5 Speed & Depth

```jsonc
{ "speed_through_water": 20.0, "is_running": true, "last_update": "..." }
{ "ocean_water_depth": 600.0, "constant_clearance": 10.0,
  "under_keel_clearance": 590.0, "is_running": true, "last_update": "..." }
```

## 4. Kompatibilitas FE — **PENTING**

FE (React + Tauri) memakai **polling axios**, bukan WebSocket.

| Endpoint | FE harapkan | Backend kirim | Status |
|---|---|---|---|
| `GET /api/gps` | JSON `{data:{latitude,...}}` | `text/plain` NMEA | 🔴 **MISMATCH** |
| `GET /api/gyro` | `{data:{...}}` | `{data:{...}}` | ✅ |
| `GET /api/weather` | `{data:{...}}` | `{data:{...}}` | ✅ |
| `GET /api/compass` | `{data:{...}}` | `{data:{...}}` | ✅ |
| `GET /api/speed` | `{data:{...}}` | `{data:{...}}` | ✅ |
| `GET /api/depth` | `{data:{...}}` | `{data:{...}}` | ✅ |
| `GET /api/navigation/config` | `{data:{...}}` | `{data:{...}}` | ✅ |

**Rekomendasi (non-breaking):**

```
GET /api/gps            → tetap NMEA text/plain (untuk integrator NMEA)
GET /api/gps?format=json→ JSON {message,data}   (untuk FE)
GET /api/gps/state      → JSON {message,data}   (alternatif eksplisit)
```

Pilih **satu** dan catat di `10-decisions.md`. Rekomendasi: `?format=json`,
karena FE hanya perlu mengubah URL dan backend tidak menambah route baru.

## 5. Contoh curl

```bash
# login
curl -X POST http://127.0.0.1:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin"}'

# set config GPS
curl -X PATCH http://127.0.0.1:8080/api/gps/config \
  -H 'Content-Type: application/json' \
  -d '{"ip":"127.0.0.1","port":1883,"update_rate":1000,"topics":["vessel/gps"]}'

# buat GPS
curl -X POST http://127.0.0.1:8080/api/gps \
  -H 'Content-Type: application/json' \
  -d '{"latitude":-6.2,"longitude":106.8,"sog":12.5,"cog":45,"is_running":true}'

# baca NMEA
curl http://127.0.0.1:8080/api/gps
```

## 6. Rencana Swagger (Fase 1)

- Library: `utoipa` 5.x + `utoipa-swagger-ui` 9.x (fitur `actix-web`).
- Endpoint: `GET /api-docs/openapi.json`, `GET /swagger-ui/`.
- Semua DTO wajib `#[derive(ToSchema)]`.
- Semua handler wajib `#[utoipa::path(...)]` dengan response 200/400/404/409.
- Dokumentasi WS/MQTT **tidak** masuk OpenAPI → ada di `03-realtime-contract.md`.

## 7. Aturan perubahan kontrak

1. **Aditif dulu**: tambah field opsional, jangan hapus/rename.
2. **Breaking** → `/api/v2` + masa transisi.
3. Setiap perubahan: update dokumen ini + `11-frontend-integration.md`.
4. FE harus bisa jalan dengan backend versi sebelumnya minimal 1 rilis.
