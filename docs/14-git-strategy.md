# 14 — Strategi Git & Publikasi

Repo GitHub `ridwanafazn/vessel-nddu` memiliki riwayat lama (2025) yang
**tidak berhubungan** dengan riwayat lokal 2026. `git push` biasa akan ditolak.

## Riwayat yang ada

| Ref | Isi |
|---|---|
| `origin/main` | Pekerjaan awal Ridwan (2025): API, WS, MQTT |
| `origin/other-ver` | Lanjutan Fijar: refactor MQTT & controller |
| `master` (lokal) | Refactor 2026 + Swagger + test + CI + Docker |

## Strategi: merge `-s ours` (tanpa force push)

`git merge -s ours` membuat **merge commit** yang:
- mempertahankan seluruh riwayat lama sebagai parent,
- **tidak** mengubah isi file (tetap memakai kode baru),
- membuat push ke `main` menjadi **fast-forward** (tidak perlu `--force`).

```powershell
git checkout master
git merge -s ours origin/main --allow-unrelated-histories `
  -m "chore: gabungkan riwayat magang 2025"

git push origin master:main
```

Hasil:
```
*   merge commit (tree = kode 2026)
|\
| * commit lama 2025 (main)
| * ...
* commit refactor 2026
```

## Alternatif

| Opsi | Kapan dipakai |
|---|---|
| Push ke branch baru `refactor-2026` | Ingin review/PR dulu |
| Repo baru `vessel-nddu-v2` | Ingin repo lama tetap sebagai arsip |
| Force push `main` | **Tidak disarankan** — menghapus riwayat |

## Atribusi

Karena kode dilanjutkan Fijar, `CONTRIBUTORS.md` dan bagian **Credits** di
README wajib dipertahankan.

## Setelah push

1. Buka **Actions** → workflow CI harus hijau.
2. Set repo description & topics:
   `rust`, `actix-web`, `nmea`, `mqtt`, `websocket`, `ship-simulator`.
3. Deploy HF: lihat `13-hf-deploy.md`.
