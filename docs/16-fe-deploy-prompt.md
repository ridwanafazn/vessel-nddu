# 16 — Initial Prompt: Deploy FE React + Tauri

Salin blok di bagian **Prompt** ke sesi baru di repo
`C:\Users\Ridwan\Git-v2\ProjectMagang-main`.

---

## Prompt

```text
Kamu adalah senior frontend engineer. Tugasmu: menyiapkan dan men-deploy
frontend React + Tauri milik Ridwan agar bisa consume backend yang SUDAH LIVE.

## Konteks proyek

- FE repo: C:\Users\Ridwan\Git-v2\ProjectMagang-main
  Stack: React 19, Vite 7, Tauri 2, axios. Navigasi berbasis state (tanpa router).
  Halaman: Login, GPS, Gyro, Weather, Compass, Speed & Depth.
- BE repo: C:\Users\Ridwan\Git-v2\vessel-nddu (Rust + Actix-web)
- BE live: https://vessel-nddu.onrender.com
- Swagger: https://vessel-nddu.onrender.com/swagger-ui/
- OpenAPI: https://vessel-nddu.onrender.com/api-docs/openapi.json
- Health:  https://vessel-nddu.onrender.com/health
- WebSocket: wss://vessel-nddu.onrender.com/ws
- Deploy BE: Render Free (Docker). Cold start ±60 detik setelah 15 menit idle.

## Kontrak API yang WAJIB dipakai

Semua response (kecuali auth) berbentuk:
  { "message": string, "data": object }

Auth:
  POST /api/auth/login
  body: { "username": "admin", "password": "admin" }
  response: { "success": true, "message": "...", "username": "admin" }
  Catatan: hardcoded demo, belum ada JWT/token.

Endpoint per halaman:
- GPS:        GET/POST/PATCH/DELETE /api/gps
              GET/POST/PATCH/DELETE /api/gps/config
- Gyro:       GET/POST/PATCH/DELETE /api/gyro
              GET/POST/PATCH/DELETE /api/gyro/config
- Weather:    GET/POST/PATCH/DELETE /api/weather
              GET/POST/PATCH/DELETE /api/weather/config
- Compass:    GET/POST/PATCH/DELETE /api/compass
              GET/POST/PATCH/DELETE /api/compass/config
- Speed:      GET/POST/PATCH/DELETE /api/speed
- Depth:      GET/POST/PATCH/DELETE /api/depth
- Navigation: GET/POST/PATCH/DELETE /api/navigation/config
- MQTT:       GET/PATCH /api/mqtt/config
- Health:     GET /health

PENTING — GPS:
  GET /api/gps            -> text/plain NMEA (BUKAN JSON)
  GET /api/gps?format=json -> { message, data: { latitude, longitude, sog, cog,
                              variation, is_running, last_update } }
  FE saat ini memanggil /api/gps tanpa ?format=json -> response.data.data undefined.
  WAJIB ubah ke ?format=json.

Bentuk data:
- GPS: latitude, longitude, sog (knots), cog (deg), variation (deg), is_running, last_update
- Gyro: yaw, pitch, roll, yaw_rate, is_running, last_update
- Weather: air_temperature, air_humidity, air_pressure, wind_direction_relative,
  wind_velocity, is_running, last_update
- Compass: heading, deviation, variation, is_running, last_update
- Speed: speed_through_water, is_running, last_update
- Depth: ocean_water_depth, constant_clearance, under_keel_clearance, is_running, last_update
- Config: ip, port, username, password, topics[], update_rate,
  enabled_sentences[] (opsional), tcp_enabled (khusus GPS)

Status penting:
- 404 = state belum dibuat (tampilkan empty state + tombol "Create")
- 409 = config belum lengkap / instance sudah ada
- 401 = login gagal
- 200/201 = sukses

WebSocket (opsional, sangat disarankan):
  wss://vessel-nddu.onrender.com/ws
  Pesan: { "type": "gps_update", "json": {...}, "gprmc": "...", ... }
         { "type": "gyro_update", "data": {...} }
         { "type": "weather_update", "data": {...} }
         { "type": "compass_update", "data": {...} }
         { "type": "speed_update", "data": {...} }
         { "type": "depth_update", "data": {...} }
  Catatan: gps_update memakai key "json", yang lain "data".

## Masalah FE saat ini yang harus diperbaiki

1. API base URL hardcoded "http://localhost:8080" di 6 file
   (GPSPage, GyroPage, WeatherPage, MagneticCompassPage, SpeedDepthPage, LoginPage).
   -> Sentralisasi ke src/api/client.js, baca import.meta.env.VITE_API_URL.
2. GPS GET tidak memakai ?format=json -> halaman GPS kosong.
3. Tidak ada handling cold start Render (request pertama bisa 60 detik).
4. setInterval bisa menumpuk request -> ganti recursive setTimeout.
5. Tidak ada indikator offline / retry / error boundary.
6. Config popup mengirim ip/port; di cloud isi default 127.0.0.1:1883,
   tcp_enabled=false.
7. Belum ada WebSocket; polling tetap boleh, tapi WS lebih baik.
8. Tauri CSP masih null; jangan rusak konfigurasi Tauri.

## Yang harus kamu lakukan

Fase A — Rapikan kode:
1. Buat src/api/client.js (axios instance, baseURL dari VITE_API_URL,
   timeout 60s, interceptor error).
2. Buat src/api/endpoints.js (fungsi per domain).
3. Ganti semua hardcoded URL.
4. Perbaiki GPS -> ?format=json.
5. Ganti setInterval -> recursive setTimeout.
6. Tambah loading state "Menghubungkan ke server..." + retry saat cold start.
7. Tambah empty state untuk 404 dan pesan jelas untuk 409.
8. Pastikan login admin/admin tetap jalan.
9. Jangan ubah src-tauri kecuali perlu.

Fase B — Env:
- Buat .env.example:
  VITE_API_URL=https://vessel-nddu.onrender.com
  VITE_WS_URL=wss://vessel-nddu.onrender.com/ws
- Buat .env.local untuk dev (boleh localhost:8080).
- Pastikan Vite membaca env (prefix VITE_).

Fase C — Deploy web (gratis, tanpa kartu):
- Pilih salah satu: Vercel / Netlify / Cloudflare Pages.
- Build command: npm run build
- Output: dist
- Set env VITE_API_URL & VITE_WS_URL di dashboard.
- SPA rewrite semua path ke /index.html.
- Tauri TIDAK di-deploy ke web; web hanya React build.
- Verifikasi semua halaman terhadap API live.

Fase D — Verifikasi:
- Login admin/admin berhasil.
- GPS: create config + state, data tampil, NMEA/JSON benar.
- Gyro, Weather, Compass, Speed/Depth: create + update + delete.
- Uji 404 (belum create) dan 409 (config kosong).
- Uji cold start: biarkan 15 menit, buka lagi, pastikan loading/retry.
- Uji WebSocket wss://.../ws (jika diimplementasikan).

Fase E — Dokumentasi:
- README FE: cara run, env, deploy, screenshot, link live.
- Catat keterbatasan: Render free cold start, state in-memory (hilang saat restart),
  auth demo, TCP disabled.

## Aturan

- Jangan mengubah backend. Jika butuh perubahan API, laporkan dulu.
- Jangan commit .env / token.
- Jangan aktifkan tcp_enabled (bisa spam MQTT).
- CORS backend allow_any_origin, jadi web & Tauri boleh.
- Setiap langkah: tunjukkan bukti (screenshot/log/curl).
- Mulai dengan membaca repo FE dan meringkas:
  (a) struktur, (b) 6 halaman, (c) hardcoded URL, (d) rencana eksekusi.
  Jangan menulis kode sebelum saya bilang "gas".
```

---

## Catatan untuk pemilik proyek

- Backend live: <https://vessel-nddu.onrender.com>
- Swagger: <https://vessel-nddu.onrender.com/swagger-ui/>
- Render free cold start ±60 detik; FE harus sabar & retry.
- State backend in-memory; setelah restart, FE perlu create ulang config/state.
- Login demo: `admin` / `admin`.
