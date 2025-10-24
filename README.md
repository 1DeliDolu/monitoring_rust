# 🚀 System-Monitoring-Agent mit Grafana-Datasource

**In aktiver Entwicklung befindliches** System-Monitoring-Tool. Bietet Rust-Backend, Grafana-Datasource-Integration und Echtzeit-Systemmetrik-Erfassung.

## 📋 Über das Projekt

Dieses Projekt ist ein Monitoring-Agent, der CPU-, Speicher-, Festplatten-, Netzwerk- und Prozessinformationen in Echtzeit sammelt und für die Visualisierung mit Grafana aufbereitet.

### Features

- ⚡ **Echtzeit-Monitoring**: Erfassung von Systemmetriken im 5-Sekunden-Intervall
- 📊 **Grafana-Integration**: Vollständig kompatibel mit JSON-API-Datasource
- 🔐 **API-Sicherheit**: Token-basierte und Query-Parameter-Authentifizierung
- 💾 **Time-Series-Storage**: Zeitreihendaten-Speicherung in einer einzigen JSON-Datei
- 🌐 **RESTful API**: Flexible und benutzerfreundliche Endpoints
- 🖥️ **Web-UI**: Einfaches HTML-Dashboard zur sofortigen Anzeige
- 📈 **24-Stunden-Historie**: Speicherung von 288 Snapshots für tägliche Daten

## 🛠️ Technologien

- **Backend**: Rust 🦀
  - Axum Web-Framework
  - Tokio Async-Runtime
  - sysinfo v0.30 (Systemmetriken)
  - chrono (Timestamp-Verarbeitung)
- **Frontend**: TypeScript/React
  - Grafana-Plugin-Infrastruktur
  - Node.js v25
- **Build-System**: Mage (Go-basiert)
  - Cross-Platform-Build-Unterstützung
  - Docker-Integration

## 📦 Installation

### Voraussetzungen

- Rust (neueste stabile Version)
- Node.js >= 22
- Go >= 1.21 (für Mage)
- Docker (optional)

### Schritte

1. **Repository klonen**

```bash
git clone https://github.com/1DeliDolu/monitoring_garafana_datasource_rust_go_typescript.git
cd monitoring_garafana_datasource_rust_go_typescript
```

2. **Umgebungsvariablen einrichten**

```bash
cp .env.example .env
# .env-Datei bearbeiten
```

3. **Rust-Agent starten**

```bash
cargo run
```

4. **Grafana-Plugin bauen** (optional)

```bash
cd monitoring-pehlione-datasource
npm install
npm run build
```

## 🚀 Verwendung

### API-Endpoints

Der Agent läuft standardmäßig unter `http://host.docker.internal:7000`.

#### Aktueller Systemstatus

```bash
GET /api/system?api_key=...
```

Zurückgegebene Daten:

```json
{
  "timestamp": 1760880641538,
  "hostname": "Musta",
  "cpu_usage_pct": 25.5,
  "memory_total_bytes": 16777216000,
  "memory_used_bytes": 8388608000,
  "disk_usage_pct": 45.2,
  "network_bytes_sent": 1024000,
  "network_bytes_received": 2048000,
  "process_count": 156
}
```

#### Zeitreihen-Snapshots (für Grafana)

```bash
GET /api/snapshots?api_key=...
```

#### Memory-Historie (mit Limit)

```bash
GET /api/history?limit=100&api_key=...
```

#### Web-Dashboard

```bash
GET /ui
```

### Grafana-Integration

1. **JSON API Datasource** Plugin installieren
2. In den Datasource-Einstellungen:
   - URL: `http://host.docker.internal:7000`
   - Query-String: `api_key=...`
3. Neues Panel erstellen und JSONPath verwenden:
   - Timestamp: `$.snapshots[*].timestamp`
   - CPU: `$.snapshots[*].cpu_usage_pct`
   - Memory: `$.snapshots[*].memory_used_bytes`

## ⚙️ Konfiguration

Konfigurierbare Variablen in der `.env`-Datei:

