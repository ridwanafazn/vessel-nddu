# 04 — Audit Temuan

Metodologi: **baca kode statis + jalankan + uji end-to-end** pada Windows 11,
Rust 1.99.1, 31 Oktober 2026.

---

## P0 — Kritis (harus selesai lebih dulu)

### P0-01 · Deadlock publikasi saat MQTT broker tidak tersedia

- **Lokasi:** `src/utils/mqtt_manager.rs:82-93`, dipakai semua `*_service.rs`
- **Akar masalah:** `AsyncClient::publish().await` mengisi channel internal
  berkapasitas **10**. Eventloop gagal connect → channel tidak dikuras →
  setelah ~10 publish, `await` menggantung **selamanya**.
- **Bukti:**
  - Tanpa broker: WebSocket **0 pesan / 5 detik** (GPS `is_running=true`).
  - Dengan broker: WebSocket **15 pesan / 5 detik**.
- **Dampak:** satu titik gagal MQTT mematikan **seluruh** pipeline publikasi
  sensor tersebut (MQTT + WS + NMEA).
- **Perbaikan:**
  1. Gunakan `try_publish` + hitung drop, atau
  2. `tokio::time::timeout(50ms, publish)`, atau
  3. Task publisher terpisah dengan bounded queue + drop-oldest.
  4. Tambahkan health flag `mqtt_connected` yang bisa dibaca `/health`.
- **Kriteria selesai:** broker dimatikan → WS tetap mengalir; log mencatat drop.

### P0-02 · Config MQTT per-sensor tidak dipakai

- **Lokasi:** `src/main.rs:65`
  ```rust
  let mut mqttoptions = MqttOptions::new("vessel-client", "127.0.0.1", 1883);
  ```
- **Fakta:** `ip`, `port`, `username`, `password` di semua config **tidak dibaca**.
  Hanya `topics`, `update_rate`, `enabled_sentences`, `tcp_enabled` yang dipakai.
- **Bukti:** ubah `ip` ke `10.0.0.5` lewat API → koneksi tetap ke `127.0.0.1`.
- **Dampak:** kontrak API menipu; tidak bisa diarahkan ke broker LEN.
- **Perbaikan:** `MqttPublisher` yang membaca config, reconnect nyata,
  `client_id` unik, TLS opsional. Atau jujur: hapus field dari config dan
  jadikan env global (pilih salah satu, catat di ADR).
- **Kriteria selesai:** ubah `ip/port` → koneksi pindah (terbukti di log).

### P0-03 · NMEA tidak valid (talker ID hilang)

- **Lokasi:** `src/data/navigation_data.rs:30,67,77,88`
- **Output aktual:**
  ```
  $VHW,0.0,T,0.0,M,15.9,N,29.5,K*56
  $DBT,2044.8,f,623.3,M,340.8,F*20
  $DPT,623.3,10.0*75
  $DBK,2012.0,f,613.3,M,335.3,F*3E
  ```
- **Standar:** wajib `$TTSSS` (≥5 karakter). `$VHW` hanya 3 → invalid.
- **Ironi:** `nmea.c` Anda sendiri mensyaratkan minimal 5 karakter → akan
  **menolak** sentence ini.
- **Dampak:** parser pihak ketiga (ECDIS/VDR) menolak → alat dianggap rusak.
- **Perbaikan:** `$IIVHW`, `$SDDBT`, `$SDDPT`, `$SDDBK` (atau talker lain yang
  sesuai). Tambahkan unit test yang memvalidasi panjang ID + checksum.
- **Kriteria selesai:** semua sentence lolos validator NMEA (termasuk `nmea.c`).

### P0-04 · TCP stream membekukan depth

- **Lokasi:** `src/services/tcp_client.rs:113-119`
  ```rust
  let depth_state = { /* dibaca SEKALI, di luar loop */ };
  let mut interval = ...;
  loop { interval.tick().await; /* depth_state tidak pernah di-refresh */ }
  ```
- **Bukti:** 6 detik `$DBT` konstan `623.3`, sementara MQTT `600.8 → 602.8`.
- **Dampak:** integrator melihat data statis → menyimpulkan simulator rusak.
- **Perbaikan:** pindahkan pembacaan depth ke dalam loop, sejajar sensor lain.
- **Kriteria selesai:** `$DBT` berubah antar tick (uji 5 detik).

---

## P1 — Serius

