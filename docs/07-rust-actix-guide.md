# 07 — Panduan Rust + Actix-web (dari latar belakang Express)

Dokumen ini menjawab: **"apakah arsitektur saya buruk?"** dan **"bagaimana yang benar?"**

---

## 1. Jawaban singkat

**Tidak buruk.** Struktur `data/services/controllers/routes` adalah pola yang sah
dan mudah dipahami. Yang bermasalah adalah **runtime pattern**, bukan layering.

Skor:

| Aspek | Skor |
|---|---|
| Struktur modul | 7/10 |
| Idiomatik Actix | 5/10 |
| Error handling | 3/10 |
| Concurrency safety | 4/10 |
| Observability | 2/10 |
| Testability | 1/10 |
| Config | 3/10 |
| API design | 6/10 |

Target realistis setelah Fase 2: **8/10 di semua aspek**.

---

## 2. Mental model: Express vs Actix

| Express | Actix-web | Catatan |
|---|---|---|
| `app = express()` | `HttpServer::new(\|\| App::new())` | App dibuat **per worker** |
| `app.use(mw)` | `.wrap(mw)` | Urutan penting |
| `app.get('/x', h)` | `.route("/x", web::get().to(h))` | — |
| `express.Router()` | `web::scope("/api")` | — |
| `req.body` | `web::Json<T>` | Otomatis 400 jika invalid |
| `req.query` | `web::Query<T>` | — |
| `req.params` | `web::Path<T>` | — |
| `res.json(x)` | `HttpResponse::Ok().json(x)` | — |
| `res.status(404)` | `HttpResponse::NotFound()` | — |
| `next(err)` | `Err(ApiError::...)` | Compile-time |
| `app.locals` | `web::Data<T>` | **Satu `AppState`** |
| `setInterval` | `tokio::time::interval` | Butuh cancellation |
| `try/catch` | `Result<T, E>` + `?` | — |
| `supertest` | `actix_web::test::init_service` | — |
| Single thread | **Multi-thread** | **Jangan blocking** |

### Perbedaan paling fatal

Express: blocking itu normal (satu thread, event loop).
Actix: blocking **menahan worker thread** → throughput turun / deadlock.

Contoh di proyek ini:

```rust
// ❌ blocking lock di handler async
let guard = config.read().unwrap();       // std::sync
tokio::time::sleep(...).await;            // guard masih hidup? bahaya
```

```rust
// ✅
let guard = state.config.read().await;    // tokio::sync
let value = guard.clone();
drop(guard);                              // lepas sebelum await
tokio::time::sleep(...).await;
```

---

## 3. Best practice 1 — Satu `AppState`

### Sekarang

```rust
// main.rs — 25 baris app_data
.app_data(web::Data::new(shared_gps_config.clone()))
.app_data(web::Data::new(shared_gps_state.clone()))
// ... 23 lagi
```

### Idiomatik

```rust
#[derive(Clone)]
pub struct AppState {
    pub gps: GpsRuntime,
    pub gyro: GyroRuntime,
    pub weather: WeatherRuntime,
    pub navigation: NavigationRuntime,
    pub compass: CompassRuntime,
    pub mqtt: Arc<MqttPublisher>,
    pub ws: WsHub,
    pub config: Arc<AppConfig>,
}

// main.rs
let state = AppState::new(&config).await?;
HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(state.clone()))
        .configure(routes::init)
})
```

```rust
// handler
pub async fn get_config(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(state.gps.config.read().await.clone()))
}
```

**Manfaat:** wiring pendek, mudah diuji, mudah menambah sensor.

---

## 4. Best practice 2 — `ApiError`

### Sekarang

```rust
let guard = config.read().unwrap(); // panic jika poisoned
if guard.ip.is_none() { return HttpResponse::Conflict().json(...); }
```

### Idiomatik

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("resource not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({ "message": self.to_string() }))
    }
}

pub async fn create_gps(
    state: web::Data<AppState>,
    body: web::Json<CreateGpsRequest>,
) -> Result<HttpResponse, ApiError> {
    let cfg = state.gps.config.read().await;
    if cfg.ip.is_none() {
        return Err(ApiError::Conflict("config incomplete".into()));
    }
    drop(cfg);
    // ...
    Ok(HttpResponse::Created().json(...))
}
```

**Manfaat:** error konsisten, tidak panic, mudah di-log.

---

## 5. Best practice 3 — Pisahkan Domain, DTO, Payload

### Sekarang

`GpsState` dipakai untuk:
- state internal,
- response REST,
- payload MQTT,
- payload WebSocket.

Perubahan satu field → merusak 4 kontrak.

### Idiomatik

```rust
// domain
pub struct GpsState { pub lat: f64, pub lon: f64, /* ... */ }

