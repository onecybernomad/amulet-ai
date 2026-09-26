# Amulet AI — Family Assistant App

## Planning Document

---

## 1. Product Overview

**Amulet AI** is a family safety and assistance app that keeps your inner circle ("Circle") connected through real-time location sharing, proactive alerts, health monitoring, and emergency response. It combines live location, geofencing, sensor-based incident detection (crash/fall), medication management, secure group chat, and premium driving/roadside features into one tiered subscription product.

---

## 2. Feature Summary

### Core (Free + Paid)
| # | Feature | Description |
|---|---------|-------------|
| 1 | **Real-Time Location Sharing** | Live map showing all Circle members' positions, updated every few seconds. |
| 2 | **Place Alerts (Geofencing)** | Automatic arrival/departure notifications for saved places (home, school, work). |
| 3 | **Location History** | Chronological log of member movements; retention varies by plan (e.g. 7 days free, 30–90 days paid). |
| 4 | **SOS Alerts** | One-tap silent or audible emergency broadcast to the entire Circle with live location. |
| 5 | **Crash Detection** | Automatic severe-accident detection via phone sensors → alert Circle + optional emergency dispatch. |
| 6 | **Fall Detection** | Phone/wearable fall detection → audible "Are you OK?" prompt → if no response, alert emergency contact. |
| 7 | **Medication Tracker & Reminders** | Medication list, dose schedule, reminders, and adherence log per member. |
| 8 | **Group Chat** | Secure in-app messaging for all Circle members. |

### Advanced & Paid
| # | Feature | Description |
|---|---------|-------------|
| 9 | **Driving Reports** | Speed, rapid acceleration, hard braking, phone-use-while-driving analytics. |
| 10 | **Emergency Dispatch** | Auto-dispatch emergency responders to crash/SOS location (Gold / Platinum tiers). |
| 11 | **Tile Tracker Integration** | Track keys, wallets, pets, bags via Tile Bluetooth hardware inside the app. |
| 12 | **Roadside & Medical Assistance** | Towing, flat-tire, medical advice hotline based on membership tier. |

---

## 3. Tech Stack

| Layer | Recommendation | Rationale |
|-------|---------------|-----------|
| **Framework** | **Tauri v2** | Rust-powered, tiny binaries (~3 MB), secure by default, cross-platform desktop + mobile. |
| **Backend (Core)** | **Rust (Axum)** | Type-safe, async, high-performance API server. Shares types with Tauri commands. |
| **Frontend** | **Vanilla JS (ES2023)** | Zero framework overhead, direct DOM control, fast load times. No build step needed (or minimal Vite). |
| **CSS** | **Vanilla CSS + CSS Custom Properties** | Lightweight, no runtime cost, full control over design system. |
| **Maps** | **Leaflet + OpenStreetMap** | Free, open-source, no API key. Dark mode via CSS filters on tiles. |
| **Real-Time Transport** | **WebSocket (native `ws` crate in Rust)** | Low-latency bidirectional for location & chat. |
| **Database** | **PostgreSQL + PostGIS** | Geospatial queries (geofencing, history) are first-class. |
| **Embedded DB (Local)** | **SQLite (rusqlite)** | Offline cache, settings, local history on each device. |
| **Cache / Pub-Sub** | **Redis** | Real-time location pub/sub, geofence state, session cache. |
| **Message Queue** | **RabbitMQ / NATS** | Async processing (dispatch, notifications, sensor pipelines). |
| **Push Notifications** | **FCM + APNs (via Tauri push plugin)** | Cross-platform reliable push. |
| **Authentication** | **OAuth2 + OTP (Tauri plugin)** | Secure login flows with native UI where possible. |
| **Subscriptions / Billing** | **Stripe** | Robust subscription management, tiered plans. |
| **Sensor / ML** | **Tauri mobile plugins + Rust native** | Access accelerometer/gyroscope via Tauri mobile APIs; Rust for signal processing. |
| **Tile Integration** | **Tile SDK (native bridge via Tauri)** | Custom Tauri plugin wrapping iOS/Android Tile SDKs. |
| **File Storage** | **AWS S3 (encrypted)** | Chat media, driving report exports. |
| **Monitoring** | **Sentry (Rust SDK) + Prometheus** | Error tracking and system health. |
| **CI/CD** | **GitHub Actions + Tauri CLI** | Automated builds for Windows, macOS, Linux, iOS, Android. |

---

