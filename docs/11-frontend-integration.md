# 11 — Panduan Integrasi Frontend (React + Tauri)

Dokumen untuk rekan FE. Fokus: **kontrak yang stabil** dan **cara konsumsi data**.

---

## 1. Ringkasan kondisi

| Hal | FE sekarang | Backend |
|---|---|---|
| Transport | REST polling (`setInterval` + axios) | REST + WS + MQTT |
| WebSocket | ❌ tidak dipakai | ✅ tersedia |
| Base URL | `http://localhost:8080` | `127.0.0.1:8080` |
| Auth | `POST /api/auth/login` | `admin/admin` |
| Halaman | GPS, Gyro, Weather, Compass, Speed/Depth, Login | Semua tersedia |

---

## 2. Kontrak yang dipakai FE

| Halaman | Endpoint | Status |
|---|---|---|
| Login | `POST /api/auth/login` | ✅ |
| GPS | `GET/POST/PATCH/DELETE /api/gps` | 🔴 **GET mismatch** |
| GPS config | `GET/PATCH/DELETE /api/gps/config` | ✅ |
| Gyro | `GET/POST/PATCH/DELETE /api/gyro` | ✅ |
| Gyro config | `GET/PATCH/DELETE /api/gyro/config` | ✅ |
| Weather | `GET/POST/PATCH/DELETE /api/weather` | ✅ |
| Weather config | `GET/PATCH/DELETE /api/weather/config` | ✅ |
| Compass | `GET/POST/PATCH/DELETE /api/compass` | ✅ |
| Compass config | `GET/PATCH/DELETE /api/compass/config` | ✅ |
| Speed | `GET/POST/PATCH/DELETE /api/speed` | ✅ |
| Depth | `GET/POST/PATCH/DELETE /api/depth` | ✅ |
| Navigation config | `GET/PATCH/DELETE /api/navigation/config` | ✅ |

### 2.1 Mismatch GPS — ✅ SUDAH DIPERBAIKI

**FE mengharapkan:**

```js
const response = await axios.get("http://localhost:8080/api/gps");
const gpsData = response.data.data; // { latitude, longitude, sog, cog, variation, is_running }
```

**Backend sekarang mendukung keduanya:**

```
GET /api/gps              → text/plain NMEA (default, untuk integrator)
GET /api/gps?format=json  → { message, data: { ... } } (untuk FE)
```

**Yang perlu dilakukan FE (satu baris):**

```js
const GPS_API = `${API_BASE_URL}/api/gps?format=json`;
```

Sudah diverifikasi: JSON mengembalikan `{data:{cog,is_running,last_update,latitude,longitude,sog,variation}}`.

---

## 3. Pola konsumsi yang disarankan

### 3.1 API client terpusat

```js
// src/api/client.js
import axios from "axios";

export const api = axios.create({
  baseURL: import.meta.env.VITE_API_URL ?? "http://localhost:8080",
  timeout: 5000,
});

api.interceptors.response.use(
  (r) => r,
  (e) => {
    if (e.code === "ERR_NETWORK") return Promise.reject({ kind: "offline" });
    if (e.response?.status === 404) return Promise.reject({ kind: "empty" });
    return Promise.reject({ kind: "error", status: e.response?.status });
  }
);
```

### 3.2 Polling yang benar

```js
useEffect(() => {
  let cancelled = false;
  let timer;

  const tick = async () => {
    try {
      const { data } = await api.get("/api/gps?format=json");
      if (!cancelled) setState(data.data);
    } catch (e) {
      if (!cancelled && e.kind === "offline") setOffline(true);
    } finally {
      if (!cancelled) timer = setTimeout(tick, rate);
    }
  };

  tick();
  return () => { cancelled = true; clearTimeout(timer); };
}, [rate]);
```

Gunakan `setTimeout` rekursif, bukan `setInterval`, agar tidak menumpuk
request saat backend lambat.

### 3.3 Migrasi ke WebSocket (opsional, Fase 3)

```js
const ws = new WebSocket("ws://127.0.0.1:8081");
ws.onmessage = (e) => {
  const msg = JSON.parse(e.data);
  switch (msg.type) {
    case "gps_update": setGps(msg.json); break;
    case "gyro_update": setGyro(msg.data); break;
    // ...
  }
};
```

Keuntungan: tanpa polling, latensi lebih rendah.
Kekurangan: perlu reconnect + fallback ke REST.

---

## 4. Tauri: hal yang perlu diperhatikan

| Hal | Catatan |
|---|---|
| CSP | `tauri.conf.json` sekarang `"csp": null` → longgar. Pertimbangkan CSP ketat |
| Origin | `tauri://localhost` / `http://tauri.localhost`; tambahkan ke CORS backend |
| HTTP plugin | Jika memakai `@tauri-apps/plugin-http`, tambahkan permission di `capabilities/default.json` |
| Dev | Vite di `localhost:1420`; CORS backend harus mengizinkan |
| WS | WebSocket dari WebView Tauri umumnya langsung jalan; uji di Windows |
| Build | `frontendDist: "../dist"`; pastikan `npm run build` sukses |

Contoh CORS backend yang benar (Fase 1):

```rust
Cors::default()
    .allowed_origin("http://localhost:1420")
    .allowed_origin("tauri://localhost")
    .allowed_origin("http://tauri.localhost")
```

---

## 5. Error handling yang disepakati

| Kondisi | Backend | FE |
|---|---|---|
| Config belum diisi | `409 {message}` | Tampilkan "isi konfigurasi dulu" |
| State belum dibuat | `404 {message}` | Tampilkan empty state |
| Login gagal | `401 {message}` | Tampilkan error |
| Backend mati | network error | Banner "offline" + retry backoff |
| Payload invalid | `400` | Bug FE; log |

---

## 6. Checklist integrasi

- [ ] Sepakati `?format=json` untuk GPS
- [ ] Ganti `setInterval` → `setTimeout` rekursif
- [ ] Tambah interceptor error
- [ ] Tambah indikator offline
- [ ] Uji CORS dari Tauri (bukan hanya browser)
- [ ] Uji saat backend mati → UI tidak crash
- [ ] Uji saat config kosong → pesan jelas
- [ ] (Opsional) migrasi ke WS setelah Fase 3

---

## 7. Yang TIDAK boleh dilakukan FE

- ❌ Mengandalkan field yang tidak ada di `02-api-contract.md`.
- ❌ Mengirim `PATCH` dengan field di luar skema.
- ❌ Memanggil endpoint individual (`/api/temperature`, dll.) tanpa koordinasi —
  backend bisa mengonsolidasikan nanti.
- ❌ Hardcode `localhost` di banyak file; gunakan satu `api client`.

---

## 8. Kontak perubahan kontrak

Setiap perubahan API:
1. Backend update `02-api-contract.md`.
2. Backend beri tahu FE (minimal 1 rilis sebelum breaking).
3. FE update tipe & client.
4. Uji bersama.
