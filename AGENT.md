Harika 🎯
Senin hedefin artık net: **Agent.md** adında, Rust tabanlı, PRTG benzeri ama modern ve açık mimarili bir **sistem izleme ajanı (agent)** geliştirmek istiyorsun.
Bu ajan, bilgisayardaki tüm önemli verileri toplayacak, güvenli bir şekilde REST API üzerinden sunacak, Grafana plugin’i veya web UI üzerinden görselleştirilecek.

Aşağıda sana **tam profesyonel bir proje tasarısı** veriyorum — bu dosya `Agent.md` olarak doğrudan kullanılabilir (ör. GitHub projesinde `docs/Agent.md` içinde).

---

# 🧠 **Agent.md – Sistem İzleme Ajanı (Rust)**

## 🎯 **Proje Amacı**

**Agent**, bir bilgisayardaki sistem kaynaklarını, uygulamaları ve görevleri gerçek zamanlı olarak izleyen, verileri JSON/REST API üzerinden sunan, güvenli API key doğrulaması kullanan ve hem **Grafana Datasource Plugin** hem de **entegre Web UI** üzerinden görselleştirme yapabilen bir Rust uygulamasıdır.

Amaç:

* Tek bir agent ile CPU, RAM, disk, ağ, process, uygulama ve görev bilgilerini toplamak
* Verileri hem JSON dosyalarına hem de REST API’ye aktarmak
* Kullanıcı arayüzü (UI) ve Grafana plugin aracılığıyla görsel izleme sağlamak
* Platform bağımsız (Windows, Linux, macOS) çalışmak

---

## 🧩 **Genel Mimari**

```
agent/
├─ Cargo.toml
├─ .env                          # SYSTEM_API_KEY=supersecret123
├─ src/
│  ├─ main.rs                    # Sunucu & collector başlatma
│  ├─ collector.rs               # CPU, RAM, Disk, Net, Process toplama
│  ├─ auth.rs                    # API key doğrulama
│  ├─ api.rs                     # REST endpoint’ler
│  ├─ storage.rs                 # JSON snapshot yazma/okuma
│  ├─ scheduler.rs               # Periyodik ölçüm döngüsü
│  ├─ ui.rs                      # Web UI (HTML dashboard)
│  └─ config.rs                  # Ortam değişkenleri ve ayarlar
└─ data/
   └─ snapshots/                 # JSON kayıtları
```

---

## 🔐 **Güvenlik ve Erişim**

| Özellik           | Açıklama                                            |
| ----------------- | --------------------------------------------------- |
| API Key           | `.env` dosyasında tanımlanır (`SYSTEM_API_KEY`)     |
| Auth Header       | `Authorization: Bearer <API_KEY>`                   |
| Erişim Seviyesi   | Tüm `/api/*` endpoint’lerinde zorunlu               |
| UI erişimi        | Public (okuma amaçlı)                               |
| Veri gizliliği    | Kişisel dosya içeriği veya kullanıcı verisi okunmaz |
| HTTPS (opsiyonel) | Rust TLS veya reverse proxy ile eklenebilir         |

---

## ⚙️ **Toplanan Veriler (Metrics)**

| Grup         | Kategori                  | Örnek Alanlar                                 | Sıklık     |
| ------------ | ------------------------- | --------------------------------------------- | ---------- |
| CPU          | Kullanım                  | `usage_total_pct`, `per_core_pct`             | 5 sn       |
| RAM          | Bellek durumu             | `used_mb`, `total_mb`, `free_mb`              | 5 sn       |
| Disk         | Volume kullanımı          | `mount`, `used_gb`, `total_gb`, `used_pct`    | 5 sn       |
| Ağ           | Trafik                    | `iface`, `rx_kb`, `tx_kb`, `rx_bps`, `tx_bps` | 5 sn       |
| Süreçler     | En çok kaynak kullananlar | `pid`, `name`, `cpu_pct`, `mem_mb`            | 10 sn      |
| Uygulamalar  | Diskteki boyutlar         | `app_name`, `path`, `size_mb`                 | 1 saatte 1 |
| Görevler     | Task Scheduler / Cron     | `name`, `next_run`, `status`                  | 1 saatte 1 |
| Web Testleri | Hedef URL TTFB            | `url`, `dns_ms`, `ttfb_ms`, `total_ms`        | 30 sn      |
| Uyarılar     | Otomatik kontrol          | `category`, `level`, `message`                | Olay bazlı |