## 4. High-Level Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     TAURI APP SHELL (Rust)                   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              WebView (Vanilla JS Frontend)               │ │
│  │  ┌───────────┐ ┌──────────┐ ┌───────────┐ ┌─────────┐  │ │
│  │  │ Leaflet   │ │ Chat UI  │ │ SOS UI    │ │ Meds /  │  │ │
│  │  │ OSM Map   │ │ (WS)     │ │           │ │ Driving │  │ │
│  │  └───────────┘ └──────────┘ └───────────┘ └─────────┘  │ │
│  │  State: vanilla JS modules + localStorage               │ │
│  └──────────────────────────┬──────────────────────────────┘ │
│                             │ Tauri IPC (invoke/events)       │
│  ┌──────────────────────────┴──────────────────────────────┐ │
│  │              TAURI CORE (Rust)                          │ │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌──────────┐  │ │
│  │  │ Auth     │ │ Local DB │ │ Sensor    │ │ Push     │  │ │
│  │  │ Manager  │ │ (SQLite) │ │ Bridge    │ │ Notif.   │  │ │
│  │  └──────────┘ └──────────┘ └───────────┘ └──────────┘  │ │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌──────────┐  │ │
│  │  │ Geofence │ │ Tile     │ │ Crash/Fall│ │ WS       │  │ │
│  │  │ Engine   │ │ Bridge   │ │ Classifier│ │ Client   │  │ │
│  │  └──────────┘ └──────────┘ └───────────┘ └──────────┘  │ │
│  └──────────────────────────┬──────────────────────────────┘ │
└─────────────────────────────┼────────────────────────────────┘
                              │ HTTP / WebSocket
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                    CLOUD SERVER (Rust / Axum)                 │
│  ┌────────────┐ ┌────────────┐ ┌──────────┐ ┌─────────────┐ │
│  │ REST API   │ │ WebSocket  │ │ Webhook  │ │ Sensor      │ │
│  │ (Auth,CRUD│ │ Server     │ │ API      │ │ Pipeline    │ │
│  │ Billing)   │ │ (Location, │ │ (Tile,   │ │ (Crash/Fall)│ │
│  │            │ │ Chat, SOS) │ │ Dispatch)│ │             │ │
│  └─────┬──────┘ └─────┬──────┘ └────┬─────┘ └──────┬──────┘ │
│        └──────────────┴─────────────┴──────────────┘        │
│                           │                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │PostgreSQL│ │  Redis   │ │ RabbitMQ │ │ S3 (encrypt) │   │
│  │+ PostGIS │ │  Cache   │ │  Queue   │ │ Media/Export │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Data Model (Key Entities)

```sql
-- Core tables (PostgreSQL + PostGIS)

CREATE TABLE users (
    id              UUID PRIMARY KEY,
    email           TEXT UNIQUE NOT NULL,
    phone           TEXT,
    display_name    TEXT NOT NULL,
    avatar_url      TEXT,
    circle_id       UUID REFERENCES circles(id),
    subscription_tier TEXT DEFAULT 'free',
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    settings        JSONB DEFAULT '{}'
);

CREATE TABLE circles (
    id          UUID PRIMARY KEY,
    name        TEXT NOT NULL,
    invite_code TEXT UNIQUE NOT NULL,
    max_members INT DEFAULT 4,
    created_by  UUID REFERENCES users(id),
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE places (
    id                UUID PRIMARY KEY,
    circle_id         UUID REFERENCES circles(id),
    name              TEXT NOT NULL,
    address           TEXT,
    geo_point         GEOGRAPHY(POINT, 4326) NOT NULL,  -- PostGIS
    radius_meters     INT DEFAULT 150,
    alert_on_arrival  BOOLEAN DEFAULT TRUE,
    alert_on_departure BOOLEAN DEFAULT TRUE
);

CREATE TABLE location_pings (
    id          UUID PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    device_id   TEXT NOT NULL,
    geo_point   GEOGRAPHY(POINT, 4326) NOT NULL,
    accuracy_m  FLOAT,
    speed_kmh   FLOAT,
    heading     FLOAT,
    battery     INT,
    ts          TIMESTAMPTZ DEFAULT NOW(),
    source      TEXT DEFAULT 'gps'
) PARTITION BY RANGE (ts);

CREATE TABLE geofence_events (
    id         UUID PRIMARY KEY,
    user_id    UUID REFERENCES users(id),
    place_id   UUID REFERENCES places(id),
    event_type TEXT CHECK (event_type IN ('arrival', 'departure')),
    entered_at TIMESTAMPTZ,
    exited_at  TIMESTAMPTZ,
    triggered_alert BOOLEAN DEFAULT FALSE
);

CREATE TABLE incidents (
    id              UUID PRIMARY KEY,
    user_id         UUID REFERENCES users(id),
    incident_type   TEXT CHECK (incident_type IN ('crash', 'fall', 'sos')),
    severity        TEXT DEFAULT 'high',
    geo_point       GEOGRAPHY(POINT, 4326),
    ts              TIMESTAMPTZ DEFAULT NOW(),
    sensor_snapshot JSONB,          -- accelerometer/gyro data
    status          TEXT DEFAULT 'detected',
                    CHECK (status IN ('detected', 'acknowledged', 'resolved', 'dispatched')),
    response_log    JSONB DEFAULT '[]'
);

CREATE TABLE medications (
    id          UUID PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    name        TEXT NOT NULL,
    dosage      TEXT,
    frequency   TEXT,              -- cron-like or custom JSON
    schedule    JSONB,             -- ["08:00", "20:00"]
    start_date  DATE,
    end_date    DATE,
    reminders_enabled BOOLEAN DEFAULT TRUE,
    adherence_log JSONB DEFAULT '[]'  -- [{ts, status}]
);

CREATE TABLE chat_messages (
    id          UUID PRIMARY KEY,
    room_id     UUID,
    sender_id   UUID REFERENCES users(id),
    body        TEXT NOT NULL,
    media_urls  TEXT[],
    ts          TIMESTAMPTZ DEFAULT NOW(),
    encrypted   BOOLEAN DEFAULT TRUE,
    read_by     UUID[] DEFAULT '{}'
);

CREATE TABLE driving_sessions (
    id          UUID PRIMARY KEY,
    user_id     UUID REFERENCES users(id),
    start_time  TIMESTAMPTZ,
    end_time    TIMESTAMPTZ,
    distance_km FLOAT,
    max_speed   FLOAT,
    avg_speed   FLOAT,
    events      JSONB DEFAULT '[]',  -- hard_brake, rapid_accel, phone_use
    score       INT,                 -- 0-100
    report_url  TEXT
);

CREATE TABLE subscriptions (
    user_id             UUID REFERENCES users(id),
    stripe_customer_id  TEXT,
    tier                TEXT DEFAULT 'free',
    status              TEXT DEFAULT 'active',
    current_period_end  TIMESTAMPTZ,
    features            TEXT[] DEFAULT '{}'
);

CREATE TABLE tile_trackers (
    id            UUID PRIMARY KEY,
    user_id       UUID REFERENCES users(id),
    tile_device_id TEXT NOT NULL,
    name          TEXT,
    last_seen     TIMESTAMPTZ,
    geo_point     GEOGRAPHY(POINT, 4326),
    battery_level INT,
    status        TEXT DEFAULT 'active'
);
```