| ID | Temuan | Lokasi | Dampak | Perbaikan |
|---|---|---|---|---|
| P1-01 | `tracing_subscriber` tidak pernah diinisialisasi | `main.rs` | Semua `tracing::info!` hilang | `telemetry::init()` |
| P1-02 | Port & bind hardcoded `127.0.0.1` | `main.rs:251,257,266` | Tidak bisa diakses LAN; bentrok port | env `HTTP_ADDR`, `WS_ADDR`, `TCP_ADDR` |
| P1-03 | Tidak ada graceful shutdown | `main.rs` | Ctrl+C mematikan paksa; thread zombie | `CancellationToken` + `tokio::signal` |
| P1-04 | Tidak ada persistence config | semua controller | Config hilang saat restart | SQLite/TOML |
| P1-05 | Auth hardcoded `admin/admin` | `auth_data.rs` | Tidak aman | JWT + middleware |
| P1-06 | CORS `allow_any_origin` | `main.rs:228` | Terbuka | whitelist origin |
| P1-07 | `MqttCommand::Reconnect` no-op | semua service | API berbohong | implementasi nyata atau hapus |
| P1-08 | Validasi config tidak konsisten | controller | Perilaku tak terduga | satu validator |
| P1-09 | Kompas tidak menghasilkan NMEA | `compass_service.rs` | Sensor kritis tidak terpakai | `HDM`/`HDG` |
| P1-10 | Timestamp tidak konsisten (UTC vs Local) | `compass_data.rs` | Data tidak bisa dikorelasikan | semua UTC RFC3339 |
| P1-11 | WMM dihitung 10×/detik | `gps_calculate.rs` | CPU boros | cache, update tiap 60 s |
| P1-12 | WS tanpa ping/pong + unbounded buffer | `net.rs` | Klien timeout / memory naik | handler ping + bounded |
| P1-13 | Log TCP salah port (`9003` vs `9001`) | `main.rs:268` | Membingungkan | perbaiki string |
| P1-14 | `GET /api/gps` mismatch dengan FE | `gps_controller.rs:113` | Halaman GPS FE kosong | `?format=json` |

---

## P2 — Kebersihan teknis

| ID | Temuan | Lokasi |
|---|---|---|
| P2-01 | `src/config/websocket.rs` dead code, import tidak valid | `src/config/` |
| P2-02 | `nmea.c`/`nmea.h` tidak dikompilasi; `parser.h` & `parser_types.h` tidak ada | root |
| P2-03 | Dependency tidak terpakai: `reqwest`, `bincode`, `lazy_static`, `wmm` | `Cargo.toml` |
| P2-04 | Duplikasi 10 service & 10 controller | `services/`, `controllers/` |
| P2-05 | `unwrap()` pada lock | hampir semua file |
| P2-06 | `std::sync::RwLock` di async | services/controllers |
| P2-07 | Domain type = API DTO = MQTT payload | `data/` |
| P2-08 | `message_data.rs` tidak dipakai | `data/` |
| P2-09 | Tidak ada test | — |
| P2-10 | Tidak ada Dockerfile / CI / README | root |
| P2-11 | Tidak ada `.git` | root |
| P2-12 | Lisensi AGPL-3.0 (perlu klarifikasi) | `LICENSE` |
| P2-13 | Weather gabungan vs individual duplikatif | `weather_service.rs` |
| P2-14 | `enabled_sentences` GPS mengatur semua sensor di TCP | `tcp_client.rs` |
| P2-15 | `flush()` per sentence (tidak efisien) | `tcp_client.rs:229` |
| P2-16 | Tidak ada `/health`, `/metrics` | — |
| P2-17 | Binary debug 16.1 MB; tidak ada release profile | `Cargo.toml` |

---

## Bukan bug (keputusan yang bisa diterima)

| Hal | Alasan |
|---|---|
| `Option<State>` untuk "belum dibuat" | Jelas dan aman |
| Kalkulasi di OS thread | Menghindari blocking async |
| NMEA encoder di domain type | Kohesif untuk ukuran proyek ini |
| Satu binary modular monolith | Tepat; microservice berlebihan |
| `enabled_sentences` semantik `None`/`[]` | Terdokumentasi |

---

## Cara reproduksi

```powershell
# 1. build & jalankan
cargo run

# 2. tanpa broker → WS kosong
node ws_test.mjs          # 0 pesan

# 3. dengan broker → WS jalan
winget install EclipseFoundation.Mosquitto
# jalankan broker, lalu ulangi → 15 pesan/5 detik

# 4. depth beku
# arahkan tcp_enabled ke listener lokal, amati $DBT konstan
```
