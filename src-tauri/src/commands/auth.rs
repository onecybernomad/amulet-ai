use crate::db::local_cache::LocalCache;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub user_id: String,
    pub otp: String,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub token: String,
    pub user: User,
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("Password hashing failed: {e}"))
}

fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed = PasswordHash::new(hash).map_err(|e| format!("Invalid hash: {e}"))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

// ── Commands ──────────────────────────────────────────────────────────────

/// Register a new user account.
#[tauri::command]
#[instrument(skip(req))]
pub async fn register(
    req: RegisterRequest,
    cache: State<'_, LocalCache>,
) -> Result<AuthResponse, String> {
    info!(email = %req.email, "Registering new user");

    // Check if user already exists
    if cache.get_user_by_email(&req.email).map_err(|e| e.to_string())?.is_some() {
        warn!(email = %req.email, "Registration failed: user already exists");
        return Err("User with this email already exists".into());
    }

    let password_hash = hash_password(&req.password)?;
    let user_id = Uuid::new_v4().to_string();

    let user = User {
        id: user_id.clone(),
        email: req.email.clone(),
        name: req.name.clone(),
        phone: req.phone.clone(),
        avatar_url: None,
        password_hash: password_hash.clone(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    cache.insert_user(&user).map_err(|e| e.to_string())?;
    let token = generate_session_token();
    cache.store_session(&user_id, &token).map_err(|e| e.to_string())?;

    info!(user_id = %user_id, "User registered successfully");
    Ok(AuthResponse {
        token,
        user: User {
            password_hash: String::new(),
            ..user
        },
    })
}

/// Log in with email and password.
#[tauri::command]
#[instrument(skip(req))]
pub async fn login(
    req: LoginRequest,
    cache: State<'_, LocalCache>,
) -> Result<AuthResponse, String> {
    info!(email = %req.email, "Login attempt");

    let user = cache
        .get_user_by_email(&req.email)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Invalid email or password".to_string())?;

    if !verify_password(&req.password, &user.password_hash)? {
        warn!(email = %req.email, "Login failed: invalid password");
        return Err("Invalid email or password".into());
    }

    let token = generate_session_token();
    cache.store_session(&user.id, &token).map_err(|e| e.to_string())?;

    info!(user_id = %user.id, "User logged in successfully");
    Ok(AuthResponse {
        token,
        user: User {
            password_hash: String::new(),
            ..user
        },
    })
}

/// Verify OTP for two-factor authentication.
#[tauri::command]
pub async fn verify_otp(
    req: VerifyOtpRequest,
    cache: State<'_, LocalCache>,
) -> Result<SessionInfo, String> {
    info!(user_id = %req.user_id, "Verifying OTP");

    let user = cache
        .get_user_by_id(&req.user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    // In production, verify TOTP against stored secret
    // For scaffolding, accept any 6-digit code
    if req.otp.len() != 6 || !req.otp.chars().all(|c| c.is_ascii_digit()) {
        return Err("Invalid OTP format".into());
    }

    let token = generate_session_token();
    cache.store_session(&req.user_id, &token).map_err(|e| e.to_string())?;

    Ok(SessionInfo {
        token,
        user: User {
            password_hash: String::new(),
            ..user
        },
    })
}

/// Log out the current user and invalidate the session.
#[tauri::command]
pub async fn logout(
    token: String,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!("Logging out user");
    cache.remove_session(&token).map_err(|e| e.to_string())?;
    debug!("Session invalidated");
    Ok(())
}
