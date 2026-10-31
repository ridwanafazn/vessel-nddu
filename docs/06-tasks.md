# 06 — Tasks

Checklist kerja. **Satu task = satu commit.**
Update status di sini setiap kali selesai.

Legenda: `[ ]` belum · `[~]` sedang · `[x]` selesai

---

## Fase 0 — Stabilisasi

### P0-01 · MQTT fail-safe
- [ ] Buat `transport/mqtt.rs` dengan `try_publish` + timeout
- [ ] Tambahkan counter `dropped_messages`
- [ ] Tambahkan flag `connected` yang bisa dibaca
- [ ] Ganti semua pemanggilan `publish_message` di 10 service
- [ ] Uji: matikan broker → WS tetap mengalir
- [ ] Uji: hidupkan broker → publish kembali normal
- **File:** `utils/mqtt_manager.rs`, semua `services/*_service.rs`
- **Acceptance:** WS menerima pesan saat broker mati; log mencatat drop

### P0-02 · Config MQTT nyata
- [ ] Tentukan model: per-sensor atau global (lihat ADR-008)
- [ ] Implementasi koneksi memakai `ip/port/username/password`
- [ ] `client_id` unik per instance
- [ ] Reconnect saat config berubah (bukan no-op)
- [ ] Uji: ubah `ip` → log menunjukkan target baru
- **File:** `main.rs`, `utils/mqtt_manager.rs`, controller config
- **Acceptance:** config benar-benar mengubah koneksi

### P0-03 · NMEA valid
- [ ] `to_vhw` → `$IIVHW`
- [ ] `to_dbt` → `$SDDBT`
- [ ] `to_dpt` → `$SDDPT`
- [ ] `to_dbk` → `$SDDBK`
- [ ] Tambah validator `is_valid_nmea(sentence)`
- [ ] Unit test: panjang ID ≥5, checksum benar
- [ ] Uji silang dengan `nmea.c` (jika diaktifkan)
- **File:** `data/navigation_data.rs`
- **Acceptance:** semua sentence lolos validator

### P0-04 · Depth TCP hidup
- [ ] Pindahkan pembacaan `depth_state` ke dalam loop
- [ ] Samakan pola dengan sensor lain
- [ ] Uji: `$DBT` berubah dalam 5 detik
- **File:** `services/tcp_client.rs`
- **Acceptance:** nilai depth tidak konstan

### P1-01 · Tracing aktif
- [ ] Buat `telemetry.rs` dengan `tracing_subscriber::fmt()`
- [ ] Level dari env `RUST_LOG`
- [ ] Ganti `println!/eprintln!` bertahap
- [ ] Uji: log publication muncul
- **File:** `main.rs`, `telemetry.rs`

### P1-02 · Config env
- [ ] `AppConfig` dari env: `HTTP_ADDR`, `WS_ADDR`, `TCP_ADDR`, `MQTT_URL`
- [ ] Default tetap kompatibel (`127.0.0.1:8080`, dst.)
- [ ] Uji: jalankan di port lain tanpa recompile
- **File:** `main.rs`, `config.rs`

### P1-13 · Log port TCP
- [ ] Ganti `9003` → nilai bind sebenarnya
- **File:** `main.rs:268`

### P1-14 · GPS JSON untuk FE
- [ ] Tambah `?format=json` pada `GET /api/gps`
- [ ] Pertahankan default `text/plain`
- [ ] Update `02-api-contract.md`
- [ ] Koordinasi dengan FE
- **File:** `controllers/gps_controller.rs`

---

## Fase 1 — Kontrak & Tooling

### D-01 · Swagger
- [ ] Tambah `utoipa`, `utoipa-swagger-ui` ke `Cargo.toml`
- [ ] `#[derive(ToSchema)]` semua DTO
- [ ] `#[utoipa::path]` semua handler
- [ ] `docs/openapi.rs` dengan `#[derive(OpenApi)]`
- [ ] Mount `/swagger-ui/` + `/api-docs/openapi.json`
- [ ] Uji: semua endpoint tampil, contoh payload benar
- **Acceptance:** Swagger UI bisa execute request

### D-02 · Repo hygiene
- [ ] `git init` + commit awal
- [ ] `README.md` root (cara run <5 menit)
- [ ] Hapus `src/config/websocket.rs`
- [ ] Putuskan nasib `nmea.c` (aktifkan atau hapus)
- [ ] Hapus dependency mati
- [ ] `.env.example`

### D-03 · Docker (opsional)
- [ ] `Dockerfile` multi-stage
- [ ] `docker-compose.yml` (app + mosquitto)
- [ ] Uji `docker compose up`

---

## Fase 2 — Kualitas

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
- [ ] `CancellationToken`
- [ ] Semua loop `select!` dengan token
- [ ] `tokio::signal::ctrl_c`
- [ ] Uji: Ctrl+C berhenti <2 s

### Q-04 · Test
- [ ] Unit NMEA (golden file)
- [ ] Unit checksum
- [ ] Integrasi REST (`actix_web::test`)
- [ ] Test regresi P0-01..04

### Q-05 · CI
- [ ] GitHub Actions: fmt, clippy, test, build
- [ ] Badge di README

### Q-06 · Release profile
- [ ] `lto = "thin"`, `strip = true`, `codegen-units = 1`
- [ ] Ukur ukuran binary

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
| — | P0-01 | ⏳ | — |
| — | P0-02 | ⏳ | — |
| — | P0-03 | ⏳ | — |
| — | P0-04 | ⏳ | — |
