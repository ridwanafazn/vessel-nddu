# 12 — Handoff & Master Prompt

Dokumen ini untuk **melanjutkan pekerjaan di sesi baru** (misalnya sesi yang
dibuka di root `C:\Users\Ridwan\Git-v2` agar bisa melihat dua repo sekaligus).

---

## 1. Status terakhir

| Item | Status |
|---|---|
| Audit kode | ✅ selesai |
| Dokumentasi (14 file) | ✅ selesai |
| Rust toolchain | ✅ 1.99.1 terpasang |
| Build | ✅ sukses, 0 warning |
| Mosquitto | ✅ terpasang & berjalan (lihat §4) |
| Port 8080 | ✅ bebas (`smile-fest-api` dimatikan) |
| Git repo | ✅ `git init` + baseline commit |
| **Fase 0 (P0-01..04, P1-01/02/13/14)** | ✅ **selesai & terverifikasi** |
| Unit test | ✅ 15 test lulus |
| Lisensi | ✅ MIT |
| Swagger | ❌ belum (Fase 1) |
| Deploy HF Spaces | ❌ belum (Fase 5 / setelah Swagger) |

---

## 2. Dua repo

| Repo | Path | Stack |
|---|---|---|
| Backend | `C:\Users\Ridwan\Git-v2\vessel-nddu` | Rust + Actix-web |
| Frontend | `C:\Users\Ridwan\Git-v2\ProjectMagang-main` | React 19 + Vite + Tauri 2 |

FE memakai **polling axios**, bukan WebSocket.
Kontrak ada di `docs/02-api-contract.md` dan `docs/11-frontend-integration.md`.

---

## 3. Aturan kerja (wajib)

1. **Baca dokumen dulu**, minimal: `README.md`, `00-context.md`,
   `02-api-contract.md`, `04-audit-findings.md`, `06-tasks.md`.
2. **Jangan mulai kode** sebelum konfirmasi pengguna.
3. **Urutan:** P0 → verifikasi → P1 → verifikasi → P2.
4. **Satu task = satu commit.** Repo sudah `git init`; lihat `git log`.
5. **Jangan breaking change** ke FE. Lihat `02-api-contract.md` §4.
6. **Setiap perbaikan wajib punya bukti** (curl/log/test).
7. Jangan mengubah `docs/` tanpa alasan; docs adalah sumber kebenaran.

---

## 4. Environment (sudah disiapkan)

```powershell
# Rust
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
rustc --version     # 1.99.1

# Build & run
cd C:\Users\Ridwan\Git-v2\vessel-nddu
cargo build
.\target\debug\vessel.exe
```

### Mosquitto

- Terpasang: `C:\Program Files\mosquitto\`
- Config lokal: `C:\Users\Ridwan\mosquitto\mosquitto.conf`
  ```
  listener 1883 127.0.0.1
  allow_anonymous true
  persistence false
  log_dest stdout
  log_type all
  ```
- Jalankan (manual, foreground/background):
  ```powershell
  & "C:\Program Files\mosquitto\mosquitto.exe" -c "C:\Users\Ridwan\mosquitto\mosquitto.conf" -v
  ```
- Catatan: service `mosquitto` bawaan installer memakai config default
  (bind `::1`, anonymous ditolak). Untuk pengembangan, gunakan config di atas.
- Verifikasi:
  ```powershell
  & "C:\Program Files\mosquitto\mosquitto_sub.exe" -h 127.0.0.1 -t "vessel/#" -v
  ```

### Port

| Port | Protokol |
|---|---|
| 8080 | REST |
| 8081 | WebSocket |
| 9001 | TCP (stub) |
| 1883 | MQTT |

---

## 5. Keputusan yang sudah final

| ADR | Keputusan |
|---|---|
| ADR-001 | Backend-only, tanpa FE |
| ADR-002 | Contract-first, backward-compatible |
| ADR-003 | Swagger: `utoipa` + `utoipa-swagger-ui` (UI + JSON) |
| ADR-004 | Mosquitto native, tanpa Docker |
| ADR-005 | **Hugging Face Spaces (Docker)** untuk portofolio |
| ADR-006 | Urutan P0 → P1 → P2 |
| ADR-007 | Talker NMEA: `IIVHW`, `SDDBT`, `SDDPT`, `SDDBK` |
| ADR-008 | Config MQTT **global** (env + `/api/mqtt/config`) |
| ADR-009 | Lisensi **MIT** |
| ADR-010 | Modular monolith |
| ADR-011 | GPS: NMEA default + `?format=json` |

## 6. Keputusan yang masih terbuka

| ADR | Pertanyaan | Rekomendasi |
|---|---|---|
| — | TCP server 9001: implementasi nyata? | Fase 3 |
| — | Auth JWT: kapan? | Fase 5 (sebelum deploy publik) |
| — | Lisensi/kepemilikan kode vs PT LEN | Konfirmasi ke pembimbing |

## 6.1 Next: Fase 1

1. `utoipa` + `utoipa-swagger-ui` → `/swagger-ui/` + `/api-docs/openapi.json`
2. `ToSchema` untuk semua DTO, `#[utoipa::path]` untuk semua handler
3. `README.md` root (cara run < 5 menit)
4. Hapus dead code (`src/config/websocket.rs`, `nmea.c` atau aktifkan)
5. Dockerfile multi-stage + `docker-compose.yml` (opsional)
6. Update `02-api-contract.md` dengan link Swagger