| Variable                   | Beschreibung                     | Standard           |
| -------------------------- | -------------------------------- | ------------------ |
| `SYSTEM_API_KEY`           | API-Authentifizierungsschlüssel  | -                  |
| `API_BIND_ADDRESS`         | Service-Adresse                  | `127.0.0.1:7000`   |
| `COLLECTION_INTERVAL_SECS` | Erfassungsintervall (Sekunden)   | `5`                |
| `HISTORY_LIMIT`            | Anzahl zu speichernder Snapshots | `288` (24 Stunden) |
| `SNAPSHOT_DIR`             | JSON-Dateiverzeichnis            | `data/snapshots`   |

## 📊 Erfasste Metriken

- **CPU**: Auslastung in Prozent, Anzahl der Kerne, Load Average
- **Speicher**: Gesamt, verwendet, frei, Swap
- **Festplatte**: Auslastung in Prozent, Gesamt-/Belegter Speicherplatz
- **Netzwerk**: Gesendete/Empfangene Bytes
- **Prozesse**: Gesamtzahl der Prozesse, laufende Prozesse
- **System**: Hostname, Uptime, Boot-Zeit

## 🔐 Sicherheit

API-Endpoints unterstützen zwei Arten der Authentifizierung:

1. **Bearer Token** (Header):

```bash
curl -H "Authorization: Bearer ..." http://host.docker.internal:7000/api/system
```

2. **Query-Parameter** (einfacher für Tests):

```bash
curl "http://host.docker.internal:7000/api/system?api_key=..."
```

Unterstützte Query-Parameter: `api_token`, `apitoken`, `token`, `key`

## 📁 Projektstruktur

```
.
├── src/                    # Rust-Backend
│   ├── main.rs            # Einstiegspunkt
│   ├── api.rs             # REST-Endpoints
│   ├── auth.rs            # Authentifizierung
│   ├── collector.rs       # Systemmetrik-Erfassung
│   ├── storage.rs         # JSON-Dateispeicherung
│   ├── scheduler.rs       # Periodische Task-Verwaltung
│   ├── state.rs           # Gemeinsamer Anwendungszustand
│   ├── config.rs          # Konfigurationsverwaltung
│   └── ui.rs              # Web-Dashboard
├── data/snapshots/        # Zeitreihen-JSON-Daten
├── monitoring-pehlione-datasource/  # Grafana-Plugin
└── Cargo.toml            # Rust-Abhängigkeiten
```

## 🏗️ Build

### Rust-Agent

```bash
cargo build --release
```

### Cross-Platform-Build (Mage)

```bash
# Für Linux
mage build:linux

# Für macOS
mage build:darwin

# Für Windows
mage build:windows

# Alle Plattformen
mage build:all
```

## 🐛 Entwicklung

### Debug-Modus

```bash
RUST_LOG=debug cargo run
```

### API testen

```bash
# Systemstatus prüfen
curl "http://host.docker.internal:7000/api/system?api_key=..."

# Snapshot-Anzahl prüfen
python3 check_snapshots.py
```

## 📝 TODO / Roadmap

- [ ] Vollständige Grafana-Plugin-Integration
- [ ] Alert-System
- [ ] Webhook-Unterstützung
- [ ] PostgreSQL/InfluxDB-Backend-Unterstützung
- [ ] Docker-Container-Monitoring
- [ ] Kubernetes-Metriken
- [ ] Multi-Host-Monitoring
- [ ] Dashboard-Vorlagen
- [ ] WebSocket-Echtzeit-Updates

## 🤝 Mitwirken

Dieses Projekt befindet sich in aktiver Entwicklung. Ihre Beiträge sind willkommen!

## 📄 Lizenz

MIT License

## 👤 Entwickler

- GitHub: [@1DeliDolu](https://github.com/1DeliDolu)

---

**⚠️ Hinweis**: Dieses Projekt befindet sich in aktiver Entwicklung. Es wird empfohlen, vor der Verwendung in Produktionsumgebungen umfassende Tests durchzuführen.
