# Amulet AI — Master Plan

> Family safety and assistance app built with **Rust + Tauri + Vanilla JS + OpenStreetMap**.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Feature Matrix](#feature-matrix)
3. [Phased Roadmap](#phased-roadmap)
4. [Data Models](#data-models)
5. [API Design](#api-design)
6. [Security](#security)
7. [Testing Strategy](#testing-strategy)
8. [Deployment](#deployment)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                     Client (Tauri v2)                    │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │
│  │  Vanilla JS  │  │  Rust Core   │  │  Local SQLite  │  │
│  │  + Leaflet   │  │  (Sensors,   │  │  (Cache/Queue) │  │
│  │  + OSM Maps  │  │   IPC, WS)  │  │                │  │
│  └─────────────┘  └──────────────┘  └────────────────┘  │
└────────────────────────┬────────────────────────────────┘
                         │ HTTPS / WSS
┌────────────────────────▼────────────────────────────────┐
│                 Cloud Server (Axum)                       │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌───────────┐  │
│  │  Auth    │ │  Realtime │ │  REST    │ │  Sensor   │  │
│  │  (JWT)   │ │  (WS Hub) │ │  API     │ │  Pipeline │  │
│  └──────────┘ └───────────┘ └──────────┘ └───────────┘  │
│  ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌───────────┐  │
│  │Geofence  │ │  Billing  │ │ Dispatch │ │  Tile     │  │
│  │ Engine   │ │  (Stripe) │ │ Service  │ │  Service  │  │
│  └──────────┘ └───────────┘ └──────────┘ └───────────┘  │
└────────────────────────┬────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────┐
│              Data Layer                                  │
│  ┌──────────────┐  ┌───────────┐  ┌──────────────────┐  │
│  │  PostgreSQL  │  │  Redis    │  │  SQLite (local)  │  │
│  │  + PostGIS   │  │  (Cache)  │  │  (Offline cache) │  │
│  └──────────────┘  └───────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Feature Matrix

### Core Features (Free Tier)

| Feature | Status | Priority | Phase |
|---------|--------|----------|-------|
| Real-Time Location Sharing | 🟢 Complete | P0 | 2 |
| Place Alerts (Geofencing) | 🟢 Complete | P0 | 2 |
| Location History | 🟢 Complete | P1 | 3 |
| SOS Alerts | 🟢 Complete | P0 | 2 |
| Crash Detection | 🟢 Complete | P1 | 3 |
| Fall Detection | 🟢 Complete | P1 | 3 |
| Medication Tracker & Reminders | 🟢 Complete | P1 | 3 |
| Group Chat | 🟢 Complete | P1 | 3 |

### Advanced & Paid Features

| Feature | Status | Priority | Phase | Tier |
|---------|--------|----------|-------|------|
| Driving Reports | 🟢 Complete | P2 | 4 | Silver+ |
| Emergency Dispatch | 🟢 Complete | P2 | 4 | Gold+ |
| Tile Tracker Integration | 🟢 Complete | P2 | 4 | Silver+ |
| Roadside & Medical Assistance | 🟢 Complete | P2 | 5 | Gold+ |

### Legend
- 🟢 Complete | 🟡 Partial | 🔴 Not Started

---

## Phased Roadmap

### Phase 1: Foundation ✅ Complete
- [x] Project scaffolding (Tauri + Axum + Vanilla JS)
- [x] Auth system (register, login, JWT, refresh tokens, 2FA/TOTP)
- [x] Database layer (PostgreSQL + SQLite local cache)
- [x] Circle/family group management
- [x] Basic REST API structure
- [x] WebSocket realtime hub
- [x] Tauri command structure

### Phase 2: Location & Safety Core ✅ Complete
- [x] **Real-Time Location Sharing**
  - [x] WebSocket location broadcast to circle members
  - [x] Live map with member markers (Leaflet)
  - [x] Location update throttling (client + server)
  - [x] Battery-efficient tracking modes (active/passive)
  - [x] Member location privacy controls

- [x] **Place Alerts (Geofencing)**
  - [x] Server-side geofence engine (Haversine distance)
  - [x] Place CRUD with map-based editor
  - [x] Enter/leave event detection
  - [x] Push notification on geofence events
  - [x] Per-member place alert rules

- [x] **SOS Alerts**
  - [x] SOS trigger (active + silent modes)
  - [x] Real-time SOS broadcast to circle
  - [x] SOS acknowledgment flow
  - [x] Escalation timer (if unacknowledged)
  - [x] Location attachment with SOS

### Phase 3: Health & Detection ✅ Complete
- [x] **Crash Detection**
  - [x] Sensor fusion (accelerometer + gyroscope)
  - [x] Crash severity classifier (minor/moderate/severe)
  - [x] Auto-SOS with crash context
  - [x] False-positive filtering
  - [x] Configurable sensitivity

- [x] **Fall Detection**
  - [x] Accelerometer pattern recognition
  - [x] Audible "Are you OK?" prompt
  - [x] Countdown timer before auto-alert
  - [x] Emergency contact notification

- [x] **Location History**
  - [x] Chronological location log
  - [x] Timeline UI with map trail
  - [x] History retention limits (free: 7 days, paid: 30-90 days)
  - [x] Export history (GPX)
  - [x] Privacy: per-member history visibility

- [x] **Medication Tracker & Reminders**
  - [x] Medication CRUD with dosage/schedule
  - [x] Local push notification reminders
  - [x] Adherence logging (taken/skipped/missed)
  - [x] Adherence reports & streaks
  - [x] Refill reminders

- [x] **Group Chat**
  - [x] Circle-scoped chat rooms
  - [x] Real-time messaging via WebSocket
  - [x] Message persistence
  - [x] Read receipts
  - [ ] Image/file sharing (paid tier) — Phase 4

### Phase 4: Premium Features ✅ Complete
- [x] **Driving Reports**
  - [x] Driving session auto-start/stop
  - [x] Speed monitoring & speeding alerts
  - [x] Rapid acceleration detection
  - [x] Hard braking detection
  - [x] Phone use while driving detection
  - [x] Trip summary & scoring
  - [x] Weekly/monthly reports

- [x] **Emergency Dispatch**
  - [x] Integration with emergency services API
  - [x] Auto-dispatch on severe crash
  - [x] Manual dispatch from SOS
  - [x] Dispatch status tracking
  - [x] Gold/Platinum tier gating

- [x] **Tile Tracker Integration**
  - [x] Tile Bluetooth pairing flow
  - [x] Item tracker management
  - [x] Ring-to-find
  - [x] Last-known location on map
  - [x] Community find network

### Phase 5: Assistance & Polish ✅ Complete
- [x] **Roadside & Medical Assistance**
  - [x] Roadside assistance request flow
  - [x] Towing/flat tire service dispatch
  - [x] Medical advice hotline integration
  - [x] Tier-based access control
  - [x] Service provider network

- [x] **Polish & UX**
  - [x] Onboarding flow
  - [x] Dark/light theme
  - [x] Offline mode improvements
  - [x] Performance optimization (lazy loading, code splitting)

---

## Data Models

### Core Entities

```
users
├── id (UUID, PK)
├── email (unique)
├── password_hash (Argon2)
├── display_name
├── phone
├── avatar_url
├── totp_secret
├── totp_enabled
├── failed_login_attempts
├── locked_until
├── email_verified
├── created_at
└── updated_at

circles
├── id (UUID, PK)
├── name
├── owner_id (FK → users)
├── invite_code (unique)
├── created_at
└── updated_at

circle_members
├── circle_id (FK → circles)
├── user_id (FK → users)
├── role (owner/member)
├── joined_at
└── PRIMARY KEY (circle_id, user_id)

locations
├── id (UUID, PK)
├── user_id (FK → users)
├── circle_id (FK → circles)
├── latitude
├── longitude
├── accuracy
├── altitude
├── speed
├── heading
├── battery_level
├── recorded_at
└── received_at

places
├── id (UUID, PK)
├── circle_id (FK → circles)
├── name
├── latitude
├── longitude
├── radius_meters
├── icon
├── created_at
└── updated_at

geofence_events
├── id (UUID, PK)
├── place_id (FK → places)
├── user_id (FK → users)
├── event_type (enter/exit)
├── latitude
├── longitude
├── triggered_at
└── notified_members (JSON)

incidents
├── id (UUID, PK)
├── circle_id (FK → circles)
├── user_id (FK → users, nullable)
├── incident_type (sos/crash/fall)
├── severity
├── latitude
├── longitude
├── status (active/acknowledged/resolved)
├── metadata (JSON)
├── created_at
└── resolved_at

incident_responses
├── id (UUID, PK)
├── incident_id (FK → incidents)
├── user_id (FK → users)
├── action (acknowledged/responded/resolved)
├── note
└── created_at

medications
├── id (UUID, PK)
├── user_id (FK → users)
├── name
├── dosage
├── schedule (JSON)
├── instructions
├── active
├── created_at
└── updated_at

medication_adherence
├── id (UUID, PK)
├── medication_id (FK → medications)
├── user_id (FK → users)
├── scheduled_for
├── taken_at
├── status (taken/skipped/missed)
├── notes
└── created_at

chat_rooms
├── id (UUID, PK)
├── circle_id (FK → circles)
├── name
├── created_at
└── updated_at

chat_messages
├── id (UUID, PK)
├── room_id (FK → chat_rooms)
├── sender_id (FK → users)
├── body
├── message_type (text/image/file)
├── metadata (JSON)
├── created_at
└── edited_at

driving_sessions
├── id (UUID, PK)
├── user_id (FK → users)
├── started_at
├── ended_at
├── distance_km
├── max_speed
├── avg_speed
├── score
└── metadata (JSON)

driving_events
├── id (UUID, PK)
├── session_id (FK → driving_sessions)
├── event_type (speeding/acceleration/braking/phone_use)
├── severity
├── latitude
├── longitude
├── speed
├── timestamp
└── metadata (JSON)

subscriptions
├── id (UUID, PK)
├── user_id (FK → users)
├── tier (free/silver/gold/platinum)
├── stripe_customer_id
├── stripe_subscription_id
├── status
├── current_period_end
├── created_at
└── updated_at

tiles
├── id (UUID, PK)
├── user_id (FK → users)
├── tile_device_id (unique)
├── name
├── item_type (keys/wallet/pet/bag/other)
├── last_latitude
├── last_longitude
├── last_seen_at
├── battery_level
├── created_at
└── updated_at
```

---

## API Design

### REST Endpoints

```
Auth
POST   /api/auth/register
POST   /api/auth/login
POST   /api/auth/refresh
POST   /api/auth/logout
POST   /api/auth/otp/send
POST   /api/auth/otp/verify
POST   /api/auth/password/reset
POST   /api/auth/password/reset/confirm
POST   /api/auth/email/verify
POST   /api/auth/email/verify/confirm
GET    /api/users/me
PUT    /api/users/me

Circles
POST   /api/circles
GET    /api/circles
GET    /api/circles/:id
POST   /api/circles/:id/join
POST   /api/circles/:id/leave
GET    /api/circles/:id/members
POST   /api/circles/:id/invite-code

Locations
GET    /api/circles/:id/locations
GET    /api/circles/:id/locations/history/:user_id
POST   /api/locations (ingest from client)

Places / Geofencing
POST   /api/circles/:id/places
GET    /api/circles/:id/places
PUT    /api/places/:id
DELETE /api/places/:id

Incidents
POST   /api/circles/:id/incidents
GET    /api/circles/:id/incidents
PUT    /api/incidents/:id/status
POST   /api/incidents/:id/respond

Medications
POST   /api/medications
GET    /api/medications
PUT    /api/medications/:id
DELETE /api/medications/:id
POST   /api/medications/:id/adherence
GET    /api/medications/adherence

Chat
POST   /api/circles/:id/rooms
GET    /api/circles/:id/rooms
GET    /api/rooms/:id/messages
POST   /api/rooms/:id/messages
POST   /api/rooms/:id/read

Driving
POST   /api/driving/sessions
PUT    /api/driving/sessions/:id
POST   /api/driving/sessions/:id/events
GET    /api/driving/reports

Subscriptions
POST   /api/subscriptions
GET    /api/subscriptions
PUT    /api/subscriptions/:id/tier
POST   /api/subscriptions/:id/cancel

Tiles
POST   /api/tiles
POST   /api/tiles/:id/ring
GET    /api/tiles/:id/locate

Realtime
WS     /ws
```

### WebSocket Protocol

```json
// Client → Server
{ "type": "location_update", "payload": { "lat": 40.7, "lng": -74.0, "accuracy": 10 } }
{ "type": "sos_trigger", "payload": { "lat": 40.7, "lng": -74.0, "mode": "active" } }
{ "type": "chat_message", "payload": { "room_id": "...", "body": "..." } }
{ "type": "ping" }

// Server → Client
{ "type": "member_location", "payload": { "user_id": "...", "lat": 40.7, "lng": -74.0 } }
{ "type": "sos_alert", "payload": { "incident_id": "...", "user_id": "...", "lat": 40.7, "lng": -74.0 } }
{ "type": "geofence_event", "payload": { "place_name": "Home", "user_name": "Alice", "event": "enter" } }
{ "type": "chat_message", "payload": { "room_id": "...", "sender_id": "...", "body": "..." } }
{ "type": "pong" }
```

---

## Security

- [x] Argon2 password hashing
- [x] JWT access tokens (15 min TTL) + refresh tokens (30 day TTL)
- [x] Token rotation with family tracking
- [x] Redis-backed token blacklist
- [x] Rate limiting (global + auth-specific)
- [x] Account lockout after failed attempts
- [x] TOTP-based 2FA
- [ ] End-to-end encryption for chat messages
- [ ] Certificate pinning in Tauri client
- [ ] Input validation on all endpoints
- [ ] SQL injection prevention (parameterized queries)
- [ ] CORS configuration
- [ ] Audit logging for sensitive operations

---

## Testing Strategy

| Layer | Tools | Coverage Target |
|-------|-------|-----------------|
| Server unit | `cargo test` | 80%+ |
| Server integration | `cargo test` + testcontainers | Key flows |
| Tauri commands | `cargo test` | Core commands |
| Frontend unit | Vitest | Components |
| Frontend E2E | Playwright | Critical paths |
| API contract | Postman / Newman | All endpoints |
| Load | k6 | 100 concurrent users |

---

## Deployment

### Development
```bash
# Terminal 1 — Server
npm run server

# Terminal 2 — Tauri app
npm run tauri dev
```

### Production
```bash
# Build frontend
npm run build

# Build Tauri app
npm run tauri build

# Server deployment
cd server && cargo build --release
# Deploy binary + migrations to cloud VM / container
```

### Infrastructure
- **Server**: Docker container on AWS ECS / Fly.io / Railway
- **Database**: Managed PostgreSQL with PostGIS (Supabase / Neon / AWS RDS)
- **Redis**: Managed Redis (Upstash / AWS ElastiCache)
- **CDN**: CloudFront / Cloudflare for static assets
- **Monitoring**: Sentry + Grafana + Prometheus

---

## Subscription Tiers

| Feature | Free | Silver | Gold | Platinum |
|---------|------|--------|------|----------|
| Circle members | 3 | 5 | 10 | Unlimited |
| Location history | 7 days | 30 days | 90 days | Unlimited |
| Driving Reports | — | ✅ | ✅ | ✅ |
| Tile Trackers | — | 3 | 10 | Unlimited |
| Emergency Dispatch | — | — | ✅ | ✅ |
| Roadside Assistance | — | — | ✅ | ✅ |
| Medical Advice | — | — | — | ✅ |
| Chat file sharing | — | — | ✅ | ✅ |
| Price | $0 | $4.99/mo | $9.99/mo | $14.99/mo |

---

*Last updated: 2026-09-29*