---

## 7. Master prompt untuk sesi baru

Salin blok berikut ke sesi baru:

```text
Kamu melanjutkan proyek backend Rust "vessel-nddu" (simulator sensor kapal +
gateway NMEA/MQTT/WS) di C:\Users\Ridwan\Git-v2\vessel-nddu.

Sebelum melakukan apa pun, baca berurutan:
1. docs/README.md
2. docs/00-context.md
3. docs/01-architecture.md
4. docs/02-api-contract.md
5. docs/04-audit-findings.md
6. docs/06-tasks.md
7. docs/07-rust-actix-guide.md
8. docs/10-decisions.md

Konteks penting:
- Ini backend murni. FE (React+Tauri) ada di C:\Users\Ridwan\Git-v2\ProjectMagang-main
  dan memakai polling axios, bukan WebSocket.
- Kontrak API TIDAK boleh breaking. GET /api/gps saat ini mengembalikan NMEA
  text/plain, sedangkan FE mengharapkan JSON. Solusi yang disepakati:
  tambah ?format=json tanpa mengubah default.
- MQTT wajib. Mosquitto sudah terpasang; config di
  C:\Users\Ridwan\mosquitto\mosquitto.conf; jalankan dengan -c file itu.
- Deployment target: Hugging Face Spaces (Docker), gratis, tanpa kartu.
  Syarat utama: Swagger/OpenAPI harus ada.
- Swagger memakai utoipa + utoipa-swagger-ui (UI + JSON).

Fase 0 SUDAH SELESAI (2026-10-31): 4 bug P0 + P1-01/02/13/14, 15 unit test,
0 warning, terverifikasi end-to-end. Lisensi MIT. MQTT global.

Tugas saat ini: Fase 1 (lihat docs/06-tasks.md), urutan:
1. Tambah utoipa + utoipa-swagger-ui
2. #[derive(ToSchema)] untuk semua DTO
3. #[utoipa::path] untuk semua handler
4. Mount /swagger-ui/ dan /api-docs/openapi.json
5. README.md root (cara run < 5 menit)
6. Hapus dead code (src/config/websocket.rs, nmea.c) atau aktifkan
7. Dockerfile multi-stage (opsional, untuk HF Spaces nanti)

Aturan:
- Jangan mulai kode sebelum saya bilang "gas".
- Satu task = satu commit (repo sudah git init).
- Setiap task: tunjukkan bukti verifikasi (curl/log/test).
- Jangan mengubah docs/02-api-contract.md tanpa memberi tahu saya.
- Jangan menyentuh FE kecuali saya minta.

Mulai dengan membaca dokumen dan meringkas: (a) status saat ini,
(b) 4 bug P0, (c) keputusan yang masih terbuka, (d) rencana eksekusi.
Jangan menulis kode dulu.
```

---

## 8. Checklist sesi baru

- [ ] Buka sesi di `C:\Users\Ridwan\Git-v2` (agar dua repo terlihat)
- [ ] Paste master prompt di atas
- [ ] Pastikan agent membaca `docs/`
- [ ] Konfirmasi keputusan terbuka (ADR-008, ADR-009, GPS JSON, git init)
- [ ] Baru mulai Fase 0

---

## 9. Jika tetap di sesi ini

Tidak masalah — sesi ini sudah punya konteks lengkap. Yang perlu dilakukan:
1. Konfirmasi keputusan terbuka.
2. Bilang "gas Fase 0".
3. Saya eksekusi task per task dengan bukti.