---

## 6. Rust Backend API (Axum)

### 6.1 Server Structure
```
server/
├── Cargo.toml
├── src/
│   ├── main.rs              # Axum app setup, router, state
│   ├── config.rs             # Environment config
│   ├── db/
│   │   ├── mod.rs           # Pool, migrations
│   │   ├── users.rs
│   │   ├── circles.rs
│   │   ├── locations.rs
│   │   ├── geofence.rs
│   │   ├── incidents.rs
│   │   ├── medications.rs
│   │   ├── chat.rs
│   │   ├── driving.rs
│   │   └── subscriptions.rs
│   ├── realtime/
│   │   ├── mod.rs           # WebSocket hub
│   │   ├── location.rs      # Location ping handler
│   │   ├── chat.rs          # Chat message handler
│   │   └── alerts.rs        # Geofence + incident alerts
│   ├── services/
│   │   ├── geofence_engine.rs
│   │   ├── sensor_pipeline.rs
│   │   ├── dispatch.rs
│   │   ├── billing.rs
│   │   └── tile.rs
│   ├── middleware/
│   │   ├── auth.rs          # JWT validation
│   │   └── rate_limit.rs
│   └── types.rs             # Shared request/response types
```

### 6.2 REST Endpoints
```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/otp/send
POST   /api/v1/auth/otp/verify

POST   /api/v1/circles
GET    /api/v1/circles/:id
POST   /api/v1/circles/:id/invite
GET    /api/v1/circles/:id/members
POST   /api/v1/circles/:id/leave

POST   /api/v1/places
GET    /api/v1/places
PATCH  /api/v1/places/:id
DELETE /api/v1/places/:id

POST   /api/v1/sos                              # Trigger SOS
GET    /api/v1/incidents?user_id=&type=         # Incident history
PATCH  /api/v1/incidents/:id/status             # Update status

POST   /api/v1/medications
GET    /api/v1/medications/:id/adherence
POST   /api/v1/medications/:id/log              # Log taken/skipped

GET    /api/v1/driving/reports
GET    /api/v1/driving/reports/:id

POST   /api/v1/tiles/link
GET    /api/v1/tiles
POST   /api/v1/tiles/:id/ring

GET    /api/v1/subscriptions/plans
POST   /api/v1/subscriptions/subscribe
POST   /api/v1/subscriptions/cancel
```

