# 01 — Arsitektur

## 1. Arsitektur saat ini

### 1.1 Peta modul

```
src/
├── main.rs                  # wiring: state, channel, service, server
├── data/                    # model + DTO + shared type alias
│   ├── gps_data.rs          # GpsState, GpsConfig, request DTO, NMEA encoder
│   ├── gyro_data.rs
│   ├── weather_data.rs      # 5 state (temp/hum/wind/pressure/combined)
│   ├── navigation_data.rs   # SpeedState, DepthState
│   ├── compass_data.rs
│   ├── auth_data.rs         # admin/admin hardcoded
│   └── message_data.rs      # tidak dipakai
├── services/                # kalkulasi + publikasi
│   ├── *_service.rs         # 10 service (calc thread + pub task)
│   └── tcp_client.rs        # outbound NMEA stream
├── controllers/             # handler HTTP
├── routes/                  # definisi scope
├── utils/
│   ├── net.rs               # WS hub + TCP stub
│   ├── mqtt_manager.rs      # wrapper publish
│   └── *_calculate.rs       # fisika simulasi
└── config/websocket.rs      # DEAD CODE (tidak di-`mod`)
```

### 1.2 Alur data

```
[REST PATCH/POST]
      │
      ▼
[Shared State: Arc<RwLock<Option<State>>>]
      │
      ├──▶ [calc thread 100ms] ── update state
      │
      ├──▶ [pub task update_rate] ──▶ MQTT (JSON + NMEA)
      │                          └──▶ WebSocket broadcast
      │
      └──▶ [TCP client] ──▶ NMEA ke host eksternal
```

### 1.3 Model konkurensi

| Komponen | Mekanisme | Interval | Catatan |
|---|---|---|---|
| Kalkulasi | `std::thread` (10 thread) | 100 ms | Blocking, terpisah dari tokio |
| Publikasi | `tokio::spawn` + `select!` | `update_rate` (default 1000 ms) | Rentan deadlock MQTT |
| TCP client | `tokio::spawn` | `update_rate` | Depth dibaca di luar loop |
| WS server | `tokio::spawn` per koneksi | event-driven | Tanpa ping/pong handler |
| MQTT eventloop | `tokio::spawn` | continuous | Reconnect 1 detik |
| HTTP | actix workers | per request | Blocking `std::RwLock` |

### 1.4 Model state

```
SharedGpsState  = Arc<RwLock<Option<GpsState>>>
SharedGpsConfig = Arc<RwLock<GpsConfig>>
```

- `Option` = "sensor belum dibuat".
- `is_running` = "simulasi aktif" (bukan "data valid").
- Config dan state dipisah — **ini keputusan yang benar**.

## 2. Penilaian

### 2.1 Yang sudah benar

| # | Hal | Alasan |
|---|---|---|
| 1 | Pemisahan `data/services/controllers/routes` | Mudah dipahami, mirip MVC Express |
| 2 | State vs Config terpisah | Config tidak ikut ter-serialize ke response |
| 3 | Kalkulasi di OS thread | Menghindari blocking runtime async |
| 4 | NMEA encoder di domain type | `to_gprmc()` kohesif |
| 5 | `enabled_sentences` fleksibel | `None`/`[]`/list punya semantik jelas |
| 6 | `web::Data` untuk shared state | Benar secara konsep |
| 7 | `configure()` per fitur | Route modular |

### 2.2 Yang belum idiomatik

| # | Masalah | Dampak |
|---|---|---|
| 1 | 25+ `web::Data` terpisah | Wiring di `main.rs` panjang, sulit diuji |
| 2 | `impl Responder` di semua handler | Error type tidak bisa disatukan |
| 3 | Tidak ada `ApiError` | Semua error jadi 500/panic |
| 4 | `unwrap()` pada lock | Panic jika poisoned |
| 5 | `std::sync::RwLock` di async | Bisa menahan worker thread |
| 6 | Tidak ada `tracing_subscriber::init()` | Log hilang |
| 7 | Config hardcoded | Tidak bisa deploy tanpa recompile |
| 8 | Domain type = API DTO = MQTT payload | Coupling; perubahan satu merusak semua |
| 9 | Duplikasi 10 service | Bug diperbaiki 10× |
| 10 | Tidak ada trait `Sensor` | Penambahan sensor = copy-paste |
| 11 | Tidak ada graceful shutdown | Thread tidak berhenti |
| 12 | `src/config/websocket.rs` dead code | Menyesatkan |

## 3. Arsitektur target