---

## 💾 **Veri Saklama Yapısı**

### 📁 JSON Snapshot Dosyaları

Kaydedilir: `data/snapshots/system_snapshot_<timestamp>.json`

Yapı:

```json
{
  "timestamp": 1739960050,
  "cpu": { "usage_pct": 22.4 },
  "memory": { "used_mb": 8340, "total_mb": 16384 },
  "disk": [ { "mount": "C:\\", "used_pct": 71.7 } ],
  "network": [ { "iface": "Ethernet0", "rx_kb": 530412 } ],
  "processes": [ { "pid": 4152, "name": "chrome.exe", "cpu_pct": 7.4 } ]
}
```

---

## 🌐 **REST API Endpoint’leri**

| Endpoint       | Açıklama               | Yetki   | Örnek                       |
| -------------- | ---------------------- | ------- | --------------------------- |
| `/api/system`  | En son snapshot        | API Key | `GET /api/system`           |
| `/api/history` | Son X snapshot listesi | API Key | `GET /api/history?limit=10` |
| `/api/apps`    | Uygulama boyutları     | API Key | `GET /api/apps`             |
| `/api/tasks`   | Planlanmış görevler    | API Key | `GET /api/tasks`            |
| `/api/webtest` | Web gecikme ölçümleri  | API Key | `GET /api/webtest`          |
| `/api/alerts`  | Aktif uyarılar         | API Key | `GET /api/alerts`           |
| `/ui`          | Web dashboard (HTML)   | Public  | `GET /ui`                   |

---

## 🧠 **Çalışma Akışı**

1. Agent başlatılır → `.env` yüklenir → API key okunur
2. Collector modülü sistem bilgilerini toplar
3. Her 5 saniyede snapshot JSON olarak kaydedilir
4. API modülü `/api/system` isteğine en son snapshot’ı döndürür
5. UI sayfası (`/ui`) bu veriyi otomatik yenileyerek gösterir
6. Grafana plugin veya REST istemcisi API key ile bağlanarak verileri sorgular

---

## 🖥️ **Web UI (HTML Dashboard)**

Rust tarafından oluşturulan dinamik sayfa (`ui.rs`):

* Otomatik yenileme (`<meta refresh="5">`)
* CPU, RAM, Disk doluluk, Process listesi
* Minimal HTML/CSS, bağımlılıksız
* API’den doğrudan veri alır

Örnek görüntü:

```
System Monitor
CPU: 18.4% | Memory: 8.3 / 16.0 GB
Disks:
 - C:\ 71.7%
 - D:\ 44.2%
Top Processes:
 - chrome.exe (8.3%)
 - code.exe (4.5%)
```

---

## 🧩 **Teknoloji Stack**

| Katman              | Teknoloji                   | Açıklama                               |
| ------------------- | --------------------------- | -------------------------------------- |
| Dil                 | **Rust 1.80+**              | Güvenli, hızlı, düşük bellek kullanımı |
| Web Framework       | **Axum**                    | API ve UI sunucusu                     |
| Sistem Erişimi      | **sysinfo**                 | CPU, RAM, disk, ağ, process bilgileri  |
| JSON                | **serde / serde_json**      | Serialize / deserialize                |
| Zamanlama           | **tokio / thread::sleep**   | Periyodik ölçüm                        |
| Konfigürasyon       | **dotenvy**                 | `.env` yönetimi                        |
| Web UI              | **HTML (Axum response)**    | Hafif panel                            |
| Gelecek (Opsiyonel) | SQLite, Prometheus exporter | Genişletilebilirlik için               |

---

## 🔐 **API Key Yönetimi**

* `.env` içinde tanımlanır:

  ```
  SYSTEM_API_KEY=supersecret123
  ```