### 6.3 WebSocket Protocol
```
ws://server/realtime?token=JWT&circle_id=UUID

Client → Server:
  { "type": "location_ping", "lat": ..., "lng": ..., "accuracy": ... }
  { "type": "chat_message", "room_id": ..., "body": ... }
  { "type": "sos", "lat": ..., "lng": ..., "silent": false }
  { "type": "incident", "incident_type": "crash"|"fall", "lat": ..., "lng": ..., "confidence": ... }

Server → Client:
  { "type": "member_location", "user_id": ..., "lat": ..., "lng": ..., "ts": ... }
  { "type": "geofence_alert", "user_id": ..., "place_name": ..., "event": "arrival"|"departure" }
  { "type": "sos_alert", "user_id": ..., "lat": ..., "lng": ..., "ts": ... }
  { "type": "chat_message", "room_id": ..., "sender_id": ..., "body": ..., "ts": ... }
```

---

## 7. Tauri App Structure (Rust Core + Vanilla JS Frontend)

```
amulet-ai/
├── src-tauri/                    # Rust backend (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/
│   └── src/
│       ├── main.rs               # Tauri app entry, plugin setup
│       ├── lib.rs                # Module exports
│       ├── commands/             # Tauri commands (IPC from JS)
│       │   ├── mod.rs
│       │   ├── auth.rs           # login, register, OTP
│       │   ├── location.rs       # start/stop GPS, send pings
│       │   ├── geofence.rs       # local geofence checks
│       │   ├── sos.rs            # trigger SOS
│       │   ├── medications.rs    # CRUD, reminders
│       │   ├── chat.rs           # send/receive messages
│       │   ├── tile.rs           # Tile SDK bridge
│       │   └── settings.rs       # Stripe checkout
│       ├── db/
│       │   ├── mod.rs            # rusqlite pool
│       │   ├── local_cache.rs    # SQLite schema + queries
│       │   └── migrations.rs
│       ├── realtime/
│       │   ├── mod.rs
│       │   ├── ws_client.rs      # WebSocket connection manager
│       │   └── handlers.rs       # Incoming message handlers
│       ├── sensors/
│       │   ├── mod.rs
│       │   ├── bridge.rs         # Tauri mobile sensor API wrapper
│       │   ├── classifier.rs     # Crash/fall signal processing
│       │   └── detector.rs       # State machine for incident detection
│       ├── push/
│       │   └── mod.rs            # Push notification manager
│       └── models/
│           └── mod.rs            # Shared Rust structs
│
├── src/                          # Vanilla JS frontend
│   ├── index.html
│   ├── css/
│   │   ├── main.css              # Design system, CSS variables
│   │   ├── components.css        # Reusable component styles
│   │   └── layout.css            # Grid/flex layouts
│   ├── js/
│   │   ├── app.js                # Entry point, app bootstrap
│   │   ├── router.js             # Hash-based SPA router
│   │   ├── state.js              # Global state store (pub/sub)
│   │   ├── api.js                # REST client (fetch wrapper)
│   │   ├── ws.js                 # WebSocket client
│   │   ├── components/
│   │   │   ├── map.js            # Leaflet OSM map wrapper
│   │   │   ├── member-marker.js
│   │   │   ├── sos-button.js
│   │   │   ├── chat-window.js
│   │   │   ├── med-card.js
│   │   │   ├── place-editor.js
│   │   │   ├── incident-alert.js
│   │   │   ├── driving-report.js
│   │   │   └── tile-panel.js
│   │   ├── pages/
│   │   │   ├── map-page.js
│   │   │   ├── places-page.js
│   │   │   ├── chat-page.js
│   │   │   ├── meds-page.js
│   │   │   ├── driving-page.js
│   │   │   ├── settings-page.js
│   │   │   └── admin-page.js
│   │   ├── lib/
│   │   │   ├── dom.js            # DOM helper utilities
│   │   │   ├── events.js         # Event bus
│   │   │   ├── storage.js        # localStorage wrapper
│   │   │   ├── format.js         # Date, distance, speed formatters
│   │   │   └── crypto.js         # Web Crypto API helpers
│   │   └── styles/
│   │       └── theme.js          # Dynamic theme switching
│   └── assets/
│       ├── icons/
│       └── images/
│
├── package.json                  # Minimal deps (vite optional)
├── vite.config.js                # Optional: for dev HMR
└── README.md
```

---

## 8. Vanilla JS Architecture

### 8.1 State Management (Pub/Sub Store)
```javascript
// state.js — lightweight reactive store
class Store {
  constructor(initialState = {}) {
    this.state = { ...initialState };
    this.listeners = new Map(); // event → Set<callback>
  }

  get(key) { return this.state[key]; }

  set(key, value) {
    const prev = this.state[key];
    this.state[key] = value;
    if (prev !== value) {
      this.emit(key, value, prev);
      this.emit('*', { key, value, prev });
    }
  }

  on(event, callback) {
    if (!this.listeners.has(event)) this.listeners.set(event, new Set());
    this.listeners.get(event).add(callback);
    return () => this.listeners.get(event).delete(callback); // unsubscribe
  }

  emit(event, ...args) {
    if (this.listeners.has(event)) {
      this.listeners.get(event).forEach(cb => cb(...args));
    }
  }
}

// Global store instance
export const store = new Store({
  user: null,
  circle: null,
  members: [],
  places: [],
  incidents: [],
  medications: [],
  activeIncident: null,
  wsConnected: false,
});
```

