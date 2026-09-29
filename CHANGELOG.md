# Changelog

All notable changes to Amulet AI will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Master plan document (`MASTER_PLAN.md`) covering full app architecture, feature matrix, phased roadmap, data models, API design, and deployment strategy
- Changelog (`CHANGELOG.md`) to track all notable changes

### Phase 2: Location & Safety Core
- **Real-Time Location Sharing**: WebSocket location broadcast, live map with member markers, location update throttling, battery-efficient tracking modes (active/balanced/passive), member location privacy controls
- **Place Alerts (Geofencing)**: Server-side geofence engine with Haversine distance, place CRUD with map-based editor, enter/leave event detection, push notifications on geofence events, per-member place alert rules
- **SOS Alerts**: SOS trigger (active + silent modes), real-time SOS broadcast to circle, SOS acknowledgment flow, escalation timer, location attachment with SOS
- New DB tables: `geofence_events`, `incident_responses`, `user_geofence_state`
- New API endpoints: `GET /api/circles/:id/geofence-events`, `POST /api/incidents/:id/acknowledge`, `GET /api/incidents/:id/responses`
- WebSocket protocol: `GeofenceEvent`, `FallDetected`, `CrashDetected` message types
- Frontend: toast notifications, incident alert modal, fall alert overlay, tracking mode selector, privacy toggle

### Phase 3: Health & Detection
- **Crash Detection**: Sensor fusion (accelerometer + gyroscope), crash severity classifier (minor/moderate/severe), auto-SOS on severe crashes, false-positive filtering, configurable sensitivity
- **Fall Detection**: Accelerometer pattern recognition, audible "Are you OK?" prompt with 30-second countdown, emergency contact notification, Tauri commands for acknowledgment/cancellation
- **Location History**: Timeline UI with map trail, GPX export, per-member history filter, history retention limits
- **Medication Tracker & Reminders**: Local push notification reminders, adherence logging (taken/skipped/missed), adherence stats & streaks, upcoming schedule view, refill date calculation
- **Group Chat**: Real-time messaging via WebSocket, message persistence, read receipts, typing indicators
- New Tauri commands: `process_sensor_reading`, `acknowledge_fall_alert`, `cancel_fall_alert`, `get_fall_alert_status`, `set_detection_sensitivity`, `reset_detection`
- New API endpoints: `POST /api/sensors/readings`, `GET /api/sensors/status`, `GET /api/medications/:id/adherence/stats`, `GET /api/medications/:id/adherence/streak`, `GET /api/medications/upcoming`
- New frontend pages: HistoryPage with timeline and map trail
- New frontend components: FallAlert with audible prompt and countdown
- New frontend services: MedReminder with push notifications and adherence tracking

### Phase 4: Premium Features
- **Driving Reports**: Auto-start/stop driving sessions, speed monitoring, rapid acceleration detection, hard braking detection, phone use detection, trip summary & safety scoring, weekly/monthly reports, driving statistics dashboard
- **Emergency Dispatch**: Tier-gated dispatch (Gold/Platinum), auto-dispatch on severe crashes, manual dispatch from SOS, dispatch status tracking, dispatch cancellation
- **Tile Tracker Integration**: Tile Bluetooth pairing flow, item tracker management, ring-to-find, last-known location on map, community find network
- New Tauri commands: `start_driving_session`, `stop_driving_session`, `report_driving_event`, `get_driving_status`, `get_driving_stats`
- New API endpoints: `GET /api/driving/sessions/:id/detail`, `GET /api/driving/stats`, `POST /api/dispatch/incident/:id`, `GET /api/dispatch/:id/status`, `POST /api/dispatch/:id/cancel`, `GET /api/tiles/:id/community-finds`, `POST /api/tiles/community-find`
- New frontend: driving stats dashboard, session controls, community find results in tile panel
- New services: dispatch tier gating, tile community find network

### Phase 5: Assistance & Polish
- **Roadside Assistance**: Request flow for towing, flat tire, jump start, lockout, fuel delivery; service provider network with ratings and ETA; cost estimates; request status tracking and cancellation
- **Medical Advice**: Platinum-tier medical hotline integration; immediate connection to medical professionals
- **Onboarding Flow**: 6-step welcome tour with feature highlights; progress indicators; skip option; completion tracking
- **Theme System**: Dark/light mode with localStorage persistence; system preference detection
- **Offline Mode**: Local SQLite cache for offline support; location caching with sync queue; medication dose logging offline
- **Performance**: Lazy-loaded routes via dynamic imports; code splitting per page; efficient WebSocket message handling
- New API endpoints: `POST /api/assistance/roadside`, `POST /api/assistance/medical`, `GET /api/assistance/:id/status`, `POST /api/assistance/:id/cancel`, `GET /api/assistance/providers`
- New frontend pages: AssistancePage with service type grid and status tracking; OnboardingPage with feature tour
- New services: assistance service with tier gating, provider network, cost estimates

### Bug Fixes & Improvements
- Fixed Rust compilation errors: type annotations, private field access, Debug/Clone/Serialize derives
- Fixed borrow checker issues: MutexGuard held across await points
- Added missing imports: `chrono::{DateTime, Utc}`, `uuid::Uuid`, `sqlx::Row`
- Fixed `DrivingSession` struct: added missing fields (`distance_km`, `max_speed`, `avg_speed`, `harsh_braking_count`, `rapid_acceleration_count`, `phone_usage_count`)
- Fixed `create_incident` calls: wrapped coordinates in `Some()` for `Option<f64>` parameters
- Removed `Default` impl for `RealtimeHub` (required `db_pool` argument)
- Added `#[derive(serde::Serialize)]` to all API response structs
- Cleaned up unused imports and variables across the codebase

---

## [0.1.0] — 2026-09-29

### Added
- Initial project scaffolding (Tauri v2 + Axum + Vanilla JS + Leaflet/OpenStreetMap)
- Auth system: register, login, JWT access/refresh tokens, logout
- Token rotation with family tracking and reuse detection (Redis-backed)
- Argon2 password hashing
- TOTP-based two-factor authentication
- Password reset flow (token-based, 1hr expiry, single-use)
- Email verification flow (token-based, 24hr expiry)
- Rate limiting (global 100 req/min, auth 5 req/min per IP+endpoint)
- Account lockout after 5 failed login attempts (15min lockout)
- Circle/family group management (create, join, leave, invite codes)
- Location tracking infrastructure (DB models, API endpoints, Tauri commands)
- Geofence/place management (CRUD, DB models, API endpoints)
- Incident/SOS system (create, acknowledge, resolve)
- Medication tracker (CRUD, adherence logging)
- Chat system (rooms, messages, read receipts)
- Driving session tracking (sessions, events, reports)
- Subscription/tier management (free, silver, gold, platinum)
- Tile tracker integration (link, ring, locate)
- Sensor pipeline (crash/fall detection scaffolding)
- WebSocket realtime hub (location broadcast, chat, alerts)
- Local SQLite cache for offline support
- Frontend pages: login, signup, map, places, medications, driving, chat, settings, admin
- Frontend components: map, member markers, SOS button, chat window, place editor, med cards, driving report, tile panel, incident alert, circle manager

---

[Unreleased]: https://github.com/your-org/amulet-ai/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/amulet-ai/releases/tag/v0.1.0