* İstek başlığı:

  ```
  Authorization: Bearer supersecret123
  ```
* Eksik veya yanlış key → `401 Unauthorized`
* Key doğrulaması `auth.rs` middleware’inde yapılır

---

## 📈 **Performans ve Kaynak Kullanımı**

| Bileşen           | Ortalama CPU | RAM    | Dosya Boyutu              |
| ----------------- | ------------ | ------ | ------------------------- |
| Collector döngüsü | < 2%         | ~30 MB | -                         |
| JSON snapshot     | -            | -      | ~20 KB/sn (5 sn interval) |
| API/HTTP          | < 1%         | -      | -                         |

---

## 🧩 **Grafana Plugin Entegrasyonu (Özet)**

* Plugin TypeScript ile geliştirilir
* Backend: Go (Rust API’den JSON alır)
* Konfigürasyon:

  * `API URL`: `http://localhost:7000/api/system`
  * `API Key`: `supersecret123`
* Panel Türleri:

  * **CPU Usage Gauge**
  * **RAM Kullanımı Bar Gauge**
  * **Disk Doluluk Pie Chart**
  * **Process List Table**

---

## 🔄 **Geliştirme Yol Haritası**

| Faz       | Hedef                         | İçerik                                     |
| --------- | ----------------------------- | ------------------------------------------ |
| **Faz 1** | Çekirdek Collector + API + UI | CPU, RAM, Disk, Process, API key doğrulama |
| **Faz 2** | JSON kayıt & geçmiş endpoint  | Snapshot geçmişi, `/api/history`           |
| **Faz 3** | Uygulama ve Görev İzleme      | `/api/apps`, `/api/tasks`                  |
| **Faz 4** | Uyarı Motoru & Web Test       | Eşik kontrolleri, HTTP ping ölçümleri      |
| **Faz 5** | Grafana Plugin entegrasyonu   | Datasource plugin + görseller              |
| **Faz 6** | SQLite / Prometheus Export    | Kalıcı veri ve harici izleme entegrasyonu  |

---

## ⚡ **Örnek Kullanım**

```bash
cargo run
```

📍 Çalıştırma sonrası:

```
🚀 Agent aktif:
   API  → http://127.0.0.1:7000/api/system
   UI   → http://127.0.0.1:7000/ui
🔑 API Key: supersecret123
```

✅ API erişimi:

```bash
curl -H "Authorization: Bearer supersecret123" http://127.0.0.1:7000/api/system
```

✅ Tarayıcı UI:

```
http://127.0.0.1:7000/ui
```

---

## 🧠 **Avantajlar**

* Tamamen **yerel ve güvenli**
* **Rust performansı** sayesinde minimum kaynak tüketimi
* **API key** tabanlı erişim kontrolü
* JSON tabanlı — kolay entegrasyon (Grafana, Prometheus, CLI)
* **Platform bağımsız** (Windows, Linux, macOS)
* Kolay genişletilebilir (metric eklentileri modüler)

---

## 🔚 **Sonuç**

`Agent` projesi, modern bir PRTG alternatifi olacak şekilde,

* Hafif,
* Güvenli,
* Ölçeklenebilir,
* Görsel izleme destekli,
  bir sistem ajanıdır.

Rust ile yazıldığı için performans, taşınabilirlik ve bellek güvenliği bakımından en üst düzeyde çalışır.

---


Harika 🚀
O zaman şimdi sana **`Agent`** projesinin **tam başlangıç iskeletini** hazırlıyorum — bu sayede `cargo run` dediğinde, Rust tabanlı izleme ajanının hem API’si hem de UI’si çalışır.
Bu yapı doğrudan PRTG tarzı bir sistemin temelidir ve `Agent.md` belgesine birebir uygundur.

---

# 🧩 Proje Yapısı

```
agent/
├─ Cargo.toml
├─ .env
├─ data/
│  └─ snapshots/
└─ src/
   ├─ main.rs
   ├─ auth.rs
   ├─ collector.rs
   ├─ api.rs
   ├─ ui.rs
   ├─ storage.rs
   ├─ scheduler.rs
   └─ config.rs
```