### 8.2 Component Pattern (Web Components style, no build step)
```javascript
// components/base.js — lightweight component base
class Component {
  constructor(rootElement) {
    this.root = rootElement;
    this.unsubscribers = [];
  }

  // Subscribe to store, auto-cleanup on destroy
  subscribe(event, callback) {
    this.unsubscribers.push(store.on(event, callback));
  }

  // Render: override in subclass
  render() { return ''; }

  // Mount to DOM
  mount(parent) {
    parent.innerHTML = this.render();
    this.afterMount();
  }

  afterMount() {}

  destroy() {
    this.unsubscribers.forEach(unsub => unsub());
  }
}
```

### 8.3 API Client
```javascript
// api.js — fetch wrapper with auth
class ApiClient {
  constructor(baseUrl) {
    this.baseUrl = baseUrl;
    this.token = localStorage.getItem('auth_token');
  }

  setToken(token) {
    this.token = token;
    localStorage.setItem('auth_token', token);
  }

  async request(method, path, body = null) {
    const headers = { 'Content-Type': 'application/json' };
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;

    const res = await fetch(`${this.baseUrl}/api/v1${path}`, {
      method,
      headers,
      body: body ? JSON.stringify(body) : null,
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({}));
      throw new ApiError(res.status, err.message || 'Request failed');
    }

    return res.json();
  }

  get(path)    { return this.request('GET', path); }
  post(path, b) { return this.request('POST', path, b); }
  patch(path, b) { return this.request('PATCH', path, b); }
  delete(path) { return this.request('DELETE', path); }
}

export const api = new ApiClient(window.__API_URL__);
```

### 8.4 WebSocket Client
```javascript
// ws.js — auto-reconnecting WebSocket
class RealtimeClient {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.handlers = new Map();
    this.reconnectDelay = 1000;
    this.maxReconnectDelay = 30000;
    this.shouldReconnect = true;
  }

  connect() {
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      this.reconnectDelay = 1000;
      store.set('wsConnected', true);
      // Authenticate
      this.send({ type: 'auth', token: api.token });
    };

    this.ws.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      const handler = this.handlers.get(msg.type);
      if (handler) handler(msg);
    };

    this.ws.onclose = () => {
      store.set('wsConnected', false);
      if (this.shouldReconnect) {
        setTimeout(() => this.connect(), this.reconnectDelay);
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
      }
    };

    this.ws.onerror = () => this.ws.close();
  }

  on(type, handler) { this.handlers.set(type, handler); }
  send(data) { if (this.ws?.readyState === WebSocket.OPEN) this.ws.send(JSON.stringify(data)); }
  disconnect() { this.shouldReconnect = false; this.ws?.close(); }
}

export const ws = new RealtimeClient(window.__WS_URL);
```

---

## 8.5 Leaflet + OpenStreetMap Map Module

### Tile Servers
| Provider | URL | Notes |
|----------|-----|-------|
| OSM Standard | `https://tile.openstreetmap.org/{z}/{x}/{y}.png` | Free, attribution required, rate-limited |
| OSM France | `https://a.tile.openstreetmap.fr/osmfr/{z}/{x}/{y}.png` | Good EU coverage |
| CartoDB Positron | `https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png` | Clean light theme, no key |
| CartoDB Dark | `https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png` | Dark theme |
| Self-hosted | `http://localhost:8080/{z}/{x}/{y}.png` | TileMaker + PostgreSQL for production scale |

### Geocoding (Nominatim)
```
GET https://nominatim.openstreetmap.org/search?q={address}&format=json&limit=5
GET https://nominatim.openstreetmap.org/reverse?lat={lat}&lon={lon}&format=json
```
- Rate limit: 1 req/s (cache results in SQLite)
- Custom Nominatim instance recommended for production

