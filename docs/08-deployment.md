# 08 — Deployment

## 1. Kebutuhan runtime

| Kebutuhan | Wajib untuk |
|---|---|
| Proses long-running (thread + timer 10 Hz) | Semua sensor |
| HTTP server (REST) | FE, Swagger |
| WebSocket server | Dashboard real-time |
| **Inbound TCP listener** (9001) | Integrator NMEA |
| Outbound TCP client | Stream ke host eksternal |
| MQTT client persisten | Publikasi |
| State in-memory | Semua |

## 2. Matriks opsi gratis

| Platform | Tanpa kartu | Binary Rust apa adanya | Inbound TCP | MQTT broker | WS | REST | Catatan |
|---|---|---|---|---|---|---|---|
| **Hugging Face Spaces** (Docker) | ✅ | ✅ | ❌ | ❌ (outbound OK) | ✅ | ✅ | 2 vCPU/16 GB, sleep 48 jam |
| **Cloudflare Workers** | ✅ | ❌ (rewrite WASM/TS) | ❌ | ❌ | ✅ (DO) | ✅ | 100k req/hari, CPU 10 ms |
| **Oracle Cloud Always Free** | ❌ kartu | ✅ | ✅ | ✅ | ✅ | ✅ | 4 ARM/24 GB, terbaik teknis |
| **Google Cloud e2-micro** | ❌ kartu | ✅ | ✅ | ✅ | ✅ | ✅ | 1 GB RAM, region US |
| **Fly.io** | ❌ kartu | ✅ | ✅ | ✅ | ✅ | ✅ | Paling mudah untuk TCP |
| **Render Free** | ⚠️ | ✅ | ❌ | ✅ | ✅ | ✅ | Sleep 15 menit |
| **Koyeb Free** | ⚠️ | ✅ | ⚠️ | ✅ | ✅ | ✅ | 512 MB |
| **Railway** | ⚠️ | ✅ | ✅ | ✅ | ✅ | ✅ | Kredit trial |
| **Lokal + Cloudflare Tunnel** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | Butuh PC hidup |

> **Catatan Oracle:** free tier biasanya **meminta kartu** untuk verifikasi
> identitas (tidak ditagih). Jika Anda benar-benar tanpa kartu, Oracle tidak bisa.

## 3. Rekomendasi

### Opsi A — Portofolio tanpa kartu (rekomendasi)

**Hugging Face Spaces (Docker) + broker MQTT gratis (HiveMQ Cloud / EMQX Serverless)**

```
┌──────────────────────────────┐
│  Hugging Face Space (Docker) │
│  vessel-sim :7860            │
│   ├── REST /api/*            │
│   ├── Swagger /swagger-ui    │
│   ├── WS   /ws               │
│   └── MQTT outbound ─────────┼──▶ HiveMQ Cloud (free)
└──────────────────────────────┘
```

- REST + WS + Swagger **jalan**.
- MQTT publish **jalan** (outbound ke broker publik).
- TCP server **tidak** (keterbatasan platform).
- TCP client outbound **jalan** (bisa demo stream ke host lain).
- Tanpa kartu, gratis, cukup untuk portofolio.

**Yang perlu diubah:**
1. Satu port HTTP (7860) → gabungkan REST + WS di port yang sama, atau
   gunakan path `/ws` pada server yang sama.
2. Bind `0.0.0.0:7860`.
3. MQTT URL dari env (bukan `127.0.0.1`).
4. `Dockerfile` multi-stage.

### Opsi B — Cloudflare Workers (jika tetap ingin CF)

**Bukan deploy, tapi rewrite.** Yang hilang:
- TCP server & client,
- MQTT broker,
- thread 10 Hz (diganti Durable Object alarm),
- `Arc<RwLock>` (diganti DO storage).

Yang didapat:
- REST + WS + Swagger di edge,
- gratis tanpa kartu,
- latensi rendah global.

**Estimasi:** 2–4 minggu. Tidak disarankan untuk Fase 0–2.

### Opsi C — Hybrid (kompromi)

```
Cloudflare Worker (edge)
  ├── /            → Swagger UI statis
  ├── /api/*       → proxy ke origin
  └── /ws          → proxy WebSocket
        │
        ▼
Hugging Face Space (origin Rust)
```

- Anda tetap "deploy di Cloudflare" (Worker + custom domain).
- Core tetap binary Rust asli.
- Tidak perlu rewrite.
- Tambahan latensi ~50–150 ms.

**Ini pilihan paling realistis** jika ingin branding Cloudflare tanpa rewrite.

## 4. Rekomendasi final

| Tujuan | Pilihan |
|---|---|
| Portofolio cepat, tanpa kartu | **Opsi A** (HF Spaces) |
| Ingin URL Cloudflare | **Opsi C** (Worker proxy + HF origin) |
| Butuh TCP server nyata | Oracle/Fly (butuh kartu) |
| Demo lokal | `localhost` + Cloudflare Tunnel |

## 5. Checklist deploy (Opsi A)

- [ ] `Dockerfile` multi-stage (`rust:1.99` → `debian:bookworm-slim`)
- [ ] Bind `0.0.0.0:7860`
- [ ] `MQTT_URL` dari env
- [ ] `RUST_LOG=info`
- [ ] Swagger UI aktif
- [ ] Healthcheck `/health`
- [ ] `.dockerignore` (`target/`)
- [ ] Ukuran image < 200 MB
- [ ] Uji `docker run -p 7860:7860`
- [ ] Push ke HF Spaces (Docker SDK)
- [ ] Set secrets (MQTT URL/credentials)
- [ ] Uji dari luar: REST, WS, Swagger

## 6. Yang tidak akan jalan di platform gratis

| Fitur | Alasan |
|---|---|
| TCP server 9001 | Platform hanya expose HTTP |
| MQTT broker | Bukan broker; pakai layanan eksternal |
| State persisten antar restart | Perlu volume/database |
| Multi-instance | State in-memory tidak dibagi |

## 7. Keputusan yang masih terbuka

Lihat `10-decisions.md` **ADR-005**. Pilih **A**, **B**, atau **C**
sebelum Fase 5 dimulai.
