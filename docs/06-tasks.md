# 06 — Tasks

Checklist kerja. **Satu task = satu commit.**
Update status di sini setiap kali selesai.

Legenda: `[ ]` belum · `[~]` sedang · `[x]` selesai

---

## Fase 0 — Stabilisasi

> **Status: ✅ SELESAI (2026-10-31)** — semua task di bawah terverifikasi.

### P0-01 · MQTT fail-safe
- [x] Buat `transport/mqtt` (implementasi di `utils/mqtt_manager.rs`) dengan `try_send` + `try_publish`
- [x] Tambahkan counter `dropped_messages`
- [x] Tambahkan flag `connected` yang bisa dibaca lewat `/api/mqtt/config`
- [x] Ganti semua pemanggilan `publish_message` di 10 service (API tetap sama)
- [x] Uji: matikan broker → WS tetap mengalir (**15 pesan/5 detik**)
- [x] Uji: hidupkan broker → publish kembali normal
- **File:** `utils/mqtt_manager.rs`, semua `services/*_service.rs`
- **Acceptance:** ✅ WS menerima pesan saat broker mati; drop dihitung

### P0-02 · Config MQTT nyata
- [x] Tentukan model: **global** (ADR-008)
- [x] Implementasi koneksi memakai `MQTT_HOST/PORT/USERNAME/PASSWORD`
- [x] `client_id` global via env
- [x] Reconnect saat config berubah (`PATCH /api/mqtt/config`)
- [x] Uji: ubah port 1884 → 1883 → `connected=true` tanpa restart
- **File:** `config.rs`, `utils/mqtt_manager.rs`, `controllers/mqtt_controller.rs`
- **Acceptance:** ✅ config benar-benar mengubah koneksi

### P0-03 · NMEA valid
- [x] `to_vhw` → `$IIVHW`
- [x] `to_dbt` → `$SDDBT`
- [x] `to_dpt` → `$SDDPT`
- [x] `to_dbk` → `$SDDBK`
- [x] Tambah validator `nmea::validate`
- [x] Unit test: panjang ID ≥5, checksum benar
- [x] Uji runtime: **0 sentence invalid** dari 57 baris TCP
- **File:** `data/navigation_data.rs`, `utils/nmea.rs`
- **Acceptance:** ✅ semua sentence lolos validator

### P0-04 · Depth TCP hidup
- [x] Pindahkan pembacaan `depth_state` ke dalam loop
- [x] Samakan pola dengan sensor lain
- [x] Uji: `$SDDBT` berubah antar tick (581.3 → 581.7 → 586.7)
- **File:** `services/tcp_client.rs`
- **Acceptance:** ✅ nilai depth tidak konstan

### P1-01 · Tracing aktif
- [x] Buat `telemetry.rs` dengan `tracing_subscriber::fmt()`
- [x] Level dari env `RUST_LOG`
- [x] Ganti log startup & MQTT ke `tracing`
- [x] Uji: log `MQTT connected`, `API server started` muncul
- **File:** `main.rs`, `telemetry.rs`

### P1-02 · Config env
- [x] `AppConfig` dari env: `HTTP_ADDR`, `WS_ADDR`, `TCP_ADDR`, `MQTT_*`
- [x] Default tetap kompatibel (`127.0.0.1:8080`, dst.)
- [x] Uji: jalankan di `HTTP_ADDR=127.0.0.1:8090` → sukses
- **File:** `main.rs`, `config.rs`, `.env.example`

### P1-13 · Log port TCP
- [x] Ganti `9003` → nilai bind sebenarnya
- **File:** `main.rs`
- **Bukti:** log `TCP server started addr=127.0.0.1:9001`

### P1-14 · GPS JSON untuk FE
- [x] Tambah `?format=json` pada `GET /api/gps`
- [x] Pertahankan default `text/plain`
- [x] Update `02-api-contract.md`
- [x] Uji: JSON `{message,data}` dan NMEA keduanya benar
- **File:** `controllers/gps_controller.rs`

---

## Fase 1 — Kontrak & Tooling

> **Status: ✅ SELESAI (2026-10-31)** — Swagger, README, Docker, dead code dibersihkan.

### D-01 · Swagger
- [x] Tambah `utoipa` + `utoipa-swagger-ui`
- [x] `#[derive(ToSchema)]` semua DTO
- [x] `#[utoipa::path]` semua handler (58 operasi)
- [x] `docs/openapi.rs` dengan `#[derive(OpenApi)]`
- [x] Mount `/swagger-ui/` + `/api-docs/openapi.json`
- [x] Uji: **17 path, 67 schema**, UI 200
- **Acceptance:** ✅ Swagger UI bisa execute request

### D-02 · Repo hygiene
- [x] `git init` + commit awal
- [x] `README.md` root (cara run <5 menit)
- [x] Hapus `src/config/websocket.rs`
- [x] Hapus `nmea.c`/`nmea.h` (tidak dikompilasi, header hilang)
- [x] Hapus dependency mati: `reqwest`, `bincode`, `lazy_static`, `wmm`
- [x] `.env.example`
- [x] Lisensi MIT

### D-03 · Docker
- [x] `Dockerfile` multi-stage
- [x] `docker-compose.yml` (app + mosquitto)
- [x] `deploy/mosquitto.conf`
- [x] `.dockerignore`
- [x] Uji image HF lokal (`--user 1000:1000`, REST + Swagger + WS) ✅
- [ ] Push ke HF Spaces (Docker SDK) — butuh akun/token