### 3.1 Prinsip

1. **Modular monolith** — satu binary, batas modul jelas. Bukan microservice.
2. **Vertical slice** — satu folder per fitur (gps, gyro, weather, ...) berisi
   `model.rs`, `sim.rs`, `nmea.rs`, `routes.rs`, `service.rs`.
3. **Satu `AppState`** — semua runtime di-inject sekali.
4. **Satu `ApiError`** — semua handler return `Result<HttpResponse, ApiError>`.
5. **Domain ≠ DTO ≠ payload** — tiga tipe terpisah bila perlu.
6. **Supervisor** — satu tempat yang mengelola lifecycle semua task.
7. **Fail-safe** — I/O eksternal tidak boleh menghentikan pipeline.
8. **Observable** — tracing + metrics sejak awal.

### 3.2 Struktur target

```
src/
├── main.rs
├── app.rs                    # build_app(AppState) -> App
├── state.rs                  # AppState
├── error.rs                  # ApiError + ResponseError
├── config.rs                 # AppConfig (env + file)
├── telemetry.rs              # tracing init, metrics
├── shutdown.rs               # CancellationToken
├── features/
│   ├── gps/
│   │   ├── model.rs          # GpsState, GpsConfig
│   │   ├── sim.rs            # calculate_next
│   │   ├── nmea.rs           # to_gprmc, ...
│   │   ├── service.rs        # publication
│   │   └── routes.rs         # handlers + OpenAPI paths
│   ├── gyro/
│   ├── weather/
│   ├── navigation/           # speed + depth
│   └── compass/
├── transport/
│   ├── mqtt.rs               # publisher + health + reconnect
│   ├── ws.rs                 # hub + ping/pong + subscription
│   └── tcp.rs                # server + client
└── docs/
    └── openapi.rs            # #[derive(OpenApi)]
```

### 3.3 Diagram target

```
                    ┌──────────────────────────┐
   REST  ──────────▶│  features/*/routes.rs    │
   Swagger ────────▶│  (validasi + ApiError)   │
                    └────────────┬─────────────┘
                                 │
                    ┌────────────▼─────────────┐
                    │  AppState (Arc)          │
                    │  registry: HashMap<Id,   │
                    │            SensorRuntime>│
                    └────────────┬─────────────┘
                                 │
              ┌──────────────────┼──────────────────┐
              ▼                  ▼                  ▼
      ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
      │ Simulation   │   │ NMEA Encoder │   │ Scenario     │
      │ Scheduler    │   │ (validated)  │   │ Engine       │
      │ (1 task)     │   └──────┬───────┘   └──────────────┘
      └──────────────┘          │
                    ┌───────────┼───────────┐
                    ▼           ▼           ▼
                 ┌─────┐    ┌─────┐    ┌──────┐
                 │MQTT │    │ WS  │    │ TCP  │
                 │pub  │    │ hub │    │ srv  │
                 └─────┘    └─────┘    └──────┘
```

### 3.4 Perubahan kunci per area

| Area | Sekarang | Target |
|---|---|---|
| State | 25 `web::Data` | 1 `AppState` |
| Error | `unwrap` | `ApiError` |
| Lock | `std::sync::RwLock` | `tokio::sync::RwLock` / `parking_lot` |
| Log | `println!` + tracing mati | `tracing` JSON |
| Config | hardcoded | env + `config.toml` |
| Scheduler | 10 OS thread | 1 tokio task + `JoinSet` |
| MQTT | satu client hardcoded | `MqttPublisher` dengan health + `try_publish` |
| WS | broadcast buta | hub + filter + ping/pong |
| TCP | client outbound + stub server | client + server nyata |
| Test | 0 | unit + integration |
| Docs | tidak ada | OpenAPI + folder ini |

## 4. Strategi migrasi

Gunakan **strangler pattern**:

1. Jangan tulis ulang semuanya sekaligus.
2. Tambahkan `AppState` **di samping** `web::Data` lama, migrasikan handler satu per satu.
3. Pertahankan path & response lama sampai FE siap.
4. Setelah semua handler pindah, hapus `web::Data` lama.
5. Baru refactor struktur folder.

**Aturan:** setiap langkah harus **tetap bisa di-build dan dijalankan**.

## 5. Yang tidak perlu diubah

- Konsep `Option<State>`.
- Pemisahan config/state.
- NMEA encoder di domain type.
- `enabled_sentences` semantik.
- Port default (bisa dijadikan config, tapi nilainya tetap).
