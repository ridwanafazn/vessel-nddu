# 05 — Roadmap

Prinsip: **stabilkan → dokumentasikan → uji → baru tambah fitur.**

```
Fase 0  Stabilisasi        🔴 P0 + P1 inti
Fase 1  Kontrak & Tooling  Swagger, docs, Docker opsional
Fase 2  Kualitas           Test, CI, error handling, shutdown
Fase 3  Fitur "hebat"      Scenario, fault injection, replay
Fase 4  Maritim lengkap    GGA/GSA/GSV, HDM, AIS, engine
Fase 5  Operasional        Auth, persistence, metrics, deploy
```

---

## Fase 0 — Stabilisasi

> **Status: ✅ SELESAI (2026-10-31)** — lihat `06-tasks.md` untuk bukti.

**Tujuan:** alat ini tidak boleh "diam-diam rusak".

| Deliverable | Kriteria selesai |
|---|---|
| P0-01 MQTT fail-safe | Broker mati → WS & NMEA tetap jalan |
| P0-02 Config MQTT nyata | Ubah ip/port → koneksi pindah |
| P0-03 NMEA valid | Semua sentence ≥5 char + checksum benar |
| P0-04 Depth TCP hidup | `$DBT` berubah antar tick |
| P1-01 tracing aktif | Log muncul di stdout, level bisa diatur |
| P1-02 config env | Port/bind dari env |
| P1-13 log port benar | Tidak ada `9003` |
| P1-14 GPS JSON | FE bisa baca `?format=json` |

**Exit criteria:** semua di atas terverifikasi dengan curl/log/test manual.

**Estimasi:** 2–3 hari.

---

## Fase 1 — Kontrak & Tooling

> **Status: ✅ SELESAI (2026-10-31)** — Swagger UI + JSON, README, Dockerfile,
> dependency mati dihapus.

| Deliverable | Kriteria selesai |
|---|---|
| Swagger UI + JSON (`utoipa`) | `/swagger-ui/` dan `/api-docs/openapi.json` hidup |
| Semua DTO `ToSchema` | Tidak ada schema kosong |
| Semua handler `#[utoipa::path]` | Response code benar |
| `docs/` lengkap | 14 dokumen (README + 00–12) |
| `README.md` root | Cara run dalam <5 menit |
| `.gitignore` + `git init` | Repo bersih |
| Dockerfile (opsional) | `docker build` sukses |
| `docker-compose.yml` (opsional) | app + mosquitto |

**Estimasi:** 2–3 hari.

---

## Fase 2 — Kualitas

| Deliverable | Kriteria selesai |
|---|---|
| `ApiError` + `ResponseError` | Tidak ada `unwrap` di handler |
| `AppState` tunggal | `main.rs` < 100 baris |
| Graceful shutdown | Ctrl+C → semua task berhenti <2 s |
| Test unit NMEA | Golden file + checksum |
| Test integrasi REST | `actix_web::test` untuk 6 endpoint utama |
| CI GitHub Actions | fmt + clippy -D warnings + test |
| Release profile | binary < 8 MB, `lto`, `strip` |
| Hapus dependency mati | `cargo machete` bersih |

**Estimasi:** 3–5 hari.

---

## Fase 3 — Fitur "hebat"

| Deliverable | Nilai |
|---|---|
| Scenario engine (rute waypoint, profil kecepatan) | Simulasi realistis, bukan random walk |
| Deterministic clock + seed | Reproducible test (syarat FAT/SAT) |
| Fault injection (dropout, stale, spike, bad checksum) | Uji robustness software klien |
| Recording & replay NMEA | Debugging & regresi |
| TCP server nyata (port 9001) | Banyak sistem maritim menghubungi sumber data |
| WS subscription filter | Hemat bandwidth |

**Estimasi:** 1–2 minggu.

---

## Fase 4 — Maritim lengkap

| Deliverable | Nilai |
|---|---|
| `GPGGA`, `GPGSA`, `GPGSV`, `GNGNS` | Wajib untuk banyak konsumen |
| `HDM` dari kompas | Integrated bridge |
| AIS `AIVDM`/`AIVDO` | Nilai jual besar |
| Engine/rudder (`RPM`, `RSA`, `VBW`, `VLW`) | Kelengkapan data kapal |
| Alarm (`ALR`/`ACK`) | Standar bridge |

**Estimasi:** 1–2 minggu.

---

## Fase 5 — Operasional

| Deliverable | Nilai |
|---|---|
| Auth JWT + middleware | Keamanan |
| Persistence config (SQLite) | Konsistensi antar sesi |
| `/health`, `/metrics` Prometheus | Operasional |
| Multi-vessel namespace | Fleet simulation |
| Deploy publik | Portofolio |

**Estimasi:** 1 minggu.

---

## Ringkasan waktu

| Fase | Hari | Kumulatif |
|---|---|---|
| 0 | 2–3 | 3 |
| 1 | 2–3 | 6 |
| 2 | 3–5 | 11 |
| 3 | 7–10 | 21 |
| 4 | 7–10 | 31 |
| 5 | 5–7 | 38 |

> Jika waktu terbatas: **Fase 0 + 1 + 2** sudah cukup untuk portofolio yang kuat.
> Fase 3 adalah pembeda "bagus" vs "sangat hebat".

## Dependensi

```
Fase 0 ──▶ Fase 1 ──▶ Fase 2 ──▶ Fase 3 ──▶ Fase 4
                                      └──▶ Fase 5
```

Fase 5 (deploy) bisa dimulai paralel setelah Fase 1, karena Swagger + REST + WS
sudah cukup untuk demo publik.