---

## Fase 2 — Kualitas

> **Status: 🟡 SEBAGIAN (2026-10-31)**
> Selesai: CI, clippy-clean, fmt, graceful shutdown, release profile,
> test OpenAPI, `/health`.
> Belum: `ApiError`, `AppState`, test integrasi REST.

### Q-01 · Error handling
- [ ] `ApiError` + `ResponseError`
- [ ] Ganti `unwrap()` lock
- [ ] Ganti `impl Responder` → `Result<HttpResponse, ApiError>`
- [ ] Uji: 404/409/500 terstruktur

### Q-02 · AppState
- [ ] Buat `AppState`
- [ ] Migrasi handler per fitur
- [ ] Hapus `web::Data` lama
- [ ] `main.rs` < 100 baris

### Q-03 · Shutdown
- [x] `CancellationToken`
- [x] Semua loop `select!` dengan token
- [x] `tokio::signal::ctrl_c`
- [ ] Uji manual Ctrl+C

### Q-04 · Test
- [x] Unit NMEA (golden file)
- [x] Unit checksum
- [x] Test OpenAPI (path & schema)
- [x] Test config & stats
- [ ] Integrasi REST (`actix_web::test`)

### Q-05 · CI
- [x] GitHub Actions: fmt, clippy, test, build
- [ ] Badge di README (perlu repo GitHub)

### Q-06 · Release profile
- [x] `lto = "thin"`, `strip = true`, `codegen-units = 1`
- [ ] Ukur ukuran binary release

### Q-07 · Health
- [x] `GET /health` dengan status MQTT + uptime

---

## Fase 3 — Fitur "hebat"

- [ ] Scenario engine (waypoint + speed profile)
- [ ] Deterministic clock + seed
- [ ] Fault injection (dropout/stale/spike/bad checksum)
- [ ] Recording & replay NMEA
- [ ] TCP server nyata (9001)
- [ ] WS subscription filter
- [ ] WS ping/pong + bounded buffer

---

## Fase 4 — Maritim lengkap

- [ ] `GPGGA`
- [ ] `GPGSA` + `GPGSV`
- [ ] `GNGNS`
- [ ] `HDM` dari kompas
- [ ] AIS `AIVDM`/`AIVDO`
- [ ] `RPM`, `RSA`, `VBW`, `VLW`
- [ ] `ALR`/`ACK`

---

## Fase 5 — Operasional

- [ ] Auth JWT + middleware
- [ ] Persistence config (SQLite)
- [ ] `/health`, `/metrics`
- [ ] Multi-vessel namespace
- [ ] Deploy publik (lihat `08-deployment.md`)

---

## Definition of Done

Sebuah task dianggap selesai jika:

1. Kode compile tanpa warning (`cargo clippy -- -D warnings`).
2. Ada bukti verifikasi (curl/log/test).
3. Dokumentasi terkait diperbarui.
4. Tidak merusak FE (cek `02-api-contract.md`).
5. Commit message jelas.

## Log progres

| Tanggal | Task | Status | Catatan |
|---|---|---|---|
| 2026-10-31 | Audit + docs | ✅ | 14 dokumen dibuat |
| 2026-10-31 | Stop `smile-fest-api` | ✅ | Port 8080 bebas |
| 2026-10-31 | Install Mosquitto | ✅ | Config di `C:\Users\Ridwan\mosquitto\mosquitto.conf` |
| 2026-10-31 | Verifikasi MQTT + WS | ✅ | 5 pesan/5 detik, 5 topik MQTT |
| 2026-10-31 | ADR-008/009 | ✅ | MQTT global, lisensi MIT |
| 2026-10-31 | P0-03 NMEA talker | ✅ | 15 unit test, 0 invalid dari 57 baris TCP |
| 2026-10-31 | P0-04 depth TCP | ✅ | `$SDDBT` berubah antar tick |
| 2026-10-31 | P1-01 tracing | ✅ | Log muncul di stdout |
| 2026-10-31 | P1-02 env config | ✅ | `HTTP_ADDR=8090` terbukti |
| 2026-10-31 | P1-13 log port | ✅ | `addr=127.0.0.1:9001` |
| 2026-10-31 | P0-01 MQTT fail-safe | ✅ | WS 15 pesan saat broker mati |
| 2026-10-31 | P0-02 MQTT global runtime | ✅ | 1884 → 1883, `connected=true` |
| 2026-10-31 | P1-14 GPS JSON | ✅ | `?format=json` + default NMEA |
| 2026-10-31 | Fase 1 Swagger | ✅ | 17 path, 67 schema, `/swagger-ui/` 200 |
| 2026-10-31 | README + Docker | ✅ | `README.md`, `Dockerfile`, `docker-compose.yml` |
| 2026-10-31 | Hygiene | ✅ | `nmea.c/h`, `config/websocket.rs`, 4 dep dihapus |
| 2026-10-31 | CI | ✅ | GitHub Actions, clippy-clean, 18 test |
| 2026-10-31 | Fase 2 inti | ✅ | shutdown, release profile, `/health` |
| 2026-10-31 | HF image | ✅ | build sukses, uji non-root, WS `/ws` |
| — | Push GitHub + HF | ⏳ | butuh akun/token manual |