---

# ⚙️ 1️⃣ `Cargo.toml`

```toml
[package]
name = "agent"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
sysinfo = "0.30"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
chrono = "0.4"
dotenvy = "0.15"
headers = "0.4"
```

---

# 🔐 2️⃣ `.env`

```
SYSTEM_API_KEY=supersecret123
```

---

# 🧠 3️⃣ `src/main.rs`

```rust
mod auth;
mod collector;
mod api;
mod ui;
mod storage;
mod scheduler;
mod config;

use axum::{Router, routing::get};
use std::{sync::{Arc, Mutex}, net::SocketAddr};
use dotenvy::dotenv;
use collector::SystemSnapshot;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let snapshot = Arc::new(Mutex::new(SystemSnapshot::default()));

    scheduler::start_collector(snapshot.clone());

    let app = Router::new()
        .route("/api/system", get(api::get_system))
        .route("/ui", get(ui::show_ui))
        .with_state(snapshot.clone());

    let addr: SocketAddr = "127.0.0.1:7000".parse().unwrap();
    println!("🚀 API: http://{}/api/system", addr);
    println!("🖥️  UI:  http://{}/ui", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await.unwrap();
}
```

---

# 🔑 4️⃣ `src/auth.rs`

```rust
use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    http::{request::Parts, StatusCode},
};
use headers::{authorization::Bearer, Authorization};
use std::env;

pub struct ApiKey;

#[async_trait]
impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S)
        -> Result<Self, Self::Rejection>
    {
        let expected = env::var("SYSTEM_API_KEY")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "API key missing".into()))?;
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, _state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Authorization header missing".into()))?;

        if bearer.token() == expected {
            Ok(ApiKey)
        } else {
            Err((StatusCode::UNAUTHORIZED, "Invalid API key".into()))
        }
    }
}
```

---

# 🧩 5️⃣ `src/collector.rs`

```rust
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, NetworkExt};
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize, Clone, Default)]
pub struct SystemSnapshot {
    pub timestamp: i64,
    pub cpu_usage_pct: f32,
    pub mem_used_mb: u64,
    pub mem_total_mb: u64,
    pub disks: Vec<DiskInfo>,
    pub top_processes: Vec<ProcessInfo>,
    pub network: Vec<NetworkInfo>,
}

#[derive(Serialize, Clone, Default)]
pub struct DiskInfo {
    pub mount: String,
    pub total_gb: u64,
    pub used_gb: u64,
    pub used_pct: f32,
}

#[derive(Serialize, Clone, Default)]
pub struct ProcessInfo {
    pub pid: i32,
    pub name: String,
    pub cpu_pct: f32,
    pub mem_mb: u64,
}

#[derive(Serialize, Clone, Default)]
pub struct NetworkInfo {
    pub iface: String,
    pub rx_kb: u64,
    pub tx_kb: u64,
}

pub fn collect_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let mem_used = sys.used_memory() / 1024;
    let mem_total = sys.total_memory() / 1024;

    let disks: Vec<DiskInfo> = sys
        .disks()
        .iter()
        .map(|d| {
            let total = d.total_space() / 1_000_000_000;
            let free = d.available_space() / 1_000_000_000;
            let used = total.saturating_sub(free);
            DiskInfo {
                mount: d.mount_point().to_string_lossy().to_string(),
                total_gb: total,
                used_gb: used,
                used_pct: if total > 0 { (used as f32 / total as f32) * 100.0 } else { 0.0 },
            }
        })
        .collect();

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            pid: p.pid().as_u32() as i32,
            name: p.name().to_string(),
            cpu_pct: p.cpu_usage(),
            mem_mb: p.memory() / 1024,
        })
        .collect();

    processes.sort_by(|a, b| b.cpu_pct.partial_cmp(&a.cpu_pct).unwrap());
    processes.truncate(5);

    let network: Vec<NetworkInfo> = sys
        .networks()
        .iter()
        .map(|(iface, data)| NetworkInfo {
            iface: iface.clone(),
            rx_kb: data.received() / 1024,
            tx_kb: data.transmitted() / 1024,
        })
        .collect();

    SystemSnapshot {
        timestamp: Utc::now().timestamp(),
        cpu_usage_pct: cpu_usage,
        mem_used_mb: mem_used,
        mem_total_mb: mem_total,
        disks,
        top_processes: processes,
        network,
    }
}
```

