# Amulet AI — Auth System Implementation Plan

## Status: All Phases Complete ✅

## Completed

### Phase 1: Server-Side Auth (Critical Security Fixes) ✅
- [x] Fix `db/users.rs` — manual row mapping with SQLite-compatible queries
- [x] Fix `types.rs` — expanded User model with all fields, added `UserProfile` type
- [x] Fix `middleware/auth.rs` — route exclusion for public paths, `AuthUser` extractor
- [x] Fix `main.rs` — all auth handlers implemented:
  - `register` — hashes password, checks for duplicates, returns tokens
  - `login` — verifies password with Argon2, returns tokens
  - `refresh_token` — validates refresh token, issues new token pair
  - `logout` — client-side token clearing
  - `send_otp` / `verify_otp` — TOTP-based 2FA
  - `get_current_user` / `update_current_user` — profile management
- [x] All handlers use `AuthUser` extractor instead of `Uuid::new_v4()` placeholders
- [x] `json_response` helper for consistent API responses
- [x] Argon2 password hashing/verification
- [x] JWT access tokens (15 min TTL) + refresh tokens (30 day TTL)
- [x] Token type differentiation in JWT claims

### Phase 2: Frontend Auth UI ✅
- [x] `src/js/pages/login-page.js` — Login form with error handling
- [x] `src/js/pages/signup-page.js` — Registration form with validation
- [x] `src/js/api.js` — `setTokens()` method, token refresh handling
- [x] `src/js/app.js` — Auth guard redirects to `/login` when unauthenticated
- [x] `src/js/ws.js` — Fixed token key (`auth_token`)
- [x] `src/css/components.css` — Auth page styles

### Phase 3: Database Layer ✅
- [x] All 9 db modules updated to use `SqlitePool` instead of `AnyPool`
- [x] Manual row mapping for all queries (SQLite-compatible)
- [x] Migration system functional

## Remaining / Future Work

### Phase 4: Token Management ✅
- [x] Redis-backed token blacklist for logout — tokens blacklisted with TTL matching their remaining lifetime
- [x] Token rotation with family tracking — reuse detection via refresh token hash comparison, family invalidation on reuse

### Phase 5: Auth Hardening ✅
- [x] Rate limiting on auth routes (stricter than global 100/60s) — 5 req/min per IP+endpoint
- [x] Account lockout after N failed attempts — 5 attempts, 15min lockout
- [x] Password reset flow — token-based, 1hr expiry, single-use
- [x] Email verification flow — token-based, 24hr expiry

### Phase 6: Tauri-Side Auth ✅
- [x] Proper TOTP implementation — real TOTP with secret generation, QR code URI, and verification
- [x] Token expiration tracking — JWT expiry decoded and stored, proactive refresh scheduled
- [x] Auto-refresh before token expiry — refreshes 1 minute before expiry, no more 401 waits