// API
#[derive(Serialize, ToSchema)]
pub struct GpsResponse { pub latitude: f64, pub longitude: f64, /* ... */ }

// MQTT
#[derive(Serialize)]
pub struct GpsMqttPayload { pub lat: f64, pub lon: f64, pub ts: i64 }
```

Untuk proyek ini, pemisahan penuh mungkin berlebihan. Minimal:
- jangan `#[serde(skip)]` field internal ke struct response,
- tambahkan `ts` dan `vessel_id` di payload MQTT/WS.

---

## 6. Best practice 4 — Config

### Sekarang

```rust
MqttOptions::new("vessel-client", "127.0.0.1", 1883);
HttpServer::new(...).bind("127.0.0.1:8080")?;
```

### Idiomatik

```rust
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: SocketAddr,
    pub ws_addr: SocketAddr,
    pub tcp_addr: SocketAddr,
    pub mqtt_url: String,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: env_or("HTTP_ADDR", "127.0.0.1:8080").parse()?,
            ws_addr: env_or("WS_ADDR", "127.0.0.1:8081").parse()?,
            tcp_addr: env_or("TCP_ADDR", "127.0.0.1:9001").parse()?,
            mqtt_url: env_or("MQTT_URL", "mqtt://127.0.0.1:1883"),
            log_level: env_or("RUST_LOG", "info"),
        })
    }
}
```

Tambahan: `dotenvy` untuk `.env`, `figment`/`config` untuk file TOML.

---

## 7. Best practice 5 — Observability

### Sekarang

```rust
println!(" Server starting...");
tracing::info!("[GPS Service]: Reconnect requested."); // TIDAK PERNAH MUNCUL
```

`tracing_subscriber` ada di `Cargo.toml` tapi **tidak pernah diinisialisasi**.

### Idiomatik

```rust
pub fn init(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .json()          // atau .compact() untuk lokal
        .init();
}
```

Gunakan `tracing::{info, warn, error, instrument}`. Untuk request:
`actix_web::middleware::Logger` atau `tracing-actix-web`.

---

## 8. Best practice 6 — Lock

| Kebutuhan | Pilihan |
|---|---|
| Baca/tulis state di async | `tokio::sync::RwLock` |
| Lock singkat, tanpa await | `parking_lot::RwLock` |
| Counter | `AtomicU64` |
| Config immutable | `Arc<AppConfig>` (tanpa lock) |

**Aturan:** jangan pernah menahan lock melewati `.await`.

```rust
// ❌
let guard = state.read().unwrap();
mqtt.publish(...).await;   // guard masih hidup

// ✅
let snapshot = { state.read().unwrap().clone() };
mqtt.publish(snapshot).await;
```

---

## 9. Best practice 7 — Background task & shutdown

### Sekarang

```rust
thread::spawn(move || loop {
    thread::sleep(Duration::from_millis(100));
    // tidak pernah berhenti
});
```

### Idiomatik

```rust
let token = CancellationToken::new();

tokio::spawn({
    let token = token.clone();
    async move {
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        loop {
            tokio::select! {
                _ = token.cancelled() => break,
                _ = tick.tick() => simulate(&state).await,
            }
        }
        tracing::info!("simulation stopped");
    }
});

// main
tokio::select! {
    _ = server => {},
    _ = tokio::signal::ctrl_c() => {
        tracing::info!("shutting down");
        token.cancel();
    }
}
```

Untuk banyak task: `JoinSet` + supervisor.

---

## 10. Best practice 8 — Middleware

```rust
App::new()
    .wrap(Logger::default())
    .wrap(Compress::default())
    .wrap(DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
    .wrap(
        Cors::default()
            .allowed_origin("http://localhost:1420") // Vite
            .allowed_origin("tauri://localhost")     // Tauri
            .allowed_methods(vec!["GET","POST","PATCH","DELETE"])
            .allowed_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
            .max_age(3600),
    )
    .wrap(from_fn(auth_middleware))
```

Catatan Tauri: origin bisa `tauri://localhost` atau `http://tauri.localhost`
tergantung platform. Uji di Windows.

---

## 11. Best practice 9 — Testing

