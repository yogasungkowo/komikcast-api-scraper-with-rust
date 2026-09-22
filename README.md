# Komikcast API Scraper With Rust

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Framework-Axum%200.8-blue.svg?style=for-the-badge&logo=rust" alt="Axum" />
  <img src="https://img.shields.io/badge/Async%20Runtime-Tokio-lightgrey.svg?style=for-the-badge&logo=tokio" alt="Tokio" />
  <img src="https://img.shields.io/badge/Author-yogasungkowo-purple.svg?style=for-the-badge" alt="Author" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg?style=for-the-badge" alt="License" />
</p>

Unofficial RESTful API Scraper untuk [komikcast.app](https://komikcast.app) yang dibangun menggunakan bahasa pemrograman **Rust**. Dirancang dengan arsitektur asynchronous modern yang berkinerja tinggi, efisien, dan sangat hemat memori.

Dibuat & dikembangkan oleh **[@yogasungkowo](https://github.com/yogasungkowo)**.

---

## ⚡ Fitur Utama

- **🚀 Ultra-Fast & Low Memory**: Memanfaatkan asynchronous runtime **Tokio** dan parsing HTML zero-cost dengan konsumsi memori hanya **~10–20 MB RAM**.
- **📦 Single Standalone Binary**: Tidak membutuhkan runtime atau dependensi eksternal saat dijalankan.
- **🔄 100% Data Parity**: Format JSON response, filter query, dan struktur data sepenuhnya kompatibel dengan frontend reader komik.
- **💾 In-Memory Genre Cache**: Caching daftar genre di dalam memori (`tokio::sync::RwLock`) untuk respon instan tanpa request berulang ke sumber data.
- **🛡️ Built-in Middleware**: Dilengkapi dengan CORS permisif, kompresi otomatis (gzip/brotli), dan request audit logger (`requests.log`).
- **🧪 Unit Tested**: Memiliki suite pengujian unit lengkap untuk memvalidasi parser komik, pagination, dan chapter reader.

---

## 🛠️ Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/) (Edition 2024 / v1.80+)
- **Web Framework**: [Axum](https://docs.rs/axum) (v0.8)
- **Async Runtime**: [Tokio](https://tokio.rs/)
- **HTTP Client**: [Reqwest](https://docs.rs/reqwest) (v0.12)
- **HTML Parser**: [Scraper](https://docs.rs/scraper) (v0.22) & [Selectors](https://docs.rs/selectors)
- **Middleware**: [Tower](https://docs.rs/tower) & [Tower-HTTP](https://docs.rs/tower-http)
- **Serialization**: [Serde](https://serde.rs/) & [serde_json](https://docs.rs/serde_json)

---

## 📡 Daftar Endpoint API

| Method | Endpoint | Deskripsi |
|---|---|---|
| `GET` | `/` | Informasi status API, endpoints, dan filter options |
| `GET` | `/explore` | Eksplorasi katalog komik dengan filter lengkap |
| `GET` | `/manga` | Alias untuk `/explore` |
| `GET` | `/ranking` | Daftar peringkat manga terpopuler (opsional: `period=all\|daily\|weekly\|monthly`) |
| `GET` | `/genres` | Daftar seluruh genre komik yang tersedia |
| `GET` | `/search?q=keyword` | Pencarian manga berdasarkan judul |
| `GET` | `/manga/:slug` | Detail komik, rating, sinopsis, dan daftar chapter |
| `GET` | `/manga/:slug/:chapterSlug` | Ambil gambar reader chapter berdasarkan slug (e.g. `one-piece-chapter-1`) |
| `GET` | `/manga/:slug/chapter/:chapter` | Ambil gambar reader chapter berdasarkan nomor chapter (e.g. `/manga/one-piece/chapter/1`) |

---

## 🔍 Parameter & Filter Options (`/explore` & `/manga`)

| Parameter | Tipe | Deskripsi / Nilai yang Didukung |
|---|---|---|
| `page` | Integer | Nomor halaman (default: `1`) |
| `type` | String | `manga`, `manhwa`, `manhua` |
| `status` | String | `ongoing`, `completed` |
| `order` / `sort` | String | `update`, `latest`, `popular`, `rating`, `title` |
| `genre` | String | Slug genre, dipisahkan koma untuk multiple (contoh: `action,fantasy`) |
| `search` / `q` | String | Kata kunci pencarian judul komik |

---

## 🚀 Cara Menjalankan (Getting Started)

### Persyaratan
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (versi 1.80 atau yang lebih baru)

### Instalasi & Menjalankan

1. **Clone repository ini:**
   ```bash
   git clone https://github.com/yogasungkowo/komikcast-api-scraper-rust.git
   cd komikcast-api-scraper-rust
   ```

2. **Jalankan dalam mode development:**
   ```bash
   cargo run
   ```
   *(Secara default server berjalan di `http://0.0.0.0:3002`)*

3. **Menjalankan dengan custom port:**
   ```bash
   PORT=8080 cargo run
   ```

4. **Build untuk performa optimal (Release):**
   ```bash
   cargo run --release
   ```

### Menjalankan Unit Test

Untuk memastikan seluruh parser scraper berjalan dengan benar:

```bash
cargo test
```

---

## 💻 Contoh Penggunaan (cURL)

```bash
# Cek info & status API
curl "http://localhost:3002/"

# Mengambil daftar genre
curl "http://localhost:3002/genres"

# Mengambil ranking komik terpopuler
curl "http://localhost:3002/ranking"

# Filter katalog: manhwa ongoing genre action
curl "http://localhost:3002/explore?genre=action&type=manhwa&status=ongoing&order=update"

# Pencarian komik
curl "http://localhost:3002/search?q=solo+leveling"

# Mengambil detail komik beserta seluruh chapter
curl "http://localhost:3002/manga/one-piece"

# Membaca gambar chapter
curl "http://localhost:3002/manga/one-piece/chapter/1"
curl "http://localhost:3002/manga/one-piece/one-piece-chapter-1"
```

---

## 👤 Author

- **yogasungkowo**
  - GitHub: [@yogasungkowo](https://github.com/yogasungkowo)

---

## 📄 Lisensi

Didistribusikan di bawah lisensi [MIT](LICENSE). Silakan gunakan secara bebas untuk pengembangan proyek Anda.

---

## ⚠️ Disclaimer

Aplikasi ini merupakan API tidak resmi (*unofficial*) yang dikembangkan untuk tujuan edukasi dan open-source. Seluruh konten komik, gambar, dan materi berhak cipta adalah hak milik masing-masing penerbit dan pemilik aslinya di [komikcast.app](https://komikcast.app).
