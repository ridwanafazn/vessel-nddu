# 15 — Deploy Render Free (gratis, tanpa kartu)

> **Menggantikan Hugging Face Spaces Docker**, karena HF Docker sekarang
> **Paid** (2026). Render Free mendukung Docker + WebSocket tanpa kartu.

## 1. Kenapa Render

| Kebutuhan | Render Free |
|---|---|
| Docker | ✅ |
| REST | ✅ |
| WebSocket | ✅ |
| MQTT internal (Mosquitto di container) | ✅ |
| Tanpa kartu | ✅ (free web service) |
| RAM | 512 MB |
| Spin-down | 15 menit idle, cold start ±1 menit |
| TCP server mentah | ❌ (tidak diekspos) |

Image sudah diuji lokal:
- `PORT=10000`, user `1000:1000`
- `/health` 200, `/swagger-ui/` 200
- `/ws` menerima 5 pesan/5 detik
- MQTT internal terhubung

## 2. File yang dipakai

| File | Fungsi |
|---|---|
| `render.yaml` | Blueprint (service, plan free, Dockerfile, health check) |
| `deploy/hf/Dockerfile` | Image Rust + nginx + Mosquitto |
| `deploy/hf/nginx.conf` | `listen __PORT__`, `/ws` → 8081, `/` → 8080 |
| `deploy/hf/entrypoint.sh` | Substitusi `$PORT`, jalankan Mosquitto + vessel + nginx |
| `deploy/hf/mosquitto.conf` | Broker internal |

> Nama folder `hf` dipertahankan agar tidak memutus skrip lama; image-nya
> generik untuk HF/Render.

## 3. Langkah deploy (manual)

### 3.1 Push ke GitHub

Sudah dilakukan: <https://github.com/ridwanafazn/vessel-nddu>

### 3.2 Buat akun Render

1. <https://dashboard.render.com/register>
2. Pilih **GitHub** → authorize
3. Tidak perlu kartu untuk Free web service

### 3.3 New Web Service

1. **New** → **Web Service**
2. Connect repo `ridwanafazn/vessel-nddu`
3. Isi:
   - **Name**: `vessel-nddu`
   - **Region**: Singapore (terdekat)
   - **Branch**: `main`
   - **Runtime**: **Docker**
   - **Dockerfile Path**: `./deploy/hf/Dockerfile`
   - **Plan**: **Free**
   - **Health Check Path**: `/health`
4. Environment variables:
   ```
   RUST_LOG=info
   MQTT_HOST=127.0.0.1
   MQTT_PORT=1883
   ```
   (`PORT` diisi otomatis oleh Render.)
5. **Create Web Service** → tunggu build (±5–10 menit).

### 3.4 Alternatif: Blueprint

Jika memakai **New → Blueprint**, Render membaca `render.yaml` otomatis.

## 4. Verifikasi

Ganti `<service>` dengan nama service Anda:

```powershell
curl.exe -s https://<service>.onrender.com/health
curl.exe -s -o NUL -w "%{http_code}`n" https://<service>.onrender.com/swagger-ui/
```

WebSocket:

```js
const ws = new WebSocket("wss://<service>.onrender.com/ws");
ws.onmessage = (e) => console.log(e.data);
```

Buat GPS agar WS mengirim data:

```powershell
curl.exe -s -X POST https://<service>.onrender.com/api/gps/config `
  -H "Content-Type: application/json" `
  -d '{\"ip\":\"127.0.0.1\",\"port\":1883,\"update_rate\":1000,\"topics\":[\"vessel/gps\"]}'

curl.exe -s -X POST https://<service>.onrender.com/api/gps `
  -H "Content-Type: application/json" `
  -d '{\"latitude\":-6.2,\"longitude\":106.8,\"sog\":12.5,\"cog\":45,\"is_running\":true}'
```

## 5. Keterbatasan Free

| Hal | Catatan |
|---|---|
| Spin-down | 15 menit tanpa traffic; cold start ±1 menit |
| RAM | 512 MB (cukup: Rust + nginx + Mosquitto) |
| Instance hours | 750 jam/bulan (satu service 24/7 ≈ 730 jam) |
| Filesystem | Ephemeral |
| TCP 9001 | Tidak diekspos |
| MQTT eksternal | Internal; untuk subscriber luar gunakan broker publik via env |

### Keep-warm (opsional)

Agar tidak spin-down, ping `/health` tiap 10 menit (cron eksternal atau
GitHub Actions schedule). Ini tetap dalam 750 jam/bulan.

## 6. Troubleshooting

| Gejala | Solusi |
|---|---|
| Build gagal `curl not found` | Pastikan builder menginstal `curl` (sudah) |
| 502 | Cek log Render; `vessel` gagal start |
| Health check gagal | Pastikan `/health` 200 dan `PORT` benar |
| WS gagal | Gunakan `wss://<service>.onrender.com/ws` |
| Swagger 404 | Cek `HTTP_ADDR=127.0.0.1:8080` dan nginx proxy |
| Cold start lama | Normal di Free; ping berkala |

## 7. Checklist

- [ ] Repo GitHub public
- [ ] Render account via GitHub (tanpa kartu)
- [ ] Web Service Docker, plan Free
- [ ] Dockerfile path `./deploy/hf/Dockerfile`
- [ ] Health check `/health`
- [ ] Build sukses
- [ ] `/health` 200
- [ ] `/swagger-ui/` 200
- [ ] `/ws` menerima pesan
- [ ] Link dimasukkan ke CV
