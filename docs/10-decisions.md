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

**Status:** 🔁 **Digantikan oleh ADR-012** (2026-10-31)

**Konteks:** Awalnya dipilih Hugging Face Spaces Docker. Ternyata **Docker
Spaces sekarang Paid** (badge "Paid" di UI HF, 2026), sehingga tidak memenuhi
syarat "gratis tanpa kartu".

**Keputusan:** HF Docker dibatalkan. Lihat **ADR-012 (Render Free)**.

---

## ADR-012 · Render Free sebagai target deploy

**Status:** ✅ Diterima (2026-10-31)

**Konteks:** Butuh host gratis tanpa kartu yang bisa menjalankan binary Rust,
REST, WebSocket, dan MQTT internal.

**Keputusan:** Deploy ke **Render Free** (Docker web service, 512 MB,
WebSocket didukung, plan Free tanpa kartu). Image memakai nginx + Mosquitto
di dalam container, listen pada `$PORT` (Render: 10000).

**Konsekuensi:**
- (+) Gratis, tanpa kartu, binary Rust asli.
- (+) REST + Swagger + WS `/ws` + MQTT internal jalan.
- (−) Spin-down 15 menit (cold start ±1 menit).
- (−) TCP server 9001 tidak diekspos.
- (+) `render.yaml` + `deploy/hf/Dockerfile` sudah diuji lokal.

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

**Status:** ✅ Diterima (2026-10-31) — **Opsi B: global**

**Konteks:** Config per-sensor punya `ip/port/username/password`, tapi
`main.rs` hardcoded `127.0.0.1:1883`.

**Keputusan:** Satu koneksi MQTT **global**. Sumber kebenaran:
environment (`MQTT_HOST`, `MQTT_PORT`, `MQTT_CLIENT_ID`, `MQTT_USERNAME`,
`MQTT_PASSWORD`, `MQTT_KEEP_ALIVE_SECS`) + endpoint runtime
`GET/PATCH /api/mqtt/config`. Field `ip/port/username/password` di config
per-sensor **tidak lagi dipakai untuk koneksi** (hanya untuk kompatibilitas FE
dan target TCP GPS).

**Konsekuensi:**
- (+) Jujur, sederhana, satu tempat konfigurasi.
- (+) Bisa diubah runtime tanpa restart.
- (−) FE yang mengira config per-sensor mengatur broker perlu dijelaskan.
- Per-sensor (Opsi C) bisa ditambahkan di Fase 3 bila benar-benar perlu.

---

## ADR-009 · Lisensi

**Status:** ✅ Diterima (2026-10-31) — **MIT**

**Konteks:** Repo sebelumnya AGPL-3.0. Untuk portofolio publik, AGPL membatasi
reuse dan bisa bermasalah untuk integrasi produk.

**Keputusan:** Ganti ke **MIT**. Hak cipta: Ridwan Ahmad Fauzan (2026).

**Konsekuensi:**
- (+) Bebas dipakai/dimodifikasi untuk portofolio.
- (−) Tidak ada proteksi copyleft; jika kode milik PT LEN, kepemilikan perlu
  diklarifikasi sebelum publikasi.

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

---

## ADR-011 · GPS: NMEA default + `?format=json`

**Status:** ✅ Diterima (2026-10-31)

**Konteks:** FE mengharapkan JSON dari `GET /api/gps`, sedangkan integrator
NMEA mengharapkan teks.

**Keputusan:** Pertahankan default `text/plain` (NMEA GPRMC) dan tambahkan
`?format=json` untuk FE.

**Konsekuensi:**
- (+) Tidak ada breaking change.
- (+) Satu endpoint, dua format, terdokumentasi.
- (−) FE harus mengubah URL (satu baris).