```rust
#[actix_web::test]
async fn create_gps_requires_config() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(AppState::for_test()))
            .configure(routes::init),
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/gps")
        .set_json(serde_json::json!({
            "latitude": 0.0, "longitude": 0.0,
            "sog": 0.0, "cog": 0.0, "is_running": true
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}
```

Untuk NMEA:

```rust
#[test]
fn vhw_has_talker_and_valid_checksum() {
    let s = SpeedState { speed_through_water: 12.5, ..Default::default() }.to_vhw();
    assert!(s.starts_with("$IIVHW,"));
    assert!(nmea_checksum_ok(&s));
}
```

---

## 12. Best practice 10 — OpenAPI

```rust
#[derive(OpenApi)]
#[openapi(
    info(title = "Vessel NDDU API", version = "1.0.0"),
    paths(
        crate::features::gps::routes::get_config,
        crate::features::gps::routes::create_gps,
    ),
    components(schemas(GpsResponse, GpsConfig, CreateGpsRequest)),
    tags((name = "GPS", description = "GPS simulator")),
)]
pub struct ApiDoc;
```

```rust
#[utoipa::path(
    get, path = "/api/gps/config", tag = "GPS",
    responses(
        (status = 200, description = "OK", body = GpsConfigEnvelope),
    )
)]
pub async fn get_config(...) -> Result<HttpResponse, ApiError> { ... }
```

---

## 13. Best practice 11 — Organisasi modul

Untuk 10 sensor, **vertical slice** lebih baik daripada layered:

```
features/gps/{model,sim,nmea,service,routes}.rs
```

Alasan:
- perubahan satu sensor terkumpul di satu folder,
- tidak ada file `controllers/` berisi 10 file hampir identik,
- mudah dihapus/ditambah.

Layered (`controllers/services/data`) tetap sah, tapi duplikasinya tinggi.

---

## 14. Best practice 12 — Hal kecil yang berdampak

| Hal | Lakukan |
|---|---|
| `impl Responder` | Ganti `Result<HttpResponse, ApiError>` |
| `unwrap()` | Ganti `?` atau `expect("alasan")` |
| `clone()` berlebihan | Snapshot sekali, bukan per field |
| `Vec<String>` topik | `SmallVec`/`Arc<str>` jika perlu |
| `flush()` per sentence | Gabung lalu flush sekali |
| `thread::sleep` | `tokio::time` di async |
| Log string | Structured field: `info!(sensor = "gps", rate = 1000, "...")` |
| Panic di task | Tangkap dengan `JoinHandle` + restart |

---

## 15. Anti-pattern yang ada di repo ini

| Anti-pattern | Lokasi | Perbaikan |
|---|---|---|
| God `main.rs` | `main.rs` | `app.rs` + `AppState` |
| Anemic error handling | semua controller | `ApiError` |
| Shared mutable global via many `Data` | `main.rs` | satu `AppState` |
| Blocking in async | services | `tokio::sync` |
| Silent failure | `mqtt_manager` | timeout + metric |
| Dead code | `config/websocket.rs` | hapus |
| Copy-paste service | 10 file | trait `Sensor` |
| No tests | — | mulai dari NMEA |
| No observability | — | `tracing` init |
| Hardcoded config | `main.rs` | env |

---

## 16. Jalur belajar yang disarankan

1. **Rust Book** bab 16 (concurrency) + bab 17 (async) — wajib.
2. **Actix-web docs**: bagian `App`, `Extractors`, `Error`, `Middleware`.
3. **tokio tutorial**: `select!`, `spawn`, `CancellationToken`, `JoinSet`.
4. **thiserror + anyhow**: error handling idiomatik.
5. **utoipa docs**: OpenAPI derive.
6. Baca ulang kode ini setelah Fase 2 — akan terasa berbeda.

---

## 17. Ringkasan satu halaman

```
SEKARANG                         TARGET
─────────────────────────────    ─────────────────────────────
25 web::Data                  →  1 AppState
unwrap()                      →  ApiError + ?
impl Responder                →  Result<HttpResponse, ApiError>
std::sync::RwLock             →  tokio::sync::RwLock
println!                      →  tracing (initialized!)
hardcoded config              →  env + file
10 OS threads                 →  1 scheduler + JoinSet
copy-paste service            →  trait Sensor
no shutdown                   →  CancellationToken
no tests                      →  unit + integration
no docs                       →  OpenAPI + docs/
```
