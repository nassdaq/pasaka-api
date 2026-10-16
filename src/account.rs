//! Accounts for the function build: the native API's `/register`, `/login`
//! and `x-api-key`, done through the host's database (`barua_fn`) instead of
//! sqlx. Same `users` table, same Argon2id hashes, so an account made here
//! signs in on the server and the other way round.
//!
//! Only the function uses this; natively `barua_fn` answers with an error and
//! the server keeps its own routes.
#![cfg_attr(not(target_os = "wasi"), allow(dead_code))]

use argon2::password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use barua_fn::{query, query_one};
use serde_json::{json, Value};

/// Lighter than the server's default so a call stays well inside the
/// function's time limit. A hash records its own parameters, so verifying
/// works on either kind.
fn argon() -> Argon2<'static> {
    Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(8 * 1024, 2, 1, None).expect("valid Argon2 params"))
}

/// The server's `AuthResponse`, as JSON text.
fn reply(success: bool, message: &str, user: Option<Value>) -> Result<String, String> {
    serde_json::to_string(&json!({ "success": success, "message": message, "user": user })).map_err(|e| e.to_string())
}

pub fn register(email: &str, password: &str) -> Result<String, String> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') || email.len() > 200 {
        return reply(false, "A valid email is required", None);
    }
    if password.len() < 8 {
        return reply(false, "The password must be at least 8 characters", None);
    }
    let hash = argon()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|e| e.to_string())?
        .to_string();
    let api_key = format!("pasaka_api_{}", uuid::Uuid::new_v4());
    match query_one(
        "insert into users (email, password_hash, api_key) values ($1, $2, $3) returning id, email, api_key",
        &[json!(email), json!(hash), json!(api_key)],
    ) {
        Ok(Some(row)) => reply(true, "Registered", Some(Value::Object(row))),
        Ok(None) => reply(false, "Nothing was created", None),
        Err(e) if e.0.contains("duplicate key") => reply(false, "An account with this email already exists", None),
        Err(e) => Err(e.0),
    }
}

pub fn login(email: &str, password: &str) -> Result<String, String> {
    let email = email.trim().to_lowercase();
    let Some(row) = query_one("select id, email, password_hash, api_key from users where email = $1", &[json!(email)]).map_err(|e| e.0)? else {
        return reply(false, "Invalid email or password", None);
    };
    let stored = row.get("password_hash").and_then(Value::as_str).unwrap_or("");
    let ok = PasswordHash::new(stored).map(|h| argon().verify_password(password.as_bytes(), &h).is_ok()).unwrap_or(false);
    if !ok {
        return reply(false, "Invalid email or password", None);
    }
    reply(true, "Logged in", Some(json!({ "id": row["id"], "email": row["email"], "api_key": row["api_key"] })))
}

/// Whether a key belongs to someone.
pub fn check_key(key: &str) -> Result<bool, String> {
    Ok(query("select 1 from users where api_key = $1", &[json!(key)]).map_err(|e| e.0)?.len() == 1)
}
