# 10 — Catatan Keputusan (ADR)

Format: **Status · Konteks · Keputusan · Konsekuensi**

---

## ADR-001 · Backend-only, tanpa FE

**Status:** ✅ Diterima (2026-10-31)

**Konteks:** FE dikerjakan rekan magang (React + Tauri). Menambah FE di repo
backend akan menduplikasi tanggung jawab.

**Keputusan:** Repo ini tetap **backend murni**. Tidak ada UI produk.

**Konsekuensi:**
- (+) Fokus, boundary jelas.
- (+) FE bebas memilih stack.
- (−) Perlu kontrak API yang ketat (lihat ADR-002).
- Catatan: Swagger UI adalah **tooling developer**, bukan FE produk.

---

## ADR-002 · Contract-first, backward-compatible

**Status:** ✅ Diterima

**Konteks:** Kontrak API dibuat manual sebelum development. FE sudah berjalan
dan memakai polling. Ditemukan mismatch `GET /api/gps` (teks vs JSON).

**Keputusan:**
1. `docs/02-api-contract.md` menjadi sumber kebenaran.
2. Perubahan bersifat **aditif**; breaking change → `/api/v2`.
3. `GET /api/gps` tetap NMEA; tambah `?format=json` untuk FE.

**Konsekuensi:**
- (+) FE tidak rusak.
- (+) Integrator NMEA tetap dapat teks.
- (−) Dua format di satu endpoint; perlu dokumentasi jelas.

---

## ADR-003 · Swagger: utoipa + utoipa-swagger-ui (Opsi A)

**Status:** ✅ Diterima (2026-10-31)

**Konteks:** Butuh dokumentasi API interaktif. Pilihan: `utoipa`, `paperclip`,
`apistos`, `aide`.

**Keputusan:** `utoipa` 5.x + `utoipa-swagger-ui` 9.x (fitur `actix-web`),
sajikan **UI + JSON**.

**Konsekuensi:**
- (+) Ekosistem besar, derive macro, aktif.
- (+) Bisa di-import ke Postman/Insomnia.
- (−) Semua DTO harus `ToSchema`; handler harus `#[utoipa::path]`.
- (−) Swagger UI adalah halaman web (dapat dimatikan via feature flag).

---

## ADR-004 · Mosquitto native, tanpa Docker

**Status:** ✅ Diterima (2026-10-31)

**Konteks:** Butuh broker MQTT lokal. Docker Desktop tersedia tapi berat dan
daemon sering mati.

**Keputusan:** Install **Mosquitto native** via winget
(`EclipseFoundation.Mosquitto`). Docker tidak dipakai untuk development.

**Konsekuensi:**
- (+) Ringan (~1 MB), bisa jadi service Windows.
- (+) Tidak menambah beban RAM.
- (−) Perlu install manual di mesin lain.
- Docker tetap opsional untuk CI/deploy (Fase 1).

---

## ADR-005 · Target deployment

**Status:** ✅ Diterima (2026-10-31) — **Opsi A: Hugging Face Spaces (Docker)**

**Konteks:** Anda menyebut dua opsi: Oracle ARM (butuh kartu) dan Cloudflare
Workers (tanpa kartu, tapi tidak bisa menjalankan binary ini). Anda menyetujui
HF Spaces gratis **dengan syarat API docs (Swagger) tersedia**.

**Keputusan:** Deploy binary Rust apa adanya ke **Hugging Face Spaces (Docker)**.
REST + WebSocket + Swagger UI berjalan di satu port (7860). MQTT publish
diarahkan ke broker eksternal (HiveMQ Cloud / EMQX Serverless) via env.
TCP server (9001) tidak tersedia di platform ini — didokumentasikan sebagai
keterbatasan.

**Konsekuensi:**
- (+) Tanpa kartu, binary asli, tidak perlu rewrite.
- (+) Swagger + REST + WS memenuhi syarat portofolio.
- (−) TCP server hilang; MQTT broker harus eksternal.
- (−) Space sleep setelah 48 jam tidak aktif (cold start).
- Opsi C (Cloudflare Worker sebagai proxy) tetap mungkin nanti tanpa rewrite.

---

## ADR-006 · Urutan kerja: P0 → P1 → P2

**Status:** ✅ Diterima

**Konteks:** Banyak temuan; mengerjakan semuanya sekaligus berisiko.

**Keputusan:** Perbaiki **P0 (4 bug kritis)** dan verifikasi, lalu **P1**,
baru **P2**. Tidak ada fitur baru sebelum P0 selesai.

**Konsekuensi:**
- (+) Stabil lebih cepat.
- (−) Fitur "hebat" (scenario engine) tertunda.

---

## ADR-007 · Talker ID NMEA

**Status:** ✅ Diterima (menunggu implementasi P0-03)

**Konteks:** `$VHW`, `$DBT`, `$DPT`, `$DBK` tidak punya talker → invalid.

**Keputusan:** Gunakan talker yang sesuai perangkat:
- Speed: `$IIVHW`
- Depth: `$SDDBT`, `$SDDPT`, `$SDDBK`

**Konsekuensi:**
- (+) Valid menurut NMEA 0183.
- (−) Perlu update dokumentasi & test golden file.
- Catatan: jika klien LEN memakai talker lain, mudah diubah (konstanta).

---

## ADR-008 · Strategi konfigurasi MQTT

**Status:** ⏳ **TERTUNDA**

**Konteks:** Config per-sensor punya `ip/port/username/password`, tapi
`main.rs` hardcoded `127.0.0.1:1883`.

**Opsi:**
- **A. Per-sensor** — tiap sensor punya koneksi sendiri (fleksibel, boros).
- **B. Global** — satu koneksi, config global via env (sederhana, jujur).
- **C. Hybrid** — global default + override per-sensor.

**Rekomendasi:** **B** untuk Fase 0 (jujur & sederhana), lalu **C** di Fase 3.

**Keputusan:** _menunggu konfirmasi_

---

## ADR-009 · Lisensi

**Status:** ⏳ **TERTUNDA**

**Konteks:** Repo memakai **AGPL-3.0**. Untuk portofolio publik ini bisa
menghalangi reuse; untuk aset LEN, AGPL mungkin tidak sesuai kebijakan.

**Opsi:**
- Tetap AGPL-3.0 (jika memang ingin copyleft).
- Ganti MIT/Apache-2.0 (portofolio).
- Private repo (aset LEN).

**Keputusan:** _menunggu konfirmasi_

---

## ADR-010 · Struktur target: modular monolith

**Status:** ✅ Diterima

**Konteks:** 10 sensor, satu binary. Microservice akan menambah kompleksitas
tanpa manfaat.

**Keputusan:** Tetap **modular monolith**, migrasi bertahap ke `features/`
(vertical slice) + `AppState`.

**Konsekuensi:**
- (+) Deploy sederhana.
- (+) Refactor bisa bertahap (strangler).
- (−) Perlu disiplin batas modul.