### Map Component Design
```javascript
// components/map.js
import L from 'leaflet';

class FamilyMap {
  constructor(containerId, options = {}) {
    this.map = L.map(containerId, {
      center: options.center || [51.505, -0.09],
      zoom: options.zoom || 13,
      zoomControl: true,
      attributionControl: true,
    });

    // Default to CartoDB Positron (clean, no key needed)
    L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
    }).addTo(this.map);

    this.memberMarkers = new Map(); // user_id → L.Marker
    this.trailLayers = new Map();   // user_id → L.Polyline
    this.placeCircles = new Map();  // place_id → L.Circle
  }

  updateMemberLocation(userId, lat, lng, metadata = {}) {
    const pos = [lat, lng];

    if (!this.memberMarkers.has(userId)) {
      const icon = L.divIcon({
        className: 'member-marker',
        html: `<div class="marker-pin" style="--marker-color:${metadata.color || '#3b82f6'}">
                 <span class="marker-initial">${metadata.initial || '?'}</span>
               </div>`,
        iconSize: [36, 36],
        iconAnchor: [18, 36],
      });
      const marker = L.marker(pos, { icon }).addTo(this.map);
      marker.bindPopup(`<b>${metadata.name}</b><br>Updated: ${metadata.time || 'now'}`);
      this.memberMarkers.set(userId, marker);
    } else {
      this.memberMarkers.get(userId).setLatLng(pos);
    }

    // Update trail
    this._addTrailPoint(userId, pos);
  }

  _addTrailPoint(userId, pos) {
    if (!this.trailLayers.has(userId)) {
      const polyline = L.polyline([], {
        color: '#3b82f6',
        weight: 3,
        opacity: 0.7,
      }).addTo(this.map);
      this.trailLayers.set(userId, polyline);
    }
    const polyline = this.trailLayers.get(userId);
    polyline.addLatLng(pos);
    // Keep last 200 points to avoid memory bloat
    const latlngs = polyline.getLatLngs();
    if (latlngs.length > 200) {
      polyline.setLatLngs(latlngs.slice(-200));
    }
  }

  addPlaceCircle(place) {
    const circle = L.circle([place.lat, place.lng], {
      radius: place.radius_meters || 150,
      color: '#10b981',
      fillColor: '#10b981',
      fillOpacity: 0.15,
      weight: 2,
    }).addTo(this.map);
    circle.bindPopup(`<b>${place.name}</b><br>Radius: ${place.radius_meters}m`);
    this.placeCircles.set(place.id, circle);
  }

  removePlaceCircle(placeId) {
    const circle = this.placeCircles.get(placeId);
    if (circle) {
      this.map.removeLayer(circle);
      this.placeCircles.delete(placeId);
    }
  }

  flyTo(lat, lng, zoom = 16) {
    this.map.flyTo([lat, lng], zoom, { duration: 1.5 });
  }

  clearTrails() {
    this.trailLayers.forEach(layer => layer.setLatLngs([]));
  }

  destroy() {
    this.map.remove();
  }
}
```

### Offline Map Caching (Tauri + SQLite)
- Cache viewed tiles in SQLite when online
- On offline, serve cached tiles via custom Leaflet TileLayer
- Pre-cache home/school/work areas on Wi-Fi

### Dark Mode Support
```javascript
// Toggle between CartoDB Positron (light) and Dark Matter (dark)
const lightTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}.png');
const darkTiles = L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}.png');

function setDarkMode(enabled) {
  if (enabled) {
    map.removeLayer(lightTiles);
    darkTiles.addTo(map);
  } else {
    map.removeLayer(darkTiles);
    lightTiles.addTo(map);
  }
}
```

---

## 9. Tauri Commands (Rust ↔ JS Bridge)

```rust
// commands/location.rs
#[tauri::command]
async fn start_location_tracking(
    window: tauri::Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Start GPS tracking, emit events to frontend
    let mut rx = state.sensor_manager.start_gps().await.map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn(async move {
        while let Some(ping) = rx.recv().await {
            window.emit("location_ping", ping).ok();
        }
    });
    Ok(())
}

#[tauri::command]
async fn send_location_ping(
    ping: LocationPing,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.ws_client.send_location(ping).await.map_err(|e| e.to_string())
}

// commands/sos.rs
#[tauri::command]
async fn trigger_sos(
    silent: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let location = state.sensor_manager.current_location().await
        .map_err(|e| e.to_string())?;
    state.dispatch_service.trigger_sos(location, silent).await
        .map_err(|e| e.to_string())
}

// commands/medications.rs
#[tauri::command]
async fn get_medications(state: State<'_, AppState>) -> Result<Vec<Medication>, String> {
    state.db.get_medications().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn log_medication_dose(
    med_id: String,
    status: String, // "taken" | "skipped"
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.log_adherence(&med_id, &status).await.map_err(|e| e.to_string())
}

// commands/geofence.rs
#[tauri::command]
async fn check_geofence(
    lat: f64,
    lng: f64,
    state: State<'_, AppState>,
) -> Result<Option<GeofenceEvent>, String> {
    state.geofence_engine.check(lat, lng).await.map_err(|e| e.to_string())
}

// commands/tile.rs
#[tauri::command]
async fn tile_ring(tile_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.tile_service.ring(&tile_id).await.map_err(|e| e.to_string())
}
```

---

## 10. Sensor Pipeline (Crash / Fall Detection)

