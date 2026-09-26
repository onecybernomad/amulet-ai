# Amulet AI

Family safety and assistance app built with **Rust + Tauri + Vanilla JS + OpenStreetMap**.

## Quick Start

### Prerequisites
- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.75+
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) (`cargo install tauri-cli`)
- PostgreSQL 15+ with PostGIS extension
- Redis 7+

### Setup

```bash
# Install JS dependencies
npm install

# Set up environment
cp .env.example .env
# Edit .env with your database and Redis URLs

# Run database migrations (server)
cd server && cargo run -- migrate

# Start the cloud server (terminal 1)
npm run server

# Start the Tauri app (terminal 2)
npm run tauri dev
```

## Project Structure

```
amulet-ai/
├── src/                  # Vanilla JS frontend (Leaflet + OSM)
│   ├── index.html
│   ├── css/
│   ├── js/
│   └── assets/
├── src-tauri/            # Rust Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── commands/     # Tauri IPC commands
│       ├── db/           # SQLite local cache
│       ├── realtime/     # WebSocket client
│       ├── sensors/      # Crash/fall detection
│       └── models/       # Shared Rust structs
├── server/               # Axum cloud server
│   ├── Cargo.toml
│   └── src/
└── PLAN.md               # Full architecture & roadmap
```

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Shell | Tauri v2 (Rust) |
| Frontend | Vanilla JS + Leaflet + OpenStreetMap |
| Backend | Rust (Axum) |
| Database | PostgreSQL + PostGIS |
| Cache | Redis |
| Realtime | WebSocket |
| Maps | OpenStreetMap (free, no API key) |

## License

MIT
