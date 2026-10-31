# 13 — Deploy Hugging Face Spaces

Panduan langkah demi langkah. Semua file image sudah disiapkan di `deploy/hf/`.

> **Tindakan manual diperlukan:** Anda perlu akun Hugging Face + token.
> Agent tidak bisa membuat akun atau menyimpan token Anda.

---

## 1. Arsitektur image HF

```
Internet
  │  HTTPS :7860
  ▼
nginx (di dalam container)
  ├── /            → 127.0.0.1:8080  (Actix: REST + Swagger)
  └── /ws          → 127.0.0.1:8081  (WebSocket)
        │
        ├── vessel (Rust)
        └── mosquitto (127.0.0.1:1883, internal)
```

Satu port publik, sesuai batasan HF Spaces.

| URL di Space | Isi |
|---|---|
| `https://<user>-<space>.hf.space/` | API root |
| `https://<user>-<space>.hf.space/swagger-ui/` | Swagger UI |
| `https://<user>-<space>.hf.space/api-docs/openapi.json` | OpenAPI |
| `wss://<user>-<space>.hf.space/ws` | WebSocket |

---

## 2. Manual: buat Space

1. Buka <https://huggingface.co/new-space>
2. Isi:
   - **Space name**: `vessel-nddu` (atau bebas)
   - **License**: MIT
   - **SDK**: **Docker**
   - **Docker template**: Blank
   - **Visibility**: Public
3. Klik **Create Space**.

## 3. Manual: buat token

1. <https://huggingface.co/settings/tokens>
2. **New token** → role **Write**
3. Simpan token (hanya muncul sekali).

## 4. Push dari repo lokal

Jalankan di `C:\Users\Ridwan\Git-v2\vessel-nddu`:

```powershell
# branch khusus HF agar Dockerfile lokal tidak berubah
git checkout -b hf

# gunakan Dockerfile & README HF
Copy-Item deploy\hf\Dockerfile .\Dockerfile -Force
Copy-Item deploy\hf\README.md .\README.md -Force

git add Dockerfile README.md deploy/hf
git commit -m "deploy: hugging face spaces"

# tambah remote (ganti <user> dan <space>)
git remote add hf https://huggingface.co/spaces/<user>/<space>

# push ke branch main Space
git push hf hf:main
```

Saat diminta password, gunakan **token** (bukan password akun).

Kembali ke branch utama:

```powershell
git checkout master
```

> File `Dockerfile` dan `README.md` di branch `master` tetap versi lokal.

---

## 5. Verifikasi

Ganti `<user>` dan `<space>`:

```powershell
curl.exe -s -o NUL -w "%{http_code}`n" https://<user>-<space>.hf.space/swagger-ui/
curl.exe -s https://<user>-<space>.hf.space/api-docs/openapi.json
```

Uji WebSocket dari browser console:

```js
const ws = new WebSocket("wss://<user>-<space>.hf.space/ws");
ws.onmessage = (e) => console.log(e.data);
```

---

## 6. Jika build gagal

| Gejala | Solusi |
|---|---|
| Build timeout | HF free punya batas waktu build; coba lagi atau kecilkan image |
| `nginx: permission denied` | Pastikan path `/tmp/nginx/*` ada (sudah dibuat di Dockerfile) |
| `mosquitto: address in use` | Hapus service mosquitto lain di image |
| 502 Bad Gateway | Cek log Space: `vessel` gagal start |
| WebSocket gagal | Pastikan path `/ws` dan header `Upgrade` benar |
| Swagger 404 | Pastikan `HTTP_ADDR=127.0.0.1:8080` dan nginx proxy `/` |

Logs: tab **Logs** di halaman Space.

---

## 7. Keterbatasan free tier

| Hal | Catatan |
|---|---|
| Sleep | Space tidur setelah ~48 jam tanpa traffic; kunjungi untuk bangun |
| CPU | 2 vCPU shared |
| Port | Hanya 7860 |
| TCP server 9001 | Tidak diekspos (batasan platform) |
| MQTT eksternal | Broker internal; untuk subscriber luar, arahkan `MQTT_HOST` ke broker publik |
| Penyimpanan | Ephemeral; config hilang saat restart (Fase 5: SQLite) |

---

## 8. Opsional: MQTT publik

Jika ingin penguji bisa subscribe dari luar:

1. Buat broker gratis (HiveMQ Cloud / EMQX Serverless).
2. Di Space, set **Variables/Secrets**:
   - `MQTT_HOST`
   - `MQTT_PORT`
   - `MQTT_USERNAME`
   - `MQTT_PASSWORD`
3. Restart Space.
4. Atau ubah runtime: `PATCH /api/mqtt/config`.

---

## 9. Opsional: auto-deploy dari GitHub

Tambahkan secret `HF_TOKEN` di repo GitHub, lalu workflow:

```yaml
name: Deploy HF
on:
  push:
    branches: [master]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Push to HF
        env:
          HF_TOKEN: ${{ secrets.HF_TOKEN }}
        run: |
          git config user.email "ci@example.com"
          git config user.name "CI"
          git checkout -b hf
          cp deploy/hf/Dockerfile Dockerfile
          cp deploy/hf/README.md README.md
          git add Dockerfile README.md
          git commit -m "deploy: hf"
          git remote add hf https://<user>:${HF_TOKEN}@huggingface.co/spaces/<user>/<space>
          git push -f hf hf:main
```

Ganti `<user>` dan `<space>`. Jangan commit token ke repo.

---

## 10. Checklist manual

- [ ] Akun Hugging Face
- [ ] Space Docker publik dibuat
- [ ] Token write dibuat
- [ ] `git push hf hf:main` sukses
- [ ] Swagger UI 200
- [ ] OpenAPI JSON 200
- [ ] WebSocket `/ws` menerima pesan
- [ ] Tautan dimasukkan ke CV/portfolio