### 10.1 Rust Signal Processing
```rust
// sensors/classifier.rs
use std::collections::VecDeque;

pub struct SensorWindow {
    samples: VecDeque<ImuSample>,  // rolling window of accelerometer + gyroscope
    window_size: usize,            // ~2 seconds at 50Hz = 100 samples
}

#[derive(Clone, Debug)]
pub struct ImuSample {
    pub ax: f32, pub ay: f32, pub az: f32,  // accelerometer (m/s²)
    pub gx: f32, pub gy: f32, pub gz: f32,  // gyroscope (rad/s)
    pub ts: u64,                            // millis
}

impl SensorWindow {
    pub fn new(window_size: usize) -> Self {
        Self { samples: VecDeque::with_capacity(window_size), window_size }
    }

    pub fn push(&mut self, sample: ImuSample) {
        if self.samples.len() >= self.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// Compute jerk (derivative of acceleration) — key crash indicator
    pub fn max_jerk(&self) -> f32 {
        self.samples.iter()
            .zip(self.samples.iter().skip(1))
            .map(|(a, b)| {
                let dt = (b.ts - a.ts) as f32 / 1000.0;
                if dt > 0.0 {
                    (((b.ax - a.ax).powi(2) + (b.ay - a.ay).powi(2) + (b.az - a.az).powi(2)).sqrt()) / dt
                } else { 0.0 }
            })
            .fold(0.0, f32::max)
    }

    /// Detect free-fall (near-zero acceleration followed by impact)
    pub fn detect_fall(&self) -> FallConfidence {
        // Pattern: free-fall (< 1g for > 200ms) → sudden stop (> 3g spike)
        // Implementation: state machine over the window
        // Returns confidence 0.0 - 1.0
        // ...
    }

    /// Detect crash: high-g spike + rapid deceleration + rotation
    pub fn detect_crash(&self) -> CrashConfidence {
        // Pattern: sustained > 4g deceleration + gyroscope rotation
        // ...
    }
}
```

### 10.2 Detection State Machine
```
IDLE → (free-fall detected) → FALL_CANDIDATE → (impact confirmed) → FALL_DETECTED
                                                                    ↓
                                                           audible prompt → 30s timeout
                                                                    ↓ (no response)
                                                           ESCALATED → alert emergency contact

IDLE → (high-g + rotation spike) → CRASH_CANDIDATE → (sustained pattern) → CRASH_DETECTED
                                                                        ↓
                                                               alert Circle + optional dispatch
```

---

## 11. Subscription Tiers

| Feature | Free | Silver | Gold | Platinum |
|--------|:----:|:------:|:----:|:--------:|
| Live location (Circle ≤ 4) | ✅ | ✅ | ✅ | ✅ |
| Place alerts | 3 places | 10 places | Unlimited | Unlimited |
| Location history | 7 days | 30 days | 90 days | 365 days |
| SOS Alerts | ✅ | ✅ | ✅ | ✅ |
| Crash Detection | ✅ | ✅ | ✅ | ✅ |
| Fall Detection | — | ✅ | ✅ | ✅ |
| Medication tracker | ✅ | ✅ | ✅ | ✅ |
| Group chat | ✅ | ✅ | ✅ | ✅ |
| Driving reports | — | Basic | Full | Full |
| Emergency dispatch | — | — | ✅ | ✅ |
| Tile tracker | — | 1 device | 5 devices | Unlimited |
| Roadside assistance | — | — | — | ✅ |
| Medical advice hotline | — | — | — | ✅ |
| Circle size | 4 | 8 | 15 | Unlimited |

---

## 12. Security & Privacy

- **End-to-end encryption** for chat messages (Signal Protocol via `libsignal-protocol` crate).
- **Location data encrypted** at rest (AES-256) and in transit (TLS 1.3).
- **Rust's memory safety** eliminates entire classes of vulnerabilities (buffer overflows, use-after-free).
- **GDPR / CCPA compliant:** data export, right to delete, explicit consent for location sharing.
- **Role-based access:** Circle admin vs. member permissions.
- **On-device processing** for crash/fall reduces raw sensor data leaving the device.
- **Tauri's security model:** CSP, isolated iframe, no Node.js integration in frontend.
- **Biometric app lock** (Windows Hello / macOS Touch ID / mobile biometrics).
- **Incident data** retained for 30 days for debugging, then purged.

---

## 13. Key Crates & Dependencies

### Rust (Tauri + Server)
```toml
[dependencies]
# Tauri
tauri = { version = "2", features = ["macos-private-api"] }
tauri-plugin-geolocation = "2"
tauri-plugin-notification = "2"
tauri-plugin-store = "2"

# Async runtime
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"

# Web framework (server)
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "uuid", "chrono", "json"] }
rusqlite = { version = "0.31", features = ["bundled", "uuid", "chrono"] }
deadpool = "0.10"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Auth
jsonwebtoken = "9"
argon2 = "0.5"
totp-rs = "5"

# Geospatial
geo = "0.28"
geohash = "0.13"
postgis = "0.9"  # or use sqlx with raw PostGIS functions

# WebSocket (server)
tokio-tungstenite = "0.24"

# Redis
redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }

# HTTP client (server → external APIs)
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Push notifications
a2 = "0.8"  # APNs
firebase-push = "0.1"  # or custom FCM client

# Error handling
thiserror = "1"
anyhow = "1"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Time
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# Config
config = "0.14"
dotenvy = "0.15"

# Stripe
stripe = "0.34"  # or use reqwest with Stripe REST API directly

# Signal Protocol (E2E chat encryption)
libsignal-protocol = "0.1"  # or vendored fork
```

