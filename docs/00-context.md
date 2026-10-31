# 00 — Konteks Proyek

## 1. Latar belakang

Proyek ini lahir dari magang di **PT LEN Industri (Persero)**. Tujuannya membangun
**simulator sensor kapal + gateway data** yang bisa menggantikan hardware asli saat
pengujian perangkat lunak maritim.

Nama `NDDU` diduga merujuk pada *Navigation Data Display/Distribution Unit*.
**Perlu konfirmasi** ke pembimbing.

## 2. Stakeholder

| Peran | Kepentingan |
|---|---|
| **Ridwan** (backend) | Simulator, API, MQTT, TCP, kualitas kode |
| **Rekan FE** (React + Tauri) | Kontrak REST yang stabil, data real-time |
| **Pembimbing LEN** | Alat uji yang bisa dipercaya, sesuai standar NMEA |
| **Konsumen akhir** | ECDIS / VDR / VMS / dashboard LEN |

## 3. Masalah yang diselesaikan

1. Hardware sensor kapal mahal dan tidak selalu tersedia.
2. Pengujian software navigasi butuh data NMEA yang **berulang dan terkontrol**.
3. Integrasi ke platform IoT butuh **MQTT**.
4. Demo ke customer butuh data yang **terlihat realistis**.

## 4. Scope

### In scope
- Simulasi 9–10 sensor (GPS, gyro, kompas, speed, depth, suhu, kelembapan, angin, tekanan).
- REST API untuk kontrol & konfigurasi.
- Publikasi MQTT (JSON + NMEA per sentence).
- WebSocket broadcast.
- TCP client streaming NMEA.
- Konfigurasi runtime (rate, topic, sentence).

### Out of scope (sekarang)
- Frontend (dikerjakan rekan, React + Tauri).
- Database / histori time-series.
- AIS / radar / engine control.
- Autentikasi produksi (masih `admin/admin`).
- Deployment multi-tenant.

## 5. Definisi sukses

### Untuk portofolio
- Bisa diakses publik (REST + WS + Swagger).
- Ada dokumentasi kontrak yang rapi.
- Ada test & CI.
- Menunjukkan pemahaman **NMEA 0183**, **MQTT**, **Rust async**, **Actix-web**.

### Untuk klien
- NMEA **valid 100%** (bisa diparse alat lain).
- Data **tidak pernah macet** walau broker mati.
- Bisa **replay skenario** yang sama.
- Bisa **mensimulasikan kegagalan sensor**.

## 6. Fakta terverifikasi (bukan asumsi)

Diuji pada Windows 11, Rust 1.99.1:

| Fakta | Bukti |
|---|---|
| Build sukses | `cargo build` 3m44s, 0 warning |
| REST hidup | `POST /api/auth/login` → `{"success":true}` |
| NMEA dihasilkan | `$GPRMC,122334.35,A,0611.9924,S,10648.0077,E,...*26` |
| TCP stream jalan | 8 baris NMEA/detik tertangkap |
| MQTT publish jalan | 11 topik terbit saat broker tersedia |
| WS jalan | 15 pesan/5 detik saat broker tersedia |
| **WS mati tanpa broker** | 0 pesan/5 detik → Bug #1 |
| **Depth beku di TCP** | `$DBT` konstan 6 detik → Bug #4 |

## 7. Asumsi yang perlu dikonfirmasi

| # | Asumsi | Dampak jika salah |
|---|---|---|
| 1 | MQTT diminta klien untuk integrasi platform LEN | Prioritas MQTT bisa turun |
| 2 | FE hanya butuh 6 halaman (GPS, Gyro, Weather, Compass, Speed/Depth, Login) | Kontrak bisa lebih sempit |
| 3 | `NDDU` = Navigation Data Display Unit | Penamaan & branding |
| 4 | Kode milik pribadi, bukan aset LEN | Lisensi AGPL bisa bermasalah |
| 5 | Deployment portofolio boleh tanpa TCP server | Pilihan platform berubah |

## 8. Risiko

| Risiko | Tingkat | Mitigasi |
|---|---|---|
| Lisensi **AGPL-3.0** tidak cocok untuk produk LEN | 🟠 | Konfirmasi kepemilikan; pertimbangkan MIT/Apache-2.0 untuk portofolio |
| FE dan backend **versi kontrak berbeda** | 🔴 | `02-api-contract.md` + versioning |
| Bug deadlock MQTT membuat demo gagal | 🔴 | P0-01 |
| Tidak ada test → refactor berbahaya | 🟠 | P1 test suite |
| Deployment gratis tidak mendukung TCP | 🟡 | Terima keterbatasan / pakai VPS |

## 9. Prinsip proyek

1. **Contract-first** — dokumentasi API sebelum implementasi.
2. **Backward-compatible** — FE sudah berjalan; jangan rusak.
3. **Fail-safe** — broker mati tidak boleh menghentikan data.
4. **Observable** — kalau tidak terlihat di log/metrics, dianggap tidak ada.
5. **Testable** — setiap bug yang diperbaiki harus punya test regresi.
