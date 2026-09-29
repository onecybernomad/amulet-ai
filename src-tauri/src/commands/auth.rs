use crate::db::local_cache::LocalCache;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD, Engine};
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

#[derive(Debug, Serialize)]
pub struct TotpSetupResponse {
    pub secret: String,
    pub qr_code_uri: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpVerifyRequest {
    pub user_id: String,
    pub otp: String,
}

#[derive(Debug, Deserialize)]
pub struct TotpDisableRequest {
    pub user_id: String,
    pub otp: String,
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

fn generate_totp_secret() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 20];
    rand::thread_rng().fill_bytes(&mut bytes);
    STANDARD.encode(&bytes)
}

fn verify_totp(secret: &str, code: &str) -> bool {
    use totp_rs::{Algorithm, TOTP};
    let totp = match TOTP::new(Algorithm::SHA1, 6, 1, 30, STANDARD.decode(secret).unwrap_or_default()) {
        Ok(t) => t,
        Err(_) => return false,
    };
    totp.check_current(code).unwrap_or(false)
}

fn generate_totp_uri(secret: &str, email: &str, issuer: &str) -> String {
    format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}&algorithm=SHA1&digits=6&period=30",
        issuer, email, secret, issuer
    )
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
        totp_secret: None,
        totp_enabled: false,
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

    // Verify TOTP against stored secret
    let secret = user.totp_secret.as_ref().ok_or_else(|| "2FA not enabled".to_string())?;
    
    if !verify_totp(secret, &req.otp) {
        warn!(user_id = %req.user_id, "OTP verification failed");
        return Err("Invalid OTP code".into());
    }

    let token = generate_session_token();
    cache.store_session(&req.user_id, &token).map_err(|e| e.to_string())?;

    Ok(SessionInfo {
        token,
        user: User {
            password_hash: String::new(),
            totp_secret: None,
            totp_enabled: false,
            ..user
        },
    })
}

/// Setup TOTP for a user - generates secret and returns QR code URI.
#[tauri::command]
pub async fn setup_totp(
    user_id: String,
    cache: State<'_, LocalCache>,
) -> Result<TotpSetupResponse, String> {
    info!(user_id = %user_id, "Setting up TOTP");

    let user = cache
        .get_user_by_id(&user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    let secret = generate_totp_secret();
    let qr_uri = generate_totp_uri(&secret, &user.email, "Amulet AI");

    // Store the secret (not yet enabled until verified)
    cache
        .update_user_totp_secret(&user_id, &secret)
        .map_err(|e| e.to_string())?;

    Ok(TotpSetupResponse {
        secret,
        qr_code_uri: qr_uri,
    })
}

/// Verify and enable TOTP for a user.
#[tauri::command]
pub async fn verify_totp_setup(
    req: TotpVerifyRequest,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!(user_id = %req.user_id, "Verifying TOTP setup");

    let user = cache
        .get_user_by_id(&req.user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    let secret = user.totp_secret.as_ref().ok_or_else(|| "TOTP not set up".to_string())?;

    if !verify_totp(secret, &req.otp) {
        return Err("Invalid OTP code".into());
    }

    cache
        .enable_user_totp(&req.user_id)
        .map_err(|e| e.to_string())?;

    info!(user_id = %req.user_id, "TOTP enabled successfully");
    Ok(())
}

/// Disable TOTP for a user.
#[tauri::command]
pub async fn disable_totp(
    req: TotpDisableRequest,
    cache: State<'_, LocalCache>,
) -> Result<(), String> {
    info!(user_id = %req.user_id, "Disabling TOTP");

    let user = cache
        .get_user_by_id(&req.user_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "User not found".to_string())?;

    let secret = user.totp_secret.as_ref().ok_or_else(|| "TOTP not enabled".to_string())?;

    if !verify_totp(secret, &req.otp) {
        return Err("Invalid OTP code".into());
    }

    cache
        .disable_user_totp(&req.user_id)
        .map_err(|e| e.to_string())?;

    info!(user_id = %req.user_id, "TOTP disabled successfully");
    Ok(())
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