### Vanilla JS (Frontend)
```json
{
  "dependencies": {
    "leaflet": "^1.9.4"
  },
  "devDependencies": {
    "vite": "^5.0.0"
  }
}
```

---

## 14. Implementation Roadmap

### Phase 1 — MVP (Weeks 1–8)
- [ ] Scaffold Tauri app + Axum server monorepo
- [ ] Auth (register, login, OTP) — Tauri commands + REST API
- [ ] Circle create/join/invite
- [ ] Real-time location sharing on Mapbox map
- [ ] Basic place alerts (geofencing)
- [ ] SOS button + alert broadcast
- [ ] Push notifications
- [ ] Basic group chat (WebSocket)

### Phase 2 — Health & Safety (Weeks 9–14)
- [ ] Crash detection (Rust signal processing + sensor bridge)
- [ ] Fall detection with audible prompt + escalation
- [ ] Medication tracker + reminders
- [ ] Location history (7/30/90 day tiers)
- [ ] Subscription billing (Stripe)

### Phase 3 — Premium Features (Weeks 15–20)
- [ ] Driving analytics + reports
- [ ] Emergency dispatch integration (Gold/Platinum)
- [ ] Tile tracker integration (custom Tauri plugin)
- [ ] Roadside assistance flow
- [ ] Medical advice hotline (Platinum)

### Phase 4 — Polish & Scale (Weeks 21–24)
- [ ] E2E encryption for chat (Signal Protocol)
- [ ] Performance optimization (batching, delta updates)
- [ ] Advanced geofence shapes (polygons)
- [ ] Apple Watch / Wear OS companion apps
- [ ] SOC 2 compliance audit
- [ ] Beta testing & launch prep

---

## 15. Key Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| False positive crash/fall detection | User trust erosion, emergency spam | On-device confidence thresholds + server-side validation + user confirmation flow |
| Battery drain from constant GPS | Poor UX, uninstalls | Adaptive ping frequency, significant-motion API, low-power mode |
| Geofence false triggers (GPS drift) | Alert fatigue | Minimum dwell time, accuracy threshold, radius tuning |
| Emergency dispatch liability | Legal / regulatory risk | Clear ToS, disclaimers, integration with established dispatch providers, optional feature |
| Tile hardware dependency | Feature unavailable without hardware | Make it optional, support multiple tracker brands long-term |
| Tauri mobile maturity | Mobile bugs / missing APIs | Desktop-first strategy; mobile as secondary target; custom native plugins |
| Scaling real-time location | Server overload at scale | Redis pub/sub sharding, edge WebSocket nodes, horizontal autoscaling |
| Vanilla JS complexity at scale | Maintenance burden | Strict module patterns, component base class, comprehensive tests |

---

## 16. Third-Party Integrations

| Service | Purpose |
|---------|---------|
| **Stripe** | Subscription billing |
| **OpenStreetMap** | Free map tiles, geocoding (Nominatim), no API key required |
| **Twilio** | SMS/email OTP, fallback alerts |
| **Firebase** | Push notifications (FCM) |
| **Sentry** | Error/crash reporting (Rust SDK) |
| **Tile SDK** | Bluetooth tracker integration |
| **Urgent.ly / RapidSOS** | Emergency dispatch |
| **Signal Protocol** | E2E encrypted chat |
| **AWS** | Infrastructure (EC2, RDS, S3, SQS) |

---

## 17. Monetization Model

- **Free tier:** 4-person Circle, 7-day history, 3 places, basic SOS & chat.
- **Silver ($4.99/mo):** 8-person Circle, 30-day history, fall detection, basic driving reports.
- **Gold ($9.99/mo):** 15-person Circle, 90-day history, emergency dispatch, 5 Tile devices.
- **Platinum ($14.99/mo):** Unlimited Circle, 365-day history, roadside assistance, medical hotline, unlimited Tile devices.
- **Family plan:** Add per-member discount for Circles > 8 on Gold/Platinum.
- **Annual billing:** 2 months free incentive.

---

## 18. Next Steps

1. **Validate** this plan with stakeholders and adjust scope.
2. **Scaffold** the monorepo (Tauri app + Axum server + shared types).
3. **Build** Phase 1 MVP features.
4. **Design** Rust sensor pipeline and crash/fall classifier.
5. **Set up** Stripe billing and subscription gating.
6. **Recruit** a beta family for early testing.
