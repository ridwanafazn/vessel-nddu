# 03 — Kontrak Real-time: WebSocket, MQTT, NMEA, TCP

## 1. WebSocket

| Item | Nilai |
|---|---|
| URL | `ws://127.0.0.1:8081` |
| Format | JSON text frame |
| Auth | ❌ belum ada |
| Ping/pong | ❌ belum ditangani (klien bisa timeout) |
| Subscribe/filter | ❌ belum ada (semua klien dapat semua) |
| Backpressure | ❌ unbounded channel |

### 1.1 Tipe pesan

```jsonc
// GPS — satu-satunya yang mengirim NMEA sekaligus
{
  "type": "gps_update",
  "json": { "latitude": -6.2, "longitude": 106.8, "sog": 12.5,
            "cog": 45.0, "variation": 0.65, "is_running": true,
            "last_update": "..." },
  "gprmc": "$GPRMC,...*26",
  "gpgll": "$GPGLL,...*73",
  "gpvtg": "$GPVTG,...*23",
  "gpzda": "$GPZDA,...*6F"
}
```

```jsonc
{ "type": "gyro_update",        "data": { "yaw": 90, "pitch": 1.2, "roll": -2, "yaw_rate": 0.5, ... } }
{ "type": "weather_update",     "data": { "air_temperature": 28.5, ... } }
{ "type": "temperature_update", "data": { "air_temperature": 28.5, ... } }
{ "type": "humidity_update",    "data": { "air_humidity": 75.0, ... } }
{ "type": "wind_update",        "data": { "wind_direction_relative": 45, "wind_velocity": 12, ... } }
{ "type": "pressure_update",    "data": { "air_pressure": 1013.2, ... } }
{ "type": "speed_update",       "data": { "speed_through_water": 20.0, ... } }
{ "type": "depth_update",       "data": { "ocean_water_depth": 600.0, ... } }
{ "type": "compass_update",     "data": { "heading": 90, "deviation": 1, "variation": 3, "last_update": "..." } }
```

> Perhatikan: `gps_update` memakai key `json`, yang lain `data`. **Tidak konsisten.**
> Usulan v2: semua pakai `data`, tambahkan `ts` (epoch ms) dan `vessel_id`.

### 1.2 Usulan protokol v2 (Fase 3)

```jsonc
// client → server
{ "op": "subscribe", "topics": ["gps", "depth"] }
{ "op": "unsubscribe", "topics": ["weather"] }
{ "op": "ping" }

// server → client
{ "type": "gps", "ts": 1761912345678, "data": { ... }, "nmea": { "gprmc": "..." } }
{ "type": "pong", "ts": 1761912345678 }
{ "type": "error", "code": "BAD_OP", "message": "..." }
```

## 2. MQTT

| Item | Nilai |
|---|---|
| Broker | Global via env `MQTT_HOST:MQTT_PORT` (default `127.0.0.1:1883`) |
| Ubah runtime | `PATCH /api/mqtt/config` (reconnect otomatis) |
| Client ID | `MQTT_CLIENT_ID` (default `vessel-client`) |
| QoS | `AtLeastOnce` (1) |
| Retain | `false` |
| LWT | ❌ belum ada |
| Auth | ✅ `MQTT_USERNAME` / `MQTT_PASSWORD` |
| Reconnect | Supervisor, retry 2 detik |
| **Fail-safe** | ✅ Queue bounded 1024; drop dihitung, tidak memblokir |

### 2.1 Peta topik

| Sensor | Topik JSON | Topik NMEA |
|---|---|---|
| GPS | `vessel/gps` | `vessel/gps/gprmc`, `/gpgll`, `/gpvtg`, `/gpzda` |
| Gyro | `vessel/gyro` | `vessel/gyro/rot`, `/hdt`, `/hdg`, `/xdr` |
| Weather (gabungan) | `vessel/weather/data` | — |
| Temperature | `vessel/weather/temperature` | `vessel/weather/wimtw` |
| Humidity | `vessel/weather/humidity` | — |
| Wind | `vessel/weather/wind` | `vessel/weather/wimwv`, `/wimwd` |
| Pressure | `vessel/weather/pressure` | — |
| Speed | `vessel/navigation/speed` | `vessel/navigation/vhw` |
| Depth | `vessel/navigation/depth` | `vessel/navigation/dbt`, `/dpt`, `/dbk` |
| Compass | `vessel/compass/heading` | — |

> Prefix diambil dari `topics[0]` config. Jika kosong → default di atas.
> ⚠️ Weather gabungan dan sensor individual **berbagi prefix default** → tabrakan.

### 2.2 Usulan v2

```
vessel/{vessel_id}/{sensor}/{format}
contoh: vessel/nddu-01/gps/nmea/gprmc
        vessel/nddu-01/gps/json
        vessel/nddu-01/_meta/status   (LWT: online/offline)
```

Tambahkan:
- `retain = true` untuk `_meta/status`.
- `LWT` = `{"status":"offline"}`.
- `QoS` configurable per sensor.
- `client_id` unik per instance.

## 3. NMEA 0183

### 3.1 Format wajib

```
$TTSSS,field1,field2,...*HH\r\n
  │ │ └── formatter (3 huruf)
  │ └──── talker (2 huruf)
  └────── start
```

**Minimal 5 karakter sebelum koma.** Ini yang dilanggar beberapa sentence.

