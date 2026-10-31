# Dokumentasi Proyek Vessel NDDU

Backend simulator & gateway data kapal (NMEA 0183 / MQTT / WebSocket / REST).
Dokumen di folder ini adalah **sumber kebenaran** untuk pengembangan bertahap.

## Cara membaca

| Urutan | Dokumen | Untuk siapa |
|---|---|---|
| 1 | [00-context.md](00-context.md) | Semua — latar belakang & tujuan |
| 2 | [01-architecture.md](01-architecture.md) | Backend — arsitektur sekarang & target |
| 3 | [02-api-contract.md](02-api-contract.md) | Backend + FE — kontrak REST |
| 4 | [03-realtime-contract.md](03-realtime-contract.md) | Backend + integrator — WS, MQTT, NMEA, TCP |
| 5 | [04-audit-findings.md](04-audit-findings.md) | Backend — temuan bug & bukti |
| 6 | [05-roadmap.md](05-roadmap.md) | Semua — fase pengembangan |
| 7 | [06-tasks.md](06-tasks.md) | Backend — checklist kerja |
| 8 | [07-rust-actix-guide.md](07-rust-actix-guide.md) | Backend — panduan idiomatik |
| 9 | [08-deployment.md](08-deployment.md) | Semua — opsi deploy gratis |
| 10 | [09-runbook.md](09-runbook.md) | Semua — cara menjalankan |
| 11 | [10-decisions.md](10-decisions.md) | Semua — catatan keputusan (ADR) |
| 12 | [11-frontend-integration.md](11-frontend-integration.md) | FE (React + Tauri) |
| 13 | [12-handoff.md](12-handoff.md) | Sesi baru — master prompt & status |
| 14 | [13-hf-deploy.md](13-hf-deploy.md) | Deploy Hugging Face Spaces (manual steps) |
| 15 | [14-git-strategy.md](14-git-strategy.md) | Strategi merge riwayat & publikasi GitHub |
| 16 | [15-render-deploy.md](15-render-deploy.md) | Deploy gratis ke Render (tanpa kartu) |

## Legenda status

| Simbol | Arti |
|---|---|
| ✅ | Selesai / terverifikasi |
| 🟡 | Sebagian |
| ❌ | Belum ada / gagal |
| ⏳ | Direncanakan |
| 🔴 P0 | Kritis — harus segera |
| 🟠 P1 | Serius — sebelum fitur baru |
| 🟡 P2 | Kebersihan teknis |

## Aturan kerja

1. **Kontrak dulu, kode kemudian.** Setiap perubahan API wajib update `02-api-contract.md`.
2. **Jangan breaking change** tanpa koordinasi FE. Lihat `11-frontend-integration.md`.
3. **Satu task = satu commit** (setelah repo di-`git init`).
4. **P0 selesai + terverifikasi** sebelum menyentuh P1. P1 selesai sebelum P2.
5. Setiap perbaikan bug **wajib** punya bukti verifikasi (curl/log/test).

## Status saat ini

| Item | Status |
|---|---|
| Build di Windows | ✅ `cargo build` sukses, 0 warning |
| REST API | ✅ Berjalan |
| WebSocket | ✅ Berjalan, **tidak tergantung broker MQTT** |
| MQTT publish | ✅ Fail-safe (queue bounded, drop dihitung) |
| MQTT config | ✅ Global, runtime via `/api/mqtt/config` |
| TCP client (outbound) | ✅ Berjalan, depth ikut update |
| TCP server (9001) | ❌ Masih stub (echo) |
| Swagger | ✅ `/swagger-ui/` + `/api-docs/openapi.json` |
| Test | ✅ 15 unit test |
| Fase 0 | ✅ Selesai & terverifikasi |
| Fase 1 | ✅ Selesai (Swagger, README, Docker) |
| Fase 2 | 🟡 Sebagian (CI, shutdown, release, `/health`) |
| Docker | ✅ Dockerfile + compose (uji lokal opsional) |
| HF Spaces | 🟡 image siap (`deploy/hf`), push manual |
| Lisensi | ✅ MIT |
| Dokumentasi | ✅ Folder ini (14 file) |