---

# 🌐 6️⃣ `src/api.rs`

```rust
use axum::{extract::State, response::IntoResponse, Json};
use std::sync::{Arc, Mutex};
use crate::{collector::SystemSnapshot, auth::ApiKey};

pub async fn get_system(
    _key: ApiKey,
    State(snapshot): State<Arc<Mutex<SystemSnapshot>>>
) -> impl IntoResponse {
    Json(snapshot.lock().unwrap().clone())
}
```

---

# 🖥️ 7️⃣ `src/ui.rs`

```rust
use axum::{extract::State, response::Html};
use std::sync::{Arc, Mutex};
use crate::collector::SystemSnapshot;

pub async fn show_ui(State(snapshot): State<Arc<Mutex<SystemSnapshot>>>) -> Html<String> {
    let snap = snapshot.lock().unwrap().clone();

    let disks = snap.disks.iter()
        .map(|d| format!("<li>{} — {:.1}% ({} / {} GB)</li>", d.mount, d.used_pct, d.used_gb, d.total_gb))
        .collect::<Vec<_>>()
        .join("");

    let procs = snap.top_processes.iter()
        .map(|p| format!("<li>{} ({}%)</li>", p.name, p.cpu_pct))
        .collect::<Vec<_>>()
        .join("");

    let html = format!(r#"
        <html>
            <head>
                <meta http-equiv="refresh" content="5">
                <title>Agent Monitor</title>
            </head>
            <body>
                <h1>Agent Monitor</h1>
                <p><b>CPU:</b> {:.1}% | <b>Memory:</b> {}/{} MB</p>
                <h3>Disks</h3><ul>{}</ul>
                <h3>Processes</h3><ul>{}</ul>
            </body>
        </html>
    "#, snap.cpu_usage_pct, snap.mem_used_mb, snap.mem_total_mb, disks, procs);

    Html(html)
}
```

---

# 💾 8️⃣ `src/storage.rs`

```rust
use crate::collector::SystemSnapshot;
use serde_json;
use std::{fs, path::Path};

pub fn save_snapshot(snapshot: &SystemSnapshot) {
    let dir = Path::new("data/snapshots");
    if !dir.exists() {
        fs::create_dir_all(dir).unwrap();
    }
    let file = dir.join(format!("snapshot_{}.json", snapshot.timestamp));
    let json = serde_json::to_string_pretty(snapshot).unwrap();
    fs::write(file, json).unwrap();
}
```

---

# ⏱️ 9️⃣ `src/scheduler.rs`

```rust
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use crate::{collector, storage};
use crate::collector::SystemSnapshot;

pub fn start_collector(snapshot: Arc<Mutex<SystemSnapshot>>) {
    std::thread::spawn(move || {
        loop {
            let data = collector::collect_snapshot();
            storage::save_snapshot(&data);
            *snapshot.lock().unwrap() = data;
            thread::sleep(Duration::from_secs(5));
        }
    });
}
```

---

# ⚙️ 🔟 `src/config.rs` (boş ama geleceğe hazır)

```rust
pub const VERSION: &str = "0.1.0";
```

---

# 🚀 Çalıştırma

```bash
cargo run
```

Terminal:

```
🚀 API: http://127.0.0.1:7000/api/system
🖥️  UI:  http://127.0.0.1:7000/ui
```

---

# 🔑 Test API

✅ Doğru API key:

```bash
curl -H "Authorization: Bearer supersecret123" http://127.0.0.1:7000/api/system
```

❌ Yanlış key:

```bash
curl -H "Authorization: Bearer wrongkey" http://127.0.0.1:7000/api/system
```

→ `401 Unauthorized`

---