### 3.2 Daftar sentence

| Sentence | Talker sekarang | Format | Contoh | Status |
|---|---|---|---|---|
| GPRMC | GP | `time,status,lat,N,lon,E,sog,cog,date,var,E,mode` | `$GPRMC,122334.35,A,0611.9924,S,10648.0077,E,12.5,45.0,311026,0.6,E,A*26` | ✅ |
| GPGLL | GP | `lat,N,lon,E,time,status,mode` | `$GPGLL,0611.9924,S,10648.0077,E,122334.35,A,A*73` | ✅ |
| GPVTG | GP | `cogT,T,cogM,M,sogN,N,sogK,K,mode` | `$GPVTG,45.0,T,44.4,M,12.5,N,23.2,K,A*23` | ✅ |
| GPZDA | GP | `time,dd,mm,yyyy,00,00` | `$GPZDA,122334.35,31,10,2026,00,00*6F` | ✅ |
| HEHDT | HE | `heading,T` | `$HEHDT,90.00,T*..` | ✅ |
| HEHDG | HE | `heading,dev,E,var,E` | `$HEHDG,90.00,0.0,E,0.0,E*..` | ✅ |
| HEXDR | HE | `A,pitch,D,PTCH,A,roll,D,ROLL,A,yaw,D,YAW` | — | ✅ |
| HEROT | HE | `rate,A` | `$HEROT,0.50,A*..` | ✅ |
| WIMWV | WI | `angle,R,speed,N,A` | `$WIMWV,45.0,R,12.0,N,A*..` | ✅ |
| WIMWD | WI | `dirT,T,dirM,M,speedN,N,speedM,M` | — | ✅ |
| WIMTW | WI | `temp,C` | `$WIMTW,28.5,C*..` | ✅ |
| VHW | II | `headingT,T,headingM,M,sogN,N,sogK,K` | `$IIVHW,0.0,T,...` | ✅ |
| DBT | SD | `feet,f,meters,M,fathoms,F` | `$SDDBT,2024.3,f,...` | ✅ |
| DPT | SD | `depth,offset` | `$SDDPT,617.0,10.0` | ✅ |
| DBK | SD | `feet,f,meters,M,fathoms,F` | `$SDDBK,2012.0,f,...` | ✅ |

**Perbaikan P0-03:** tambahkan talker.
- Speed → `$IIVHW` (integrated instrumentation) atau `$SDVHW` (sounder).
- Depth → `$SDDBT`, `$SDDPT`, `$SDDBK`.
- Sesuaikan dengan perangkat yang disimulasikan; dokumentasikan pilihannya.

### 3.3 Sentence yang belum ada (usulan)

| Sentence | Kegunaan | Prioritas |
|---|---|---|
| `GPGGA` | Fix quality, altitude, satellite count | 🔴 Tinggi (banyak konsumen wajib) |
| `GPGSA` | DOP, active satellites | 🟠 |
| `GPGSV` | Satellites in view | 🟠 |
| `GNGNS` | Multi-GNSS fix | 🟡 |
| `HDM` | Heading magnetic (kompas) | 🔴 Tinggi |
| `MDA` | Meteorological composite | 🟡 |
| `MMB` | Barometer | 🟡 |
| `RSA` | Rudder angle | 🟡 |
| `RPM` | Engine RPM | 🟡 |
| `VBW` | Dual ground/water speed | 🟡 |
| `VLW` | Distance traveled | 🟡 |
| `ALR` / `ACK` | Alarm | 🟡 |
| `AIVDM` / `AIVDO` | AIS | 🟠 (nilai jual) |

### 3.4 Validasi checksum

XOR semua byte antara `$` dan `*`. Contoh:

```
body = "GPRMC,122334.35,A,0611.9924,S,10648.0077,E,12.5,45.0,311026,0.6,E,A"
xor  = 0x26
output = "$" + body + "*26\r\n"
```

## 4. TCP

### 4.1 Client (outbound) — **berfungsi**

| Item | Nilai |
|---|---|
| Target | `gps_config.ip:gps_config.port` |
| Aktif jika | `gps_config.tcp_enabled == true` |
| Interval | `gps_config.update_rate` |
| Isi | Semua sentence yang enabled dari GPS, gyro, wind, temp, speed, depth |
| Reconnect | 5 detik |
| Status | ✅ Depth dibaca di dalam loop; NMEA divalidasi sebelum dikirim |
| Catatan | `enabled_sentences` GPS masih mengatur **semua** sensor (coupling, P2) |

### 4.2 Server (inbound, port 9001) — **stub**

```rust
// utils/net.rs — hanya echo
let _ = socket.write_all(b"TCP received").await;
```

Belum ada server NMEA nyata. Usulan Fase 3:
- Terima koneksi, kirim stream NMEA sesuai filter klien.
- Protokol opsional: klien kirim `SUB GPRMC,GPVTG\n` untuk memilih sentence.
- Atau sederhana: semua sentence, seperti client mode.

> ⚠️ Log di `main.rs` mencetak `tcp://127.0.0.1:9003`, padahal bind ke `9001`.

## 5. Kompatibilitas

| Konsumen | Protokol yang dipakai |
|---|---|
| FE React + Tauri | REST polling |
| Integrator NMEA | TCP client / MQTT per-sentence |
| Dashboard LEN | MQTT JSON |
| Demo portofolio | REST + WS + Swagger |
